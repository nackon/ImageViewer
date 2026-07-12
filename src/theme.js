// Theme preference handling.
// Preference is what the user chose ('system' | 'light' | 'dark'),
// theme is what actually gets applied ('light' | 'dark').

export const THEME_PREFERENCES = ['system', 'light', 'dark'];

// Resolve the effective theme from a preference and the OS setting
export function resolveTheme(preference, systemPrefersDark) {
    if (preference === 'light' || preference === 'dark') {
        return preference;
    }
    return systemPrefersDark ? 'dark' : 'light';
}

// Cycle preference: system -> light -> dark -> system
export function nextThemePreference(current) {
    const index = THEME_PREFERENCES.indexOf(current);
    return THEME_PREFERENCES[(index + 1) % THEME_PREFERENCES.length];
}

// Sanitize a value restored from storage
export function normalizeThemePreference(value) {
    return THEME_PREFERENCES.includes(value) ? value : 'system';
}

// Human readable status, e.g. "Theme: System (Dark)"
export function themeStatusLabel(preference, resolvedTheme) {
    const capitalize = (s) => s.charAt(0).toUpperCase() + s.slice(1);
    if (preference === 'system') {
        return `Theme: System (${capitalize(resolvedTheme)})`;
    }
    return `Theme: ${capitalize(preference)}`;
}
