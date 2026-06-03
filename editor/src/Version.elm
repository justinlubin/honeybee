module Version exposing (build, stable)

-- Note: The <<<X>>> tags in this file get replaced after compilation; see the
--       Makefile for how this gets done!


build : String
build =
    "<<<COMMIT-SHORT-HASH>>>"


stable : Bool
stable =
    "<<<UNSTABLE-INDICATOR>>>" == "STABLE"
