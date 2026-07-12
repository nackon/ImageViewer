/**
 * Maps native menu item ids (see src-tauri/src/menu.rs) to the handler name
 * that already implements the matching keyboard shortcut in main.js.
 */
export const MENU_ACTION_HANDLERS = {
  'next-image': 'nextImage',
  'previous-image': 'previousImage',
  'first-image': 'firstImage',
  'last-image': 'lastImage',
  'zoom-in': 'zoomIn',
  'zoom-out': 'zoomOut',
  'actual-size': 'actualSize',
  'fit-to-window': 'applyFitZoom',
  'toggle-thumbnails': 'toggleThumbnailView',
};

/**
 * Actions that only make sense while the image view is showing. These mirror
 * the keyboard shortcuts, which are likewise only bound while
 * `viewMode === 'image'` (see main.js) — the thumbnail view has its own,
 * unrelated bindings for the same keys (e.g. arrow keys move the grid
 * selection). Invoking them from the thumbnail view would silently mutate
 * image/zoom state behind a hidden view instead of doing nothing, as the
 * keyboard shortcut does.
 */
const IMAGE_VIEW_ONLY_ACTIONS = new Set([
  'next-image',
  'previous-image',
  'first-image',
  'last-image',
  'zoom-in',
  'zoom-out',
  'actual-size',
  'fit-to-window',
]);

/**
 * @param {string} actionId - Id emitted by the Rust `menu-command` event.
 * @returns {boolean} Whether this action should be ignored outside the image view.
 */
export function isImageViewOnlyAction(actionId) {
  return IMAGE_VIEW_ONLY_ACTIONS.has(actionId);
}

/**
 * Dispatches a menu action id to the matching handler.
 * @param {string} actionId - Id emitted by the Rust `menu-command` event.
 * @param {Object<string, Function>} handlers - Map of handler name to function.
 * @param {Object} [options]
 * @param {string} [options.viewMode='image'] - Current view mode ('image' or 'thumbnail').
 *   Image-view-only actions are ignored unless this is 'image'.
 * @returns {*} The handler's return value, or undefined if no handler ran.
 */
export function handleMenuAction(actionId, handlers, { viewMode = 'image' } = {}) {
  const handlerName = MENU_ACTION_HANDLERS[actionId];
  if (!handlerName) {
    return undefined;
  }

  if (isImageViewOnlyAction(actionId) && viewMode !== 'image') {
    return undefined;
  }

  const handler = handlers[handlerName];
  if (typeof handler !== 'function') {
    return undefined;
  }

  return handler();
}
