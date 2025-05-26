module Model exposing (Model, init)

import Assoc exposing (Assoc)
import Core exposing (Library, Value, WorkingProgram)
import Port


type alias Model =
    { library : Library
    , program : WorkingProgram
    , pbnStatus : Maybe Port.PbnStatusMessage
    , goalSuggestions : Assoc String (List Value)
    }


init : Library -> Model
init library =
    { library = library
    , program = Core.empty
    , pbnStatus = Nothing
    , goalSuggestions = []
    }
