(annotation Seq
  (at Int) (data Str))

(analysis VolcanoPlot
  (start Int) (end Int))

(computation foo VolcanoPlot
  ((s1 Seq) (s2 Seq))
  ((= (.start ret) (.at s1))
   (= (.end ret) (.at s2))))
