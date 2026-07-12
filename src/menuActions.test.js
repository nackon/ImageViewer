import { describe, it, expect, vi } from 'vitest';
import { handleMenuAction, MENU_ACTION_HANDLERS } from './menuActions.js';

describe('handleMenuAction', () => {
  it('calls the mapped handler for every known action id', () => {
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
});
