module Model exposing (Model, init)

import Core exposing (..)
import OrderedDict as OD


type alias Model =
    { library : Library
    , workflow : Workflow
    }


init : Model
init =
    { library = OD.empty
    , workflow =
        { steps =
            [ { name = "RNA-seq", args = OD.fromList [ ( "day", VInt 1 ) ] }
            , { name = "RNA-seq", args = OD.empty }
            ]
        , goal =
            Just
                { name = "Differential gene expression"
                , args = OD.empty
                }
        }
    }
