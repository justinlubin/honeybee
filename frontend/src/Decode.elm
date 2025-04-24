module Decode exposing (library)

import Core exposing (..)
import Json.Decode exposing (..)


valueType : Decoder ValueType
valueType =
    string
        |> andThen
            (\s ->
                case s of
                    "Bool" ->
                        succeed VTBool

                    "Int" ->
                        succeed VTInt

                    "Str" ->
                        succeed VTStr

                    _ ->
                        fail "Unknown value type"
            )


stepSignature : Decoder StepSignature
stepSignature =
    map (\p -> { params = p })
        (field "params" <| keyValuePairs valueType)


library : Decoder Library
library =
    map2 (\props types -> { props = props, types = types })
        (field "props" <| keyValuePairs stepSignature)
        (field "types" <| keyValuePairs stepSignature)
