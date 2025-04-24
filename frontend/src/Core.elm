module Core exposing (..)

import Assoc exposing (Assoc)


type ValueType
    = VTBool
    | VTInt
    | VTStr


type Value
    = VBool Bool
    | VInt Int
    | VStr String


type alias Library =
    { props : Assoc String StepSignature
    , types : Assoc String StepSignature
    }


type alias StepSignature =
    { params : Assoc String ValueType
    }


type alias Step =
    { name : String
    , args : Assoc String Value
    }


type alias Workflow =
    { steps : List Step
    , goal : Maybe Step
    }
