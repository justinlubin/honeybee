port module Port exposing (..)

import Assoc exposing (Assoc)



-- Outgoing messages


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


type alias PbnStatusMessage =
    { workingExpression : String
    , choices : Assoc Int String
    , valid : Bool
    }


port receivePbnStatus : (PbnStatusMessage -> msg) -> Sub msg
