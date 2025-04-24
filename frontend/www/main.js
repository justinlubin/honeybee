const app = Elm.Main.init({
    node: document.getElementById("app"),
});

import init, * as Honeybee from "./pkg/honeybee.js";

await init();

const libraryRes = await fetch("bio.hblib.toml");
const librarySrc = await libraryRes.text();
const library = Honeybee.parse_library(librarySrc);
window._lib = library;

for (const [name, { params }] of library.Ok.Prop) {
    console.log(name, params);
}

for (const [name, { params }] of library.Ok.Type) {
    console.log(name, params);
}
