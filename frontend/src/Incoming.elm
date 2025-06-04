port module Incoming exposing (..)

import Assoc exposing (Assoc)
import Cell exposing (..)
import Core
import Json.Decode as D



--------------------------------------------------------------------------------
-- PBN


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
    }


decodeCodeCell : D.Decoder CodeCell
decodeCodeCell =
    D.map2 CodeCell
        (D.field "title" <| D.nullable D.string)
        (D.field "code" D.string)


decodeMetadataChoice : D.Decoder MetadataChoice
decodeMetadataChoice =
    D.map2 MetadataChoice
        (D.field "metadata" <| D.keyValuePairs decodeValue)
        (D.field "choice_index" D.int)


decodeFunctionChoice : D.Decoder FunctionChoice
decodeFunctionChoice =
    D.map5 FunctionChoice
        (D.field "function_title" D.string)
        (D.field "function_description" <| D.nullable D.string)
        (D.field "code" <| D.nullable D.string)
        (D.field "metadata_choices" <| D.list decodeMetadataChoice)
        (D.succeed Nothing)


decodeChoiceCell : D.Decoder ChoiceCell
decodeChoiceCell =
    D.map5 ChoiceCell
        (D.field "var_name" D.string)
        (D.field "type_title" D.string)
        (D.field "type_description" <| D.nullable D.string)
        (D.field "function_choices" <| D.list decodeFunctionChoice)
        (D.succeed Nothing)


decodeCell : D.Decoder Cell
decodeCell =
    D.oneOf
        [ D.field "Code" <| D.map Code decodeCodeCell
        , D.field "Choice" <| D.map Choice decodeChoiceCell
        ]


decodePbnStatus : D.Decoder PbnStatusMessage
decodePbnStatus =
    D.map2 PbnStatusMessage
        (D.field "cells" <| D.list decodeCell)
        (D.field "output" <| D.nullable D.string)


port iPbnStatus_ : (D.Value -> msg) -> Sub msg


iPbnStatus : (Result D.Error PbnStatusMessage -> msg) -> Sub msg
iPbnStatus f =
    iPbnStatus_ (D.decodeValue decodePbnStatus >> f)
