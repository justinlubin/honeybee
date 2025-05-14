module Decode exposing (library)

import Core exposing (..)
import Dict
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
    map3
        (\p pl o ->
            { params = p
            , kind = kind
            , paramLabels = Dict.fromList pl
            , overview = Debug.log "Ever????" o
            }
        )
        (field "params" <| keyValuePairs valueType)
        (field "paramLabels" <| keyValuePairs string)
        (field "overview" <| nullable string)


library : Decoder Library
library =
    map2 (\props types -> props ++ types)
        (field "props" <| keyValuePairs (stepSignature Prop))
        (field "types" <| keyValuePairs (stepSignature Type))
