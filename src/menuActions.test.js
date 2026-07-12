import { describe, it, expect, vi } from 'vitest';
import {
  handleMenuAction,
  isImageViewOnlyAction,
  MENU_ACTION_HANDLERS,
} from './menuActions.js';

describe('handleMenuAction', () => {
  it('calls the mapped handler for every known action id (default image view)', () => {
    for (const [actionId, handlerName] of Object.entries(MENU_ACTION_HANDLERS)) {
      const handler = vi.fn().mockReturnValue('called');
      const result = handleMenuAction(actionId, { [handlerName]: handler });

      expect(handler).toHaveBeenCalledTimes(1);
      expect(result).toBe('called');
    }
  });

  it('does not call unrelated handlers', () => {
    const nextImage = vi.fn();
    const zoomIn = vi.fn();

    handleMenuAction('next-image', { nextImage, zoomIn });

    expect(nextImage).toHaveBeenCalledTimes(1);
    expect(zoomIn).not.toHaveBeenCalled();
  });

  it('returns undefined and does nothing for an unknown action id', () => {
    const nextImage = vi.fn();

    const result = handleMenuAction('not-a-real-action', { nextImage });

    expect(result).toBeUndefined();
    expect(nextImage).not.toHaveBeenCalled();
  });

  it('returns undefined when the mapped handler is missing', () => {
    const result = handleMenuAction('next-image', {});
    expect(result).toBeUndefined();
  });

  it('ignores image-view-only actions while in thumbnail view', () => {
    for (const [actionId, handlerName] of Object.entries(MENU_ACTION_HANDLERS)) {
      if (!isImageViewOnlyAction(actionId)) continue;

      const handler = vi.fn();
      const result = handleMenuAction(
        actionId,
        { [handlerName]: handler },
        { viewMode: 'thumbnail' }
      );

      expect(handler).not.toHaveBeenCalled();
      expect(result).toBeUndefined();
    }
  });

  it('still runs view-mode-independent actions (e.g. toggle-thumbnails) in thumbnail view', () => {
    const toggleThumbnailView = vi.fn().mockReturnValue('toggled');

    const result = handleMenuAction(
      'toggle-thumbnails',
      { toggleThumbnailView },
      { viewMode: 'thumbnail' }
    );

    expect(toggleThumbnailView).toHaveBeenCalledTimes(1);
    expect(result).toBe('toggled');
  });

  it('runs image-view-only actions when explicitly in image view', () => {
    const nextImage = vi.fn().mockReturnValue('next');

    const result = handleMenuAction('next-image', { nextImage }, { viewMode: 'image' });

    expect(nextImage).toHaveBeenCalledTimes(1);
    expect(result).toBe('next');
  });
});

describe('isImageViewOnlyAction', () => {
  it('flags navigation and zoom actions', () => {
    for (const actionId of [
      'next-image',
      'previous-image',
      'first-image',
      'last-image',
      'zoom-in',
      'zoom-out',
      'actual-size',
      'fit-to-window',
    ]) {
      expect(isImageViewOnlyAction(actionId)).toBe(true);
    }
  });

  it('does not flag toggle-thumbnails or unknown ids', () => {
    expect(isImageViewOnlyAction('toggle-thumbnails')).toBe(false);
    expect(isImageViewOnlyAction('not-a-real-action')).toBe(false);
  });
});
