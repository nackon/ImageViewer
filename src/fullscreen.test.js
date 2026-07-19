import { describe, it, expect, vi } from 'vitest';
import { toggleFullscreen, escapeAction } from './fullscreen.js';

function createFakeWindow(initialFullscreen) {
  let fullscreen = initialFullscreen;
  return {
    isFullscreen: vi.fn(async () => fullscreen),
    setFullscreen: vi.fn(async (value) => {
      fullscreen = value;
    }),
    get state() {
      return fullscreen;
    },
  };
}

describe('toggleFullscreen', () => {
  it('should enter fullscreen when not fullscreen', async () => {
    const win = createFakeWindow(false);
    const result = await toggleFullscreen(win);
    expect(result).toBe(true);
    expect(win.setFullscreen).toHaveBeenCalledWith(true);
    expect(win.state).toBe(true);
  });

  it('should exit fullscreen when fullscreen', async () => {
    const win = createFakeWindow(true);
    const result = await toggleFullscreen(win);
    expect(result).toBe(false);
    expect(win.setFullscreen).toHaveBeenCalledWith(false);
    expect(win.state).toBe(false);
  });

  it('should return to the original state after toggling twice', async () => {
    const win = createFakeWindow(false);
    await toggleFullscreen(win);
    await toggleFullscreen(win);
    expect(win.state).toBe(false);
    expect(win.setFullscreen).toHaveBeenNthCalledWith(1, true);
    expect(win.setFullscreen).toHaveBeenNthCalledWith(2, false);
  });

  it('should query the current state before setting', async () => {
    const win = createFakeWindow(false);
    await toggleFullscreen(win);
    expect(win.isFullscreen).toHaveBeenCalledTimes(1);
  });
});

describe('escapeAction', () => {
  it('should exit fullscreen when in fullscreen', () => {
    expect(escapeAction(true)).toBe('exit-fullscreen');
  });

  it('should close the app when not in fullscreen', () => {
    expect(escapeAction(false)).toBe('close');
  });
});
