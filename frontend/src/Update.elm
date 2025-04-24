module Update exposing (Msg(..), update)


type Msg
    = Noop


update : Msg -> b -> b
update msg model =
    case msg of
        Noop ->
            model
