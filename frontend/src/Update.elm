module Update exposing (Msg(..), subscriptions, update)

import Assoc exposing (Assoc)
import Compile
import Core exposing (..)
import Model exposing (Model)
import Port


type Msg
    = AddBlankStep
    | SetStep StepIndex String
    | ClearStep StepIndex
    | RemoveStep Int
    | SetArgumentByString ValueType StepIndex String String
    | StartNavigating { programSource : String }
    | MakePbnChoice Int
    | ReceivePbnStatus Port.PbnStatusMessage
    | Download Port.DownloadMessage
    | PbnCheck Port.PbnCheckMessage
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


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        AddBlankStep ->
            ( { model
                | workflow =
                    Core.insertStep
                        (List.length (steps model.workflow))
                        SHole
                        model.workflow
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
                            }
            in
            ( newModel
            , Port.sendPbnCheck
                { programSource = Compile.compile newModel.workflow }
            )

        ClearStep si ->
            let
                newModel =
                    { model | workflow = Core.setStep si SHole model.workflow }
            in
            ( newModel
            , Port.sendPbnCheck
                { programSource = Compile.compile newModel.workflow }
            )

        RemoveStep i ->
            let
                newModel =
                    { model | workflow = Core.removeStep i model.workflow }
            in
            ( newModel
            , Port.sendPbnCheck
                { programSource = Compile.compile newModel.workflow }
            )

        SetArgumentByString vt si param str ->
            let
                v =
                    valueFromString vt str
            in
            let
                newModel =
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
                    }
            in
            ( newModel
            , Port.sendPbnCheck
                { programSource = Compile.compile newModel.workflow }
            )

        StartNavigating x ->
            ( model
            , Port.sendPbnInit x
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

        PbnCheck x ->
            ( model
            , Port.sendPbnCheck x
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
                                List.map
                                    (\( argName, argChoices ) ->
                                        ( argName
                                        , suggestions args argName argChoices
                                        )
                                    )
                                    choices
                          }
                        , Cmd.none
                        )


suggestions :
    Assoc String Value
    -> String
    -> List String
    -> List Value
suggestions existingArgs argName newArgStrings =
    case Assoc.get argName existingArgs of
        Nothing ->
            []

        Just v ->
            let
                vt =
                    Core.valueType v
            in
            List.map (valueFromString vt) newArgStrings


subscriptions : Model -> Sub Msg
subscriptions _ =
    Sub.batch
        [ Port.receivePbnStatus ReceivePbnStatus
        , Port.receiveValidGoalMetadata ReceiveValidGoalMetadata
        ]
