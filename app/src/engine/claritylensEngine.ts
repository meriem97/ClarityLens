import init, {
  inspect_image_bytes,
} from "../../engine/claritylens-engine/pkg/claritylens_engine.js";

export type ImageByteInspection = {
  status: string;
  message: string;
};

export type LuminanceInspection = {
  status: "luminance_ready";
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
};

let wasmReady: Promise<unknown> | null = null;

function ensureWasmReady() {
  if (!wasmReady) {
    wasmReady = init();
  }

  return wasmReady;
}

export async function inspectImageBytes(
  bytes: Uint8ClampedArray,
  width: number,
  height: number,
): Promise<LuminanceInspection> {
  await ensureWasmReady();

  const raw = inspect_image_bytes(
    bytes,
    width,
    height,
  );

  console.log("Raw WASM result:", raw);
  console.log("Raw WASM keys:", Object.keys(raw));

  return raw as unknown as LuminanceInspection;
}


export type ContrastInspection = {
  status: "contrast_ready";
  foreground_luminance: number;
  background_luminance: number;
  contrast_ratio: number;
  normal_text_passes: boolean;
  large_text_passes: boolean;
};

export type LuminanceResult = {
  status: "luminance_ready";
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
};