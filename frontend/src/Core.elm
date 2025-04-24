module Core exposing (..)

import OrderedDict exposing (OrderedDict)


type ValueType
    = VTBool
    | VTInt
    | VTStr


type Value
    = VBool Bool
    | VInt Int
    | VStr String


type alias Library =
    OrderedDict String StepSignature


type alias StepSignature =
    { params : OrderedDict String ValueType
    }


type alias Step =
    { name : String
    , args : OrderedDict String Value
    }


type alias Workflow =
    { steps : List Step
    , goal : Maybe Step
    }
