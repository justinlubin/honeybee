(facts
  (InputRaster
    (.name "Landsat Scene")
    (.data "LC09_L1TP_044034_20240508_20240508_02_T1.tif")
    (.resolution 30)
    (.crs "EPSG:32610")
    (.bands 11)
    (.sensor "landsat-9")))

(goal
  (Raster
    (.name "Landsat NVDI")
    (.resolution 90)
    (.crs "EPSG:26943")
    (.bands 1)))