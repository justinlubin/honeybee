module Main exposing (..)

import Browser
import Incoming
import Json.Decode
import Model
import Update
import View


main : Program Json.Decode.Value Model.Model Update.Msg
main =
    Browser.element
        { init =
            \v ->
                Model.init
                    { library =
                        v
                            |> Json.Decode.decodeValue (Json.Decode.field "library" Incoming.library)
                            |> Result.mapError (Debug.log "'library' decode error")
                            |> Result.withDefault { props = [], types = [] }
                    , sound =
                        v
                            |> Json.Decode.decodeValue (Json.Decode.field "sound" Json.Decode.bool)
                            |> Result.mapError (Debug.log "'sound' decode error")
                            |> Result.withDefault True
                    , log =
                        v
                            |> Json.Decode.decodeValue (Json.Decode.field "log" Json.Decode.bool)
                            |> Result.mapError (Debug.log "'log' decode error")
                            |> Result.withDefault False
                    , partid =
                        v
                            |> Json.Decode.decodeValue (Json.Decode.field "partid" Json.Decode.string)
                            |> Result.mapError (Debug.log "'partid' decode error")
                            |> Result.withDefault "000"
                    }
        , update = Update.update
        , view = View.view
        , subscriptions = Update.subscriptions
        }
