module Complete exposing (complete)

import Assoc
import Core exposing (..)
import Util


fillHole : ValueType -> Value
fillHole vt =
    case vt of
        VTInt ->
            VInt 0

        VTBool ->
            VBool False

        VTStr ->
            VStr ""


value : Bool -> ValueType -> String -> Maybe Value
value allowHoles vt str =
    case Core.parse vt str of
        ParseSuccess v ->
            Just v

        _ ->
            if allowHoles then
                Just (fillHole vt)

            else
                Nothing


fact : Bool -> Fact String -> Maybe (Fact Value)
fact allowHoles f =
    Maybe.map (\args -> { name = f.name, args = args, sig = f.sig })
        (f.args
            |> Assoc.map
                (\_ ( a, vt ) ->
                    Maybe.map (\v -> ( v, vt )) (value allowHoles vt a)
                )
            |> Assoc.sequence
        )


complete :
    { allowGoalHoles : Bool }
    -> WorkingProgram
    -> Maybe CompleteProgram
complete { allowGoalHoles } prog =
    Maybe.map2 (\p g -> { props = p, goal = g })
        (prog.props
            |> List.map (Maybe.andThen (fact False))
            |> Util.sequence
        )
        (prog.goal
            |> Maybe.andThen (fact allowGoalHoles)
        )
