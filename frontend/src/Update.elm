module Update exposing (Msg(..), subscriptions, update)

import Assoc exposing (Assoc)
import Compile
import Core exposing (..)
import Json.Decode as D
import Model exposing (Model)
import Port
import Util


type Msg
    = Nop
    | AddBlankStep
    | SetStep StepIndex String
    | ClearStep StepIndex
    | RemoveStep Int
    | SetArgumentByString ValueType StepIndex String String
    | SetArgumentTextField Port.SetTextFieldMessage StepIndex String Value
    | StartNavigating { programSource : String }
    | MakePbnChoice Int
    | ReceivePbnStatus Port.PbnStatusMessage
    | Download Port.DownloadMessage
    | ReceiveValidGoalMetadata Port.ValidGoalMetadataMessage


valueFromString : ValueType -> String -> Value
valueFromString vt str =
    if String.isEmpty str then
        VHole vt

    else
        case vt of
            VTInt ->
                str
                    |> String.toInt
                    |> Maybe.map VInt
                    |> Maybe.withDefault (VHole VTInt)

            VTBool ->
                case String.toLower str of
                    "true" ->
                        VBool True

                    "false" ->
                        VBool False

                    _ ->
                        VHole VTBool

            VTStr ->
                VStr str


setArgument : Model -> StepIndex -> String -> Value -> Model
setArgument model si param v =
    { model
        | workflow =
            Core.modifyStep si
                (\s ->
                    case s of
                        SHole ->
                            SHole

                        SConcrete { name, args } ->
                            SConcrete
                                { name = name
                                , args = args |> Assoc.set param v
                                }
                )
                model.workflow
        , pbnStatus = Nothing
    }


syncGoalSuggestions : ( Model, Cmd msg ) -> ( Model, Cmd msg )
syncGoalSuggestions ( model, cmd ) =
    case Compile.compile { allowGoalHoles = True } model.workflow of
        Just programSource ->
            ( model
            , Cmd.batch
                [ cmd
                , Port.sendPbnCheck { programSource = programSource }
                ]
            )

        Nothing ->
            ( { model | goalSuggestions = [] }, cmd )


consistentSuggestions :
    Assoc String Value
    -> List (Assoc String Value)
    -> Assoc String (List Value)
consistentSuggestions args choices =
    Assoc.map
        (\argName argValue ->
            case argValue of
                VHole _ ->
                    choices
                        |> List.filterMap
                            (\choice ->
                                if Core.argsConsistent args choice then
                                    Assoc.get argName choice

                                else
                                    Nothing
                            )
                        |> Util.unique
                        |> List.sortBy
                            (Core.unparseValue
                                >> Maybe.withDefault ""
                            )

                _ ->
                    []
        )
        args


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        Nop ->
            ( model, Cmd.none )

        AddBlankStep ->
            ( { model
                | workflow =
                    Core.insertStep
                        (List.length (steps model.workflow))
                        SHole
                        model.workflow
                , pbnStatus = Nothing
              }
            , Cmd.none
            )

        SetStep si name ->
            let
                newModel =
                    case Assoc.get name model.library of
                        Nothing ->
                            model

                        Just sig ->
                            { model
                                | workflow =
                                    Core.setStep si
                                        (freshStep name sig)
                                        model.workflow
                                , pbnStatus = Nothing
                            }
            in
            syncGoalSuggestions ( newModel, Cmd.none )

        ClearStep si ->
            let
                newModel =
                    { model
                        | workflow = Core.setStep si SHole model.workflow
                        , pbnStatus = Nothing
                    }
            in
            syncGoalSuggestions ( newModel, Cmd.none )

        RemoveStep i ->
            let
                newModel =
                    { model
                        | workflow = Core.removeStep i model.workflow
                        , pbnStatus = Nothing
                    }
            in
            syncGoalSuggestions ( newModel, Cmd.none )

        SetArgumentByString vt si param str ->
            syncGoalSuggestions
                ( setArgument model si param (valueFromString vt str)
                , Cmd.none
                )

        SetArgumentTextField x si param v ->
            syncGoalSuggestions
                ( setArgument model si param v
                , Port.sendSetTextField x
                )

        StartNavigating x ->
            ( model
            , Cmd.batch
                [ Port.scrollTo { x = 0, y = 0 }
                , Port.sendPbnInit x
                ]
            )

        MakePbnChoice i ->
            ( model
            , Port.sendPbnChoice { choice = i }
            )

        ReceivePbnStatus status ->
            ( { model | pbnStatus = Just status }
            , Cmd.none
            )

        Download x ->
            ( model
            , Port.sendDownload x
            )

        ReceiveValidGoalMetadata { goalName, choices } ->
            case goal model.workflow of
                SHole ->
                    ( model, Cmd.none )

                SConcrete { name, args } ->
                    if name /= goalName then
                        ( model, Cmd.none )

                    else
                        ( { model
                            | goalSuggestions =
                                consistentSuggestions args choices
                          }
                        , Cmd.none
                        )


subscriptions : Model -> Sub Msg
subscriptions _ =
    Sub.batch
        [ Port.receivePbnStatus ReceivePbnStatus
        , Port.receiveValidGoalMetadata <|
            \val ->
                case D.decodeValue Port.decodeValidGoalMetadata val of
                    Ok vgm ->
                        Debug.log "Msg" <| ReceiveValidGoalMetadata vgm

                    Err _ ->
                        ReceiveValidGoalMetadata { goalName = "", choices = [] }
        ]
