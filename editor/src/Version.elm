module Version exposing (fullVersion)


shortVersion : String
shortVersion =
    "0.4.0"


fullVersion : String
fullVersion =
    shortVersion ++ "+<<<COMMIT-SHORT-HASH>>>"
