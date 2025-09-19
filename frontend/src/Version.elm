module Version exposing (fullVersion)


shortVersion : String
shortVersion =
    "0.3.0"


fullVersion : String
fullVersion =
    shortVersion ++ "+<<<COMMIT-SHORT-HASH>>>"
