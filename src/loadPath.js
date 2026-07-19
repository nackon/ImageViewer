/**
 * Resolves the path that should actually be displayed after a `load_image` call.
 *
 * `path` passed into `load_image` may be a folder (drag & drop, "Open Folder...").
 * In that case the Rust backend resolves it to the first image inside and reports
 * it as `images[index]`; the frontend must display that resolved image path, not
 * the raw folder path that was originally passed in.
 * @param {string} path - The path originally passed to `load_image`.
 * @param {string[] | undefined} images - The gallery list returned by `load_image`.
 * @param {number} index - The index of the loaded image within `images`.
 * @returns {string} The path to actually display.
 */
export function resolveLoadedPath(path, images, index) {
    return (images && images[index]) || path;
}
