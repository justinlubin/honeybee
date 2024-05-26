(annotations
  (Transfect (.sample "A") (.at 1) (.library "library.csv"))
  (Sort (.in-sample "A") (.on-sample "B") (.off-sample "C") (.at 5))
  (Seq (.sample "B") (.at 5) (.data "initial.fastq"))
  (Seq (.sample "C") (.at 5) (.data "final.fastq")))

(goal
  (Growth-Between (.sample1 "B") (.sample2 "C") (.at 5)))
