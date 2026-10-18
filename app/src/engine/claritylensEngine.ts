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
): Promise<ImageByteInspection> {
  await ensureWasmReady();

  return inspect_image_bytes(bytes, width, height) as ImageByteInspection;
}