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


factSignature : Decoder FactSignature
factSignature =
    map3
        (\p pl o ->
            { params = p
            , paramLabels = Dict.fromList pl
            , overview = o
            }
        )
        (field "params" <| keyValuePairs valueType)
        (field "paramLabels" <| keyValuePairs string)
        (field "overview" <| nullable string)


factLibrary : Decoder FactLibrary
factLibrary =
    keyValuePairs factSignature


library : Decoder Library
library =
    map2 (\p t -> { props = p, types = t })
        (field "props" factLibrary)
        (field "types" factLibrary)
