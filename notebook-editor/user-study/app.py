import base64
import zlib
import json
import flask
from flask import Flask
import time

app = Flask(__name__)


@app.get("/<path:path>")
def simple(path):
    return flask.send_from_directory(
        directory="../www",
        path=path,
    )


@app.post("/__log")
def log():
    body = flask.request.json
    body["timestamp"] = round(time.time() * 1000)
    line = (
        base64.a85encode(zlib.compress(json.dumps(body).encode()))
        + "\n".encode()
    )
    with open("log.txt", "ab") as f:
        f.write(line)

    return flask.Response(status=204)


@app.route("/")
def home():
    return simple("index.html")
