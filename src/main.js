import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { listen } from '@tauri-apps/api/event';
import { convertFileSrc } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { toggleFullscreen, escapeAction } from './fullscreen.js';
import { handleMenuAction } from './menuActions.js';
import { resolveTheme, nextThemePreference, normalizeThemePreference, themeStatusLabel } from './theme.js';

console.log('=== ImageViewer JS loaded ===');

// DOM elements
const imageEl = document.getElementById('image');
const imageWrapper = document.getElementById('image-wrapper');
const filenameEl = document.getElementById('filename');
const infoEl = document.getElementById('info');
const dropZone = document.getElementById('drop-zone');
const imageContainer = document.getElementById('image-container');
const thumbnailView = document.getElementById('thumbnail-view');
const thumbnailGrid = document.getElementById('thumbnail-grid');
const footer = document.getElementById('footer');
const imagePosition = document.getElementById('image-position');
const zoomLevel = document.getElementById('zoom-level');
const thumbnailHint = document.getElementById('thumbnail-hint');
const thumbnailFooter = document.getElementById('thumbnail-footer');

// State
let currentPath = null;
let currentZoom = 1.0; // 100%
let zoomMode = 'fit'; // 'fit' or 'manual'
let viewMode = 'image'; // 'image' or 'thumbnail'
let allImages = [];
let currentIndex = 0;
let selectedThumbnailIndex = 0;

// Constants
const ZOOM_STEP = 0.25; // 25%
const THEME_STORAGE_KEY = 'themePreference';

// Theme
const darkModeQuery = window.matchMedia('(prefers-color-scheme: dark)');
let themePreference = normalizeThemePreference(localStorage.getItem(THEME_STORAGE_KEY));
let themeStatusTimer = null;

function applyTheme() {
    document.documentElement.dataset.theme = resolveTheme(themePreference, darkModeQuery.matches);
}

function cycleTheme() {
    themePreference = nextThemePreference(themePreference);
    localStorage.setItem(THEME_STORAGE_KEY, themePreference);
    applyTheme();

    // Briefly show the new theme in the header
    infoEl.textContent = themeStatusLabel(themePreference, resolveTheme(themePreference, darkModeQuery.matches));
    clearTimeout(themeStatusTimer);
    themeStatusTimer = setTimeout(() => {
        infoEl.textContent = '';
    }, 2000);
}

applyTheme();
darkModeQuery.addEventListener('change', applyTheme);

// Load image
async function loadImage(path) {
    try {
        console.log('Loading image:', path);
        const result = await invoke('load_image', { path });
        currentPath = path;
        allImages = result.images || [];
        currentIndex = result.index || 0;

        // Convert file path to URL
        const assetUrl = convertFileSrc(path);
        console.log('Asset URL:', assetUrl);

        imageEl.src = assetUrl;
        imageWrapper.classList.add('loaded');
        dropZone.style.display = 'none';

        // Update filename and info
        const filename = path.split('/').pop();
        filenameEl.textContent = `${filename} - ${result.width} × ${result.height} - ${formatFileSize(result.size)}`;

        // Update footer
        updateFooter();

        // Reset zoom mode to fit for new image
        zoomMode = 'fit';

        // Apply zoom
        imageEl.onload = () => {
            console.log('Image loaded successfully');
            applyFitZoom();
        };

        imageEl.onerror = () => {
            console.error('Image element failed to load');
        };
    } catch (error) {
        console.error('Failed to load image:', error);
        filenameEl.textContent = 'Error loading image';
    }
}

