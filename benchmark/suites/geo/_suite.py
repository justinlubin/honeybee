from dataclasses import dataclass

import rasterio
from rasterio.io import DatasetReader

@dataclass
class InputRaster:
    name: str
    data: str
    resolution: int
    projection: str
    bands: int
    sensor: str

@dataclass
class Raster:
    @dataclass
    class M:
        name: str
        resolution: int
        projection: str
        bands: int

    @dataclass
    class D:
        dataset: DatasetReader

    m: M
    d: D

def load_raster(raster: InputRaster) -> Raster.D:
    dataset = rasterio.open(raster.data)
    return dataset

@dataclass
class InputResamplingMethod:
    method: str

@dataclass
class ResamplingMethod:
    @dataclass
    class M:
        method: str

    @dataclass
    class D:
        method: str
    
    m: M
    d: D

# Resampling

def nearest() -> ResamplingMethod.D:
    return ResamplingMethod.D(method="nearest")

def bilinear() -> ResamplingMethod.D:
    return ResamplingMethod.D(method="bilinear")

def cubic() -> ResamplingMethod.D:
    return ResamplingMethod.D(method="cubic")

def cubic_spline() -> ResamplingMethod.D:
    return ResamplingMethod.D(method="cubic_spline")

def lanczos() -> ResamplingMethod.D:
    return ResamplingMethod.D(method="lanczos")

def average() -> ResamplingMethod.D:
    return ResamplingMethod.D(method="average")

def mode() -> ResamplingMethod.D:
    return ResamplingMethod.D(method="mode")

def gauss() -> ResamplingMethod.D:
    return ResamplingMethod.D(method="gauss")

def max() -> ResamplingMethod.D:
    return ResamplingMethod.D(method="max")

def min() -> ResamplingMethod.D:
    return ResamplingMethod.D(method="min")

def med() -> ResamplingMethod.D:
    return ResamplingMethod.D(method="med")

def q1() -> ResamplingMethod.D:
    return ResamplingMethod.D(method="q1")

def q3() -> ResamplingMethod.D:
    return ResamplingMethod.D(method="q3")

def sum() -> ResamplingMethod.D:
    return ResamplingMethod.D(method="sum")

def rms() -> ResamplingMethod.D:
    return ResamplingMethod.D(method="rms")

def resample(raster: Raster, resolution: int, method: ResamplingMethod) -> Raster.D:
    dataset = ...
    return Raster.D(dataset=dataset)

# Reprojection

def warp(raster: Raster, projection: str) -> Raster.D:
    dataset = ...
    return Raster.D(dataset=dataset)

# Remote Sensing

def ndvi(raster: Raster, red: int, nir: int) -> Raster.D:
    dataset = ...
    return Raster.D(dataset=dataset)

def ndwi(raster: Raster, green: int, nir: int) -> Raster.D:
    dataset = ...
    return Raster.D(dataset=dataset)