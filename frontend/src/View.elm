module View exposing (view)

import Assoc exposing (Assoc)
import Cell
import Compile
import Complete
import Core exposing (..)
import Dict exposing (Dict)
import Html exposing (..)
import Html.Attributes as A
import Html.Events as E
import Incoming
import Json.Encode
import Model exposing (Model)
import Update exposing (Msg(..))



--------------------------------------------------------------------------------
-- Generic


circled : Attribute msg
circled =
    A.class "circled"


menuBar : List (Attribute msg) -> List (Html msg) -> List (Html msg) -> List (Html msg) -> Html msg
menuBar attrs left middle right =
    div
        (A.class "menu-bar" :: attrs)
        [ div [ A.class "menu-bar-left" ] left
        , div [ A.class "menu-bar-middle" ] middle
        , div [ A.class "menu-bar-right" ] right
        ]


pane : List (Attribute msg) -> Html msg -> List (Html msg) -> Html msg
pane attrs headerContent bodyContent =
    section
        (A.class "pane" :: attrs)
        [ header [ A.class "pane-header" ] [ headerContent ]
        , div [ A.class "pane-body" ] bodyContent
        ]


paneHeading : List (Attribute msg) -> List (Html msg) -> Html msg
paneHeading attrs content =
    h1 (A.class "pane-heading" :: attrs) content


group : List (Attribute msg) -> Html msg -> List (Html msg) -> Html msg
group attrs headerContent bodyContent =
    section
        (A.class "group" :: attrs)
        [ header [ A.class "group-header" ] [ headerContent ]
        , div [ A.class "group-body" ] bodyContent
        ]


groupHeading : List (Attribute msg) -> List (Html msg) -> Html msg
groupHeading attrs content =
    h2 (A.class "group-heading" :: attrs) content


card : List (Attribute msg) -> Html msg -> List (Html msg) -> Html msg
card attrs headerContent bodyContent =
    section
        (A.class "card" :: attrs)
        [ header [ A.class "card-header" ] [ headerContent ]
        , div [ A.class "card-body" ] bodyContent
        ]


cardHeading :
    List (Attribute msg)
    -> List (Html msg)
    -> List (Html msg)
    -> List (Html msg)
    -> Html msg
cardHeading attrs prefix content suffix =
    div
        [ A.class "card-heading-wrapper" ]
        [ span [ A.class "card-heading-prefix" ] prefix
        , span [ A.class "card-heading-prefix-separator" ] []
        , h3 [ A.class "card-heading" ] content
        , span [ A.class "card-heading-suffix" ] suffix
        ]


cardHeadingSubtitle : List (Attribute msg) -> List (Html msg) -> Html msg
cardHeadingSubtitle attrs content =
    span [ A.class "card-heading-subtitle" ] content


fancyCode : List (Attribute msg) -> { language : String, code : String } -> Html msg
fancyCode attrs { language, code } =
    node "fancy-code"
        ([ A.attribute "language" language
         , A.property "code" (Json.Encode.string code)
         ]
            ++ attrs
        )
        []



