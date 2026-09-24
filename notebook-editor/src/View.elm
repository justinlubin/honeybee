module View exposing (view)

import Annotations
import Cell
import Compile
import Complete
import Core
import Html exposing (..)
import Html.Attributes as A
import Html.Events as E
import Html.Keyed
import Incoming
import Json.Decode as D
import Markdown
import Model exposing (Model)
import SyntaxHighlight
import Update exposing (Msg(..))
import Util
import Version


help : Maybe String -> String -> List (Html Msg) -> Html Msg
help activeHelp id body =
    let
        active =
            activeHelp == Just id
    in
    div
        [ A.classList [ ( "help", True ), ( "help-active", active ) ]
        ]
        [ span
            [ A.class "help-button"
            , E.stopPropagationOn "click" (D.succeed ( UserClickedHelp id, True ))
            ]
            [ text "?" ]
        , div [ A.class "help-content" ] body
        ]


cellTitle : Cell.Cell -> String
cellTitle c =
    Annotations.removeAll <|
        case c of
            Cell.Code { title } ->
                title

            Cell.Choice { typeTitle } ->
                typeTitle


type SearchEngine
    = Google
    | DuckDuckGo


searchEngineUrl : SearchEngine -> String -> String
searchEngineUrl se query =
    let
        prefix =
            case se of
                Google ->
                    "https://google.com/search?q="

                DuckDuckGo ->
                    "https://duckduckgo.com/?q="

        encodedQuery =
            String.replace " " "+" query
    in
    prefix ++ encodedQuery


markdown : List (Attribute msg) -> String -> Html msg
markdown attrs s =
    Markdown.toHtmlWith
        { githubFlavored = Just { tables = True, breaks = False }
        , defaultHighlighting = Nothing
        , sanitize = False
        , smartypants = True
        }
        (A.class "markdown" :: attrs)
        s


inlineMarkdown : List (Attribute msg) -> String -> Html msg
inlineMarkdown attrs s =
    markdown (A.class "inline" :: attrs) s


pythonCode : String -> Html msg
pythonCode codeString =
    case SyntaxHighlight.python codeString of
        Ok hCode ->
            SyntaxHighlight.toBlockHtml Nothing hCode

        Err _ ->
            code [] [ text codeString ]


factSelect :
    Core.FactLibrary
    -> String
    -> Core.ProgramIndex
    -> Maybe (Core.Fact String)
    -> Bool
    -> Html Msg
factSelect lib blankName pi mfact enabled =
    let
        selectedName =
            case mfact of
                Just fact ->
                    Just fact.name

                Nothing ->
                    Nothing

        wrap sortKey key title =
            ( sortKey
            , option
                [ A.value key
                , A.selected (Just key == selectedName)
                ]
                [ text title
                ]
            )

        wrappedOptions =
            wrap "" blankName blankName
                :: List.filterMap
                    (\( key, sig ) ->
                        case sig.title of
                            Nothing ->
                                Just (wrap key key key)

                            Just title ->
                                if Annotations.contains Annotations.Intermediate title then
                                    Nothing

                                else
                                    Just (wrap title key title)
                    )
                    lib

        options =
            wrappedOptions
                |> List.sortBy (\( title, _ ) -> title)
                |> List.map (\( _, el ) -> el)
    in
    select
        [ A.disabled (not enabled)
        , A.value (selectedName |> Maybe.withDefault blankName)
        , E.onInput <|
            \key ->
                if key == blankName then
                    UserClearedStep pi

                else
                    UserSetStep pi key
        ]
        options


pane : Bool -> List (Html Msg) -> Html Msg
pane active body =
    div
        [ A.classList
            [ ( "pane", True )
            , ( "pane-inactive", not active )
            ]
        ]
        body


type alias Panel =
    { header : List (Html Msg)
    , body : List (Html Msg)
    , footer : Maybe (List (Html Msg))
    }


