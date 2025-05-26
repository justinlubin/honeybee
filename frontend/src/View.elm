module View exposing (view)

import Assoc exposing (Assoc)
import Compile
import Complete
import Core exposing (..)
import Dict exposing (Dict)
import Html exposing (..)
import Html.Attributes as A
import Html.Events as E
import Json.Encode
import Model exposing (Model)
import Port
import Update exposing (Msg)
import Util


arg :
    ProgramIndex
    -> Dict String String
    -> String
    -> ( ( String, ValueType ), List Value )
    -> Html Msg
arg pi argLabels argName ( ( valueStr, _ ), suggestions ) =
    let
        id =
            "step-argument"
                ++ (case pi of
                        Goal ->
                            "GOAL"

                        Prop i ->
                            String.fromInt i
                   )
                ++ argName
    in
    div
        [ A.class "step-arg"
        ]
        [ label
            [ A.for id
            ]
            [ argLabels
                |> Dict.get argName
                |> Maybe.withDefault argName
                |> text
            ]
        , input
            [ E.onInput (Update.SetArgumentByString pi argName)
            , A.id id
            , A.placeholder "Enter value here…"
            , A.value valueStr
            ]
            []
        , if List.isEmpty suggestions then
            text ""

          else
            div [ A.class "suggestion-tip" ] <|
                text "Try one of the following: "
                    :: List.intersperse
                        (text ", ")
                        (List.map
                            (\sug ->
                                let
                                    s =
                                        Core.unparse sug
                                in
                                button
                                    [ E.onClick
                                        (Update.SetArgumentByString
                                            pi
                                            argName
                                            s
                                        )
                                    ]
                                    [ text s ]
                            )
                            suggestions
                        )
        ]


args :
    ProgramIndex
    -> Dict String String
    -> Assoc String ( ( String, ValueType ), List Value )
    -> List (Html Msg)
args pi argLabels a =
    Assoc.mapCollapse (arg pi argLabels) a


step :
    FactLibrary
    -> Assoc String (List Value)
    -> ProgramIndex
    -> Maybe (Fact String)
    -> Html Msg
step library suggestions pi s =
    let
        blankName =
            "Choose a step…"

        deleteButton =
            case pi of
                Prop i ->
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
                        Update.ClearStep pi

                    else
                        Update.SetStep pi k

        ( selectedName, extras ) =
            case s of
                Nothing ->
                    ( blankName, [] )

                Just f ->
                    ( f.name
                    , args
                        pi
                        f.sig.paramLabels
                        (Assoc.leftMergeWith [] f.args suggestions)
                    )

        options =
            ( blankName, blankName )
                :: Assoc.mapCollapse
                    (\k sig -> ( k, Maybe.withDefault k sig.overview ))
                    library

        dropdown =
            select
                [ A.class "step-title"
                , inputEvent
                ]
                (List.map
                    (\( name, displayName ) ->
                        option
                            [ A.selected (name == selectedName)
                            , A.value name
                            ]
                            [ text displayName ]
                    )
                    options
                )
    in
    div
        [ A.class "step" ]
        (dropdown :: deleteButton :: extras)


program :
    { m | library : Library, goalSuggestions : Assoc String (List Value) }
    -> WorkingProgram
    -> Html Msg
program ctx prog =
    div [ A.class "workflow" ]
        [ h3 [] [ text "Experimental workflow" ]
        , ol [ A.class "steps" ]
            (List.indexedMap
                (\i s -> li [] [ step ctx.library.props [] (Prop i) s ])
                prog.props
            )
        , button
            [ A.class "step-add"
            , E.onClick Update.AddBlankStep
            ]
            [ text "Add step" ]
        , h3 [] [ text "Goal of experiment" ]
        , step ctx.library.types ctx.goalSuggestions Goal prog.goal
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
                                        |> Util.unSubscriptNumbers
                                        |> String.toInt
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
                        [ node "fancy-code"
                            [ A.attribute "language" "python"
                            , A.property "code" (Json.Encode.string line)
                            ]
                            []
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


startNavigation : WorkingProgram -> Html Msg
startNavigation prog =
    let
        ( attrs, extras ) =
            case
                prog
                    |> Complete.complete { allowGoalHoles = False }
                    |> Maybe.map Compile.compile
            of
                Nothing ->
                    ( [ A.disabled True ]
                    , [ div [ A.class "subtitle" ]
                            [ text "(Complete experimental workflow first)" ]
                      ]
                    )

                Just programSource ->
                    ( [ E.onClick <|
                            Update.StartNavigating { programSource = programSource }
                      ]
                    , []
                    )
    in
    button
        ([ A.class "start-navigation", A.class "standout-button" ] ++ attrs)
        (text "Start navigating" :: extras)


pbnStatus : Maybe Port.PbnStatusMessage -> Html Msg
pbnStatus ms =
    case ms of
        Nothing ->
            text ""

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
                            [ text "Download analysis script" ]
                        ]

                  else
                    text ""
                ]


view : Model -> Html Msg
view model =
    main_
        []
        [ header []
            [ h1 []
                [ span [ A.class "pbn" ] [ text "Programming by Navigation" ]
                , text " with "
                , span [ A.class "honeybee" ] [ text "Honeybee" ]
                ]
            , p [] [ text "Honeybee is a tool you can use to write code to analyze experimental data." ]
            , p [] [ text "It works in two steps:" ]
            , ol []
                [ li [] [ text "First, you write down your experimental workflow." ]
                , li [] [ text "Then, Honeybee helps you navigate among all possible programs to analyze the experiment you wrote down." ]
                ]
            , p [] [ text "Using your biology expertise, you can navigate to the program that fits your need!" ]
            ]
        , div [ A.class "specification-pane" ]
            [ h2 []
                [ span [] [ text "Step 1: " ]
                , span [] [ text "Write down your experimental workflow" ]
                ]
            , program model model.program
            , startNavigation model.program
            ]
        , div [ A.class "navigation-pane" ]
            [ h2
                [ A.class <|
                    if model.pbnStatus == Nothing then
                        "inactive-pane-header"

                    else
                        "active-pane-header"
                ]
                [ span [] [ text "Step 2: " ]
                , span [] [ text "Create an analysis script for this experiment" ]
                ]
            , pbnStatus model.pbnStatus
            ]
        , button
            [ A.id "devmode"
            , E.onClick Update.LoadExample
            ]
            [ text "devmode" ]
        ]
