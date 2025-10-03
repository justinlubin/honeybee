module Update exposing (Msg(..), subscriptions, update)

import Assoc exposing (Assoc)
import Cell
import Compile
import Complete
import Core exposing (..)
import Incoming
import Model exposing (Model)
import Outgoing
import Util



--------------------------------------------------------------------------------
-- Messages


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
    | UserSelectedFunction { cellIndex : Int } Int
    | UserDeselectedFunction { cellIndex : Int }
    | UserSelectedMetadata { cellIndex : Int, functionIndex : Int } Int
    | UserMadePbnChoice Int
    | UserRequestedDownload Outgoing.DownloadMessage
    | UserClickedDevMode
      -- Backend actions
    | BackendSentPbnStatus Incoming.PbnStatusMessage
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
            |> Complete.complete { allowGoalHoles = True }
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
                            { model
                                | program =
                                    Core.set pi
                                        (Just (Core.fresh name sig))
                                        model.program
                                , pbnStatus = Nothing
                            }
            in
            syncGoalSuggestions ( newModel, Cmd.none )

        UserClearedStep pi ->
            let
                newModel =
                    { model
                        | program = Core.set pi Nothing model.program
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

        UserStartedNavigation x ->
            ( model
            , Cmd.batch
                [ Outgoing.oScrollIntoView { selector = "#navigation-pane" }
                , Outgoing.oPbnInit x
                ]
            )

        UserSelectedFunction { cellIndex } functionIndex ->
            ( setFunctionChoice
                { cellIndex = cellIndex, functionIndex = Just functionIndex }
                model
            , Cmd.none
            )

        UserDeselectedFunction { cellIndex } ->
            ( setFunctionChoice
                { cellIndex = cellIndex, functionIndex = Nothing }
                model
            , Cmd.none
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
            , Outgoing.oPbnChoose { choice = choice }
            )

        UserRequestedDownload x ->
            ( model
            , Outgoing.oDownload x
            )

        UserClickedDevMode ->
            ( { model | program = Core.example }
            , Cmd.none
            )

        BackendSentPbnStatus status ->
            ( { model | pbnStatus = Just status }
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


subscriptions : Model -> Sub Msg
subscriptions _ =
    Sub.batch
        [ Incoming.iPbnStatus <|
            \psResult ->
                case psResult of
                    Ok ps ->
                        BackendSentPbnStatus ps

                    Err e ->
                        let
                            _ =
                                Debug.log "error" e
                        in
                        BackendSentPbnStatus { cells = [], output = Nothing }
        , Incoming.iValidGoalMetadata <|
            \vgmResult ->
                case vgmResult of
                    Ok vgm ->
                        BackendSentValidGoalMetadata vgm

                    Err _ ->
                        BackendSentValidGoalMetadata
                            { goalName = "", choices = [] }
        ]
