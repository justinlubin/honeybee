port module Outgoing exposing (..)

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
    { programSource : String
    }


port oPbnCheck : PbnCheckMessage -> Cmd msg


type alias PbnInitMessage =
    { programSource : String
    }


port oPbnInit : PbnInitMessage -> Cmd msg


type alias PbnChooseMessage =
    { choice : Int
    }


port oPbnChoose : PbnChooseMessage -> Cmd msg
