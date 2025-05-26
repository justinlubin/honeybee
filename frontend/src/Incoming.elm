port module Incoming exposing (..)

import Assoc exposing (Assoc)
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
    { workingExpression : String
    , choices : Assoc Int String
    , valid : Bool
    }


port iPbnStatus_ : (PbnStatusMessage -> msg) -> Sub msg


iPbnStatus : (PbnStatusMessage -> msg) -> Sub msg
iPbnStatus f =
    iPbnStatus_ f
