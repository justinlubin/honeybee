module Util exposing (..)


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
