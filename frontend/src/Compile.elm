module Compile exposing (compile)

import Assoc exposing (Assoc)
import Core exposing (..)
import Util


value : Bool -> Value -> Maybe String
value fillHoles v =
    case v of
        VInt n ->
            Just (String.fromInt n)

        VBool True ->
            Just "true"

        VBool False ->
            Just "false"

        VStr s ->
            Just ("\"" ++ s ++ "\"")

        VHole vt ->
            if fillHoles then
                case vt of
                    VTInt ->
                        Just "0"

                    VTBool ->
                        Just "false"

                    VTStr ->
                        Just "\"\""

            else
                Nothing


arg : Bool -> String -> Value -> Maybe String
arg fillHoles argName v =
    v
        |> value fillHoles
        |> Maybe.map (\s -> "args." ++ argName ++ " = " ++ s)


stepBody : Bool -> String -> Assoc String Core.Value -> Maybe String
stepBody fillHoles name args =
    args
        |> Assoc.mapCollapse (arg fillHoles)
        |> Util.sequence
        |> Maybe.map
            (\argStrings ->
                "name = \""
                    ++ name
                    ++ "\"\n"
                    ++ String.join "\n" argStrings
            )


step : Bool -> String -> Core.Step -> Maybe String
step fillHoles prefix s =
    case s of
        Core.SHole ->
            Nothing

        Core.SConcrete { name, args } ->
            stepBody fillHoles name args
                |> Maybe.map (\body -> prefix ++ body)


prop : Core.Step -> Maybe String
prop s =
    step False "[[Prop]]\n" s


goal : Bool -> Core.Step -> Maybe String
goal fillHoles s =
    step fillHoles "[Goal]\n" s


compile :
    { allowGoalHoles : Bool }
    -> Core.Workflow
    -> Maybe String
compile { allowGoalHoles } w =
    Maybe.map (String.join "\n\n") <|
        Util.sequence <|
            List.map prop (Core.steps w)
                ++ [ goal allowGoalHoles (Core.goal w) ]