panel : Float -> List (Html.Attribute Msg) -> Panel -> Html Msg
panel fraction attrs p =
    let
        bgLine =
            span [ A.class "bg-line" ] []
    in
    div
        (A.class "panel"
            :: A.style "width" (String.fromFloat (100 * fraction) ++ "%")
            :: attrs
        )
        [ header [] ([ bgLine, bgLine, bgLine ] ++ p.header)
        , section [] p.body
        , case p.footer of
            Just f ->
                footer [] f

            Nothing ->
                text ""
        ]


startNavigationButton : Model -> Html Msg
startNavigationButton model =
    let
        attrs =
            case
                model.program
                    |> Complete.complete { allowPropHoles = True, allowGoalHoles = True }
                    |> Maybe.map Compile.compile
            of
                Just programSource ->
                    [ E.onClick (UserStartedNavigation { programSource = programSource }) ]

                Nothing ->
                    [ A.disabled True ]
    in
    button
        (A.class "right" :: attrs)
        [ text "Continue" ]


goalSpecification : Model -> Panel
goalSpecification model =
    let
        workflowComplete =
            case Util.at 0 model.program.props of
                Just (Just _) ->
                    True

                _ ->
                    False
    in
    { header =
        [ h1 [] [ text "Control Panel" ] ]
    , body =
        [ pane True
            [ h2 []
                [ text "Experimental workflow"
                , help
                    model.activeHelp
                    "experimental-workflow"
                    [ text "This is the assay that you ran for your experiment. It determines what kinds of analyses you can run." ]
                ]
            , factSelect
                model.library.props
                "Choose an assay…"
                (Core.Prop 0)
                (model.program.props |> Util.asSingleton |> Util.joinMaybe)
                True
            ]
        , pane workflowComplete
            [ h2 []
                [ text "Goal of experiment"
                , help
                    model.activeHelp
                    "goal"
                    [ text "This is the computational analysis you want to run on your data. It’s the reason for running the experiment." ]
                ]
            , factSelect
                (model.library.types
                    |> List.filter
                        (\( name, _ ) ->
                            List.member name model.goalSuggestions
                        )
                )
                "Choose a goal…"
                Core.Goal
                model.program.goal
                workflowComplete
            ]
        ]
    , footer =
        Just [ startNavigationButton model ]
    }


codePanel : Model -> Float -> Html Msg
codePanel model fraction =
    panel
        fraction
        [ A.id "code-panel" ]
        { header =
            [ h1 [] [ text "Code Panel" ]
            , case model.pbnStatus of
                Nothing ->
                    text ""

                Just status ->
                    text ""

            -- Outline!
            -- div
            --     [ A.id "outline" ]
            --     [ details []
            --         [ summary [] [ text "Outline" ]
            --         , ul [] <|
            --             List.indexedMap
            --                 (\_ c ->
            --                     li []
            --                         [ a
            --                             [ A.href "#" ]
            --                           <|
            --                             (if Cell.isChoice c then
            --                                 [ span
            --                                     [ A.class "card-reference"
            --                                     , A.class "cell-choice"
            --                                     ]
            --                                     [ text "Choice"
            --                                     ]
            --                                 , text " "
            --                                 ]
            --                              else
            --                                 []
            --                             )
            --                                 ++ [ text (cellTitle c)
            --                                    ]
            --                         ]
            --                 )
            --                 status.cells
            --         ]
            --     ]
            ]
        , body =
            case model.pbnStatus of
                Nothing ->
                    [ pane False
                        [ p
                            [ A.class "waiting" ]
                            [ text "No code yet! Use the control panel on the right." ]
                        ]
                    ]

                Just status ->
                    let
                        lastChoiceIndex =
                            status.cells
                                |> List.indexedMap
                                    (\i c ->
                                        case c of
                                            Cell.Choice _ ->
                                                Just i

                                            Cell.Code _ ->
                                                Nothing
                                    )
                                |> Util.justs
                                |> Util.last
                    in
                    List.indexedMap
                        (\i c ->
                            cell
                                { lastChoiceIndex = lastChoiceIndex
                                , index = i
                                , diff =
                                    case model.speculativePbnStatus of
                                        Just specStatus ->
                                            Util.findFirst2
                                                (\x y ->
                                                    case ( x, y ) of
                                                        ( Cell.Choice _, Cell.Code _ ) ->
                                                            True

                                                        _ ->
                                                            False
                                                )
                                                (List.reverse status.cells)
                                                (List.reverse specStatus.cells)

                                        Nothing ->
                                            Nothing
                                }
                                c
                        )
                        status.cells
        , footer = Nothing
        }


