module Assoc exposing (..)


type alias Assoc k v =
    List ( k, v )


map : (k -> a -> b) -> Assoc k a -> Assoc k b
map f =
    List.map (\( x, y ) -> ( x, f x y ))


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


set : k -> v -> Assoc k v -> Assoc k v
set k v a =
    if List.any (\( k2, _ ) -> k2 == k) a then
        map
            (\k2 v2 ->
                if k2 == k then
                    v

                else
                    v2
            )
            a

    else
        ( k, v ) :: a


leftMerge : b -> Assoc k a -> Assoc k b -> Assoc k ( a, b )
leftMerge missing left right =
    map
        (\k v1 ->
            ( v1
            , case get k right of
                Nothing ->
                    missing

                Just v2 ->
                    v2
            )
        )
        left


getAll : a -> List (Assoc a b) -> List b
getAll k a =
    List.filterMap (get k) a
