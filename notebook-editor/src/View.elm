module View exposing (view)

import Annotations
import Cell
import Compile
import Complete
import Core
import Html exposing (..)
import Html.Attributes as A
import Html.Events as E
import Incoming
import Json.Encode
import Markdown
import Model exposing (Model)
import Update exposing (Msg(..))
import Util


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


fancyCode : List (Attribute msg) -> { language : String, code : String } -> Html msg
fancyCode attrs { language, code } =
    node "fancy-code"
        ([ A.attribute "language" language
         , A.property "code" (Json.Encode.string code)
         ]
            ++ attrs
        )
        []


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

        wrap key title =
            option
                [ A.value key
                ]
                [ text title
                ]

        options =
            wrap blankName blankName
                :: List.filterMap
                    (\( key, sig ) ->
                        case sig.title of
                            Nothing ->
                                Just (wrap key key)

                            Just title ->
                                if Annotations.contains Annotations.Intermediate title then
                                    Nothing

                                else
                                    Just (wrap key title)
                    )
                    lib
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


panel : List (Html.Attribute Msg) -> Panel -> Html Msg
panel attrs p =
    div
        (A.class "panel" :: attrs)
        [ header [] p.header
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
    button attrs [ text "Continue" ]


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
            [ h2 [] [ text "Experimental workflow" ]
            , factSelect
                model.library.props
                "Choose an assay…"
                (Core.Prop 0)
                (model.program.props |> Util.asSingleton |> Util.joinMaybe)
                True
            ]
        , pane workflowComplete
            [ h2 [] [ text "Goal of experiment" ]
            , factSelect
                model.library.types
                "Choose a goal…"
                Core.Goal
                model.program.goal
                workflowComplete
            ]
        ]
    , footer =
        Just [ startNavigationButton model ]
    }


codePanel : Model -> Html Msg
codePanel model =
    panel
        [ A.id "code-panel" ]
        { header = [ h1 [] [ text "Code Panel" ] ]
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
                    List.map cell status.cells
        , footer = Nothing
        }


cell : Cell.Cell -> Html Msg
cell c =
    case c of
        Cell.Code cc ->
            section
                [ A.class "cell" ]
                [ h2 [] [ text cc.title ]
                , div [ A.class "code-container" ]
                    [ fancyCode []
                        { language = "python"
                        , code = cc.code
                        }
                    ]
                ]

        Cell.Choice cc ->
            section
                [ A.class "cell", A.style "color" "red" ]
                [ h2 [] [ text "CHOICE" ]
                ]


choice : Incoming.PbnStatusMessage -> Panel
choice status =
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
    case nextChoice of
        Just ( cellIndex, cc ) ->
            let
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
                    , text cc.typeTitle
                    ]
                , case cc.typeDescription of
                    Just desc ->
                        markdown [] desc

                    Nothing ->
                        text ""
                , h3 [] [ text "Choices for next step" ]
                , ul
                    [ A.class "function-choices" ]
                    (List.indexedMap
                        (\functionIndex fc ->
                            functionChoice
                                { cellIndex = cellIndex
                                , functionIndex = functionIndex
                                , selected =
                                    Just functionIndex == cc.selectedFunctionChoice
                                }
                                fc
                        )
                        cc.functionChoices
                    )
                ]
            , footer =
                Just
                    [ button [ A.class "left" ] [ text "Undo" ]
                    , button
                        [ A.disabled (not selectionMade)
                        , E.onClick <|
                            UserDeselectedFunction
                                { cellIndex = cellIndex }
                        ]
                        [ text "Clear selection" ]
                    , button [ A.disabled (not selectionMade) ] [ text "Continue" ]
                    ]
            }

        Nothing ->
            { header = header, body = [ text "all done!" ], footer = Nothing }


functionChoice :
    { cellIndex : Int, functionIndex : Int, selected : Bool }
    -> Cell.FunctionChoice
    -> Html Msg
functionChoice ctx fc =
    li []
        [ label []
            [ input
                [ A.name "function-choice"
                , A.type_ "radio"
                , A.checked ctx.selected
                , E.onInput <|
                    \_ ->
                        UserSelectedFunction
                            { cellIndex = ctx.cellIndex }
                            ctx.functionIndex
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


controlPanel : Model -> Html Msg
controlPanel model =
    panel
        [ A.id "control-panel" ]
        (case model.pbnStatus of
            Nothing ->
                goalSpecification model

            Just status ->
                choice status
        )


view : Model -> Html Msg
view model =
    main_ []
        [ codePanel model
        , controlPanel model
        ]
