import { describe, it, expect } from 'vitest';
import { resolveLoadedPath } from './loadPath.js';

describe('resolveLoadedPath', () => {
    it('returns the resolved image at the given index when a folder was opened', () => {
        const images = ['/some/folder/a.jpg', '/some/folder/b.jpg'];
        expect(resolveLoadedPath('/some/folder', images, 0)).toBe('/some/folder/a.jpg');
    });

    it('returns the exact file path unchanged for a normal single-file open', () => {
        const images = ['/dir/a.jpg', '/dir/b.jpg', '/dir/c.jpg'];
        expect(resolveLoadedPath('/dir/b.jpg', images, 1)).toBe('/dir/b.jpg');
    });

    it('falls back to the original path when images is empty', () => {
        expect(resolveLoadedPath('/some/file.jpg', [], 0)).toBe('/some/file.jpg');
    });

    it('falls back to the original path when images is undefined', () => {
        expect(resolveLoadedPath('/some/file.jpg', undefined, 0)).toBe('/some/file.jpg');
    });
});
