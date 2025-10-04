module View exposing (view)

import Annotations
import Assoc exposing (Assoc)
import Cell
import Compile
import Complete
import Core exposing (..)
import Dict exposing (Dict)
import Html exposing (..)
import Html.Attributes as A
import Html.Events as E
import Html.Keyed
import Incoming
import Json.Encode
import Markdown
import Model exposing (Model)
import Regex
import Update exposing (Msg(..))
import Util
import Version



--------------------------------------------------------------------------------
-- Generic


circled : Attribute msg
circled =
    A.class "circled"


menuBar :
    List (Attribute msg)
    -> List (Html msg)
    -> List (Html msg)
    -> List (Html msg)
    -> Html msg
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


type CollapseConfig
    = NotCollapsible
    | Collapsible { openByDefault : Bool }


type alias CardConfig =
    { collapse : CollapseConfig
    }


card : CardConfig -> List (Attribute msg) -> Html msg -> List (Html msg) -> Html msg
card config attrs headerContent bodyContent =
    let
        ( overallWrapper, headerWrapper ) =
            case config.collapse of
                NotCollapsible ->
                    ( div [], div [] )

                Collapsible { openByDefault } ->
                    ( details
                        (if openByDefault then
                            [ A.attribute "open" "" ]

                         else
                            []
                        )
                    , summary []
                    )
    in
    section
        (A.class "card" :: attrs)
        [ overallWrapper
            [ headerWrapper [ header [ A.class "card-header" ] [ headerContent ] ]
            , div [ A.class "card-body" ] bodyContent
            ]
        ]


cardHeading :
    List (Attribute msg)
    -> List (Html msg)
    -> List (Html msg)
    -> List (Html msg)
    -> Html msg
cardHeading attrs prefix content suffix =
    div
        (A.class "card-heading-wrapper" :: attrs)
        [ span [ A.class "card-heading-prefix" ] prefix
        , span [ A.class "card-heading-prefix-separator" ] []
        , h3 [ A.class "card-heading" ] content
        , span [ A.class "card-heading-suffix" ] suffix
        ]


cardHeadingSubtitle : List (Attribute msg) -> List (Html msg) -> Html msg
cardHeadingSubtitle attrs content =
    span (A.class "card-heading-subtitle" :: attrs) content


cardInnerHeading : List (Attribute msg) -> List (Html msg) -> Html msg
cardInnerHeading attrs content =
    h4 (A.class "card-inner-heading" :: attrs) content


fancyCode : List (Attribute msg) -> { language : String, code : String } -> Html msg
fancyCode attrs { language, code } =
    node "fancy-code"
        ([ A.attribute "language" language
         , A.property "code" (Json.Encode.string code)
         ]
            ++ attrs
        )
        []


tabbedMenu :
    List (Attribute msg)
    -> { selectionEvent : Int -> msg, deselectionEvent : msg, selectedIndex : Maybe Int }
    -> List { heading : Html msg, body : Html msg }
    -> Html msg
