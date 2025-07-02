import typescriptLogo from "./typescript.svg";
import viteLogo from "/vite.svg";
import { setupCounter } from "./counter.ts";
import * as icebrk from "icebrk";

import "./style.scss";

// Poor man's JQuery
const $ = function(id: string): Element { return document.getElementById(id)};

const form = $("form");

const platformInput = $("platform-input");
const versionInput = $("version-input");

const inquiryNumberInput = $("inquiry-number-input");

const dateFieldset = $("date-fieldset");
const dayInput = $("day-input");
const monthInput = $("month-input");

const deviceIdFieldset = $("device-id-fieldset");

let masterKeyLabel = $("master-key-label");

enum AlgorithmVersion {
  v0,
  v1,
  v2,
  v3,
  v4,
}

let currentPlatform: icebrk.Platform = null;
let currentAlgorithmVersion: AlgorithmVersion = null;

function setElementVisibility(element: Element, enable: bool) {
  if (enable) {
    element.disabled = false;
    element.style.display = "";
  } else {
    element.disabled = true;
    element.style.display = "none";
  }
}

function setExtendedInquiryNumber(value: bool) {
  if (value) {
    inquiryNumberInput.max = "9999999999";
  } else {
    inquiryNumberInput.max = "99999999";
  }
}

setElementVisibility(dateFieldset, false);
setElementVisibility(deviceIdFieldset, false);
setExtendedInquiryNumber(false);

const versionRangesMap = {
  wii: [["1.0", "4.3"]],

  dsi: [["1.1", "1.4.5"]],

  wiiu: [
    ["1.0.0", "4.1.0", AlgorithmVersion.v0],
    ["5.0.0", "5.5.5", AlgorithmVersion.v2],
  ],

  "3ds": [
    ["1.0.0", "6.3.0", AlgorithmVersion.v0],
    ["7.0.0", "7.1.0", AlgorithmVersion.v1],
    ["7.2.0", "11.15.0", AlgorithmVersion.v2],
  ],

  switch: [
    ["1.0.0", "7.0.1", AlgorithmVersion.v3],
    ["8.0.0", "14.1.2", AlgorithmVersion.v4],
  ],
};

const platformMap = {
  wii: icebrk.Platform.Wii,
  dsi: icebrk.Platform.Dsi,
  wiiu: icebrk.Platform.Wiiu,
  "3ds": icebrk.Platform.The3ds,
  switch: icebrk.Platform.Switch,
};

platformInput.addEventListener("change", (e) => {
  const value = platformInput.options[platformInput.selectedIndex].value;

  currentPlatform = platformMap[value];
  const versions = versionRangesMap[value];

  let options = [];

  function newOption(value: string): Element {
    let option = document.createElement("option");
    option.appendChild(document.createTextNode(value));

    return option;
  }

  if (versions.length > 1) {
    let option = newOption("Select a version...");
    option.selected = true;
    option.disabled = true;

    options.push(option);
  }

  versions.forEach((version) => {
    let option = newOption(`From ${version[0]} to ${version[1]}`);
    let algorithmVersion = version[2];

    if (algorithmVersion !== undefined) {
      option.value = algorithmVersion;
    }

    options.push(option);
  });

  versionInput.replaceChildren(...options);

  if (value === "wii" || value === "dsi") {
    setElementVisibility(dateFieldset, true);
    setElementVisibility(deviceIdFieldset, false);
    setExtendedInquiryNumber(false);

    currentAlgorithmVersion = AlgorithmVersion.v0;

    return;
  }

  setElementVisibility(dateFieldset, false);
  setElementVisibility(deviceIdFieldset, false);
});

versionInput.addEventListener("change", (e) => {
  const value = versionInput.options[versionInput.selectedIndex].value;
  // The internal number of the enum is coerced into a string when put on a HTML tag
  currentAlgorithmVersion = Number(value);

  setExtendedInquiryNumber(value !== "v0");

  switch (currentAlgorithmVersion) {
    case AlgorithmVersion.v0:
    case AlgorithmVersion.v1:
    case AlgorithmVersion.v2:
      setElementVisibility(dateFieldset, true);
      setElementVisibility(deviceIdFieldset, false);
      break;

    case AlgorithmVersion.v3:
      setElementVisibility(dateFieldset, false);
      setElementVisibility(deviceIdFieldset, false);
      break;

    case AlgorithmVersion.v4:
      setElementVisibility(dateFieldset, false);
      setElementVisibility(deviceIdFieldset, true);
      break;
  }
});

form.addEventListener("submit", (e) => {
	e.preventDefault();

  switch (currentAlgorithmVersion) {
    case AlgorithmVersion.v0:
      masterKeyLabel.textContent = icebrk.calculate_v0_master_key(
        currentPlatform,
        inquiryNumberInput.value,
        dayInput.value,
        monthInput.value,
      );
      break;
  }
});
