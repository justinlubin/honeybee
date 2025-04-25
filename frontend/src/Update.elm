module Update exposing (Msg(..), update)

import Assoc
import Core exposing (..)
import Model exposing (Model)


type Msg
    = AddBlankStep
    | SetStep (Maybe Int) String
    | ClearStep (Maybe Int)


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    Debug.log "updated model, cmd msg:" <|
        case msg of
            AddBlankStep ->
                let
                    oldWorkflow =
                        model.workflow

                    newWorkflow =
                        { oldWorkflow | steps = oldWorkflow.steps ++ [ SHole ] }
                in
                ( { model | workflow = newWorkflow }
                , Cmd.none
                )

            SetStep mi name ->
                case Assoc.get name model.library of
                    Nothing ->
                        ( model, Cmd.none )

                    Just sig ->
                        let
                            oldWorkflow =
                                model.workflow

                            newWorkflow =
                                case mi of
                                    Nothing ->
                                        { oldWorkflow | goal = newStep name sig }

                                    Just i ->
                                        { oldWorkflow
                                            | steps =
                                                List.indexedMap
                                                    (\j s ->
                                                        if i == j then
                                                            newStep name sig

                                                        else
                                                            s
                                                    )
                                                    oldWorkflow.steps
                                        }
                        in
                        ( { model | workflow = newWorkflow }
                        , Cmd.none
                        )

            ClearStep mi ->
                let
                    oldWorkflow =
                        model.workflow

                    newWorkflow =
                        case mi of
                            Nothing ->
                                { oldWorkflow | goal = SHole }

                            Just i ->
                                { oldWorkflow
                                    | steps =
                                        List.indexedMap
                                            (\j s ->
                                                if i == j then
                                                    SHole

                                                else
                                                    s
                                            )
                                            oldWorkflow.steps
                                }
                in
                ( { model | workflow = newWorkflow }
                , Cmd.none
                )
