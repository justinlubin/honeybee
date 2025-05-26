////////////////////////////////////////////////////////////////////////////////
// Honeybee loading

import init, * as Honeybee from "./pkg/honeybee.js";

await init();

const libraryResponse = await fetch("std-bio.hblib.toml");
const librarySource = await libraryResponse.text();
const library = Honeybee.parse_library(librarySource);

const flags = { props: {}, types: {} };

function loadFact(kvs) {
    let overview = kvs.info.get("overview");

    if (overview === undefined) {
        overview = null;
    }

    let paramLabels = kvs.info.get("params");
    if (paramLabels) {
        paramLabels = Object.fromEntries(paramLabels);
    } else {
        paramLabels = {};
    }

    return {
        params: Object.fromEntries(kvs.params),
        overview: overview,
        paramLabels: paramLabels,
    };
}

for (const [name, kvs] of library.Prop) {
    flags.props[name] = loadFact(kvs);
}

for (const [name, kvs] of library.Type) {
    flags.types[name] = loadFact(kvs);
}

console.log(flags);

////////////////////////////////////////////////////////////////////////////////
// Custom elements

customElements.define(
    "fancy-code",
    class extends HTMLElement {
        constructor() {
            super();
            this._code = null;
        }

        set code(value) {
            this._code = value;

            const preElement = document.createElement("pre");
            const codeElement = document.createElement("code");

            const language = this.getAttribute("language");
            if (language) {
                codeElement.className = "language-" + language;
            }

            codeElement.innerText = this._code;

            Prism.highlightElement(codeElement);

            this.textContent = "";
            preElement.appendChild(codeElement);
            this.appendChild(preElement);
        }

        get code() {
            return this._code;
        }
    },
);

////////////////////////////////////////////////////////////////////////////////
// Elm initialization

const app = Elm.Main.init({
    node: document.getElementById("app"),
    flags: flags,
});

////////////////////////////////////////////////////////////////////////////////
// Elm ports

app.ports.scrollIntoView.subscribe((msg) => {
    document.querySelector(msg.selector).scrollIntoView({ behavior: "smooth" });
});

app.ports.sendPbnCheck.subscribe((msg) => {
    try {
        const validGoalMetadataMessage = Honeybee.valid_goal_metadata(
            librarySource,
            msg.programSource,
        );
        validGoalMetadataMessage.choices = validGoalMetadataMessage.choices.map(
            (m) => Object.fromEntries(m),
        );
        console.log(validGoalMetadataMessage);
        app.ports.receiveValidGoalMetadata.send(validGoalMetadataMessage);
    } catch (e) {
        console.error(e);
    }
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
