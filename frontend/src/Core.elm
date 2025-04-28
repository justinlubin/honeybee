module Core exposing
    ( Library
    , Step(..)
    , StepIndex(..)
    , StepKind(..)
    , StepSignature
    , Value(..)
    , ValueType(..)
    , Workflow
    , emptyWorkflow
    , freshStep
    , goal
    , insertStep
    , modifyStep
    , props
    , removeStep
    , setStep
    , steps
    , types
    , valueType
    )

import Assoc exposing (Assoc)
import Util


type ValueType
    = VTBool
    | VTInt
    | VTStr


type Value
    = VBool Bool
    | VInt Int
    | VStr String
    | VHole ValueType


valueType : Value -> ValueType
valueType v =
    case v of
        VBool _ ->
            VTBool

        VInt _ ->
            VTInt

        VStr _ ->
            VTStr

        VHole vt ->
            vt


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
        , args : Assoc String Value
        }


freshStep : String -> StepSignature -> Step
freshStep name sig =
    SConcrete
        { name = name
        , args = List.map (\( k, vt ) -> ( k, VHole vt )) sig.params
        }


type alias Library =
    Assoc String StepSignature


props : Library -> Library
props =
    List.filter (\( _, s ) -> s.kind == Prop)


types : Library -> Library
types =
    List.filter (\( _, s ) -> s.kind == Type)


type Workflow
    = W
        { steps : List Step
        , goal : Step
        }


emptyWorkflow : Workflow
emptyWorkflow =
    W { steps = [], goal = SHole }


type StepIndex
    = Goal
    | Step Int


steps : Workflow -> List Step
steps (W w) =
    w.steps


goal : Workflow -> Step
goal (W w) =
    w.goal


setStep : StepIndex -> Step -> Workflow -> Workflow
setStep si step w =
    modifyStep si (\_ -> step) w


modifyStep : StepIndex -> (Step -> Step) -> Workflow -> Workflow
modifyStep si modify (W w) =
    case si of
        Goal ->
            W { w | goal = modify w.goal }

        Step i ->
            W
                { w
                    | steps =
                        w.steps
                            |> List.indexedMap
                                (\j s ->
                                    if i == j then
                                        modify s

                                    else
                                        s
                                )
                }


insertStep : Int -> Step -> Workflow -> Workflow
insertStep i step (W w) =
    if i == List.length w.steps then
        W { w | steps = w.steps ++ [ step ] }

    else
        W
            { w
                | steps =
                    w.steps
                        |> List.indexedMap
                            (\j s ->
                                if i == j then
                                    [ step, s ]

                                else
                                    [ s ]
                            )
                        |> List.concat
            }


removeStep : Int -> Workflow -> Workflow
removeStep i (W w) =
    W
        { w
            | steps = w.steps |> Util.indexedFilter (\j _ -> i /= j)
        }
