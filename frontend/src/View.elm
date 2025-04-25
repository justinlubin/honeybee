module View exposing (view)

import Assoc exposing (Assoc)
import Core exposing (..)
import Html exposing (..)
import Html.Attributes as A
import Html.Events as E
import Json.Decode
import Model exposing (Model)
import Update exposing (Msg)



-- onChange : msg -> Attribute msg
-- onChange m =
--     E.on "change" (Json.Decode.succeed m)


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


step : Library -> Maybe Int -> Step -> Html Msg
step lib mi s =
    case s of
        SHole ->
            select
                [ E.onInput
                    (\k ->
                        if k == "<blank>" then
                            Update.ClearStep mi

                        else
                            Update.SetStep mi k
                    )
                ]
            <|
                option [ A.selected True ] [ text "<blank>" ]
                    :: Assoc.mapCollapse
                        (\k _ -> option [] [ text k ])
                        lib

        SConcrete { name, args } ->
            span [] <|
                (select
                    [ E.onInput
                        (\k ->
                            if k == "<blank>" then
                                Update.ClearStep mi

                            else
                                Update.SetStep mi k
                        )
                    ]
                 <|
                    option [] [ text "<blank>" ]
                        :: Assoc.mapCollapse
                            (\k _ ->
                                option [ A.selected (k == name) ] [ text k ]
                            )
                            lib
                )
                    :: Assoc.mapCollapse arg args


goal : Library -> Step -> Html Msg
goal lib g =
    div []
        [ b [] [ text "Goal: " ]
        , step (types lib) Nothing g
        ]


workflow : Library -> Workflow -> Html Msg
workflow lib w =
    div []
        [ goal lib w.goal
        , button
            [ E.onClick Update.AddBlankStep
            ]
            [ text "Add step"
            ]
        , ol []
            (List.indexedMap
                (\i s ->
                    li [] [ step (props lib) (Just i) s ]
                )
                w.steps
            )
        ]


view : Model -> Html Msg
view model =
    workflow model.library model.workflow
