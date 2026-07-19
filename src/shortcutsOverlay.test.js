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
  const styleMatch = html.match(/<style>([\s\S]*?)<\/style>/);
  const bodyMatch = html.match(/<body>([\s\S]*)<\/body>/);
  document.head.innerHTML = `<style>${styleMatch[1]}</style>`;
  document.body.innerHTML = bodyMatch[1];
});

describe('shortcuts overlay visibility (index.html)', () => {
  it('is not marked hidden by class alone (would already fail here on regression)', () => {
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
    expect(getComputedStyle(overlay).display).toBe('flex');
    overlay.classList.add('hidden');
  });
});
