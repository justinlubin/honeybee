module View exposing (view)

import Annotations
import Core
import Html exposing (..)
import Html.Attributes as A
import Html.Events as E
import Model exposing (Model)
import Update exposing (Msg(..))
import Util


codePanel : Model -> Html Msg
codePanel model =
    div [ A.class "panel" ] [ header [] [ h1 [] [ text "Code Panel" ] ] ]


factSelect :
    Core.FactLibrary
    -> String
    -> Core.ProgramIndex
    -> Maybe (Core.Fact String)
    -> Html Msg
factSelect lib blankName pi mfact =
    let
        wrap key title =
            option [ A.value key ] [ text title ]

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
    case mfact of
        Nothing ->
            select
                [ E.onInput <|
                    \key ->
                        if key == blankName then
                            UserClearedStep pi

                        else
                            UserSetStep pi key
                ]
                options

        Just fact ->
            text "selected"


goalSpecification : Model -> Html Msg
goalSpecification model =
    section []
        [ h2 [] [ text "Experimental workflow" ]
        , p []
            [ factSelect
                model.library.props
                "Choose an assay…"
                (Core.Prop 0)
                (model.program.props |> Util.asSingleton |> Util.joinMaybe)
            ]
        , h2 [] [ text "Goal of experiment" ]
        , p []
            [ factSelect
                model.library.types
                "Choose a goal…"
                Core.Goal
                model.program.goal
            ]
        ]


controlPanel : Model -> Html Msg
controlPanel model =
    div
        [ A.class "panel"
        ]
        [ header [] [ h1 [] [ text "Control Panel" ] ]
        , case model.pbnStatus of
            Nothing ->
                goalSpecification model

            Just status ->
                text "status"
        ]


view : Model -> Html Msg
view model =
    main_ []
        [ codePanel model
        , controlPanel model
        ]
