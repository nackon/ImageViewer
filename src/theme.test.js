import { describe, it, expect } from 'vitest';
import {
    THEME_PREFERENCES,
    resolveTheme,
    nextThemePreference,
    normalizeThemePreference,
    themeStatusLabel,
} from './theme.js';

describe('resolveTheme', () => {
    it('follows the OS setting when preference is system', () => {
        expect(resolveTheme('system', true)).toBe('dark');
        expect(resolveTheme('system', false)).toBe('light');
    });

    it('returns the explicit preference regardless of the OS setting', () => {
        expect(resolveTheme('light', true)).toBe('light');
        expect(resolveTheme('light', false)).toBe('light');
        expect(resolveTheme('dark', true)).toBe('dark');
        expect(resolveTheme('dark', false)).toBe('dark');
    });
});

describe('nextThemePreference', () => {
    it('cycles system -> light -> dark -> system', () => {
        expect(nextThemePreference('system')).toBe('light');
        expect(nextThemePreference('light')).toBe('dark');
        expect(nextThemePreference('dark')).toBe('system');
    });

    it('starts the cycle over from an unknown value', () => {
        // indexOf returns -1, so the next preference is the first entry
        expect(nextThemePreference('bogus')).toBe(THEME_PREFERENCES[0]);
    });
});

describe('normalizeThemePreference', () => {
    it('accepts valid preferences', () => {
        expect(normalizeThemePreference('system')).toBe('system');
        expect(normalizeThemePreference('light')).toBe('light');
        expect(normalizeThemePreference('dark')).toBe('dark');
    });

    it('falls back to system for invalid values', () => {
        expect(normalizeThemePreference(null)).toBe('system');
        expect(normalizeThemePreference(undefined)).toBe('system');
        expect(normalizeThemePreference('')).toBe('system');
        expect(normalizeThemePreference('blue')).toBe('system');
    });
});

describe('themeStatusLabel', () => {
    it('shows the resolved theme when following the system', () => {
        expect(themeStatusLabel('system', 'dark')).toBe('Theme: System (Dark)');
        expect(themeStatusLabel('system', 'light')).toBe('Theme: System (Light)');
    });

    it('shows only the preference when explicitly set', () => {
        expect(themeStatusLabel('light', 'light')).toBe('Theme: Light');
        expect(themeStatusLabel('dark', 'dark')).toBe('Theme: Dark');
    });
});
