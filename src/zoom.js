/**
 * Calculate the scale factor to fit an image within a container
 * @param {number} imageWidth - Natural width of the image
 * @param {number} imageHeight - Natural height of the image
 * @param {number} containerWidth - Width of the container
 * @param {number} containerHeight - Height of the container
 * @returns {number} Scale factor (never exceeds 1.0)
 */
export function calculateFitScale(imageWidth, imageHeight, containerWidth, containerHeight) {
  if (imageWidth <= 0 || imageHeight <= 0 || containerWidth <= 0 || containerHeight <= 0) {
    return 1.0;
  }

  const scaleX = containerWidth / imageWidth;
  const scaleY = containerHeight / imageHeight;

  // Never scale up beyond 100%
  return Math.min(scaleX, scaleY, 1.0);
}

/**
 * Apply zoom in operation
 * @param {number} currentScale - Current scale factor
 * @param {number} factor - Zoom factor (default 1.2)
 * @param {number} maxScale - Maximum allowed scale (default 5.0)
 * @returns {number} New scale factor
 */
export function applyZoomIn(currentScale, factor = 1.2, maxScale = 5.0) {
  return Math.min(currentScale * factor, maxScale);
}

/**
 * Apply zoom out operation
 * @param {number} currentScale - Current scale factor
 * @param {number} factor - Zoom factor (default 1.2)
 * @param {number} minScale - Minimum allowed scale (default 0.1)
 * @returns {number} New scale factor
 */
export function applyZoomOut(currentScale, factor = 1.2, minScale = 0.1) {
  return Math.max(currentScale / factor, minScale);
}

/**
 * Format zoom percentage for display
 * @param {number} scale - Scale factor
 * @returns {string} Formatted percentage string
 */
export function formatZoomPercentage(scale) {
  return `${Math.round(scale * 100)}%`;
}
