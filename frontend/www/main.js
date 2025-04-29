import init, * as Honeybee from "./pkg/honeybee.js";

await init();

const libraryResponse = await fetch("bio.hblib.toml");
const librarySource = await libraryResponse.text();
const library = Honeybee.parse_library(librarySource);
window._lib = library;

const flags = { props: {}, types: {} };

for (const [name, { params }] of library.Ok.Prop) {
    flags.props[name] = { params: Object.fromEntries(params) };
}

for (const [name, { params }] of library.Ok.Type) {
    flags.types[name] = { params: Object.fromEntries(params) };
}

const app = Elm.Main.init({
    node: document.getElementById("app"),
    flags: flags,
});

app.ports.send.subscribe((msg) => {
    try {
        const synthesisResult = Honeybee.autopilot(
            librarySource,
            msg.programSource,
        );
        app.ports.receive.send({
            synthesisResult: synthesisResult,
        });
    } catch (e) {
        console.error(e);
    }
});
