import { inspectImageBytes } from "./engine/claritylensEngine";

type AnalysisRequest = {
  pixels: Uint8ClampedArray;
  width: number;
  height: number;
  devicePixelRatio: 1 | 2 | 3;
};

type AnalysisResponse =
  | {
      type: "success";
      result: {
        status: string;
        width: number;
        height: number;
        pixel_count: number;

        average_luminance: number;
        brightness_level: "low" | "medium" | "high";

        sampled_color_pixel_count: number;
        average_red: number;
        average_green: number;
        average_blue: number;

        average_red_8bit: number;
        average_green_8bit: number;
        average_blue_8bit: number;

        devicePixelRatio: 1 | 2 | 3;
      };
    }
  | {
      type: "error";
      message: string;
    };

self.addEventListener(
  "message",
  async (event: MessageEvent<AnalysisRequest>) => {
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

      const response: AnalysisResponse = {
        type: "success",
        result: {
          ...result,
          devicePixelRatio,
        },
      };

      self.postMessage(response);
    } catch (error) {
      const message =
        error instanceof Error
          ? error.message
          : "WASM analysis failed.";

      const response: AnalysisResponse = {
        type: "error",
        message,
      };

      self.postMessage(response);
    }
  },
);