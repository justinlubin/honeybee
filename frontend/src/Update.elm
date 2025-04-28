module Update exposing (Msg(..), update)

import Assoc
import Core exposing (..)
import Model exposing (Model)


type Msg
    = AddBlankStep
    | SetStep StepIndex String
    | ClearStep StepIndex
    | RemoveStep Int
    | SetArgumentByString ValueType StepIndex String String


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
            case Assoc.get name model.library of
                Nothing ->
                    ( model, Cmd.none )

                Just sig ->
                    ( { model
                        | workflow =
                            Core.setStep si
                                (freshStep name sig)
                                model.workflow
                      }
                    , Cmd.none
                    )

        ClearStep si ->
            ( { model | workflow = Core.setStep si SHole model.workflow }
            , Cmd.none
            )

        RemoveStep i ->
            ( { model | workflow = Core.removeStep i model.workflow }
            , Cmd.none
            )

        SetArgumentByString vt si param str ->
            let
                v =
                    valueFromString vt str
            in
            ( { model
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
            , Cmd.none
            )
