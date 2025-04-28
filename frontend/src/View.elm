module View exposing (view)

import Assoc exposing (Assoc)
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


arg : String -> Maybe Value -> Html Msg
arg argName mv =
    text <|
        argName
            ++ ": "
            ++ (case mv of
                    Nothing ->
                        "?"

                    Just v ->
                        stringFromValue v
               )
            ++ ". "


step : Library -> StepIndex -> Step -> Html Msg
step lib si s =
    let
        deleteButton =
            case si of
                Step i ->
                    span []
                        [ button [ E.onClick (Update.RemoveStep i) ]
                            [ text "X" ]
                        , text " "
                        ]

                Goal ->
                    text ""

        result =
            case s of
                SHole ->
                    [ select
                        [ E.onInput
                            (\k ->
                                if k == "<blank>" then
                                    Update.ClearStep si

                                else
                                    Update.SetStep si k
                            )
                        ]
                      <|
                        option [ A.selected True ] [ text "<blank>" ]
                            :: Assoc.mapCollapse
                                (\k _ -> option [] [ text k ])
                                lib
                    ]

                SConcrete { name, args } ->
                    [ select
                        [ E.onInput
                            (\k ->
                                if k == "<blank>" then
                                    Update.ClearStep si

                                else
                                    Update.SetStep si k
                            )
                        ]
                      <|
                        option [] [ text "<blank>" ]
                            :: Assoc.mapCollapse
                                (\k _ ->
                                    option [ A.selected (k == name) ] [ text k ]
                                )
                                lib
                    ]
                        ++ Assoc.mapCollapse arg args
    in
    div [] (deleteButton :: result)


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
    workflow model.library model.workflow
