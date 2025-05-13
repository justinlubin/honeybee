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
import Util


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
    div
        [ A.class "step-arg"
        ]
        [ label
            [ A.for id
            ]
            [ text <|
                argName
                    ++ (if False && Config.debug then
                            " (" ++ stringFromValue v ++ ")"

                        else
                            ""
                       )
            ]
        , input
            [ E.onInput (Update.SetArgumentByString (valueType v) si argName)
            , A.id id
            , A.placeholder "Enter value here…"
            ]
            []
        , if List.isEmpty suggestions then
            text ""

          else
            div [ A.class "suggestion-tip" ] <|
                text "Try one of the following: "
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
        blankName =
            "Choose a step…"

        deleteButton =
            case si of
                Step i ->
                    button
                        [ A.class "step-delete"
                        , E.onClick (Update.RemoveStep i)
                        ]
                        [ text "×" ]

                Goal ->
                    text ""

        inputEvent =
            E.onInput <|
                \k ->
                    if k == blankName then
                        Update.ClearStep si

                    else
                        Update.SetStep si k

        ( name, extras ) =
            case s of
                SHole ->
                    ( blankName, [] )

                SConcrete st ->
                    ( st.name
                    , args
                        si
                        (Assoc.leftMerge [] st.args goalSuggestions)
                    )

        options =
            blankName :: Assoc.mapCollapse (\k _ -> k) library

        dropdown =
            select
                [ A.class "step-title"
                , inputEvent
                ]
                (List.map
                    (\k -> option [ A.selected (k == name) ] [ text k ])
                    options
                )
    in
    div
        [ A.class "step" ]
        (deleteButton :: dropdown :: extras)


workflow :
    { m | library : Library, goalSuggestions : Assoc String (List Value) }
    -> Workflow
    -> Html Msg
workflow ctx w =
    div [ A.class "workflow" ]
        [ h3 [] [ text "Experimental workflow" ]
        , ol [ A.class "steps" ]
            (List.indexedMap
                (\i s -> li [] [ step (props ctx.library) ctx.goalSuggestions (Step i) s ])
                (steps w)
            )
        , button
            [ A.class "step-add"
            , E.onClick Update.AddBlankStep
            ]
            [ text "Add step" ]
        , h3 [] [ text "Goal of experiment" ]
        , step (types ctx.library) ctx.goalSuggestions Goal (goal w)
        ]


directManipulationPbn : Port.PbnStatusMessage -> Html Msg
directManipulationPbn { workingExpression, choices } =
    let
        collectedChoices =
            choices
                |> List.indexedMap (\i ( h, f ) -> ( h, ( f, i ) ))
                |> Assoc.collect

        codeLines =
            workingExpression
                |> String.lines
                |> List.map
                    (\line ->
                        case String.split "?" line of
                            [ left, right ] ->
                                case
                                    right
                                        |> String.split ","
                                        |> List.head
                                        |> Maybe.map Util.unSubscriptNumbers
                                        |> Maybe.andThen String.toInt
                                of
                                    Just h ->
                                        ( left, Just h )

                                    Nothing ->
                                        ( line, Nothing )

                            _ ->
                                ( line, Nothing )
                    )

        impossible =
            List.isEmpty collectedChoices
                && List.all (\( line, _ ) -> String.isEmpty line) codeLines
    in
    if impossible then
        div [ A.class "pbn-impossible" ]
            [ p [] [ text "Honeybee can't figure out how to make analysis script for this experiment." ]
            , p [] [ text "There might be missing steps (or typos) in your experiment or the Honeybee library might not include the computational steps you need." ]
            ]

    else
        div [ A.class "direct-manipulation-pbn" ] <|
            List.map
                (\( line, maybeHole ) ->
                    div
                        [ A.class "code-line" ]
                        [ code [] [ pre [] [ text line ] ]
                        , case maybeHole of
                            Just h ->
                                case Assoc.get h collectedChoices of
                                    Just hChoices ->
                                        select
                                            [ A.class "h-choices"
                                            , E.onInput <|
                                                \s ->
                                                    case String.toInt s of
                                                        Just i ->
                                                            Update.MakePbnChoice i

                                                        Nothing ->
                                                            Update.Nop
                                            ]
                                            (option
                                                [ A.value "" ]
                                                [ text "Choose a step…" ]
                                                :: List.map
                                                    (\( f, i ) ->
                                                        option
                                                            [ A.value (String.fromInt i) ]
                                                            [ text f ]
                                                    )
                                                    hChoices
                                            )

                                    Nothing ->
                                        text ""

                            Nothing ->
                                text ""
                        ]
                )
                codeLines


startNavigation : Workflow -> Html Msg
startNavigation w =
    button
        [ A.class "start-navigation"
        , A.class "standout-button"
        , case Compile.compile { allowGoalHoles = False } w of
            Nothing ->
                A.disabled True

            Just programSource ->
                E.onClick <|
                    Update.StartNavigating { programSource = programSource }
        ]
        [ text "Start navigating"
        ]


pbnStatus : Maybe Port.PbnStatusMessage -> Html Msg
pbnStatus ms =
    case ms of
        Nothing ->
            div
                [ A.class "pbn-inactive" ]
                [ div []
                    [ p [] [ text "Please complete your experimental workflow." ]
                    , p [] [ text "Then, click the \"Start navigating\" button." ]
                    ]
                ]

        Just msg ->
            div
                [ A.class "pbn" ]
                [ directManipulationPbn msg
                , if msg.valid then
                    div [ A.class "pbn-completed" ]
                        [ h3 [] [ text "All done!" ]
                        , button
                            [ A.class "standout-button"
                            , E.onClick
                                (Update.Download
                                    { filename = "analysis.py"
                                    , text = msg.workingExpression
                                    }
                                )
                            ]
                            [ text "Download script" ]
                        ]

                  else
                    text ""
                ]


view : Model -> Html Msg
view model =
    main_
        []
        [ div [ A.class "specification-pane" ]
            [ h2 []
                [ span [] [ text "Step 1: " ]
                , span [] [ text "Write down your experimental workflow" ]
                ]
            , workflow model model.workflow
            , startNavigation model.workflow
            ]
        , div [ A.class "navigation-pane" ]
            [ h2
                [ A.class <|
                    if model.pbnStatus == Nothing then
                        "inactive-pane-header"

                    else
                        ""
                ]
                [ span [] [ text "Step 2: " ]
                , span [] [ text "Create an analysis script for this experiment" ]
                ]
            , pbnStatus model.pbnStatus

            -- , pbnInactiveOverlay model.pbnStatus
            ]
        ]
