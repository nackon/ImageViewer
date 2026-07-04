import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { listen } from '@tauri-apps/api/event';
import { convertFileSrc } from '@tauri-apps/api/core';

console.log('=== ImageViewer JS loaded ===');

const imageEl = document.getElementById('image');
const filenameEl = document.getElementById('filename');
const infoEl = document.getElementById('info');
const dropZone = document.getElementById('drop-zone');

console.log('Elements:', { imageEl, filenameEl, infoEl, dropZone });

let currentPath = null;

async function loadImage(path) {
    try {
        console.log('Loading image:', path);
        await invoke('load_image', { path });
        currentPath = path;

        // Convert file path to URL that Tauri can serve
        const assetUrl = convertFileSrc(path);
        console.log('Asset URL:', assetUrl);
        imageEl.src = assetUrl;
        imageEl.classList.add('loaded');
        dropZone.style.display = 'none';

        // Update filename
        const filename = path.split('/').pop();
        filenameEl.textContent = filename;

        // Get image dimensions when loaded
        imageEl.onload = () => {
            console.log('Image loaded successfully');
            infoEl.textContent = `${imageEl.naturalWidth} × ${imageEl.naturalHeight}`;
        };

        imageEl.onerror = () => {
            console.error('Image element failed to load');
        };
    } catch (error) {
        console.error('Failed to load image:', error);
        filenameEl.textContent = 'Error loading image';
    }
}

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

async function openFile() {
    console.log('openFile() called');
    try {
        const selected = await open({
            multiple: false,
            filters: [{
                name: 'Image',
                extensions: ['jpg', 'jpeg', 'png', 'gif', 'bmp', 'webp']
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

// Button handlers
console.log('Setting up button handlers');
document.getElementById('open-btn').addEventListener('click', () => {
    console.log('Open button clicked');
    openFile();
});
document.getElementById('next-btn').addEventListener('click', () => {
    console.log('Next button clicked');
    nextImage();
});
document.getElementById('prev-btn').addEventListener('click', () => {
    console.log('Previous button clicked');
    previousImage();
});

// Keyboard handlers
document.addEventListener('keydown', (e) => {
    if (e.key === 'ArrowRight' || e.key === ' ') {
        e.preventDefault();
        nextImage();
    } else if (e.key === 'ArrowLeft') {
        e.preventDefault();
        previousImage();
    } else if (e.key === ' ') {
        e.preventDefault();
        openFile();
    }
});

// File drop handler
console.log('Setting up drag-drop listener');
listen('tauri://drag-drop', (event) => {
    console.log('Drag-drop event:', event);
    const files = event.payload.paths;
    if (files && files.length > 0) {
        console.log('Dropped file:', files[0]);
        loadImage(files[0]);
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

// Handle file opened from Finder (macOS "Open With")
(async () => {
    console.log('Setting up open-file listener');
    await listen('open-file', (event) => {
        console.log('Received open-file event:', event.payload);
        loadImage(event.payload);
    });
    console.log('open-file listener registered');
})();
