module View exposing (view)

import Core exposing (..)
import Html exposing (..)
import Model exposing (Model)
import OrderedDict as OD
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


arg : String -> Value -> Html Msg
arg argName v =
    text <| argName ++ ": " ++ stringFromValue v ++ ". "


step : Step -> Html Msg
step s =
    span [] <|
        b [] [ text <| s.name ++ ". " ]
            :: (OD.map arg s.args |> OD.values)


goal : Maybe Step -> Html Msg
goal g =
    case g of
        Just s ->
            span []
                [ b [] [ text "Goal: " ]
                , text s.name
                ]

        Nothing ->
            text "No goal yet!"


workflow : Workflow -> Html Msg
workflow w =
    div []
        [ ol [] (List.map (\s -> li [] [ step s ]) w.steps)
        , goal w.goal
        ]


view : Model -> Html Msg
view model =
    workflow model.workflow
