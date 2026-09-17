module View exposing (view)

import Annotations
import Cell
import Compile
import Complete
import Core
import Html exposing (..)
import Html.Attributes as A
import Html.Events as E
import Json.Encode
import Model exposing (Model)
import Update exposing (Msg(..))
import Util


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


controlPanel : Model -> Html Msg
controlPanel model =
    panel
        [ A.id "control-panel" ]
        (case Debug.log "status" model.pbnStatus of
            Nothing ->
                goalSpecification model

            Just _ ->
                { header = [], body = [], footer = Nothing }
        )


view : Model -> Html Msg
view model =
    main_ []
        [ codePanel model
        , controlPanel model
        ]