cell :
    { lastChoiceIndex : Maybe Int
    , index : Int
    , diff : Maybe ( Cell.Cell, Cell.Cell )
    }
    -> Cell.Cell
    -> Html Msg
cell ctx c =
    case c of
        Cell.Code cc ->
            section
                [ A.class "cell" ]
                [ h2 [] [ text cc.title ]
                , div
                    [ A.class "code-container" ]
                    [ pythonCode cc.code ]
                ]

        Cell.Choice _ ->
            if Just ctx.index == ctx.lastChoiceIndex then
                case ctx.diff of
                    Just ( _, Cell.Code specCode ) ->
                        section
                            [ A.id "active-choice-cell"
                            , A.class "cell"
                            , A.class "speculating"
                            ]
                            [ h2 [] [ text specCode.title ]
                            , div
                                [ A.class "code-container" ]
                                [ pythonCode specCode.code ]
                            ]

                    _ ->
                        section
                            [ A.id "active-choice-cell"
                            , A.class "cell"
                            , A.class "waiting-for-speculation"
                            ]
                            [ span [ A.class "choice" ] [ text "Choice" ]
                            , text " Choose code for this slot in the control panel"
                            ]

            else
                section
                    [ A.class "cell", A.class "dormant" ]
                    [ text "The code that goes here will be filled out using the control panel later"
                    ]


choice : Maybe String -> Incoming.PbnStatusMessage -> Panel
choice activeHelp status =
    let
        nextChoice =
            status.cells
                |> List.indexedMap
                    (\i c ->
                        case c of
                            Cell.Code _ ->
                                Nothing

                            Cell.Choice cc ->
                                Just ( i, cc )
                    )
                |> Util.justs
                |> Util.last

        header =
            [ h1 [] [ text "Control Panel" ] ]
    in
    case status.output of
        Just output ->
            { header = header
            , body =
                [ h2 [] [ text "All done!" ]
                , div [ A.class "markdown" ]
                    [ p [] [ text "You have completed all the choices you need to make." ]
                    , p [] [ text "Here’s what to do next:" ]
                    , ol []
                        [ li [] [ text "Download the notebook using the button below." ]
                        , li [] [ text "Open the notebook in Jupyter Lab." ]
                        , li [] [ text "Set the parameter variables at the top of the notebook." ]
                        , li [] [ text "Fill out any necessary sample sheets in a spreadsheet editor." ]
                        , li [] [ text "Run the code on your data!" ]
                        ]
                    ]
                ]
            , footer =
                Just
                    [ button
                        [ A.class "left"
                        , E.onClick UserClickedUndo
                        ]
                        [ text "Undo" ]
                    , button
                        [ A.class "big"
                        , E.onClick
                            (UserRequestedDownload
                                { filename = "pipeline.ipynb"
                                , text = output
                                }
                            )
                        ]
                        [ text "Download notebook" ]
                    ]
            }

        Nothing ->
            case nextChoice of
                Just ( cellIndex, cc ) ->
                    let
                        maybePbnChoiceIndex =
                            cc.selectedFunctionChoice
                                |> Maybe.andThen
                                    (\fci -> Util.at fci cc.functionChoices)
                                |> Maybe.andThen
                                    (\fc -> Util.at fc.selectedMetadataChoice fc.metadataChoices)
                                |> Maybe.map
                                    (\mc -> mc.choiceIndex)

                        selectionMade =
                            case cc.selectedFunctionChoice of
                                Just _ ->
                                    True

                                Nothing ->
                                    False
                    in
                    { header = header
                    , body =
                        [ h2 []
                            [ span [ A.class "choice" ] [ text "Choice" ]
                            , text " "
                            , text (Annotations.removeAll cc.typeTitle)
                            , help
                                activeHelp
                                "choice-type-title"
                                [ text "This is information about the part of the analysis you are currently working on. You’ll need to choose one of the “next steps” below based on what you feel is right for your experiment!" ]
                            ]
                        , case cc.typeDescription of
                            Just desc ->
                                markdown [] desc

                            Nothing ->
                                text ""
                        , h3
                            [ A.class "choices-header" ]
                            [ text "Choices for next step"
                            , help
                                activeHelp
                                "choice-next-steps"
                                [ text "These are the next steps you can choose between for this part of the analysis." ]
                            ]
                        , Html.Keyed.ul
                            [ A.class "function-choices" ]
                            (List.indexedMap
                                (\functionIndex fc ->
                                    ( fc.functionTitle
                                    , functionChoice
                                        { cellIndex = cellIndex
                                        , functionIndex = functionIndex
                                        , selected =
                                            Just functionIndex == cc.selectedFunctionChoice
                                        }
                                        fc
                                    )
                                )
                                cc.functionChoices
                            )
                        ]
                    , footer =
                        Just
                            [ button
                                [ A.class "left"
                                , E.onClick UserClickedUndo
                                ]
                                [ text "Undo" ]
                            , button
                                [ A.class "right"
                                , A.disabled (not selectionMade)
                                , E.onClick <|
                                    UserDeselectedFunction
                                        { cellIndex = cellIndex }
                                ]
                                [ text "Clear selection" ]
                            , button
                                ([ A.class "right"
                                 , A.disabled (not selectionMade)
                                 ]
                                    ++ (case maybePbnChoiceIndex of
                                            Just i ->
                                                [ E.onClick (UserMadePbnChoice i) ]

                                            Nothing ->
                                                []
                                       )
                                )
                                [ text "Continue" ]
                            ]
                    }

                Nothing ->
                    { header = header
                    , body = [ p [] [ text "Something has gone wrong… please try refreshing the page!" ] ]
                    , footer = Nothing
                    }