tabbedMenu attrs { selectionEvent, deselectionEvent, selectedIndex } content =
    let
        ( headers, bodies ) =
            List.unzip <|
                List.indexedMap
                    (\i { heading, body } ->
                        let
                            selected =
                                selectedIndex == Just i

                            selectedAttr =
                                A.classList
                                    [ ( "tabbed-menu-selected"
                                      , selectedIndex == Just i
                                      )
                                    ]
                        in
                        ( div
                            [ A.class "tabbed-menu-header"
                            , selectedAttr
                            , E.onClick <|
                                if selected then
                                    deselectionEvent

                                else
                                    selectionEvent i
                            ]
                            [ heading
                            ]
                        , div
                            [ A.class "tabbed-menu-body", selectedAttr ]
                            [ body ]
                        )
                    )
                    content
    in
    div
        ([ A.class "tabbed-menu"
         , A.classList [ ( "closed", selectedIndex == Nothing ) ]
         ]
            ++ attrs
        )
        [ div [ A.class "tabbed-menu-headers" ] headers
        , div [ A.class "tabbed-menu-bodies" ] bodies
        ]



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
            , A.class "card-inner-heading"
            ]
            [ argLabels
                |> Dict.get argName
                |> Maybe.map Annotations.removeAll
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
        , if
            List.isEmpty suggestions
                || (argLabels
                        |> Dict.get argName
                        |> Maybe.map (Annotations.contains Annotations.NoSuggest)
                        |> (==) (Just True)
                   )
          then
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
            List.filter
                (\( _, displayName ) ->
                    not <|
                        Annotations.contains
                            Annotations.Intermediate
                            displayName
                )
            <|
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
        { collapse = NotCollapsible }
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


functionChoice :
    { cellIndex : Int, functionIndex : Int }
    -> Cell.FunctionChoice
    -> { heading : Html Msg, body : Html Msg }
functionChoice ctx fc =
    let
        selectAdditionalInformation =
            [ p
                [ A.class "tabbed-menu-body-label" ]
                [ text "Select additional information…" ]
            , select
                [ A.class "tabbed-menu-body-dropdown"
                , E.onInput <|
                    \v ->
                        case String.toInt v of
                            Just n ->
                                UserSelectedMetadata ctx n

                            Nothing ->
                                Nop
                ]
              <|
                List.indexedMap
                    (\i mc ->
                        option
                            [ A.value (String.fromInt i)
                            ]
                            [ mc.metadata
                                |> Assoc.mapCollapse (\k v -> k ++ " = " ++ Compile.value v)
                                |> String.join ", "
                                |> text
                            ]
                    )
                    fc.metadataChoices
            ]
    in
    { heading = text fc.functionTitle
    , body =
        div [] <|
            [ Markdown.toHtml
                [ A.class "markdown" ]
                (Maybe.withDefault "" fc.functionDescription)
            ]
                ++ (if List.length fc.metadataChoices > 1 then
                        selectAdditionalInformation

                    else
                        []
                   )
                ++ (case fc.code of
                        Nothing ->
                            []

                        Just c ->
                            let
                                cleanCode =
                                    Regex.replaceAtMost 1
                                        (Maybe.withDefault Regex.never <|
                                            Regex.fromString "\"\"\"(.|\n)*?\"\"\"\\s*"
                                        )
                                        (\_ -> "")
                                        c
                            in
                            [ p [ A.class "tabbed-menu-body-label" ] [ text "Code preview…" ]
                            , div
                                [ A.class "code-preview" ]
                                [ fancyCode
                                    []
                                    { language = "python"
                                    , code = cleanCode
                                    }
                                ]
                            ]
                   )
    }


functionChoices :
    { a | cellIndex : Int, selectedFunctionChoice : Maybe Int }
    -> List Cell.FunctionChoice
    -> Html Msg
functionChoices ctx fcs =
    tabbedMenu
        []
        { selectionEvent =
            UserSelectedFunction { cellIndex = ctx.cellIndex }
        , deselectionEvent =
            UserDeselectedFunction { cellIndex = ctx.cellIndex }
        , selectedIndex =
            ctx.selectedFunctionChoice
        }
        (List.indexedMap
            (\i fc ->
                functionChoice
                    { cellIndex = ctx.cellIndex, functionIndex = i }
                    fc
            )
            fcs
        )


cellId : Int -> String
cellId cellIndex =
    "cell" ++ String.fromInt cellIndex


cellTitle : Cell.Cell -> String
cellTitle c =
    Annotations.removeAll <|
        case c of
            Cell.Code { title, functionTitle } ->
                case ( title, functionTitle ) of
                    ( Just t, _ ) ->
                        t

                    ( _, Just t ) ->
                        t

                    _ ->
                        ""

            Cell.Choice { typeTitle } ->
                typeTitle


cell : { cellIndex : Int } -> Cell.Cell -> Html Msg
cell ctx c =
    case c of
        Cell.Code { code } ->
            card
                { collapse = Collapsible { openByDefault = ctx.cellIndex /= 0 } }
                ([ A.class "cell-code"
                 , A.id (cellId ctx.cellIndex)
                 ]
                    ++ (if ctx.cellIndex /= 0 then
                            [ A.attribute "data-popinkey" code
                            ]

                        else
                            []
                       )
                )
                (cardHeading []
                    [ text "Code" ]
                    [ text (cellTitle c) ]
                    []
                )
                [ fancyCode [] { language = "python", code = code }
                ]

        Cell.Choice x ->
            let
                suffix =
                    if List.length x.functionChoices == 1 then
                        " (there's only one option in this case)"

                    else
                        ""
            in
            card
                { collapse = Collapsible { openByDefault = True } }
                [ A.class "cell-choice"
                , A.id (cellId ctx.cellIndex)
                ]
                (cardHeading []
                    [ span []
                        [ text "Choice"
                        , cardHeadingSubtitle [] [ text x.varName ]
                        ]
                    ]
                    [ text (cellTitle c) ]
                    []
                )
                [ Markdown.toHtml
                    [ A.class "markdown" ]
                    (Maybe.withDefault "" x.typeDescription)

                -- , cardInnerHeading [] [ text "Notes" ]
                -- , textarea [] []
                , cardInnerHeading [] [ text ("Choices" ++ suffix) ]
                , functionChoices
                    { cellIndex = ctx.cellIndex
                    , selectedFunctionChoice = x.selectedFunctionChoice
                    }
                    x.functionChoices
                , let
                    maybePbnChoiceIndex =
                        x.selectedFunctionChoice
                            |> Maybe.andThen (\fci -> Util.at fci x.functionChoices)
                            |> Maybe.andThen
                                (\fc ->
                                    Util.at fc.selectedMetadataChoice
                                        fc.metadataChoices
                                )
                            |> Maybe.map (\mc -> mc.choiceIndex)

                    disabled =
                        maybePbnChoiceIndex == Nothing

                    event =
                        maybePbnChoiceIndex
                            |> Maybe.map UserMadePbnChoice
                            |> Maybe.withDefault Nop
                  in
                  button
                    [ A.class "standout-button"
                    , A.disabled disabled
                    , E.onClick event
                    ]
                    (text "Make selection"
                        :: (if disabled then
                                [ div
                                    [ A.class "subtitle" ]
                                    [ text "(Choose an analysis first)" ]
                                ]

                            else
                                []
                           )
                    )
                ]


directManipulationPbn : List Cell.Cell -> List ( String, Html Msg )
directManipulationPbn cells =
    List.indexedMap
        (\i c ->
            ( Cell.key c
            , cell { cellIndex = i } c
            )
        )
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
        ([ A.id "start-navigating", A.class "standout-button" ] ++ attrs)
        (text "Start navigating" :: extras)


nextChoice : List Cell.Cell -> Maybe Int
nextChoice cells =
    cells
        |> List.indexedMap Tuple.pair
        |> Util.findFirst (\( _, c ) -> Cell.isChoice c)
        |> Maybe.map Tuple.first


solutionPrefix : String
solutionPrefix =
    "################################################################################\n"
        ++ "# Script originally created using\n"
        ++ "#     Honeybee (https://honeybee-lang.org), version "
        ++ Version.fullVersion
        ++ "\n\n"


pbnStatus : Maybe Incoming.PbnStatusMessage -> List (Html Msg)
pbnStatus ms =
    case ms of
        Nothing ->
            []

        Just { cells, output } ->
            let
                outline =
                    div
                        [ A.class "outline-wrapper" ]
                        [ nav
                            [ A.class "outline" ]
                            [ h3
                                [ A.class "outline-heading" ]
                                [ text "Outline" ]
                            , ul [] <|
                                List.indexedMap
                                    (\cellIndex c ->
                                        li []
                                            [ a
                                                [ A.href ("#" ++ cellId cellIndex) ]
                                              <|
                                                (if Cell.isChoice c then
                                                    [ span
                                                        [ A.class "card-reference"
                                                        , A.class "cell-choice"
                                                        ]
                                                        [ text "Choice"
                                                        ]
                                                    , text " "
                                                    ]

                                                 else
                                                    []
                                                )
                                                    ++ [ text (cellTitle c)
                                                       ]
                                            ]
                                    )
                                    cells
                                    ++ [ button
                                            [ A.class "standout-button"
                                            , A.class "post-popin-attention"
                                            ]
                                            [ case nextChoice cells of
                                                Just i ->
                                                    a
                                                        [ A.href ("#" ++ cellId i) ]
                                                        [ text "Next "
                                                        , span
                                                            [ A.class "card-reference"
                                                            , A.class "cell-choice"
                                                            ]
                                                            [ text "Choice"
                                                            ]
                                                        ]

                                                Nothing ->
                                                    a
                                                        [ A.href "#pbn-completed" ]
                                                        [ text "Go to download button!"
                                                        ]
                                            ]
                                       ]
                            ]
                        ]

                downloadButton =
                    case output of
                        Nothing ->
                            text ""

                        Just solutionString ->
                            div [ A.id "pbn-completed" ]
                                [ button
                                    [ A.class "standout-button"
                                    , E.onClick
                                        (UserRequestedDownload
                                            { filename = "analysis.py"
                                            , text = solutionPrefix ++ solutionString
                                            }
                                        )
                                    ]
                                    [ text "Download analysis script" ]
                                ]
            in
            [ p
                [ A.class "tip" ]
                [ text "Just like in Jupyter notebooks, this interface consists of many "
                , span [ A.class "card-reference", A.class "cell-code" ] [ text "Code" ]
                , text " cells. However, now there are also "
                , span [ A.class "card-reference", A.class "cell-choice" ] [ text "Choice" ]
                , text " cells! When you see a "
                , span [ A.class "card-reference", A.class "cell-choice" ] [ text "Choice" ]
                , text " cell, decide which analysis to run for that part of the code. When you make the selection, the "
                , span [ A.class "card-reference", A.class "cell-choice" ] [ text "Choice" ]
                , text " cell will become a "
                , span [ A.class "card-reference", A.class "cell-code" ] [ text "Code" ]
                , text " cell."
                ]
            , p
                [ A.class "tip" ]
                [ text "Choosing between analyses in a "
                , span [ A.class "card-reference", A.class "cell-choice" ] [ text "Choice" ]
                , text " cell can be quite challenging. Please take your time, read the information at each step, and search the Internet for resources that could help you make your decision!"
                ]
            , outline
            , Html.Keyed.node "pop-in" [] (directManipulationPbn cells)
            , downloadButton
            ]


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
                [ a [ A.href "https://honeybee-lang.org" ] [ text "Honeybee" ]
                , text " 🐝"
                ]
            ]
            []
            [ button
                [ A.id "devmode"
                , E.onClick UserClickedDevMode
                ]
                [ text "devmode" ]
            , span
                [ A.class "version-number" ]
                [ text <| " version " ++ Version.fullVersion
                ]
            ]
        , pane
            []
            (paneHeading [] [ i [ circled ] [ text "i" ], text "Getting Started" ])
            [ p [] [ text "Honeybee is a programming tool you can use to help you write Python code to analyze experimental data. It works in two steps:" ]
            , ol []
                [ li [] [ text "First, you write down your experimental workflow and goal." ]
                , li []
                    [ text "Then, Honeybee helps you work "
                    , b [] [ text "backward" ]
                    , text " from your goal to write a program to analyze your experimental data."
                    ]
                ]
            , p [] [ text "Once you finish the first step (filling out the details of your experiment), the next step (navigating to an analysis program) works like this:" ]
            , img [ A.src "assets/navigation-overview.svg" ] []
            , p [] [ text "You’ll keep working backward until there are no steps left." ]
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
            (pbnStatus model.pbnStatus)
        ]
