module Model exposing (Model, init)

import Core exposing (..)


type alias Model =
    { library : Library
    , workflow : Workflow
    }


init : Library -> Model
init library =
    { library = library
    , workflow =
        { steps = []
        , goal = SHole
        }
    }
