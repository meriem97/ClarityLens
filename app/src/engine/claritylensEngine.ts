import init, {
  analyze_placeholder,
} from "../../engine/claritylens-engine/pkg/claritylens_engine.js";

export type PlaceholderAnalysis = {
  status: string;
  message: string;
};

let wasmReady: Promise<unknown> | null = null;

function ensureWasmReady() {
  if (!wasmReady) {
    wasmReady = init();
  }

  return wasmReady;
}

export async function analyzePlaceholder(
  width: number,
  height: number,
  devicePixelRatio: number,
): Promise<PlaceholderAnalysis> {
  await ensureWasmReady();

  return analyze_placeholder(
    width,
    height,
    devicePixelRatio,
  ) as PlaceholderAnalysis;
}