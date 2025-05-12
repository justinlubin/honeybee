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


stepSignature : StepKind -> Decoder StepSignature
stepSignature kind =
    map (\p -> { params = p, kind = kind })
        (field "params" <| keyValuePairs valueType)


library : Decoder Library
library =
    map2 (\props types -> props ++ types)
        (field "props" <| keyValuePairs (stepSignature Prop))
        (field "types" <| keyValuePairs (stepSignature Type))
