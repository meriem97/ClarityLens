import "./style.css";
import {validateScreenshot} from "./validate";
import {normalizeScreenshot} from "./normalize"
import AnalysisWorker from "./analysis.worker?worker"



const fileInput = document.querySelector<HTMLInputElement>("#screenshot");
const dprInput = document.querySelector<HTMLSelectElement>("#dpr"); //dpr stands for device pixel ratio
const analyzeButton = document.querySelector<HTMLButtonElement>("#analyze");
const status = document.querySelector<HTMLParagraphElement>("#status");
const preview = document.querySelector<HTMLImageElement>("#preview");

let selectedFile:File | null=null

if (!fileInput || !dprInput || !analyzeButton || !status || !preview) {
  throw new Error("ClarityLens UI elements not found.");
} // if any required element is missing , stop the app with clear error

function getSelectedDpr(select:HTMLSelectElement): 1 | 2 | 3 {
  const value = Number(select.value);

  if (value === 1 || value === 3) {
    return value;
  }

  return 2;
}
//function getSelectedDpr2() {
  //const value = Number(dprInput.value);
  //return value;

//} you may say values from the selection menu are already 1 or 2 or 3 only and that this getSelectedDpr2() is enough and that the if statement is useless
//BUT even using standard developer tools ( like inspect in the right click menu) allows the user to manipulate the value , so even if he change vlue to 6 on inspection tool it will select 2 by default

fileInput.addEventListener("change", () => {
  const file = fileInput.files?.[0];

  if (!file) {
    status.textContent = "";
    preview.hidden = true;
    preview.src = "";
    analyzeButton.disabled = true;
    return;
  }
  const result = validateScreenshot(file);
  if (!result.valid) {
    status.textContent = result.message;
    preview.hidden = true;
    preview.src = "";
    analyzeButton.disabled = true;
    return;
  }

  selectedFile = result.file

  status.textContent = `Screenshot ready. Device pixel ratio: ${getSelectedDpr(dprInput)}x.`;
  preview.src = URL.createObjectURL(selectedFile);
  preview.hidden = false;
  analyzeButton.disabled = false;
});

analyzeButton.addEventListener("click", async () => {
  if (!selectedFile) {
    status.textContent = "Please select a valid screenshot first.";
    return;
  }

  analyzeButton.disabled = true;
  status.textContent = "Preparing analysis…";

  const worker = new AnalysisWorker();

  worker.onmessage = (event: MessageEvent) => {
  console.log("Worker response:", event.data);

  const data = event.data;

  if (data.type === "success") {
    const result = data.result;

    status.textContent =
      `Average luminance: ${result.average_luminance.toFixed(4)}`;
  } else if (data.type === "error") {
    status.textContent =
      `Analysis failed: ${data.message}`;
  } else {
    status.textContent =
      "Received an unknown analysis response.";
  }

  worker.terminate();
  analyzeButton.disabled = false;
};

  worker.onerror = (error) => {
  console.error("Analysis Worker error:", error);

  status.textContent =
    "Analysis could not complete. Please try again.";

  worker.terminate();
  analyzeButton.disabled = false;
};

  try {
  const normalized = await normalizeScreenshot(
    selectedFile,
    getSelectedDpr(dprInput)
  );

  const context = normalized.canvas.getContext("2d");

  if (!context) {
    throw new Error("Canvas is not supported in this browser.");
  }

  const imageData = context.getImageData(
    0,
    0,
    normalized.width,
    normalized.height
  );

  

  

  worker.postMessage({
  pixels: imageData.data,
  width: normalized.width,
  height: normalized.height,
  devicePixelRatio: normalized.devicePixelRatio,
});
} catch (error) {
  console.error("Normalization error:", error);
  status.textContent = "Could not prepare this screenshot.";
  worker.terminate();
  analyzeButton.disabled = false;
}
});