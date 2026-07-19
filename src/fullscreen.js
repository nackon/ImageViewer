/**
 * Toggle the fullscreen state of a window.
 * @param {{isFullscreen: () => Promise<boolean>, setFullscreen: (value: boolean) => Promise<void>}} win
 *   Window-like object (e.g. Tauri's getCurrentWindow())
 * @returns {Promise<boolean>} The new fullscreen state
 */
export async function toggleFullscreen(win) {
  const isFullscreen = await win.isFullscreen();
  const next = !isFullscreen;
  await win.setFullscreen(next);
  return next;
}

/**
 * Decide what the Escape key should do based on the fullscreen state.
 * In fullscreen, Escape exits fullscreen instead of closing the app.
 * @param {boolean} isFullscreen - Current fullscreen state
 * @returns {'exit-fullscreen' | 'close'}
 */
export function escapeAction(isFullscreen) {
  return isFullscreen ? 'exit-fullscreen' : 'close';
}
