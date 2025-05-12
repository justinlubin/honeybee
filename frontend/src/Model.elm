module Model exposing (Model, init)

import Assoc exposing (Assoc)
import Config
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
    , workflow =
        if Config.debug then
            exampleWorkflow

        else
            emptyWorkflow
    , pbnStatus = Nothing
    , goalSuggestions = []

    -- [ ( "sample1", [ VStr "pepita", VStr "moko" ] )
    -- , ( "sample2", [ VStr "pepita" ] )
    -- ]
    }
