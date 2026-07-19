import { describe, it, expect } from 'vitest';
import { SHORTCUTS, shortcutsForContext, nextShortcutsHelpVisibility } from './shortcuts.js';

describe('SHORTCUTS', () => {
  it('has a non-empty list', () => {
    expect(SHORTCUTS.length).toBeGreaterThan(0);
  });

  it('every entry has keys, a description, and a valid context', () => {
    for (const shortcut of SHORTCUTS) {
      expect(shortcut.keys).toBeTruthy();
      expect(shortcut.description).toBeTruthy();
      expect(['image', 'thumbnail', 'global']).toContain(shortcut.context);
    }
  });

  it('includes the help shortcut itself', () => {
    expect(SHORTCUTS.some((s) => s.keys === '?')).toBe(true);
  });
});

describe('shortcutsForContext', () => {
  it('includes image-specific and global shortcuts for "image"', () => {
    const result = shortcutsForContext('image');
    expect(result.some((s) => s.description === 'Next image')).toBe(true);
    expect(result.some((s) => s.description === 'Toggle fullscreen')).toBe(true);
    expect(result.some((s) => s.description === 'Move thumbnail selection')).toBe(false);
  });

  it('includes thumbnail-specific and global shortcuts for "thumbnail"', () => {
    const result = shortcutsForContext('thumbnail');
    expect(result.some((s) => s.description === 'Move thumbnail selection')).toBe(true);
    expect(result.some((s) => s.description === 'Toggle fullscreen')).toBe(true);
    expect(result.some((s) => s.description === 'Next image')).toBe(false);
  });

  it('returns only global shortcuts for an unknown context', () => {
    const result = shortcutsForContext('bogus');
    expect(result.length).toBeGreaterThan(0);
    expect(result.every((s) => s.context === 'global')).toBe(true);
  });
});

describe('nextShortcutsHelpVisibility', () => {
  it('opens the overlay when "?" is pressed while closed', () => {
    expect(nextShortcutsHelpVisibility(false, '?')).toBe(true);
  });

  it('closes the overlay when "?" is pressed while open', () => {
    expect(nextShortcutsHelpVisibility(true, '?')).toBe(false);
  });

  it('closes the overlay on Escape when open', () => {
    expect(nextShortcutsHelpVisibility(true, 'Escape')).toBe(false);
  });

  it('leaves the overlay closed on Escape when already closed', () => {
    expect(nextShortcutsHelpVisibility(false, 'Escape')).toBe(false);
  });

  it('leaves visibility unchanged for unrelated keys', () => {
    expect(nextShortcutsHelpVisibility(false, 'a')).toBe(false);
    expect(nextShortcutsHelpVisibility(true, 'a')).toBe(true);
  });
});
