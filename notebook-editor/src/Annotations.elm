module Annotations exposing
    ( Annotation(..)
    , contains
    , getAll
    , removeAll
    )


type Annotation
    = NoSuggest
    | Intermediate


parseAnnotation : String -> Maybe Annotation
parseAnnotation s =
    case s of
        "nosuggest" ->
            Just NoSuggest

        "intermediate" ->
            Just Intermediate

        _ ->
            Nothing


split : String -> ( List Annotation, String )
split s =
    if String.startsWith "@" s then
        case List.head (String.indexes ":" s) of
            Nothing ->
                ( [], s )

            Just i ->
                ( s
                    |> String.slice 1 i
                    |> String.split ","
                    |> List.filterMap parseAnnotation
                , String.dropLeft (i + 1) s
                )

    else
        ( [], s )


getAll : String -> List Annotation
getAll s =
    Tuple.first (split s)


removeAll : String -> String
removeAll s =
    Tuple.second (split s)


contains : Annotation -> String -> Bool
contains a s =
    List.member a (getAll s)
