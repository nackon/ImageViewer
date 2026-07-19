import { describe, it, expect, beforeAll } from 'vitest';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

// Regression test for issue #39: the shortcuts help overlay was visible on
// startup (and pressing Escape there quit the app instead of closing it)
// because `#shortcuts-overlay { display: flex }` has higher CSS specificity
// than `.hidden { display: none }`, so the "hidden" class had no effect.

const indexHtmlPath = path.join(
  path.dirname(fileURLToPath(import.meta.url)),
  '..',
  'index.html'
);
const html = readFileSync(indexHtmlPath, 'utf-8');

beforeAll(() => {
  const doc = new DOMParser().parseFromString(html, 'text/html');

  const styleEl = doc.querySelector('style');
  if (!styleEl) {
    throw new Error('index.html has no <style> element to load into the test document');
  }
  if (!doc.getElementById('shortcuts-overlay')) {
    throw new Error('index.html has no #shortcuts-overlay element');
  }

  document.head.innerHTML = `<style>${styleEl.textContent}</style>`;
  document.body.innerHTML = doc.body.innerHTML;
});

describe('shortcuts overlay visibility (index.html)', () => {
  it('has the "hidden" class in the markup by default', () => {
    const overlay = document.getElementById('shortcuts-overlay');
    expect(overlay.classList.contains('hidden')).toBe(true);
  });

  it('is not displayed on initial load, when the hidden class is present', () => {
    const overlay = document.getElementById('shortcuts-overlay');
    expect(getComputedStyle(overlay).display).toBe('none');
  });

  it('is displayed once the hidden class is removed', () => {
    const overlay = document.getElementById('shortcuts-overlay');
    overlay.classList.remove('hidden');
    try {
      expect(getComputedStyle(overlay).display).toBe('flex');
    } finally {
      overlay.classList.add('hidden');
    }
  });
});
