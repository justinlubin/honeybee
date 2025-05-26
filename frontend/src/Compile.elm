module Compile exposing (compile)

import Assoc exposing (Assoc)
import Core exposing (..)


value : Value -> String
value v =
    case v of
        VInt n ->
            String.fromInt n

        VBool True ->
            "true"

        VBool False ->
            "false"

        VStr s ->
            "\"" ++ s ++ "\""


arg : String -> ( Value, ValueType ) -> String
arg argName ( v, _ ) =
    "args." ++ argName ++ " = " ++ value v


factBody : String -> Assoc String ( Value, ValueType ) -> String
factBody name args =
    "name = \""
        ++ name
        ++ "\"\n"
        ++ String.join "\n" (Assoc.mapCollapse arg args)


fact : String -> Fact Value -> String
fact prefix f =
    prefix ++ factBody f.name f.args


prop : Fact Value -> String
prop f =
    fact "[[Prop]]\n" f


goal : Fact Value -> String
goal f =
    fact "[Goal]\n" f


compile : CompleteProgram -> String
compile prog =
    String.join "\n\n"
        (List.map prop prog.props ++ [ goal prog.goal ])
