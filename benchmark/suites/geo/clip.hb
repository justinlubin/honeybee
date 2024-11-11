(facts
  (InputRaster
    (.name "NED 10m DEM")
    (.data "ned10.tif")
    (.resolution 10)
    (.crs "EPSG:32610")
    (.bands 1)
    (.sensor "unknown"))
  (InputVector
    (.name "Conterminous United States (CONUS)")
    (.data "conus.geojson")
    (.crs "EPSG:4326")))

(goal
  (Raster
    (.name "Clipped DEM")
    (.resolution 10)
    (.crs "EPSG:26943")
    (.bands 1)))