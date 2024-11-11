from pathlib import Path

import numpy as np
import rasterio
from rasterio.enums import Resampling

BAND_DESIGNATIONS = {"landsat-9": {"green": 3, "nir": 5}}

DATA = "data/ndvi"


def compute_ndvi(
    input_path: Path,
    output_path: Path,
    sensor: str,
) -> None:
    with rasterio.open(input_path) as scene:
        green = scene.read(BAND_DESIGNATIONS.get(sensor).get("green")).astype(float)
        nir = scene.read(BAND_DESIGNATIONS.get(sensor).get("nir")).astype(float)

        ndvi = np.where((nir + green) > 0.0, (nir - green) / (nir + green), 0.0)

        profile = scene.profile
        profile.update(dtype=rasterio.float32, count=1)

        with rasterio.open(output_path, "w", **profile) as dst:
            dst.write(ndvi.astype(rasterio.float32), 1)


def resample_raster(
    input_path: Path,
    output_path: Path,
    scale_factor: float,
    resampling_method: Resampling,
) -> None:
    with rasterio.open(input_path) as src:
        new_width = int(src.width * scale_factor)
        new_height = int(src.height * scale_factor)

        transform = src.transform * src.transform.scale(
            (src.width / new_width), (src.height / new_height)
        )

        profile = src.profile
        profile.update(width=new_width, height=new_height, transform=transform)

        data = src.read(
            out_shape=(src.count, new_height, new_width), resampling=resampling_method
        )

        with rasterio.open(output_path, "w", **profile) as dst:
            dst.write(data)


if __name__ == "__main__":
    scene_path = Path(f"{DATA}/LC09_L1TP_044034_20240508_20240508_02_T1.TIF").resolve()
    ndvi_path = Path(
        f"{DATA}/LC09_L1TP_044034_20240508_20240508_02_T1_NDVI.TIF"
    ).resolve()
    resampled_ndvi_path = Path(
        f"{DATA}/LC09_L1TP_044034_20240508_20240508_02_T1_NDVI_RESAMPLED.TIF"
    ).resolve()

    compute_ndvi(scene_path, ndvi_path, "landsat-9")
    resample_raster(ndvi_path, resampled_ndvi_path, 1 / 3, Resampling.bilinear)
