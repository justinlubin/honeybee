import init, * as Honeybee from "./pkg/honeybee.js";

await init();

const libraryResponse = await fetch("bio.hblib.toml");
const librarySrc = await libraryResponse.text();
const library = Honeybee.parse_library(librarySrc);
window._lib = library;

const flags = { props: {}, types: {} };

for (const [name, { params }] of library.Ok.Prop) {
    flags.props[name] = { params: params };
}

for (const [name, { params }] of library.Ok.Type) {
    flags.types[name] = { params: params };
}

const app = Elm.Main.init({
    node: document.getElementById("app"),
    flags: flags,
});
