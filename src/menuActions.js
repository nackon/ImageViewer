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
  'open-file': 'openFile',
  'open-folder': 'openFolder',
};

/**
 * Dispatches a menu action id to the matching handler.
 * @param {string} actionId - Id emitted by the Rust `menu-command` event.
 * @param {Object<string, Function>} handlers - Map of handler name to function.
 * @returns {*} The handler's return value, or undefined if no handler matched.
 */
export function handleMenuAction(actionId, handlers) {
  const handlerName = MENU_ACTION_HANDLERS[actionId];
  if (!handlerName) {
    return undefined;
  }

  const handler = handlers[handlerName];
  if (typeof handler !== 'function') {
    return undefined;
  }

  return handler();
}
