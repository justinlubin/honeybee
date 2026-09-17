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
                ( v
                    |> Json.Decode.decodeValue Incoming.library
                    |> Result.mapError (Debug.log "error")
                    |> Result.withDefault { props = [], types = [] }
                    |> Model.init
                , Cmd.none
                )
        , update = Update.update
        , view = View.view
        , subscriptions = Update.subscriptions
        }
