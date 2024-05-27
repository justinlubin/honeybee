(facts
  (Seq (.sample "healthy") (.at 1) (.data "healthy.fastq"))
  (Seq (.sample "tle") (.at 1) (.data "cri.fastq")))

(goal
  (DifferentialGeneExpression
    (.sample1 "healthy") (.sample2 "tle") (.at 1)))
