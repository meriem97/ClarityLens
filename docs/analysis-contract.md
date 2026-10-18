# ClarityLens Analysis Contract

## Version

v0

## Current milestone

Step 5.8 — Define the next analysis contract

## First measurement

ClarityLens v0 will first calculate average relative luminance from the normalized screenshot pixels.

This measurement describes the average brightness of the screenshot. It does not determine whether the interface is usable, readable, accessible, or visually well-structured.

## Input

The WASM engine receives:

- `bytes`: normalized screenshot pixels in RGBA order.
- `width`: normalized image width in pixels.
- `height`: normalized image height in pixels.

The expected byte count is:

```text
width × height × 4
```

The four bytes for each pixel are:

```text
R, G, B, A
```

## Input validation

The engine must reject the input when:

```text
bytes.length !== width × height × 4
```

The engine must also reject zero width or zero height.

## Color space

The input is treated as 8-bit sRGB.

Each RGB channel is normalized:

```text
channel_srgb = channel_8bit / 255
```

## Luminance formula

For each normalized sRGB channel:

```text
if channel_srgb <= 0.04045:
    channel_linear = channel_srgb / 12.92
else:
    channel_linear = ((channel_srgb + 0.055) / 1.055) ^ 2.4
```

For each pixel:

```text
pixel_luminance =
    0.2126 × red_linear +
    0.7152 × green_linear +
    0.0722 × blue_linear
```

The formula follows the WCAG relative-luminance definition.

## Alpha policy

Alpha is ignored in v0.

The engine calculates luminance from RGB regardless of the alpha byte.

## Average

The engine calculates:

```text
average_luminance =
    sum(pixel_luminance) / pixel_count
```

The internal calculation uses `f64`.

The final returned value is rounded to four decimal places.

## Output contract

The future WASM result will have this shape:

```json
{
  "status": "luminance_ready",
  "width": 390,
  "height": 845,
  "pixel_count": 329550,
  "average_luminance": 0.4821
}
```

Fields:

- `status`: `"luminance_ready"` when calculation succeeds.
- `width`: normalized image width.
- `height`: normalized image height.
- `pixel_count`: `width × height`.
- `average_luminance`: number between `0.0` and `1.0`.

## Expected test cases

| Input | Expected result |
|---|---:|
| Black pixel `(0, 0, 0, 255)` | `0.0` |
| White pixel `(255, 255, 255, 255)` | `1.0` |
| Red pixel `(255, 0, 0, 255)` | Approximately `0.2126` |
| Green pixel `(0, 255, 0, 255)` | Approximately `0.7152` |
| Blue pixel `(0, 0, 255, 255)` | Approximately `0.0722` |
| Black and white pixels | `0.5` |
| Transparent red pixel | Same as opaque red because alpha is ignored |

## Limitations

Average luminance does not identify:

- Text.
- Buttons.
- Background regions.
- Contrast failures.
- Visual hierarchy.
- Usability problems.
- Accessibility compliance.
- Conversion problems.

It is a deterministic visual measurement and not a complete UX diagnosis.