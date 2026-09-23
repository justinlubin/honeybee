module Update exposing (Msg(..), subscriptions, update)

import Assoc exposing (Assoc)
import Browser.Events
import Cell
import Compile
import Complete
import Core exposing (..)
import Incoming
import Json.Decode as D
import Model exposing (Model)
import Outgoing
import Util



--------------------------------------------------------------------------------
-- Messages


type alias Key =
    { cmd : Bool, ctrl : Bool, shift : Bool, key : String }


type
    Msg
    -- No-op
    = Nop
      -- User actions
    | UserAddedBlankStep
    | UserSetStep ProgramIndex String
    | UserClearedStep ProgramIndex
    | UserRemovedStep Int
    | UserSetArgument ProgramIndex String String
    | UserStartedNavigation { programSource : String }
    | UserSelectedFunction { cellIndex : Int } Int (Maybe Int)
    | UserDeselectedFunction { cellIndex : Int }
    | UserSelectedMetadata { cellIndex : Int, functionIndex : Int } Int
    | UserMadePbnChoice Int
    | UserRequestedDownload Outgoing.DownloadMessage
    | UserClickedExample
    | UserClickedUndo
    | UserClickedHelp String
    | UserPressedShortcut Key
    | UserMouseDownedHandle
      -- General handlers
    | UserClicked
    | UserMouseMoved Bool Float
    | UserMouseUpped Float
      -- Backend actions
    | BackendSentPbnStatus { speculative : Bool } Incoming.PbnStatusMessage
    | BackendSentValidGoalMetadata Incoming.ValidGoalMetadataMessage



--------------------------------------------------------------------------------
-- Model helpers


setArgument : ProgramIndex -> String -> String -> Model -> Model
setArgument pi param s model =
    { model
        | program =
            Core.modify pi
                (Maybe.map
                    (\f ->
                        { f
                            | args =
                                Assoc.modify
                                    param
                                    (\( _, vt ) -> ( s, vt ))
                                    f.args
                        }
                    )
                )
                model.program
        , pbnStatus = Nothing
    }


setFunctionChoice : { cellIndex : Int, functionIndex : Maybe Int } -> Model -> Model
setFunctionChoice { cellIndex, functionIndex } model =
    case model.pbnStatus of
        Nothing ->
            model

        Just status ->
            let
                newStatus =
                    { status
                        | cells =
                            List.indexedMap
                                (\i c ->
                                    case c of
                                        Cell.Code _ ->
                                            c

                                        Cell.Choice ch ->
                                            if i == cellIndex then
                                                Cell.Choice
                                                    { ch
                                                        | selectedFunctionChoice =
                                                            functionIndex
                                                    }

                                            else
                                                c
                                )
                                status.cells
                    }
            in
            { model | pbnStatus = Just newStatus }


setMetadataChoice :
    { cellIndex : Int, functionIndex : Int, metadataIndex : Int }
    -> Model
    -> Model
setMetadataChoice { cellIndex, functionIndex, metadataIndex } model =
    case model.pbnStatus of
        Nothing ->
            model

        Just status ->
            let
                updateFunctionChoices =
                    List.indexedMap
                        (\fci fc ->
                            if fci == functionIndex then
                                { fc | selectedMetadataChoice = metadataIndex }

                            else
                                fc
                        )

                updateCells =
                    List.indexedMap
                        (\i c ->
                            case c of
                                Cell.Code _ ->
                                    c

                                Cell.Choice ch ->
                                    if i == cellIndex then
                                        Cell.Choice
                                            { ch
                                                | functionChoices =
                                                    updateFunctionChoices
                                                        ch.functionChoices
                                            }

                                    else
                                        c
                        )

                newStatus =
                    { status | cells = updateCells status.cells }
            in
            { model | pbnStatus = Just newStatus }



--------------------------------------------------------------------------------
-- Suggestion helpers


syncGoalSuggestions : ( Model, Cmd msg ) -> ( Model, Cmd msg )
syncGoalSuggestions ( model, cmd ) =
    case
        model.program
            |> Complete.complete { allowPropHoles = True, allowGoalHoles = True }
            |> Maybe.map Compile.compile
    of
        Just programSource ->
            ( model
            , Cmd.batch
                [ cmd
                , Outgoing.oPbnCheck { programSource = programSource }
                ]
            )

        Nothing ->
            ( { model | goalSuggestions = [] }, cmd )


