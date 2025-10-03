module Assoc exposing (..)

import Util


type alias Assoc k v =
    List ( k, v )


length : Assoc k v -> Int
length a =
    List.length a


all : (k -> v -> Bool) -> Assoc k v -> Bool
all p a =
    List.all (\( x, y ) -> p x y) a


map : (k -> a -> b) -> Assoc k a -> Assoc k b
map f =
    List.map (\( x, y ) -> ( x, f x y ))


mapCollapse : (k -> v -> b) -> Assoc k v -> List b
mapCollapse f =
    List.map (\( x, y ) -> f x y)


sequence : Assoc k (Maybe v) -> Maybe (Assoc k v)
sequence a =
    a
        |> mapCollapse (\k mv -> Maybe.map (\v -> ( k, v )) mv)
        |> Util.sequence


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


modify : k -> (v -> v) -> Assoc k v -> Assoc k v
modify k f a =
    map
        (\k2 v ->
            if k2 == k then
                f v

            else
                v
        )
        a


leftMergeWith : b -> Assoc k a -> Assoc k b -> Assoc k ( a, b )
leftMergeWith missing left right =
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


leftMerge : Assoc k a -> Assoc k b -> Assoc k ( a, Maybe b )
leftMerge left right =
    leftMergeWith Nothing left (map (\_ v -> Just v) right)


merge : Assoc k a -> Assoc k b -> Maybe (Assoc k ( a, b ))
merge left right =
    leftMerge left right
        |> map (\_ ( x, my ) -> Maybe.map (\y -> ( x, y )) my)
        |> sequence


getAll : a -> List (Assoc a b) -> List b
getAll k a =
    List.filterMap (get k) a


collect : List ( a, b ) -> Assoc a (List b)
collect xys =
    case xys of
        [] ->
            []

        ( x, y ) :: tl ->
            let
                collect_tl =
                    collect tl
            in
            set x
                (y :: (get x collect_tl |> Maybe.withDefault []))
                collect_tl
