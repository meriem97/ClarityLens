export type ValidationResult =
  | { valid: true; file: File }
  | { valid: false; message: string };

const ACCEPTED_TYPES = ["image/png", "image/jpeg", "image/webp"];
const MAX_SIZE_BYTES = 10 * 1024 * 1024;

export function validateScreenshot(file: File): ValidationResult {
  if (!ACCEPTED_TYPES.includes(file.type)) {
    return {
      valid: false,
      message: "Please upload a PNG, JPG, or WebP image.",
    };
  }

  if (file.size > MAX_SIZE_BYTES) {
    return {
      valid: false,
      message: "File is too large. Please upload an image under 10 MB.",
    };
  }

  return { valid: true, file };
}