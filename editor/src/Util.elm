module Util exposing (..)


justs : List (Maybe a) -> List a
justs xs =
    List.filterMap identity xs


findFirst : (a -> Bool) -> List a -> Maybe a
findFirst f xs =
    List.head (List.filter f xs)


indexedFilter : (Int -> a -> Bool) -> List a -> List a
indexedFilter pred xs =
    xs
        |> List.indexedMap (\i x -> ( i, x ))
        |> List.filterMap
            (\( i, x ) ->
                if pred i x then
                    Just x

                else
                    Nothing
            )


sequence : List (Maybe a) -> Maybe (List a)
sequence xs =
    let
        result =
            List.filterMap (\x -> x) xs
    in
    if List.length xs == List.length result then
        Just result

    else
        Nothing


unique : List a -> List a
unique xs =
    case xs of
        [] ->
            []

        hd :: tl ->
            if List.member hd tl then
                unique tl

            else
                hd :: unique tl


subscriptNumbers : String -> String
subscriptNumbers s =
    String.map
        (\c ->
            case c of
                '0' ->
                    '₀'

                '1' ->
                    '₁'

                '2' ->
                    '₂'

                '3' ->
                    '₃'

                '4' ->
                    '₄'

                '5' ->
                    '₅'

                '6' ->
                    '₆'

                '7' ->
                    '₇'

                '8' ->
                    '₈'

                '9' ->
                    '₉'

                _ ->
                    c
        )
        s


unSubscriptNumbers : String -> String
unSubscriptNumbers s =
    String.map
        (\c ->
            case c of
                '₀' ->
                    '0'

                '₁' ->
                    '1'

                '₂' ->
                    '2'

                '₃' ->
                    '3'

                '₄' ->
                    '4'

                '₅' ->
                    '5'

                '₆' ->
                    '6'

                '₇' ->
                    '7'

                '₈' ->
                    '8'

                '₉' ->
                    '9'

                _ ->
                    c
        )
        s


at : Int -> List a -> Maybe a
at i xs =
    xs
        |> List.drop i
        |> List.head
