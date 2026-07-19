/**
 * Keyboard shortcuts shown in the "?" help overlay.
 * `context` marks which view the shortcut applies to: 'image', 'thumbnail',
 * or 'global' (shown in both views).
 */
export const SHORTCUTS = [
  { keys: '→ / Space / N', description: 'Next image', context: 'image' },
  { keys: '← / Backspace / P', description: 'Previous image', context: 'image' },
  { keys: 'Home', description: 'First image', context: 'image' },
  { keys: 'End', description: 'Last image', context: 'image' },
  { keys: '+ / =', description: 'Zoom in', context: 'image' },
  { keys: '-', description: 'Zoom out', context: 'image' },
  { keys: '0', description: 'Actual size (100%)', context: 'image' },
  { keys: 'W', description: 'Fit to window', context: 'image' },
  { keys: '↑ ↓ ← →', description: 'Move thumbnail selection', context: 'thumbnail' },
  { keys: 'Enter', description: 'Open selected thumbnail', context: 'thumbnail' },
  { keys: 'T', description: 'Toggle thumbnail view', context: 'global' },
  { keys: 'F', description: 'Toggle fullscreen', context: 'global' },
  { keys: 'D', description: 'Cycle theme', context: 'global' },
  { keys: 'Esc', description: 'Close this help, exit fullscreen, or go back', context: 'global' },
  { keys: 'Q', description: 'Quit', context: 'global' },
  { keys: '?', description: 'Show / hide this shortcut list', context: 'global' },
];

/**
 * Shortcuts relevant to a given view, including the ones shared by both views.
 * @param {'image' | 'thumbnail'} context
 * @returns {typeof SHORTCUTS}
 */
export function shortcutsForContext(context) {
  return SHORTCUTS.filter((shortcut) => shortcut.context === context || shortcut.context === 'global');
}

/**
 * Decide the next visibility of the shortcuts help overlay for a keydown.
 * '?' always toggles it; Escape closes it if it's open; any other key
 * leaves the visibility unchanged.
 * @param {boolean} isVisible - Current visibility of the overlay
 * @param {string} key - `KeyboardEvent.key` of the pressed key
 * @returns {boolean} The next visibility
 */
export function nextShortcutsHelpVisibility(isVisible, key) {
  if (key === '?') {
    return !isVisible;
  }
  if (key === 'Escape' && isVisible) {
    return false;
  }
  return isVisible;
}
