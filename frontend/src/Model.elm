module Model exposing (Model, init)

import Assoc exposing (Assoc)
import Core exposing (Library, Value, WorkingProgram)
import Incoming


type alias Model =
    { library : Library
    , program : WorkingProgram
    , pbnStatus : Maybe Incoming.PbnStatusMessage
    , goalSuggestions : Assoc String (List Value)
    }


init : Library -> Model
init library =
    { library = library
    , program = Core.empty
    , pbnStatus = Nothing
    , goalSuggestions = []
    }
