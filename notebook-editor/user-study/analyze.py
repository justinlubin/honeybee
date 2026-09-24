import zlib
import base64
import polars as pl

line_dtype = pl.Struct(
    [
        pl.Field("timestamp", pl.Int64),
        pl.Field("msg", pl.String),
    ]
)


def decode(line: str) -> str:
    return zlib.decompress(base64.a85decode(line)).decode()


data = (
    pl.read_lines(
        "log.txt",
    )
    .with_columns(
        pl.col("line")
        .map_elements(
            decode,
            return_dtype=pl.String,
        )
        .str.json_decode(line_dtype)
    )
    .unnest()
)

print(data)
