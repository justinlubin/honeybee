module Version exposing (fullVersion, stable)

-- Note: The <<<X>>> tags in this file get replaced after compilation; see the
--       Makefile for how this gets done!


shortVersion : String
shortVersion =
    "0.4.0"


fullVersion : String
fullVersion =
    shortVersion ++ "+<<<COMMIT-SHORT-HASH>>>"


stable : Bool
stable =
    "<<<UNSTABLE-INDICATOR>>>" == "STABLE"
