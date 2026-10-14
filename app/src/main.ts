import "./style.css";
import {validateScreenshot} from "./validate";

const fileInput = document.querySelector<HTMLInputElement>("#screenshot");
const dprInput = document.querySelector<HTMLSelectElement>("#dpr"); //dpr stands for device pixel ratio
const analyzeButton = document.querySelector<HTMLButtonElement>("#analyze");
const status = document.querySelector<HTMLParagraphElement>("#status");
const preview = document.querySelector<HTMLImageElement>("#preview");

if (!fileInput || !dprInput || !analyzeButton || !status || !preview) {
  throw new Error("ClarityLens UI elements not found.");
} // if any required element is missing , stop the app with clear error

function getSelectedDpr(): 1 | 2 | 3 {
  const value = Number(dprInput.value);

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
  status.textContent = `Screenshot ready. Device pixel ratio: ${getSelectedDpr()}x.`;
  preview.src = URL.createObjectURL(file);
  preview.hidden = false;
  analyzeButton.disabled = false;
});