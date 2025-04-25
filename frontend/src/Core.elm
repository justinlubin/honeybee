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


type StepKind
    = Prop
    | Type


type alias StepSignature =
    { params : Assoc String ValueType
    , kind : StepKind
    }


type Step
    = SHole
    | SConcrete
        { name : String
        , args : Assoc String (Maybe Value)
        }


newStep : String -> StepSignature -> Step
newStep name sig =
    SConcrete
        { name = name
        , args = List.map (\( k, _ ) -> ( k, Nothing )) sig.params
        }


type alias Library =
    Assoc String StepSignature


props : Library -> Library
props =
    List.filter (\( _, s ) -> s.kind == Prop)


types : Library -> Library
types =
    List.filter (\( _, s ) -> s.kind == Type)


type alias Workflow =
    { steps : List Step
    , goal : Step
    }
