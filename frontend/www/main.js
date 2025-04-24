const app = Elm.Main.init({
    node: document.getElementById("app"),
});

import init, * as Honeybee from "./pkg/honeybee.js";

await init();

alert(Honeybee.greet("World"));

const libraryRes = await fetch("bio.hblib.toml");
const librarySrc = await libraryRes.text();
console.log(librarySrc);
// const library = Honeybee.parse_library(librarySrc);
