# import zlib
# import base64

import polars as pl

line_dtype = pl.Struct(
    [
        pl.Field("logfmt", pl.Int64),
        pl.Field("build", pl.String),
        pl.Field("partid", pl.String),
        pl.Field("sessionid", pl.String),
        pl.Field("timestamp", pl.Int64),
        pl.Field("sound", pl.Boolean),
        pl.Field("msg", pl.String),
    ]
)


def decode(line: str) -> str:
    # return zlib.decompress(base64.a85decode(line)).decode()
    return line


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
    .with_columns(
        pl.from_epoch(
            "timestamp",
            time_unit="ms",
        ).dt.convert_time_zone(
            time_zone="America/Los_Angeles",
        )
    )
)

print(
    data.filter(pl.col("logfmt") == 0).filter(
        pl.col("msg").str.starts_with("UserStartedNavigation")
        # | pl.col("msg").str.starts_with("Backend")
        | pl.col("msg").str.starts_with("UserClickedUndo")
        | pl.col("msg").str.starts_with("UserMade"),
    )
)
