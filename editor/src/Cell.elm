module Cell exposing (..)

import Assoc exposing (Assoc)
import Core


type alias MetadataChoice =
    { metadata : Assoc String Core.Value
    , choiceIndex : Int
    }


type alias Hyperparameter =
    { name : String
    , default : String
    , comment : String
    }


type alias FunctionChoice =
    { functionTitle : String
    , functionDescription : Maybe String
    , code : Maybe String
    , metadataChoices : List MetadataChoice
    , selectedMetadataChoice : Int
    , googleScholarId : Maybe String
    , citation : Maybe String
    , additionalCitations : Maybe (List String)
    , hyperparameters : List Hyperparameter
    , use : Maybe String
    , pmid : Maybe String
    , search : Bool
    }


type alias CodeCell =
    { title : String
    , code : String
    , openWhenEditing : Bool
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


isChoice : Cell -> Bool
isChoice c =
    case c of
        Code _ ->
            False

        Choice _ ->
            True


key : Cell -> String
key c =
    case c of
        Code x ->
            x.code

        Choice x ->
            x.varName
