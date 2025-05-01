module View exposing (view)

import Assoc exposing (Assoc)
import Compile
import Config
import Core exposing (..)
import Html exposing (..)
import Html.Attributes as A
import Html.Events as E
import Model exposing (Model)
import Port
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


arg : StepIndex -> String -> ( Value, List Value ) -> Html Msg
arg si argName ( v, suggestions ) =
    let
        id =
            "step-"
                ++ (case si of
                        Goal ->
                            "GOAL"

                        Step i ->
                            String.fromInt i
                   )
                ++ "-argument-"
                ++ argName
    in
    span []
        [ b [ A.class "argument-name" ] [ text (argName ++ ": ") ]
        , input
            [ E.onInput (Update.SetArgumentByString (valueType v) si argName)
            , A.class "argument-input"
            , A.id id
            ]
            []
        , if List.isEmpty suggestions then
            text ""

          else
            span [] <|
                text " [Tip: Try "
                    :: List.intersperse
                        (text ", ")
                        (List.filterMap
                            (\sug ->
                                Maybe.map
                                    (\s ->
                                        button
                                            [ E.onClick
                                                (Update.SetArgumentTextField
                                                    { id = id
                                                    , text = s
                                                    }
                                                    si
                                                    argName
                                                    sug
                                                )
                                            ]
                                            [ text s ]
                                    )
                                    (Core.unparseValue sug)
                            )
                            suggestions
                        )
                    ++ [ text "]" ]
        , text <|
            if Config.debug then
                " (" ++ stringFromValue v ++ ")"

            else
                ""
        ]


args : StepIndex -> Assoc String ( Value, List Value ) -> List (Html Msg)
args si a =
    Assoc.mapCollapse (arg si) a


step :
    Library
    -> Assoc String (List Value)
    -> StepIndex
    -> Step
    -> Html Msg
step library goalSuggestions si s =
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
                    ( st.name
                    , args
                        si
                        (Assoc.leftMerge [] st.args goalSuggestions)
                    )

        options =
            "<blank>" :: Assoc.mapCollapse (\k _ -> k) library

        dropdown =
            select
                [ inputEvent ]
                (List.map
                    (\k -> option [ A.selected (k == name) ] [ text k ])
                    options
                )
    in
    div [] (deleteButton :: dropdown :: extras)


workflow :
    { m | library : Library, goalSuggestions : Assoc String (List Value) }
    -> Workflow
    -> Html Msg
workflow ctx w =
    div [ A.class "workflow" ]
        [ h2 [] [ text "Experimental Workflow" ]
        , button
            [ E.onClick Update.AddBlankStep ]
            [ text "Add step" ]
        , ol []
            (List.indexedMap
                (\i s -> li [] [ step (props ctx.library) ctx.goalSuggestions (Step i) s ])
                (steps w)
            )
        , h2 [] [ text "Goal of Experiment" ]
        , step (types ctx.library) ctx.goalSuggestions Goal (goal w)
        ]


pbnStatus : Maybe Port.PbnStatusMessage -> Html Msg
pbnStatus ms =
    case ms of
        Nothing ->
            text ""

        Just { workingExpression, choices, valid } ->
            div
                []
                [ h2 [] [ text "Python script to analyze this experiment" ]
                , code [] [ pre [] [ text workingExpression ] ]
                , if valid then
                    div []
                        [ h2 [] [ text "All done!" ]
                        , button
                            [ E.onClick
                                (Update.Download
                                    { filename = "analysis.py"
                                    , text = workingExpression
                                    }
                                )
                            ]
                            [ text "Download script" ]
                        ]

                  else
                    div []
                        [ h2 [] [ text "Possible next steps" ]
                        , ol []
                            (List.indexedMap
                                (\i ( h, f ) ->
                                    li []
                                        [ button [ E.onClick (Update.MakePbnChoice i) ]
                                            [ span [] [ text "?" ]
                                            , sub [] [ text (String.fromInt h) ]
                                            , span [] [ text " ↦ " ]
                                            , span [] [ text f ]
                                            ]
                                        ]
                                )
                                choices
                            )
                        ]
                ]


view : Model -> Html Msg
view model =
    div
        []
        [ workflow model model.workflow
        , button
            [ case Compile.compile { allowGoalHoles = False } model.workflow of
                Nothing ->
                    A.disabled True

                Just programSource ->
                    E.onClick <|
                        Update.StartNavigating
                            { programSource = programSource }
            ]
            [ text "Start navigating" ]
        , pbnStatus model.pbnStatus
        ]
