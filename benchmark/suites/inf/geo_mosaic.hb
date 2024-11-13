(facts
  (CRS (.val "EPSG:32610"))
  (CRS (.val "EPSG:32611"))

  (InputRaster
    (.name "Hillshade 1")
    (.data "hillshade-1.tif")
    (.resolution 30)
    (.crs "EPSG:32610")
    (.bands 1)
    (.sensor "unknown"))
  (InputRaster
    (.name "Hillshade 2")
    (.data "hillshade-2.tif")
    (.resolution 30)
    (.crs "EPSG:32611")
    (.bands 2)
    (.sensor "unknown")))

(goal
  (Raster
    (.name "Mosaiced Hillshade")
    (.crs "EPSG:32610")
    (.resolution 30)
    (.bands 1)))