functionChoice :
    { cellIndex : Int, functionIndex : Int, selected : Bool }
    -> Cell.FunctionChoice
    -> Html Msg
functionChoice ctx fc =
    let
        maybePbnChoiceIndex =
            Util.at fc.selectedMetadataChoice fc.metadataChoices
                |> Maybe.map (\mc -> mc.choiceIndex)
    in
    li [ A.classList [ ( "selected", ctx.selected ) ] ]
        [ label []
            [ input
                [ A.name "function-choice"
                , A.type_ "radio"
                , A.checked ctx.selected
                , E.onClick <|
                    UserSelectedFunction
                        { cellIndex = ctx.cellIndex }
                        ctx.functionIndex
                        maybePbnChoiceIndex
                ]
                []
            , strong [] [ text fc.functionTitle ]
            , case fc.use of
                Just use ->
                    span
                        [ A.class "use" ]
                        [ text " "
                        , inlineMarkdown [] use
                        ]

                Nothing ->
                    text ""
            ]
        , ul [ A.class "tool-search-info" ] <|
            List.concat
                [ if fc.search then
                    let
                        searchEngineQuery =
                            fc.functionTitle ++ " bioinformatics"
                    in
                    [ li []
                        [ img [ A.src "assets/google.webp" ] []
                        , a
                            [ A.href (searchEngineUrl Google searchEngineQuery) ]
                            [ text "Google" ]
                        ]
                    , li []
                        [ img [ A.src "assets/duckduckgo.png" ] []
                        , a
                            [ A.href (searchEngineUrl DuckDuckGo searchEngineQuery) ]
                            [ text "DuckDuckGo" ]
                        ]
                    ]

                  else
                    []
                , case fc.pmid of
                    Just pmid ->
                        [ li []
                            [ img [ A.src "assets/nih.png" ] []
                            , a
                                [ A.href <|
                                    "https://pubmed.ncbi.nlm.nih.gov/"
                                        ++ pmid
                                        ++ "/"
                                ]
                                [ text "PubMed" ]
                            ]
                        ]

                    Nothing ->
                        []
                , case fc.googleScholarId of
                    Just gsid ->
                        [ li []
                            [ img [ A.src "assets/google_scholar.png" ] []
                            , a
                                [ A.href <| "https://scholar.google.com/scholar?cites=" ++ gsid ]
                                [ text "Google Scholar" ]
                            ]
                        ]

                    Nothing ->
                        []
                ]
        , case fc.functionDescription of
            Just desc ->
                details
                    []
                    [ summary [] [ text "More info…" ]
                    , div []
                        [ markdown [] desc
                        , if List.isEmpty fc.hyperparameters then
                            text ""

                          else
                            details []
                                [ summary [] [ text "Parameters you’ll need to set…" ]
                                , div [ A.class "markdown" ]
                                    [ p []
                                        [ text "Once you download your script, you will need to set the following parameters at the top of the file:"
                                        ]
                                    , ul []
                                        (List.map
                                            (\h ->
                                                li []
                                                    [ code [] [ text h.name ]
                                                    , text <|
                                                        ": "
                                                            ++ h.comment
                                                            ++ " (default: "
                                                            ++ h.default
                                                            ++ ")"
                                                    ]
                                            )
                                            fc.hyperparameters
                                        )
                                    ]
                                ]
                        , case fc.citation of
                            Just citation ->
                                details [] <|
                                    [ summary [] [ text "Citation information..." ]
                                    , div [ A.class "markdown" ] <|
                                        [ p []
                                            [ text <|
                                                "If you use "
                                                    ++ fc.functionTitle
                                                    ++ ", please cite it as:"
                                            ]
                                        , blockquote [] [ text citation ]
                                        ]
                                            ++ (case fc.additionalCitations of
                                                    Just acs ->
                                                        [ p [] [ text "Please also cite:" ]
                                                        ]
                                                            ++ List.map
                                                                (\c ->
                                                                    blockquote
                                                                        []
                                                                        [ text c ]
                                                                )
                                                                acs

                                                    Nothing ->
                                                        []
                                               )
                                    ]

                            Nothing ->
                                text ""
                        , if List.length fc.metadataChoices > 1 then
                            text "TODO"

                          else
                            text ""
                        ]
                    ]

            Nothing ->
                text ""
        ]


