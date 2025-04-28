module Update exposing (Msg(..), update)

import Assoc
import Core exposing (..)
import Model exposing (Model)


type Msg
    = AddBlankStep
    | SetStep StepIndex String
    | ClearStep StepIndex
    | RemoveStep Int


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
