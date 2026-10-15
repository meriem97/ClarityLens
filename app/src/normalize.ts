export interface NormalizedImage {
  canvas: HTMLCanvasElement;
  width: number;
  height: number;
  devicePixelRatio: 1 | 2 | 3;
}

export async function normalizeScreenshot(
  file: File,
  devicePixelRatio: 1 | 2 | 3
): Promise<NormalizedImage> {
  const bitmap = await createImageBitmap(file);

  const targetWidth = 390;
  const cssWidth = bitmap.width / devicePixelRatio;
  const cssHeight = bitmap.height / devicePixelRatio;

  const scale = targetWidth / cssWidth;

  const canvas = document.createElement("canvas");
  canvas.width = targetWidth;
  canvas.height = Math.round(cssHeight * scale);

  const context = canvas.getContext("2d");

  if (!context) {
    throw new Error("Canvas is not supported in this browser.");
  }

  context.drawImage(bitmap, 0, 0, canvas.width, canvas.height);

  return {
    canvas,
    width: canvas.width,
    height: canvas.height,
    devicePixelRatio,
  };
}