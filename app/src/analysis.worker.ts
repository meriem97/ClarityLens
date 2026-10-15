self.addEventListener("message", (event: MessageEvent) => {
  const { imageData, devicePixelRatio } = event.data;

  self.postMessage({
    status: "success",
    message: `Received image data: ${imageData.width} * ${imageData.height} at ${devicePixelRatio}x.`,
  });
});