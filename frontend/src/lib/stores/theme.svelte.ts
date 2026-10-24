export type ThemeName = 'light' | 'dark';

const STORAGE_KEY = 'cacalm_theme';

const state = $state<{ name: ThemeName }>({ name: 'dark' });

export function getTheme() {
  return state;
}

export function isDark(): boolean {
  return state.name === 'dark';
}

function applyToDocument(name: ThemeName) {
  if (typeof document === 'undefined') return;
  const root = document.documentElement;
  root.classList.toggle('dark', name === 'dark');
  root.setAttribute('data-theme', name === 'dark' ? 'dark' : 'light');
}

export function setTheme(name: ThemeName) {
  state.name = name;
  applyToDocument(name);
  try {
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem(STORAGE_KEY, name);
    }
  } catch {
    // Private mode / blocked storage
  }
}

export function toggleTheme() {
  setTheme(state.name === 'dark' ? 'light' : 'dark');
}

export function initTheme() {
  let saved: string | null = null;
  try {
    if (typeof localStorage !== 'undefined') {
      saved = localStorage.getItem(STORAGE_KEY);
    }
  } catch {
    saved = null;
  }
  setTheme(saved === 'light' ? 'light' : 'dark');
}
