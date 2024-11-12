(facts
  (Reference (.data "hg38.fasta"))
  (Seq (.sample "Healthy") (.at 5) (.data "healthy.fastq"))
  (Seq (.sample "TLE") (.at 5) (.data "tle.fastq")))

(goal
  (DifferentialGeneExpression
    (.sample1 "Healthy") (.sample2 "TLE") (.at 5)))
