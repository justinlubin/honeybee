////////////////////////////////////////////////////////////////////////////////
// Helpers

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

function elmify(m) {
    if (m === undefined) {
        return null;
    } else if (m instanceof Map) {
        const obj = {};
        for (const [k, v] of m) {
            obj[k] = elmify(v);
        }
        return obj;
    } else if (Array.isArray(m)) {
        const arr = [];
        for (const v of m) {
            arr.push(elmify(v));
        }
        return arr;
    } else if (m instanceof Object) {
        const obj = {};
        for (const [k, v] of Object.entries(m)) {
            obj[k] = elmify(v);
        }
        return obj;
    } else {
        return m;
    }
}

////////////////////////////////////////////////////////////////////////////////
// Honeybee loading

import init, * as Honeybee from "./pkg/honeybee.js";

await init();

const libraryResponse = await fetch("std-bio.hblib.toml");
const librarySource = await libraryResponse.text();
const library = Honeybee.parse_library(librarySource);

const flags = {
    props: elmify(library.Prop),
    types: elmify(library.Type),
};

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

            codeElement.textContent = this._code;

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

app.ports.oScrollIntoView.subscribe((msg) => {
    window.setTimeout(() => {
        document
            .querySelector(msg.selector)
            .scrollIntoView({ behavior: "smooth" });
    }, 100);
});

// PBN

app.ports.oPbnCheck.subscribe((msg) => {
    try {
        const validGoalMetadataMessage = Honeybee.valid_goal_metadata(
            librarySource,
            msg.programSource,
        );
        validGoalMetadataMessage.choices = validGoalMetadataMessage.choices.map(
            (m) => Object.fromEntries(m),
        );
        app.ports.iValidGoalMetadata_.send(validGoalMetadataMessage);
    } catch (e) {
        console.error(e);
    }
});

app.ports.oPbnInit.subscribe((msg) => {
    try {
        const pbnStatusMessage = elmify(
            Honeybee.pbn_init(librarySource, msg.programSource),
        );
        console.log(pbnStatusMessage);
        app.ports.iPbnStatus_.send(pbnStatusMessage);
    } catch (e) {
        console.error(e);
    }
});

app.ports.oPbnChoose.subscribe((msg) => {
    try {
        const pbnStatusMessage = elmify(Honeybee.pbn_choose(msg.choice));
        app.ports.iPbnStatus_.send(pbnStatusMessage);
    } catch (e) {
        console.error(e);
    }
});

app.ports.oDownload.subscribe((msg) => {
    download(msg.filename, msg.text);
});