// Format file size
function formatFileSize(bytes) {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

// Update footer
function updateFooter() {
    if (viewMode === 'image' && allImages.length > 0) {
        imagePosition.textContent = `${currentIndex + 1}/${allImages.length}`;
        imagePosition.classList.remove('hidden');
        zoomLevel.textContent = `Zoom: ${Math.round(currentZoom * 100)}%`;
        zoomLevel.classList.remove('hidden');
        thumbnailHint.classList.remove('hidden');
        thumbnailFooter.classList.add('hidden');
    } else if (viewMode === 'thumbnail') {
        imagePosition.classList.add('hidden');
        zoomLevel.classList.add('hidden');
        thumbnailHint.classList.add('hidden');
        thumbnailFooter.classList.remove('hidden');
        footer.classList.add('thumbnail-mode');
    }
}

// Apply zoom
function applyZoom(zoom) {
    currentZoom = Math.max(0.25, Math.min(zoom, 10)); // Limit: 25% to 1000%
    imageEl.style.transform = `scale(${currentZoom})`;
    imageEl.style.transformOrigin = 'center center';
    updateFooter();
}

// Fit zoom
function applyFitZoom() {
    zoomMode = 'fit';
    imageEl.style.transform = '';
    imageEl.style.maxWidth = '';
    imageEl.style.maxHeight = '';
    imageEl.style.transformOrigin = '';

    // Calculate actual fit zoom level
    if (imageEl.naturalWidth > 0 && imageEl.naturalHeight > 0) {
        const containerWidth = imageContainer.clientWidth;
        const containerHeight = imageContainer.clientHeight;

        // If the container isn't currently visible (e.g. thumbnail view), avoid reporting 0%.
        if (containerWidth > 0 && containerHeight > 0) {
            const scaleX = containerWidth / imageEl.naturalWidth;
            const scaleY = containerHeight / imageEl.naturalHeight;
            // Never scale up beyond 100%
            currentZoom = Math.min(scaleX, scaleY, 1.0);
        } else {
            currentZoom = 1.0;
        }
    } else {
        currentZoom = 1.0;
    }

    updateFooter();
}

// Zoom in
function zoomIn() {
    zoomMode = 'manual';
    imageEl.style.maxWidth = 'none';
    imageEl.style.maxHeight = 'none';
    applyZoom(currentZoom + ZOOM_STEP);
}

// Zoom out
function zoomOut() {
    zoomMode = 'manual';
    imageEl.style.maxWidth = 'none';
    imageEl.style.maxHeight = 'none';
    applyZoom(currentZoom - ZOOM_STEP);
}

// Actual size (100%)
function actualSize() {
    zoomMode = 'manual';
    imageEl.style.maxWidth = 'none';
    imageEl.style.maxHeight = 'none';
    applyZoom(1.0);
}

// Navigation
async function nextImage() {
    try {
        const path = await invoke('next_image');
        if (path) {
            await loadImage(path);
        }
    } catch (error) {
        console.error('Failed to get next image:', error);
    }
}

async function previousImage() {
    try {
        const path = await invoke('previous_image');
        if (path) {
            await loadImage(path);
        }
    } catch (error) {
        console.error('Failed to get previous image:', error);
    }
}

async function firstImage() {
    try {
        const path = await invoke('first_image');
        if (path) {
            await loadImage(path);
        }
    } catch (error) {
        console.error('Failed to get first image:', error);
    }
}

async function lastImage() {
    try {
        const path = await invoke('last_image');
        if (path) {
            await loadImage(path);
        }
    } catch (error) {
        console.error('Failed to get last image:', error);
    }
}

// Open file dialog
async function openFile() {
    console.log('openFile() called');
    try {
        const selected = await open({
            multiple: false,
            filters: [{
                name: 'Image',
                extensions: ['jpg', 'jpeg', 'png', 'gif', 'bmp', 'webp', 'tiff']
            }]
        });

        console.log('Selected file:', selected);
        if (selected) {
            await loadImage(selected);
        }
    } catch (error) {
        console.error('Failed to open file:', error);
    }
}

// Open folder dialog
async function openFolder() {
    console.log('openFolder() called');
    try {
        const selected = await open({
            directory: true,
            multiple: false,
        });

        console.log('Selected folder:', selected);
        if (selected) {
            await loadImage(selected);
        }
    } catch (error) {
        console.error('Failed to open folder:', error);
    }
}

// Toggle thumbnail view
async function toggleThumbnailView() {
    if (viewMode === 'image') {
        // Switch to thumbnail view
        viewMode = 'thumbnail';
        imageContainer.style.display = 'none';
        thumbnailView.classList.add('active');
        footer.classList.add('thumbnail-mode');
        filenameEl.textContent = `Thumbnails - ${allImages.length} images`;
        infoEl.textContent = '';
        updateFooter();

        // Generate thumbnails
        await generateThumbnails();
        selectedThumbnailIndex = currentIndex;
        updateThumbnailSelection();
    } else {
        // Switch back to image view
        viewMode = 'image';
        imageContainer.style.display = 'flex';
        thumbnailView.classList.remove('active');
        footer.classList.remove('thumbnail-mode');
        updateFooter();

        // Reload current image info
        if (currentPath) {
            const result = await invoke('get_image_info', { path: currentPath });
            const filename = currentPath.split('/').pop();
            filenameEl.textContent = `${filename} - ${result.width} × ${result.height} - ${formatFileSize(result.size)}`;
        }
    }
}

// Generate thumbnails
async function generateThumbnails() {
    thumbnailGrid.innerHTML = '';

    for (let i = 0; i < allImages.length; i++) {
        const imagePath = allImages[i];
        const thumbnailItem = document.createElement('div');
        thumbnailItem.className = 'thumbnail-item';
        thumbnailItem.dataset.index = i;

        // Create thumbnail image
        const img = document.createElement('img');
        const thumbnailUrl = await invoke('get_thumbnail', { path: imagePath });
        img.src = convertFileSrc(thumbnailUrl);

        thumbnailItem.appendChild(img);
        thumbnailItem.addEventListener('click', () => selectThumbnail(i));

        thumbnailGrid.appendChild(thumbnailItem);
    }
}

// Update thumbnail selection
function updateThumbnailSelection() {
    const items = thumbnailGrid.querySelectorAll('.thumbnail-item');
    items.forEach((item, index) => {
        if (index === selectedThumbnailIndex) {
            item.classList.add('selected');
            item.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
        } else {
            item.classList.remove('selected');
        }
    });
}

// Select thumbnail
function selectThumbnail(index) {
    selectedThumbnailIndex = index;
    updateThumbnailSelection();
}

// View selected thumbnail
async function viewSelectedThumbnail() {
    if (selectedThumbnailIndex >= 0 && selectedThumbnailIndex < allImages.length) {
        await loadImage(allImages[selectedThumbnailIndex]);
        await toggleThumbnailView();
    }
}

// Thumbnail navigation
function navigateThumbnails(direction) {
    const columns = Math.floor(thumbnailGrid.offsetWidth / 165); // 150px + 15px gap

    switch (direction) {
        case 'up':
            selectedThumbnailIndex = Math.max(0, selectedThumbnailIndex - columns);
            break;
        case 'down':
            selectedThumbnailIndex = Math.min(allImages.length - 1, selectedThumbnailIndex + columns);
            break;
        case 'left':
            selectedThumbnailIndex = Math.max(0, selectedThumbnailIndex - 1);
            break;
        case 'right':
            selectedThumbnailIndex = Math.min(allImages.length - 1, selectedThumbnailIndex + 1);
            break;
    }

    updateThumbnailSelection();
}

// Keyboard handlers
document.addEventListener('keydown', async (e) => {
    // Prevent default for navigation keys
    if (['ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown', 'Home', 'End', ' '].includes(e.key)) {
        e.preventDefault();
    }

    if (viewMode === 'image') {
        // Image view shortcuts
        switch (e.key) {
            case 'ArrowRight':
            case ' ':
            case 'n':
            case 'N':
                await nextImage();
                break;
            case 'ArrowLeft':
            case 'Backspace':
            case 'p':
            case 'P':
                await previousImage();
                break;
            case 'Home':
                await firstImage();
                break;
            case 'End':
                await lastImage();
                break;
            case 't':
            case 'T':
                await toggleThumbnailView();
                break;
            case 'q':
            case 'Q':
                await getCurrentWindow().close();
                break;
            case 'Escape': {
                const win = getCurrentWindow();
                if (escapeAction(await win.isFullscreen()) === 'exit-fullscreen') {
                    await win.setFullscreen(false);
                } else {
                    await win.close();
                }
                break;
            }
            case '+':
            case '=':
                zoomIn();
                break;
            case '-':
                zoomOut();
                break;
            case '0':
                actualSize();
                break;
            case 'f':
            case 'F':
                await toggleFullscreen(getCurrentWindow());
                break;
            case 'w':
            case 'W':
                applyFitZoom();
                break;
            case 'd':
            case 'D':
                cycleTheme();
                break;
        }
    } else {
        // Thumbnail view shortcuts
        switch (e.key) {
            case 'ArrowUp':
                navigateThumbnails('up');
                break;
            case 'ArrowDown':
                navigateThumbnails('down');
                break;
            case 'ArrowLeft':
                navigateThumbnails('left');
                break;
            case 'ArrowRight':
                navigateThumbnails('right');
                break;
            case 'Enter':
                await viewSelectedThumbnail();
                break;
            case 'f':
            case 'F':
                await toggleFullscreen(getCurrentWindow());
                break;
            case 't':
            case 'T':
            case 'Escape':
                await toggleThumbnailView();
                break;
            case 'd':
            case 'D':
                cycleTheme();
                break;
            case 'q':
            case 'Q':
                await exit(0);
                break;
        }
    }
});

// Mouse wheel zoom
imageContainer.addEventListener('wheel', (e) => {
    if (viewMode === 'image' && imageWrapper.classList.contains('loaded')) {
        e.preventDefault();
        if (e.deltaY < 0) {
            zoomIn();
        } else {
            zoomOut();
        }
    }
});

// Drop zone click to open
dropZone.addEventListener('click', () => {
    openFile();
});

// File drop handler
console.log('Setting up drag-drop listener');
listen('tauri://drag-drop', async (event) => {
    console.log('Drag-drop event:', event);
    const paths = event.payload.paths;
    if (paths && paths.length > 0) {
        const droppedPath = paths[0];
        console.log('Dropped path:', droppedPath);

        try {
            await loadImage(droppedPath);
        } catch (error) {
            console.error('Failed to load dropped file:', error);
            filenameEl.textContent = 'Error loading dropped item';
        }
    }
});

// Drop zone visual feedback
dropZone.addEventListener('dragover', (e) => {
    e.preventDefault();
    dropZone.classList.add('active');
});

dropZone.addEventListener('dragleave', () => {
    dropZone.classList.remove('active');
});

// Menu bar commands (mirrors the keyboard shortcuts handled above)
listen('menu-command', (event) => {
    handleMenuAction(event.payload, {
        nextImage,
        previousImage,
        firstImage,
        lastImage,
        zoomIn,
        zoomOut,
        actualSize,
        applyFitZoom,
        toggleThumbnailView,
        openFile,
        openFolder,
    });
});

// Window resize handler
window.addEventListener('resize', () => {
    if (zoomMode === 'fit' && imageEl.naturalWidth > 0) {
        applyFitZoom();
    }
});

// Handle file opened from Finder (macOS "Open With")
(async () => {
    console.log('Setting up open-file listener');
    await listen('open-file', (event) => {
        console.log('Received open-file event:', event.payload);
        loadImage(event.payload);
    });
    console.log('open-file listener registered');
})();

// Handle file open from OS (double-click)
(async () => {
    console.log('[JS] Setting up open-file-from-os listener');
    await listen('open-file-from-os', (event) => {
        const targetFilePath = event.payload;
        console.log("[JS] Received file from OS:", targetFilePath);
        loadImage(targetFilePath);
    });
    console.log('[JS] open-file-from-os listener registered');

    // Get buffered files
    setTimeout(async () => {
        console.log('[JS] Calling frontend_ready');
        try {
            const bufferedPaths = await invoke('frontend_ready');
            console.log('[JS] frontend_ready returned:', bufferedPaths);
            if (bufferedPaths && bufferedPaths.length > 0) {
                console.log('[JS] Loading buffered file:', bufferedPaths[0]);
                loadImage(bufferedPaths[0]);
            }
        } catch (e) {
            console.error('[JS] frontend_ready error:', e);
        }
    }, 100);
})();