consistentSuggestions :
    Fact String
    -> List (Assoc String Value)
    -> Assoc String (List Value)
consistentSuggestions goalFact choices =
    Assoc.map
        (\argName ( argStr, argType ) ->
            case Core.parse argType argStr of
                ParseFail ->
                    []

                ParseSuccess _ ->
                    []

                Blank ->
                    choices
                        |> List.filterMap
                            (\choice ->
                                if Core.consistent goalFact choice then
                                    Assoc.get argName choice

                                else
                                    Nothing
                            )
                        |> Util.unique
                        |> List.sortBy Core.unparse
        )
        goalFact.args



--------------------------------------------------------------------------------
-- Main update


doUndo : Model -> ( Model, Cmd Msg )
doUndo model =
    case model.pbnStatus of
        Just status ->
            if status.canUndo then
                ( model
                , Cmd.batch
                    [ Outgoing.oPbnUndo {}
                    , Outgoing.oScrollIntoView { selector = "#active-choice-cell" }
                    ]
                )

            else
                ( { model | pbnStatus = Nothing }
                , Cmd.none
                )

        Nothing ->
            ( model, Cmd.none )


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        Nop ->
            ( model, Cmd.none )

        UserAddedBlankStep ->
            ( { model
                | program =
                    Core.insert
                        (List.length model.program.props)
                        Nothing
                        model.program
                , pbnStatus = Nothing
              }
            , Cmd.none
            )

        UserSetStep pi name ->
            let
                newModel =
                    case Core.getSigFor pi name model.library of
                        Nothing ->
                            model

                        Just sig ->
                            let
                                intermediateProgram =
                                    Core.set pi
                                        (Just (Core.fresh name sig))
                                        model.program

                                newProgram =
                                    case pi of
                                        Goal ->
                                            intermediateProgram

                                        Prop _ ->
                                            { intermediateProgram | goal = Nothing }
                            in
                            { model
                                | program = newProgram
                                , pbnStatus = Nothing
                            }
            in
            syncGoalSuggestions ( newModel, Cmd.none )

        UserClearedStep pi ->
            let
                intermediateProgram =
                    Core.set pi Nothing model.program

                newProgram =
                    { intermediateProgram | goal = Nothing }

                newModel =
                    { model
                        | program = newProgram
                        , pbnStatus = Nothing
                    }
            in
            syncGoalSuggestions ( newModel, Cmd.none )

        UserRemovedStep i ->
            let
                newModel =
                    { model
                        | program = Core.remove i model.program
                        , pbnStatus = Nothing
                    }
            in
            syncGoalSuggestions ( newModel, Cmd.none )

        UserSetArgument pi param str ->
            syncGoalSuggestions
                ( setArgument pi param str model
                , Cmd.none
                )

        UserStartedNavigation { programSource } ->
            ( model
            , Cmd.batch
                [ Outgoing.oScrollIntoView { selector = "#active-choice-cell" }
                , Outgoing.oPbnInit { programSource = programSource, sound = model.sound }
                ]
            )

        UserSelectedFunction { cellIndex } functionIndex speculateChoice ->
            ( setFunctionChoice
                { cellIndex = cellIndex, functionIndex = Just functionIndex }
                model
            , Cmd.batch
                [ Outgoing.oScrollIntoView { selector = "#active-choice-cell" }
                , case speculateChoice of
                    Just choice ->
                        Outgoing.oPbnSpeculate { choice = choice }

                    Nothing ->
                        Cmd.none
                ]
            )

        UserDeselectedFunction { cellIndex } ->
            ( setFunctionChoice
                { cellIndex = cellIndex, functionIndex = Nothing }
                { model | speculativePbnStatus = Nothing }
            , Outgoing.oScrollIntoView { selector = "#active-choice-cell" }
            )

        UserSelectedMetadata { cellIndex, functionIndex } metadataIndex ->
            ( setMetadataChoice
                { cellIndex = cellIndex
                , functionIndex = functionIndex
                , metadataIndex = metadataIndex
                }
                model
            , Cmd.none
            )

        UserMadePbnChoice choice ->
            ( model
            , Cmd.batch
                [ Outgoing.oPbnChoose { choice = choice }
                , Outgoing.oScrollIntoView { selector = "#active-choice-cell" }
                ]
            )

        UserRequestedDownload x ->
            ( model
            , Outgoing.oDownload x
            )

        UserClickedExample ->
            ( { model | program = Core.example model.library }
            , Cmd.none
            )

        UserClickedUndo ->
            doUndo model

        UserClickedHelp id ->
            if model.activeHelp == Just id then
                ( { model | activeHelp = Nothing }, Cmd.none )

            else
                ( { model | activeHelp = Just id }, Cmd.none )

        UserPressedShortcut { cmd, ctrl, key } ->
            if (cmd || ctrl) && key == "Z" then
                doUndo model

            else
                ( model, Cmd.none )

        UserMouseDownedHandle ->
            ( { model
                | dragHandleState =
                    Model.Moving
                        (Model.toFraction model.dragHandleState)
              }
            , Cmd.none
            )

        UserClicked ->
            ( { model | activeHelp = Nothing }, Cmd.none )

        UserMouseMoved isDown fraction ->
            ( { model
                | dragHandleState =
                    if isDown then
                        Model.Moving fraction

                    else
                        Model.Static (Model.toFraction model.dragHandleState)
              }
            , Cmd.none
            )

        UserMouseUpped fraction ->
            ( { model | dragHandleState = Model.Static fraction }
            , Cmd.none
            )

        BackendSentPbnStatus { speculative } status ->
            ( if speculative then
                { model | speculativePbnStatus = Just status }

              else
                { model
                    | pbnStatus = Just status
                    , speculativePbnStatus = Nothing
                }
            , Cmd.none
            )

        BackendSentValidGoalMetadata { goalName, choices } ->
            case model.program.goal of
                Nothing ->
                    ( model, Cmd.none )

                Just goalFact ->
                    if goalFact.name /= goalName then
                        ( model, Cmd.none )

                    else
                        ( { model
                            | goalSuggestions =
                                consistentSuggestions goalFact choices
                          }
                        , Cmd.none
                        )



