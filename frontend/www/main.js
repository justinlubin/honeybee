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
        app.ports.iPbnStatus_.send(pbnStatusMessage);
    } catch (e) {
        alert(
            `Honeybee cannot figure out how to make an analysis script for this experiment.

Here are some things to try:

1. Make sure there are no missing steps in your experimental workflow or typos in your descriptions of the steps.

2. Make sure your selected goal is actually the goal you have for the experiment.

3. Make sure your selected goal can actually be achieved using the steps in your experiment.

If none of these steps help, it is likely that the Honeybee library does not (yet!) include the comptuational steps you need.

In any case, please feel free reach out to Justin at justinlubin@berkeley.edu with a screenshot of this page for help! ☺`,
        );
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

////////////////////////////////////////////////////////////////////////////////
// Smooth-scroll clicks

// https://stackoverflow.com/a/33616981
document.addEventListener("click", (e) => {
    const target = e.target.closest("a");
    if (target) {
        const href = target.getAttribute("href");
        if (href?.startsWith("#")) {
            e.preventDefault();
            document.querySelector(href).scrollIntoView({ behavior: "smooth" });
        }
    }
});

////////////////////////////////////////////////////////////////////////////////
// Animations

// https://developer.mozilla.org/en-US/docs/Web/API/MutationObserver

let seen = new Set();

const target = document.querySelector("#navigation-pane .pane-body");

document.getElementById("start-navigating").addEventListener("click", () => {
    seen = new Set();
});

function findElementToFocus(target) {
    for (const node of target.childNodes) {
        if (!node.classList?.contains("cell-code")) {
            continue;
        }
        if (node.dataset.key.startsWith("from dataclasses")) {
            continue;
        }
        if (seen.has(node.dataset.key)) {
            continue;
        }
        return node;
    }
    return null;
}

const observer = new MutationObserver((_mutations, _obs) => {
    const el = findElementToFocus(target);
    if (el) {
        seen.add(el.dataset.key);
        el.scrollIntoView({ behavior: "instant" });
        el.classList.add("just-added");
        window.setTimeout(() => {
            el.classList.remove("just-added");
        }, 500);
    }
});

observer.observe(target, {
    childList: true,
});
