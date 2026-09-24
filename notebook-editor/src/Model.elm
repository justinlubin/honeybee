module Model exposing
    ( DragHandleState(..)
    , Model
    , init
    , toFraction
    )

import Assoc exposing (Assoc)
import Core exposing (Library, Value, WorkingProgram)
import Incoming



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
    , log : Bool
    , partid : String
    , library : Library
    , program : WorkingProgram
    , pbnStatus : Maybe Incoming.PbnStatusMessage
    , speculativePbnStatus : Maybe Incoming.PbnStatusMessage
    , goalSuggestions : Assoc String (List Value)
    , activeHelp : Maybe String
    , dragHandleState : DragHandleState
    }


type alias Flags =
    { library : Library, sound : Bool, log : Bool, partid : String }


init : Flags -> Model
init { library, sound, log, partid } =
    { sound = sound
    , log = log
    , partid = partid
    , library = library
    , program = Core.example library -- Core.empty
    , pbnStatus = Nothing
    , speculativePbnStatus = Nothing
    , goalSuggestions = []
    , activeHelp = Nothing
    , dragHandleState = Static 0.55
    }
