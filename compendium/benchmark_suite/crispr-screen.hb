(facts
  (Transfect (.sample "A") (.at 1) (.library "library.csv"))
  (Seq (.sample "A") (.at 2) (.data "initial.fastq"))
  (Seq (.sample "A") (.at 3) (.data "final.fastq")))

(goal
  (GrowthPhenotype (.sample "A") (.from 2) (.to 3)))
