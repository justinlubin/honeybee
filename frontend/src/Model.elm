module Model exposing (Model, init)

import Assoc exposing (Assoc)
import Core exposing (..)
import Port


type alias Model =
    { library : Library
    , workflow : Workflow
    , pbnStatus : Maybe Port.PbnStatusMessage
    , goalSuggestions : Assoc String (List Value)
    }


init : Library -> Model
init library =
    { library = library
    , workflow = emptyWorkflow
    , pbnStatus = Nothing
    , goalSuggestions = []
    }
