import { describe, it, expect } from 'vitest';
import { calculateFitScale, applyZoomIn, applyZoomOut, formatZoomPercentage } from './zoom.js';

describe('calculateFitScale', () => {
  it('should scale down large image to fit container', () => {
    // Image 2000x1000, container 1000x500
    const scale = calculateFitScale(2000, 1000, 1000, 500);
    expect(scale).toBe(0.5);
  });

  it('should not scale up small image', () => {
    // Image 500x500, container 1000x1000
    const scale = calculateFitScale(500, 500, 1000, 1000);
    expect(scale).toBe(1.0);
  });

  it('should fit to width when image is wide', () => {
    // Image 2000x500, container 1000x1000
    const scale = calculateFitScale(2000, 500, 1000, 1000);
    expect(scale).toBe(0.5);
  });

  it('should fit to height when image is tall', () => {
    // Image 500x2000, container 1000x1000
    const scale = calculateFitScale(500, 2000, 1000, 1000);
    expect(scale).toBe(0.5);
  });

  it('should handle edge case with zero dimensions', () => {
    expect(calculateFitScale(0, 1000, 1000, 1000)).toBe(1.0);
    expect(calculateFitScale(1000, 0, 1000, 1000)).toBe(1.0);
    expect(calculateFitScale(1000, 1000, 0, 1000)).toBe(1.0);
    expect(calculateFitScale(1000, 1000, 1000, 0)).toBe(1.0);
  });

  it('should handle square image in square container', () => {
    const scale = calculateFitScale(1000, 1000, 800, 800);
    expect(scale).toBe(0.8);
  });
});

describe('applyZoomIn', () => {
  it('should increase scale by default factor', () => {
    const newScale = applyZoomIn(1.0);
    expect(newScale).toBe(1.2);
  });

  it('should not exceed maximum scale', () => {
    const newScale = applyZoomIn(4.9, 1.2, 5.0);
    expect(newScale).toBe(5.0);
  });

  it('should use custom zoom factor', () => {
    const newScale = applyZoomIn(1.0, 1.5);
    expect(newScale).toBe(1.5);
  });

  it('should handle multiple zoom ins', () => {
    let scale = 1.0;
    scale = applyZoomIn(scale);
    scale = applyZoomIn(scale);
    expect(scale).toBeCloseTo(1.44, 2);
  });
});

describe('applyZoomOut', () => {
  it('should decrease scale by default factor', () => {
    const newScale = applyZoomOut(1.2);
    expect(newScale).toBe(1.0);
  });

  it('should not go below minimum scale', () => {
    const newScale = applyZoomOut(0.11, 1.2, 0.1);
    expect(newScale).toBe(0.1);
  });

  it('should use custom zoom factor', () => {
    const newScale = applyZoomOut(1.5, 1.5);
    expect(newScale).toBe(1.0);
  });

  it('should handle multiple zoom outs', () => {
    let scale = 1.44;
    scale = applyZoomOut(scale);
    scale = applyZoomOut(scale);
    expect(scale).toBeCloseTo(1.0, 2);
  });
});

describe('formatZoomPercentage', () => {
  it('should format scale as percentage', () => {
    expect(formatZoomPercentage(1.0)).toBe('100%');
    expect(formatZoomPercentage(0.5)).toBe('50%');
    expect(formatZoomPercentage(2.0)).toBe('200%');
  });

  it('should round to nearest integer', () => {
    expect(formatZoomPercentage(0.666)).toBe('67%');
    expect(formatZoomPercentage(1.234)).toBe('123%');
  });
});
