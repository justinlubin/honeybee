module Model exposing
    ( DragHandleState(..)
    , Model
    , inUserStudy
    , init
    , toFraction
    )

import Assoc exposing (Assoc)
import Compile
import Complete
import Core exposing (Library, Value, WorkingProgram)
import Incoming
import Outgoing



-- Drag handling based on the following example (BSD-3 license, Evan Czaplicki)
-- https://github.com/elm/browser/blob/1.0.2/examples/src/Drag.elm


type DragHandleState
    = Static Float
    | Moving Float


toFraction : DragHandleState -> Float
toFraction dragState =
    case dragState of
        Static fraction ->
            fraction

        Moving fraction ->
            fraction


type alias Model =
    { sound : Bool
    , partid : Maybe String
    , library : Library
    , program : WorkingProgram
    , pbnStatus : Maybe Incoming.PbnStatusMessage
    , speculativePbnStatus : Maybe Incoming.PbnStatusMessage
    , currentGoalMetadataSuggestions : Assoc String (List Value)
    , goalSuggestions : List String
    , activeHelp : Maybe String
    , dragHandleState : DragHandleState
    }


type alias Flags =
    { library : Library
    , sound : Bool
    , partid : Maybe String
    }


init : Flags -> ( Model, Cmd msg )
init { library, sound, partid } =
    let
        model =
            { sound = sound
            , partid = partid
            , library = library
            , program = Core.empty

            -- , program = Core.example library
            , pbnStatus = Nothing
            , speculativePbnStatus = Nothing
            , currentGoalMetadataSuggestions = []
            , goalSuggestions = []
            , activeHelp = Nothing
            , dragHandleState = Static 0.5
            }

        cmd =
            case
                model.program
                    |> Complete.completeProps { allowPropHoles = True }
                    |> Maybe.map Compile.props
            of
                Just propsSource ->
                    Outgoing.oPbnCheck { propsSource = propsSource }

                Nothing ->
                    Cmd.none
    in
    ( model, cmd )


inUserStudy : Model -> Bool
inUserStudy m =
    m.partid /= Nothing
