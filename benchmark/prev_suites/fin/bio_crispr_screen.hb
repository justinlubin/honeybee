(facts
  (Transfection (.sample "A") (.at 1) (.library "library.csv"))
  (Seq (.sample "A") (.at 2) (.data "initial.fastq"))
  (Seq (.sample "A") (.at 3) (.data "final.fastq")))

(goal
  (Enrichment (.sample "A") (.start 2) (.end 3)))
