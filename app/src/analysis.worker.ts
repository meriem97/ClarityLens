import { inspectImageBytes } from "./engine/claritylensEngine";

type AnalysisRequest = {
  pixels: Uint8ClampedArray;
  width: number;
  height: number;
  devicePixelRatio: 1 | 2 | 3;
};

self.addEventListener("message", async (event: MessageEvent<AnalysisRequest>) => {
  try {
    const {
      pixels,
      width,
      height,
      devicePixelRatio,
    } = event.data;

    const result = await inspectImageBytes(
      pixels,
      width,
      height,
    );

    self.postMessage({
      type: "success",
      result: {
        ...result,
        devicePixelRatio,
      },
    });
  } catch (error) {
    const message =
      error instanceof Error
        ? error.message
        : "WASM analysis failed.";

    self.postMessage({
      type: "error",
      message,
    });
  }
});