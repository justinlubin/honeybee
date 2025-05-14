module Core exposing
    ( Library
    , Step(..)
    , StepIndex(..)
    , StepKind(..)
    , StepSignature
    , Value(..)
    , ValueType(..)
    , Workflow
    , argsConsistent
    , consistent
    , emptyWorkflow
    , exampleWorkflow
    , freshStep
    , goal
    , insertStep
    , modifyStep
    , props
    , removeStep
    , setStep
    , steps
    , types
    , unparseValue
    , valueType
    )

import Assoc exposing (Assoc)
import Dict exposing (Dict)
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


consistent : Value -> Value -> Bool
consistent v1 v2 =
    case ( v1, v2 ) of
        ( VInt n1, VInt n2 ) ->
            n1 == n2

        ( VBool b1, VBool b2 ) ->
            b1 == b2

        ( VStr s1, VStr s2 ) ->
            s1 == s2

        ( VHole vt1, _ ) ->
            vt1 == valueType v2

        ( _, VHole vt2 ) ->
            valueType v1 == vt2

        _ ->
            False


unparseValue : Value -> Maybe String
unparseValue v =
    case v of
        VBool True ->
            Just "true"

        VBool False ->
            Just "false"

        VInt n ->
            Just (String.fromInt n)

        VStr s ->
            Just s

        VHole _ ->
            Nothing


type StepKind
    = Prop
    | Type


type alias StepSignature =
    { params : Assoc String ValueType
    , kind : StepKind
    , paramLabels : Dict String String
    , overview : Maybe String
    }


type alias SConcreteData =
    { name : String
    , args : Assoc String Value
    , argLabels : Dict String String
    , overview : Maybe String
    }


type Step
    = SHole
    | SConcrete SConcreteData


freshStep : String -> StepSignature -> Step
freshStep name sig =
    SConcrete
        { name = name
        , args = List.map (\( k, vt ) -> ( k, VHole vt )) sig.params
        , argLabels = sig.paramLabels
        , overview = sig.overview
        }


argsConsistent : Assoc String Value -> Assoc String Value -> Bool
argsConsistent args1 args2 =
    if List.length args1 /= List.length args2 then
        False

    else
        List.all
            (\( k1, v1 ) ->
                case Assoc.get k1 args2 of
                    Nothing ->
                        False

                    Just v2 ->
                        consistent v1 v2
            )
            args1


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


exampleWorkflow : Workflow
exampleWorkflow =
    W
        { steps =
            [ SConcrete
                { name = "RNAseq"
                , args =
                    [ ( "sample", VStr "1" )
                    , ( "path", VStr "raw_data/rnaseq/control/" )
                    ]
                , argLabels = Dict.empty
                , overview = Nothing
                }
            , SConcrete
                { name = "RNAseq"
                , args =
                    [ ( "sample", VStr "2" )
                    , ( "path", VStr "raw_data/rnaseq/experimental/" )
                    ]
                , argLabels = Dict.empty
                , overview = Nothing
                }
            ]
        , goal =
            SConcrete
                { name = "DifferentialGeneExpression"
                , args =
                    [ ( "sample1", VStr "1" )
                    , ( "sample2", VStr "2" )
                    ]
                , argLabels = Dict.empty
                , overview = Nothing
                }
        }


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
