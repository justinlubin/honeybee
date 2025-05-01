port module Port exposing (..)

import Assoc exposing (Assoc)
import Core
import Json.Decode as D



-- Outgoing messages


type alias PbnCheckMessage =
    { programSource : String
    }


port sendPbnCheck : PbnCheckMessage -> Cmd msg


type alias PbnInitMessage =
    { programSource : String
    }


port sendPbnInit : PbnInitMessage -> Cmd msg


type alias PbnChoiceMessage =
    { choice : Int
    }


port sendPbnChoice : PbnChoiceMessage -> Cmd msg


type alias DownloadMessage =
    { filename : String
    , text : String
    }


port sendDownload : DownloadMessage -> Cmd msg



-- Incoming messages


type alias ValidGoalMetadataMessage =
    { goalName : String
    , choices : List (Assoc String Core.Value)
    }


port receiveValidGoalMetadata : (D.Value -> msg) -> Sub msg


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


type alias PbnStatusMessage =
    { workingExpression : String
    , choices : Assoc Int String
    , valid : Bool
    }


port receivePbnStatus : (PbnStatusMessage -> msg) -> Sub msg