--------------------------------------------------------------------------------
-- Subscriptions


decodeFraction : D.Decoder Float
decodeFraction =
    D.map2 (/)
        (D.field "pageX" D.float)
        (D.at [ "currentTarget", "defaultView", "innerWidth" ] D.float)


decodeButtons : D.Decoder Bool
decodeButtons =
    D.field "buttons" (D.map (\buttons -> buttons == 1) D.int)


subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.batch
        [ Browser.Events.onKeyDown <|
            D.map UserPressedShortcut <|
                D.map4 Key
                    (D.field "metaKey" D.bool)
                    (D.field "ctrlKey" D.bool)
                    (D.field "shiftKey" D.bool)
                    (D.field "key" D.string |> D.map String.toUpper)
        , Browser.Events.onClick (D.succeed UserClicked)
        , case model.dragHandleState of
            Model.Static _ ->
                Sub.none

            Model.Moving _ ->
                Sub.batch
                    [ Browser.Events.onMouseMove
                        (D.map2 UserMouseMoved decodeButtons decodeFraction)
                    , Browser.Events.onMouseUp
                        (D.map UserMouseUpped decodeFraction)
                    ]
        , Incoming.iPbnStatus <|
            \psResult ->
                case psResult of
                    Ok ps ->
                        BackendSentPbnStatus { speculative = False } ps

                    Err e ->
                        let
                            _ =
                                Debug.log "status error" e
                        in
                        BackendSentPbnStatus { speculative = False }
                            { cells = []
                            , output = Nothing
                            , canUndo = False
                            }
        , Incoming.iPbnSpeculativeStatus <|
            \psResult ->
                case psResult of
                    Ok ps ->
                        BackendSentPbnStatus { speculative = True } ps

                    Err e ->
                        let
                            _ =
                                Debug.log "speculative status error" e
                        in
                        BackendSentPbnStatus { speculative = True }
                            { cells = []
                            , output = Nothing
                            , canUndo = False
                            }
        , Incoming.iValidGoalMetadata <|
            \vgmResult ->
                case vgmResult of
                    Ok vgm ->
                        BackendSentValidGoalMetadata vgm

                    Err _ ->
                        BackendSentValidGoalMetadata
                            { goalName = "", choices = [] }
        ]
