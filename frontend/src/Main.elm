module Main exposing (..)

import Browser
import Compile
import Decode
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
                    |> Json.Decode.decodeValue Decode.library
                    |> Result.withDefault []
                    |> Model.init
                , Cmd.none
                )
        , update = Update.update
        , view = View.view
        , subscriptions = \_ -> Sub.none
        }
