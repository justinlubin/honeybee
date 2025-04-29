module Compile exposing (compile)

import Assoc exposing (Assoc)
import Core


value : Core.Value -> String
value v =
    case v of
        Core.VInt n ->
            String.fromInt n

        Core.VBool True ->
            "true"

        Core.VBool False ->
            "false"

        Core.VStr s ->
            "\"" ++ s ++ "\""

        Core.VHole _ ->
            "?"


arg : String -> Core.Value -> String
arg argName v =
    "args." ++ argName ++ " = " ++ value v


stepBody : String -> Assoc String Core.Value -> String
stepBody name args =
    "name = \""
        ++ name
        ++ "\"\n"
        ++ String.join "\n" (Assoc.mapCollapse arg args)


step : String -> Core.Step -> String
step prefix s =
    let
        body =
            case s of
                Core.SHole ->
                    "?"

                Core.SConcrete { name, args } ->
                    stepBody name args
    in
    prefix ++ body


prop : Core.Step -> String
prop s =
    step "[[Prop]]\n" s


goal : Core.Step -> String
goal s =
    step "[Goal]\n" s


compile : Core.Workflow -> String
compile w =
    String.join "\n\n" <|
        List.map prop (Core.steps w)
            ++ [ goal (Core.goal w) ]
