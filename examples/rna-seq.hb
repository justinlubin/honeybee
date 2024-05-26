(facts
  (Seq (.sample "CRISPRoff") (.at 3) (.data "croff.fastq"))
  (Seq (.sample "CRISPRi") (.at 3) (.data "cri.fastq")))

(goal
  (DifferentialGeneExpression (.at 3)))