controlPanel : Model -> Float -> Html Msg
controlPanel model fraction =
    panel
        fraction
        [ A.id "control-panel" ]
        (case model.pbnStatus of
            Nothing ->
                goalSpecification model

            Just status ->
                choice model.activeHelp status
        )


mainMenuBar : Html Msg
mainMenuBar =
    div
        [ A.class "menu-bar" ]
        [ div [ A.class "menu-bar-left" ]
            [ span []
                [ text "🐝 "
                , b
                    []
                    [ a [ A.href "https://honeybee-lang.org" ] [ text "Honeybee" ]
                    ]
                , text " (homepage)"
                ]
            , span []
                [ img
                    [ A.src "assets/zulip-icon-circle.svg"
                    , A.width 20
                    , A.height 20
                    ]
                    []
                , text " "
                , b []
                    [ a
                        [ A.href "https://chat.honeybee-lang.org" ]
                        [ text "Zulip" ]
                    ]
                , text " (say hi, ask for help)"
                ]
            , span []
                [ text "📓 "
                , b []
                    [ a
                        [ A.href "launch-notebook.sh"
                        , A.download "launch-notebook.sh"
                        ]
                        [ text "Download notebook launcher" ]
                    ]
                ]
            ]
        , div [ A.class "menu-bar-right" ]
            [ span
                [ A.class "version-number" ]
                [ text <| " build " ++ Version.build
                , if not Version.stable then
                    span [ A.class "unstable-indicator" ] [ text " UNSTABLE" ]

                  else
                    text ""
                ]
            ]
        ]


dragHandle : Html Msg
dragHandle =
    div
        [ A.id "drag-handle"
        , E.onMouseDown UserMouseDownedHandle
        ]
        []


view : Model -> Html Msg
view model =
    div
        [ A.id "root"
        , A.style "user-select" <|
            case model.dragHandleState of
                Model.Static _ ->
                    "auto"

                Model.Moving _ ->
                    "none"
        ]
        [ mainMenuBar
        , main_ []
            [ codePanel
                model
                (Model.toFraction model.dragHandleState)
            , dragHandle
            , controlPanel
                model
                (1 - Model.toFraction model.dragHandleState)
            ]
        ]
