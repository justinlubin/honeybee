module View exposing (view)

import Assoc exposing (Assoc)
import Compile
import Core exposing (..)
import Html exposing (..)
import Html.Attributes as A
import Html.Events as E
import Model exposing (Model)
import Update exposing (Msg)


stringFromValue : Value -> String
stringFromValue v =
    case v of
        VBool True ->
            "True"

        VBool False ->
            "False"

        VInt n ->
            String.fromInt n

        VStr s ->
            "\"" ++ s ++ "\""

        VHole _ ->
            "?"


arg : StepIndex -> String -> Value -> Html Msg
arg si argName v =
    span []
        [ b [ A.class "argument-name" ] [ text (argName ++ ": ") ]
        , input
            [ E.onInput (Update.SetArgumentByString (valueType v) si argName)
            , A.class "argument-input"
            ]
            []
        , text <| " (" ++ stringFromValue v ++ ")"
        ]


args : StepIndex -> Assoc String Value -> List (Html Msg)
args si a =
    Assoc.mapCollapse (arg si) a


step : Library -> StepIndex -> Step -> Html Msg
step lib si s =
    let
        deleteButton =
            case si of
                Step i ->
                    button
                        [ A.class "delete-button"
                        , E.onClick (Update.RemoveStep i)
                        ]
                        [ text "X" ]

                Goal ->
                    text ""

        inputEvent =
            E.onInput <|
                \k ->
                    if k == "<blank>" then
                        Update.ClearStep si

                    else
                        Update.SetStep si k

        ( name, extras ) =
            case s of
                SHole ->
                    ( "<blank>", [] )

                SConcrete st ->
                    ( st.name, args si st.args )

        options =
            "<blank>" :: Assoc.mapCollapse (\k _ -> k) lib

        dropdown =
            select
                [ inputEvent ]
                (List.map
                    (\k -> option [ A.selected (k == name) ] [ text k ])
                    options
                )
    in
    div [] (deleteButton :: dropdown :: extras)


workflow : Library -> Workflow -> Html Msg
workflow lib w =
    div [ A.class "workflow" ]
        [ h2 [] [ text "Goal of Experiment" ]
        , step (types lib) Goal (goal w)
        , h2 [] [ text "Experimental Workflow" ]
        , button
            [ E.onClick Update.AddBlankStep ]
            [ text "Add step" ]
        , ol []
            (List.indexedMap
                (\i s -> li [] [ step (props lib) (Step i) s ])
                (steps w)
            )
        ]


view : Model -> Html Msg
view model =
    div
        []
        [ workflow model.library model.workflow
        , pre [] [ text <| Compile.compile model.workflow ]
        ]
