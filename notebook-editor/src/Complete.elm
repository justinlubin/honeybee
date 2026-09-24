module Complete exposing (complete, completeProps)

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
    { allowPropHoles : Bool, allowGoalHoles : Bool }
    -> WorkingProgram
    -> Maybe CompleteProgram
complete { allowPropHoles, allowGoalHoles } prog =
    if List.isEmpty prog.props then
        Nothing

    else
        Maybe.map2 (\p g -> { props = p, goal = g })
            (prog.props
                |> List.map (Maybe.andThen (fact allowPropHoles))
                |> Util.sequence
            )
            (prog.goal
                |> Maybe.andThen (fact allowGoalHoles)
            )


completeProps : { allowPropHoles : Bool } -> WorkingProgram -> Maybe (List (Fact Value))
completeProps { allowPropHoles } prog =
    prog.props
        |> List.map (Maybe.andThen (fact allowPropHoles))
        |> Util.sequence



-- tryGoal : List (Maybe (Fact String)) -> String -> FactSignature -> Maybe CompleteProgram
-- tryGoal props goalName goalSig =
--     complete
--         { allowPropHoles = True, allowGoalHoles = True }
--         { props = props
--         , goal = Just (Core.fresh goalName goalSig)
--         }