--------------------------------------------------------------------------------
-- Program construction


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
            [ E.onInput (UserSetArgument pi argName)
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
                                    [ E.onClick (UserSetArgument pi argName s) ]
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

        ( prefix, class, deleteButton ) =
            case pi of
                Prop i ->
                    ( "Step"
                    , "step"
                    , button
                        [ A.class "step-delete"
                        , E.onClick (UserRemovedStep i)
                        ]
                        [ text "×"
                        ]
                    )

                Goal ->
                    ( "Goal", "goal", text "" )

        inputEvent =
            E.onInput <|
                \k ->
                    if k == blankName then
                        UserClearedStep pi

                    else
                        UserSetStep pi k

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
                    (\k sig -> ( k, Maybe.withDefault k sig.title ))
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
    card
        [ A.class class ]
        (cardHeading [] [ text prefix ] [ dropdown ] [ deleteButton ])
        extras


program :
    { m | library : Library, goalSuggestions : Assoc String (List Value) }
    -> WorkingProgram
    -> List (Html Msg)
program ctx prog =
    [ group [] (groupHeading [] [ text "Experimental workflow" ]) <|
        List.indexedMap
            (\i s -> step ctx.library.props [] (Prop i) s)
            prog.props
            ++ [ button
                    [ A.class "step-add"
                    , E.onClick UserAddedBlankStep
                    ]
                    [ text "Add step" ]
               ]
    , group []
        (groupHeading [] [ text "Goal of experiment" ])
        [ step ctx.library.types ctx.goalSuggestions Goal prog.goal
        ]
    ]



--------------------------------------------------------------------------------
-- Direct manipulation Programming by Navigation


functionChoices :
    { cellIndex : Int, selectedFunctionChoice : Maybe Int }
    -> List Cell.FunctionChoice
    -> Html Msg
functionChoices ctx fcs =
    div
        []
        (List.map (\x -> button [] [ text x.functionTitle ]) fcs)


cell : { cellIndex : Int } -> Cell.Cell -> Html Msg
cell ctx c =
    case c of
        Cell.Code { title, code } ->
            card
                [ A.class "cell-code" ]
                (cardHeading []
                    [ text "Code" ]
                    (case title of
                        Just t ->
                            [ text t ]

                        Nothing ->
                            []
                    )
                    []
                )
                [ fancyCode [] { language = "python", code = code }
                ]

        Cell.Choice x ->
            card
                [ A.class "cell-choice" ]
                (cardHeading []
                    [ span []
                        [ text "Choice"
                        , cardHeadingSubtitle [] [ text x.varName ]
                        ]
                    ]
                    [ text x.typeTitle
                    ]
                    []
                )
                [ case x.typeDescription of
                    Nothing ->
                        text ""

                    Just t ->
                        p [] [ text t ]
                , functionChoices
                    { cellIndex = ctx.cellIndex
                    , selectedFunctionChoice = x.selectedFunctionChoice
                    }
                    x.functionChoices
                ]


directManipulationPbn : List Cell.Cell -> List (Html Msg)
directManipulationPbn cells =
    List.indexedMap
        (\i c -> cell { cellIndex = i } c)
        cells



--------------------------------------------------------------------------------
-- Glue


startNavigationButton : WorkingProgram -> Html Msg
startNavigationButton prog =
    let
        ( attrs, extras ) =
            case
                prog
                    |> Complete.complete { allowGoalHoles = False }
                    |> Maybe.map Compile.compile
            of
                Nothing ->
                    ( [ A.disabled True ]
                    , [ div
                            [ A.class "subtitle" ]
                            [ text "(Complete experimental workflow first)" ]
                      ]
                    )

                Just programSource ->
                    ( [ E.onClick <|
                            UserStartedNavigation { programSource = programSource }
                      ]
                    , []
                    )
    in
    button
        ([ A.class "standout-button" ] ++ attrs)
        (text "Start navigating" :: extras)


pbnStatus : Maybe Incoming.PbnStatusMessage -> Html Msg
pbnStatus ms =
    case ms of
        Nothing ->
            text ""

        Just { cells, output } ->
            let
                ( impossible, downloadButton ) =
                    case output of
                        Nothing ->
                            ( List.all
                                (\c ->
                                    case c of
                                        Cell.Code _ ->
                                            True

                                        Cell.Choice x ->
                                            List.isEmpty x.functionChoices
                                )
                                cells
                            , text ""
                            )

                        Just solutionString ->
                            ( False
                            , div [ A.class "pbn-completed" ]
                                [ h3 [] [ text "All done!" ]
                                , button
                                    [ A.class "standout-button"
                                    , E.onClick
                                        (UserRequestedDownload
                                            { filename = "analysis.py"
                                            , text = solutionString
                                            }
                                        )
                                    ]
                                    [ text "Download analysis script" ]
                                ]
                            )
            in
            div [ A.class "pbn" ] <|
                (if impossible then
                    [ div [ A.class "pbn-impossible" ]
                        [ p [] [ text "Honeybee can't figure out how to make analysis script for this experiment." ]
                        , p [] [ text "There might be missing steps (or typos) in your experiment. Alternatively, the Honeybee library might not include the computational steps you need." ]
                        , p []
                            [ text "Please reach out to Justin at"
                            , a
                                [ A.href "mailto://justinlubin@berkeley.edu" ]
                                [ text "justinlubin@berkeley.edu" ]
                            , text "for help!"
                            ]
                        ]
                    ]

                 else
                    directManipulationPbn cells
                )
                    ++ [ downloadButton ]


view : Model -> Html Msg
view model =
    div
        [ A.id "root"
        ]
        [ menuBar
            []
            [ b [] [ text "Programming by Navigation" ]
            , text " with "
            , b
                []
                [ text "Honeybee 🐝" ]
            ]
            []
            [ button
                [ A.id "devmode"
                , E.onClick UserClickedDevMode
                , A.title "Sets fun value to 65"
                ]
                [ text "devmode" ]
            ]
        , pane
            []
            (paneHeading [] [ i [ circled ] [ text "i" ], text "Getting Started" ])
            [ p [] [ text "Honeybee is a tool you can use to write code to analyze experimental data." ]
            , p [] [ text "It works in two steps:" ]
            , ol []
                [ li [] [ text "First, you write down your experimental workflow." ]
                , li [] [ text "Then, Honeybee helps you navigate among all possible programs to analyze the experiment you wrote down." ]
                ]
            , p [] [ text "Using your biology expertise, you can navigate to the program that fits your need!" ]
            ]
        , pane
            []
            (paneHeading []
                [ span [ circled ] [ text "1" ]
                , span [] [ text "Experimental Workflow" ]
                ]
            )
          <|
            program model model.program
                ++ [ startNavigationButton model.program ]
        , pane
            [ A.id "navigation-pane"
            , A.classList [ ( "pane-inactive", model.pbnStatus == Nothing ) ]
            ]
            (paneHeading
                []
                [ span [ circled ] [ text "2" ]
                , span [] [ text "Navigation" ]
                ]
            )
            [ p
                []
                [ text "When you see a "
                , span
                    [ A.class "card-reference"
                    , A.class "cell-choice"
                    ]
                    [ text "Choice"
                    ]
                , text " cell, decide which analysis to run for that part of the code."
                ]
            , pbnStatus model.pbnStatus
            ]
        ]
