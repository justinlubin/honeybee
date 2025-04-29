port module Port exposing (..)


type alias SendMessage =
    { programSource : String
    }


type alias ReceiveMessage =
    { synthesisResult : String
    }


port send : SendMessage -> Cmd msg


port receive : (ReceiveMessage -> msg) -> Sub msg
