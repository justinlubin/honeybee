port module Incoming exposing (..)

import Assoc exposing (Assoc)
import Cell exposing (..)
import Core
import Dict
import Json.Decode as D
import Json.Decode.Pipeline as P



--------------------------------------------------------------------------------
-- Ports


type alias ValidGoalMetadataMessage =
    { goalName : String
    , choices : List (Assoc String Core.Value)
    }


decodeValue : D.Decoder Core.Value
decodeValue =
    D.oneOf
        [ D.map Core.VInt D.int
        , D.map Core.VBool D.bool
        , D.map Core.VStr D.string
        ]


decodeValidGoalMetadata : D.Decoder ValidGoalMetadataMessage
decodeValidGoalMetadata =
    D.map2 ValidGoalMetadataMessage
        (D.field "goalName" D.string)
        (D.field "choices" <| D.list <| D.keyValuePairs decodeValue)


port iValidGoalMetadata_ : (D.Value -> msg) -> Sub msg


iValidGoalMetadata : (Result D.Error ValidGoalMetadataMessage -> msg) -> Sub msg
iValidGoalMetadata f =
    iValidGoalMetadata_ (D.decodeValue decodeValidGoalMetadata >> f)


type alias PbnStatusMessage =
    { cells : List Cell
    , output : Maybe String
    , canUndo : Bool
    }


decodeCodeCell : D.Decoder CodeCell
decodeCodeCell =
    D.map2 CodeCell
        (D.field "title" D.string)
        (D.field "code" D.string)


decodeMetadataChoice : D.Decoder MetadataChoice
decodeMetadataChoice =
    D.map2 MetadataChoice
        (D.field "metadata" <| D.keyValuePairs decodeValue)
        (D.field "choice_index" D.int)


decodeHyperparameter : D.Decoder Hyperparameter
decodeHyperparameter =
    D.map3 Hyperparameter
        (D.field "name" D.string)
        (D.field "default" D.string)
        (D.field "comment" D.string)


decodeFunctionChoice : D.Decoder FunctionChoice
decodeFunctionChoice =
    D.succeed FunctionChoice
        |> P.required "function_title" D.string
        |> P.required "function_description" (D.nullable D.string)
        |> P.required "code" (D.nullable D.string)
        |> P.required "metadata_choices" (D.list decodeMetadataChoice)
        |> P.hardcoded 0
        |> P.optionalAt [ "info", "google_scholar_id" ]
            (D.nullable D.string)
            Nothing
        |> P.optionalAt [ "info", "citation" ]
            (D.nullable D.string)
            Nothing
        |> P.optionalAt [ "info", "additional_citations" ]
            (D.nullable <| D.list D.string)
            Nothing
        |> P.optionalAt [ "info", "hyperparameters" ]
            (D.nullable <| D.list decodeHyperparameter)
            Nothing
        |> P.optionalAt [ "info", "use" ]
            (D.nullable <| D.string)
            Nothing


decodeChoiceCell : D.Decoder ChoiceCell
decodeChoiceCell =
    D.map5 ChoiceCell
        (D.field "var_name" D.string)
        (D.field "type_title" D.string)
        (D.field "type_description" <| D.nullable D.string)
        (D.field "function_choices" <|
            D.map (List.sortBy (\x -> x.functionTitle)) <|
                D.list decodeFunctionChoice
        )
        (D.succeed Nothing)


decodeCell : D.Decoder Cell
decodeCell =
    D.oneOf
        [ D.field "Code" <| D.map Code decodeCodeCell
        , D.field "Choice" <| D.map Choice decodeChoiceCell
        ]


decodePbnStatus : D.Decoder PbnStatusMessage
decodePbnStatus =
    D.map3 PbnStatusMessage
        (D.field "cells" <| D.list decodeCell)
        (D.field "output" <| D.nullable D.string)
        (D.field "can_undo" <| D.bool)


port iPbnStatus_ : (D.Value -> msg) -> Sub msg


iPbnStatus : (Result D.Error PbnStatusMessage -> msg) -> Sub msg
iPbnStatus f =
    iPbnStatus_ (D.decodeValue decodePbnStatus >> f)



--------------------------------------------------------------------------------
-- Other decoders


valueType : D.Decoder Core.ValueType
valueType =
    D.string
        |> D.andThen
            (\s ->
                case s of
                    "Bool" ->
                        D.succeed Core.VTBool

                    "Int" ->
                        D.succeed Core.VTInt

                    "Str" ->
                        D.succeed Core.VTStr

                    _ ->
                        D.fail "Unknown value type"
            )


factSignature : D.Decoder Core.FactSignature
factSignature =
    D.map3
        (\p pl t ->
            { params = p
            , paramLabels = Dict.fromList (Maybe.withDefault [] pl)
            , title = t
            }
        )
        (D.field "params" <| D.keyValuePairs valueType)
        (D.maybe <| D.at [ "info", "params" ] <| D.keyValuePairs D.string)
        (D.maybe <| D.at [ "info", "title" ] D.string)


factLibrary : D.Decoder Core.FactLibrary
factLibrary =
    D.keyValuePairs factSignature


library : D.Decoder Core.Library
library =
    D.map2 (\p t -> { props = p, types = t })
        (D.field "props" factLibrary)
        (D.field "types" factLibrary)
