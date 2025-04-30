module Model exposing (Model, init)

import Config
import Core exposing (..)
import Port


type alias Model =
    { library : Library
    , workflow : Workflow
    , pbnStatus : Maybe Port.PbnStatusMessage
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
    }
