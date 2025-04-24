module View exposing (view)

import Assoc
import Core exposing (..)
import Html exposing (..)
import Model exposing (Model)
import Update exposing (Msg)


type alias Context a =
    { a | library : Library }


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


arg : String -> Value -> Html Msg
arg argName v =
    text <| argName ++ ": " ++ stringFromValue v ++ ". "


step : Context a -> Step -> Html Msg
step ctx s =
    span [] <|
        b [] [ text <| s.name ++ ". " ]
            :: Assoc.mapCollapse arg s.args


goal : Context a -> Maybe Step -> Html Msg
goal ctx g =
    case g of
        Just s ->
            span []
                [ b [] [ text "Goal: " ]
                , text s.name
                ]

        Nothing ->
            text "No goal yet!"


workflow : Context a -> Workflow -> Html Msg
workflow ctx w =
    div []
        [ ol [] (List.map (\s -> li [] [ step ctx s ]) w.steps)
        , goal ctx w.goal
        , button [] [ text "Add step" ]
        ]


view : Model -> Html Msg
view model =
    workflow model model.workflow
