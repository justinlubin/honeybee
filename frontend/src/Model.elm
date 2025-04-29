module Model exposing (Model, init)

import Core exposing (..)


type alias Model =
    { library : Library
    , workflow : Workflow
    , synthesisResult : Maybe String
    }


init : Library -> Model
init library =
    { library = library
    , workflow = exampleWorkflow
    , synthesisResult = Nothing
    }
