module Core exposing
    ( CompleteProgram
    , Fact
    , FactLibrary
    , FactSignature
    , Library
    , Program
    , ProgramIndex(..)
    , Value(..)
    , ValueParseResult(..)
    , ValueType(..)
    , WorkingProgram
    , consistent
    , empty
    , example
    , fresh
    , getSigFor
    , insert
    , modify
    , parse
    , remove
    , set
    , unparse
    , valueType
    )

import Assoc exposing (Assoc)
import Dict exposing (Dict)
import Util



--------------------------------------------------------------------------------
-- Values


type ValueType
    = VTBool
    | VTInt
    | VTStr


type Value
    = VBool Bool
    | VInt Int
    | VStr String


valueType : Value -> ValueType
valueType v =
    case v of
        VBool _ ->
            VTBool

        VInt _ ->
            VTInt

        VStr _ ->
            VTStr


unparse : Value -> String
unparse v =
    case v of
        VBool True ->
            "true"

        VBool False ->
            "false"

        VInt n ->
            String.fromInt n

        VStr s ->
            s


type ValueParseResult
    = ParseFail
    | Blank
    | ParseSuccess Value


parse : ValueType -> String -> ValueParseResult
parse vt str =
    if String.isEmpty (String.trim str) then
        Blank

    else
        case vt of
            VTInt ->
                str
                    |> String.toInt
                    |> Maybe.map (VInt >> ParseSuccess)
                    |> Maybe.withDefault ParseFail

            VTBool ->
                case String.toLower str of
                    "true" ->
                        ParseSuccess (VBool True)

                    "false" ->
                        ParseSuccess (VBool False)

                    _ ->
                        ParseFail

            VTStr ->
                ParseSuccess (VStr str)



--------------------------------------------------------------------------------
-- Facts


type alias FactSignature =
    { params : Assoc String ValueType
    , paramLabels : Dict String String
    , title : Maybe String
    }


type alias FactLibrary =
    Assoc String FactSignature


type alias Library =
    { props : FactLibrary, types : FactLibrary }


type alias Fact v =
    { name : String
    , args : Assoc String ( v, ValueType )
    , sig : FactSignature
    }


fresh : String -> FactSignature -> Fact String
fresh name sig =
    { name = name
    , args = Assoc.map (\_ vt -> ( "", vt )) sig.params
    , sig = sig
    }


consistent :
    Fact String
    -> Assoc String Value
    -> Bool
consistent fact args =
    Assoc.all
        (\k ( s1, vt ) ->
            case Assoc.get k args of
                Just v2 ->
                    case parse vt s1 of
                        ParseFail ->
                            False

                        Blank ->
                            True

                        ParseSuccess v1 ->
                            v1 == v2

                Nothing ->
                    False
        )
        fact.args



--------------------------------------------------------------------------------
-- Programs


type alias Program v =
    { props : List v
    , goal : v
    }


type alias WorkingProgram =
    Program (Maybe (Fact String))


type alias CompleteProgram =
    Program (Fact Value)


type ProgramIndex
    = Goal
    | Prop Int


getSigFor : ProgramIndex -> String -> Library -> Maybe FactSignature
getSigFor pi name lib =
    let
        relevantLibrary =
            case pi of
                Goal ->
                    lib.types

                Prop _ ->
                    lib.props
    in
    Assoc.get name relevantLibrary


set : ProgramIndex -> v -> Program v -> Program v
set pi x prog =
    modify pi (\_ -> x) prog


modify : ProgramIndex -> (v -> v) -> Program v -> Program v
modify pi func prog =
    case pi of
        Goal ->
            { prog | goal = func prog.goal }

        Prop i ->
            let
                newProps =
                    prog.props
                        |> List.indexedMap
                            (\j x ->
                                if i == j then
                                    func x

                                else
                                    x
                            )
            in
            { prog | props = newProps }


insert : Int -> v -> Program v -> Program v
insert i x prog =
    if i == List.length prog.props then
        { prog | props = prog.props ++ [ x ] }

    else
        let
            newProps =
                prog.props
                    |> List.indexedMap
                        (\j y ->
                            if i == j then
                                [ x, y ]

                            else
                                [ y ]
                        )
                    |> List.concat
        in
        { prog | props = newProps }


remove : Int -> Program v -> Program v
remove i prog =
    let
        newProps =
            prog.props
                |> Util.indexedFilter (\j _ -> i /= j)
    in
    { prog | props = newProps }


empty : Program (Maybe (Fact v))
empty =
    { props = [], goal = Nothing }


example : Library -> WorkingProgram
example library =
    case ( Assoc.get "P_LocalRnaSeq" library.props, Assoc.get "DifferentialGeneExpression" library.types ) of
        ( Just propSig, Just typeSig ) ->
            { props =
                [ Just
                    { name = "P_SraRnaSeq"
                    , args =
                        [ ( "label", ( "main", VTStr ) )
                        , ( "sample_sheet", ( "metadata/samples.csv", VTStr ) )
                        ]
                    , sig =
                        propSig
                    }
                ]
            , goal =
                Just
                    { name = "DifferentialGeneExpression"
                    , args =
                        [ ( "label", ( "main", VTStr ) )
                        , ( "comparison_sheet", ( "metadata/comparisons.csv", VTStr ) )
                        ]
                    , sig =
                        typeSig
                    }
            }

        _ ->
            { props = []
            , goal = Nothing
            }
