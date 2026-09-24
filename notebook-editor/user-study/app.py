import base64
import zlib
import json
import flask
from flask import Flask
import time

app = Flask(__name__)

NEWLINE = "\n".encode()


@app.get("/<path:path>")
def simple(path):
    return flask.send_from_directory(
        directory="../www",
        path=path,
    )


@app.post("/__log")
def log():
    line = base64.a85encode(zlib.compress(flask.request.data)) + NEWLINE
    with open("log.txt", "ab") as f:
        f.write(line)

    return flask.Response(status=204)


@app.route("/")
def home():
    return simple("index.html")
