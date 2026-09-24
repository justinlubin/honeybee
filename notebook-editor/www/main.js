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

import init, * as Honeybee from "./pkg/honeybee_js.js";

await init();

const libraryResponse = await fetch("bio.hblib.toml");
const librarySource = await libraryResponse.text();
const library = Honeybee.parse_library(librarySource);

let sound = true;
let log = false;

// Study mode (running on user study server)
if (window.location.href.includes(":5000")) {
  log = true;

  const urlParams = new URLSearchParams(window.location.search);
  const condition = urlParams.get("condition");

  if (condition == "5b7886fb8748aee0") {
    sound = true;
  } else if (condition == "368d77182b69ccb7") {
    sound = false;
  } else {
    let error = `Error\n\nPlease report the following message to the investigator.\n\nUnknown condition '${condition}'`;
    alert(error);
    throw error;
  }
}

const flags = {
  sound: sound,
  log: log,
  library: {
    props: elmify(library.Prop),
    types: elmify(library.Type),
  },
};

// https://developer.mozilla.org/en-US/docs/Web/API/MutationObserver

// let seen = new Set();

// function findElementToFocus(target) {
//   for (const node of target.childNodes) {
//     if (!node.dataset.popinkey) {
//       continue;
//     }
//     if (seen.has(node.dataset.popinkey)) {
//       continue;
//     }
//     return node;
//   }
//   return null;
// }

// customElements.define(
//   "pop-in",
//   class extends HTMLElement {
//     constructor() {
//       super();

//       const observer = new MutationObserver((_mutations, _obs) => {
//         const el = findElementToFocus(this);
//         if (el) {
//           seen.add(el.dataset.popinkey);

//           // Important to scroll before adding just-added class
//           el.scrollIntoView({ behavior: "instant" });

//           el.classList.add("just-added");
//           window.setTimeout(() => {
//             el.classList.remove("just-added");
//           }, 500);

//           window.setTimeout(() => {
//             document.querySelectorAll(".post-popin-attention").forEach((x) => {
//               x.classList.add("attention");
//               window.setTimeout(() => {
//                 x.classList.remove("attention");
//               }, 500);
//             });
//           }, 1000);
//         }
//       });

//       observer.observe(this, {
//         childList: true,
//       });
//     }
//   },
// );

////////////////////////////////////////////////////////////////////////////////
// Elm initialization

const app = Elm.Main.init({
  node: document.getElementById("app"),
  flags: flags,
});

let askBeforeLeaving = false;
window.onbeforeunload = () => {
  // Override confirmation when in development
  if (window.location.href.includes("127.0.0.1")) {
    return;
  }
  if (askBeforeLeaving) {
    return "Are you sure you would like to leave?";
  }
};

// document.getElementById("start-navigating").addEventListener("click", () => {
//   seen = new Set();
// });

////////////////////////////////////////////////////////////////////////////////
// Elm ports

app.ports.oScrollIntoView.subscribe((msg) => {
  window.setTimeout(() => {
    document.querySelector(msg.selector).scrollIntoView({ behavior: "smooth" });
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
      Honeybee.pbn_init(librarySource, msg.programSource, msg.sound),
    );
    app.ports.iPbnStatus_.send(pbnStatusMessage);
    askBeforeLeaving = true;
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

app.ports.oPbnSpeculate.subscribe((msg) => {
  try {
    const pbnStatusMessage = elmify(Honeybee.pbn_choose(msg.choice));
    Honeybee.pbn_undo();
    app.ports.iPbnSpeculativeStatus_.send(pbnStatusMessage);
  } catch (e) {
    console.error(e);
  }
});

app.ports.oPbnUndo.subscribe((_msg) => {
  try {
    const pbnStatusMessage = elmify(Honeybee.pbn_undo());
    app.ports.iPbnStatus_.send(pbnStatusMessage);
  } catch (e) {
    console.error(e);
  }
});

app.ports.oDownload.subscribe((msg) => {
  download(msg.filename, msg.text);
});

app.ports.oLog.subscribe((msg) => {
  // https://stackoverflow.com/a/47065313
  fetch("/__log", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(msg),
  });
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
// Debug functionality

window.__auto = function () {
  let pbnStatusMessage = null;
  while (true) {
    try {
      pbnStatusMessage = elmify(Honeybee.pbn_choose(0));
    } catch (e) {
      break;
    }
  }
  if (pbnStatusMessage) {
    app.ports.iPbnStatus_.send(pbnStatusMessage);
  }
};
