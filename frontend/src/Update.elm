module Update exposing (Msg(..), subscriptions, update)

import Assoc exposing (Assoc)
import Compile
import Complete
import Core exposing (..)
import Incoming
import Model exposing (Model)
import Outgoing
import Util


type Msg
    = Nop
    | AddBlankStep
    | SetStep ProgramIndex String
    | ClearStep ProgramIndex
    | RemoveStep Int
    | SetArgumentByString ProgramIndex String String
    | StartNavigating { programSource : String }
    | MakePbnChoice Int
    | ReceivePbnStatus Incoming.PbnStatusMessage
    | Download Outgoing.DownloadMessage
    | ReceiveValidGoalMetadata Incoming.ValidGoalMetadataMessage
    | LoadExample


setArgument : Model -> ProgramIndex -> String -> String -> Model
setArgument model pi param s =
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


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        Nop ->
            ( model, Cmd.none )

        AddBlankStep ->
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

        SetStep pi name ->
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

        ClearStep pi ->
            let
                newModel =
                    { model
                        | program = Core.set pi Nothing model.program
                        , pbnStatus = Nothing
                    }
            in
            syncGoalSuggestions ( newModel, Cmd.none )

        RemoveStep i ->
            let
                newModel =
                    { model
                        | program = Core.remove i model.program
                        , pbnStatus = Nothing
                    }
            in
            syncGoalSuggestions ( newModel, Cmd.none )

        SetArgumentByString pi param str ->
            syncGoalSuggestions
                ( setArgument model pi param str
                , Cmd.none
                )

        StartNavigating x ->
            ( model
            , Cmd.batch
                [ Outgoing.oScrollIntoView { selector = ".navigation-pane" }
                , Outgoing.oPbnInit x
                ]
            )

        MakePbnChoice i ->
            ( model
            , Outgoing.oPbnChoose { choice = i }
            )

        ReceivePbnStatus status ->
            ( { model | pbnStatus = Just status }
            , Cmd.none
            )

        Download x ->
            ( model
            , Outgoing.oDownload x
            )

        ReceiveValidGoalMetadata { goalName, choices } ->
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

        LoadExample ->
            ( { model | program = Core.example }
            , Cmd.none
            )


subscriptions : Model -> Sub Msg
subscriptions _ =
    Sub.batch
        [ Incoming.iPbnStatus ReceivePbnStatus
        , Incoming.iValidGoalMetadata <|
            \vgmResult ->
                case vgmResult of
                    Ok vgm ->
                        ReceiveValidGoalMetadata vgm

                    Err _ ->
                        ReceiveValidGoalMetadata { goalName = "", choices = [] }
        ]
