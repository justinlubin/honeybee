port module Outgoing exposing (..)

import Json.Encode



--------------------------------------------------------------------------------
-- General


type alias ScrollIntoViewMessage =
    { selector : String
    }


port oScrollIntoView : ScrollIntoViewMessage -> Cmd msg


type alias DownloadMessage =
    { filename : String
    , text : String
    }


port oDownload : DownloadMessage -> Cmd msg



--------------------------------------------------------------------------------
-- PBN


type alias PbnCheckMessage =
    { propsSource : String
    }


port oPbnCheck : PbnCheckMessage -> Cmd msg


type alias PbnInitMessage =
    { programSource : String
    , sound : Bool
    }


port oPbnInit : PbnInitMessage -> Cmd msg


type alias PbnChooseMessage =
    { choice : Int
    }


port oPbnChoose : PbnChooseMessage -> Cmd msg


port oPbnSpeculate : PbnChooseMessage -> Cmd msg


type alias PbnUndoMessage =
    {}


port oPbnUndo : PbnUndoMessage -> Cmd msg



--------------------------------------------------------------------------------
-- Logging


type alias LogMessage =
    { partid : String
    , msg : Json.Encode.Value
    , build : String
    , logfmt : Int
    }


port oLog : LogMessage -> Cmd msg
