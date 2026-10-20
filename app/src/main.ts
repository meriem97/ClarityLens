import "./style.css";
import { validateScreenshot } from "./validate";
import { normalizeScreenshot } from "./normalize";
import AnalysisWorker from "./analysis.worker?worker";

const fileInput =
  document.querySelector<HTMLInputElement>("#screenshot");

const dprInput =
  document.querySelector<HTMLSelectElement>("#dpr");

const analyzeButton =
  document.querySelector<HTMLButtonElement>("#analyze");

const statusElement =
  document.querySelector<HTMLParagraphElement>("#status");

const preview =
  document.querySelector<HTMLImageElement>("#preview");

let selectedFile: File | null = null;

if (
  !fileInput ||
  !dprInput ||
  !analyzeButton ||
  !statusElement ||
  !preview
) {
  throw new Error("ClarityLens UI elements not found.");
}

function getSelectedDpr(
  select: HTMLSelectElement,
): 1 | 2 | 3 {
  const value = Number(select.value);

  if (value === 1 || value === 3) {
    return value;
  }

  return 2;
}

fileInput.addEventListener("change", () => {
  const file = fileInput.files?.[0];

  if (!file) {
    selectedFile = null;
    statusElement.textContent = "";
    preview.hidden = true;
    preview.src = "";
    analyzeButton.disabled = true;
    return;
  }

  const result = validateScreenshot(file);

  if (!result.valid) {
    selectedFile = null;
    statusElement.textContent = result.message;
    preview.hidden = true;
    preview.src = "";
    analyzeButton.disabled = true;
    return;
  }

  selectedFile = result.file;

  statusElement.textContent =
    `Screenshot ready. Device pixel ratio: ` +
    `${getSelectedDpr(dprInput)}x.`;

  preview.src = URL.createObjectURL(selectedFile);
  preview.hidden = false;
  analyzeButton.disabled = false;
});

analyzeButton.addEventListener("click", async () => {
  if (!selectedFile) {
    statusElement.textContent =
      "Please select a valid screenshot first.";
    return;
  }

  analyzeButton.disabled = true;
  statusElement.textContent = "Preparing analysis…";

  const worker = new AnalysisWorker();

  worker.onmessage = (event: MessageEvent) => {
    console.log("Worker response:", event.data);

    const data = event.data;

    if (data.type === "success") {
      const result = data.result;

      const brightnessLabel =
        result.brightness_level[0].toUpperCase() +
        result.brightness_level.slice(1);

      statusElement.textContent =
        `Average luminance: ` +
        `${result.average_luminance.toFixed(4)} · ` +
        `Overall brightness: ${brightnessLabel}`;
    } else if (data.type === "error") {
      statusElement.textContent =
        `Analysis failed: ${data.message}`;
    } else {
      statusElement.textContent =
        "Received an unknown analysis response.";
    }

    worker.terminate();
    analyzeButton.disabled = false;
  };

  worker.onerror = (error) => {
    console.error("Analysis Worker error:", error);

    statusElement.textContent =
      "Analysis could not complete. Please try again.";

    worker.terminate();
    analyzeButton.disabled = false;
  };

  try {
    const normalized = await normalizeScreenshot(
      selectedFile,
      getSelectedDpr(dprInput),
    );

    const context = normalized.canvas.getContext("2d");

    if (!context) {
      throw new Error(
        "Canvas is not supported in this browser.",
      );
    }

    const imageData = context.getImageData(
      0,
      0,
      normalized.width,
      normalized.height,
    );

    worker.postMessage({
      pixels: imageData.data,
      width: normalized.width,
      height: normalized.height,
      devicePixelRatio: normalized.devicePixelRatio,
    });
  } catch (error) {
    console.error("Normalization error:", error);

    statusElement.textContent =
      "Could not prepare this screenshot.";

    worker.terminate();
    analyzeButton.disabled = false;
  }
});