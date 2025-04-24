module Assoc exposing (..)


type alias Assoc k v =
    List ( k, v )


mapCollapse : (k -> v -> b) -> Assoc k v -> List b
mapCollapse f =
    List.map (\( x, y ) -> f x y)


get : k -> Assoc k v -> Maybe v
get k a =
    case a of
        [] ->
            Nothing

        ( k2, v ) :: tl ->
            if k2 == k then
                Just v

            else
                get k tl
