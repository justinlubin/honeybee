module Cell exposing (..)

import Assoc exposing (Assoc)
import Core


type alias MetadataChoice =
    { metadata : Assoc String Core.Value
    , choiceIndex : Int
    }


type alias FunctionChoice =
    { functionTitle : String
    , functionDescription : Maybe String
    , code : Maybe String
    , metadataChoices : List MetadataChoice
    , selectedMetadataChoice : Int
    }


type alias CodeCell =
    { title : Maybe String
    , functionTitle : Maybe String
    , code : String
    }


type alias ChoiceCell =
    { varName : String
    , typeTitle : String
    , typeDescription : Maybe String
    , functionChoices : List FunctionChoice
    , selectedFunctionChoice : Maybe Int
    }


type Cell
    = Code CodeCell
    | Choice ChoiceCell
