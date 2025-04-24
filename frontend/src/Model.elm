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
        { steps =
            [ { name = "RNA-seq"
              , args = [ ( "day", VInt 1 ) ]
              }
            , { name = "RNA-seq"
              , args = []
              }
            ]
        , goal =
            Just
                { name = "Differential gene expression"
                , args = []
                }
        }
    }
