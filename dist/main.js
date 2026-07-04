import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { listen } from '@tauri-apps/api/event';
import { convertFileSrc } from '@tauri-apps/api/core';

const imageEl = document.getElementById('image');
const filenameEl = document.getElementById('filename');
const infoEl = document.getElementById('info');
const dropZone = document.getElementById('drop-zone');

let currentPath = null;

async function loadImage(path) {
    try {
        await invoke('load_image', { path });
        currentPath = path;

        // Convert file path to URL that Tauri can serve
        const assetUrl = convertFileSrc(path);
        imageEl.src = assetUrl;
        imageEl.classList.add('loaded');
        dropZone.style.display = 'none';

        // Update filename
        const filename = path.split('/').pop();
        filenameEl.textContent = filename;

        // Get image dimensions when loaded
        imageEl.onload = () => {
            infoEl.textContent = `${imageEl.naturalWidth} × ${imageEl.naturalHeight}`;
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
    try {
        const selected = await open({
            multiple: false,
            filters: [{
                name: 'Image',
                extensions: ['jpg', 'jpeg', 'png', 'gif', 'bmp', 'webp']
            }]
        });

        if (selected) {
            await loadImage(selected);
        }
    } catch (error) {
        console.error('Failed to open file:', error);
    }
}

// Button handlers
document.getElementById('open-btn').addEventListener('click', openFile);
document.getElementById('next-btn').addEventListener('click', nextImage);
document.getElementById('prev-btn').addEventListener('click', previousImage);

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
listen('tauri://drag-drop', (event) => {
    const files = event.payload.paths;
    if (files && files.length > 0) {
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

// Handle file opened from Finder (macOS)
window.addEventListener('DOMContentLoaded', async () => {
    // Check if app was opened with a file
    const args = window.__TAURI__?.process?.args || [];
    const fileArg = args.find(arg =>
        arg.endsWith('.jpg') ||
        arg.endsWith('.jpeg') ||
        arg.endsWith('.png') ||
        arg.endsWith('.gif') ||
        arg.endsWith('.bmp') ||
        arg.endsWith('.webp')
    );

    if (fileArg) {
        await loadImage(fileArg);
    }
});
