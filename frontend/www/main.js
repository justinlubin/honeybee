import init, * as Honeybee from "./pkg/honeybee.js";

await init();

const libraryResponse = await fetch("bio.hblib.toml");
const librarySource = await libraryResponse.text();
const library = Honeybee.parse_library(librarySource);
window._lib = library;

const flags = { props: {}, types: {} };

for (const [name, { params }] of library.Prop) {
    flags.props[name] = { params: Object.fromEntries(params) };
}

for (const [name, { params }] of library.Type) {
    flags.types[name] = { params: Object.fromEntries(params) };
}

const app = Elm.Main.init({
    node: document.getElementById("app"),
    flags: flags,
});

app.ports.sendPbnInit.subscribe((msg) => {
    try {
        const pbnStatusMessage = Honeybee.pbn_init(
            librarySource,
            msg.programSource,
        );
        app.ports.receivePbnStatus.send(pbnStatusMessage);
    } catch (e) {
        console.error(e);
    }
});

app.ports.sendPbnChoice.subscribe((msg) => {
    try {
        const pbnStatusMessage = Honeybee.pbn_choose(msg.choice);
        app.ports.receivePbnStatus.send(pbnStatusMessage);
    } catch (e) {
        console.error(e);
    }
});

// https://stackoverflow.com/a/18197341
function download(filename, text) {
    const element = document.createElement("a");
    element.setAttribute(
        "href",
        "data:text/plain;charset=utf-8," + encodeURIComponent(text),
    );
    element.setAttribute("download", filename);

    element.style.display = "none";
    document.body.appendChild(element);

    element.click();

    document.body.removeChild(element);
}

app.ports.sendDownload.subscribe((msg) => {
    download(msg.filename, msg.text);
});
