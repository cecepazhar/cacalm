import en, { type Dictionary } from './locales/en';
import id from './locales/id';
import { setLocale as setCauiLocale, getLocale as getCauiLocale } from '@cecepazhar/caui';

export type Locale = 'en' | 'id';

const DICTIONARIES: Record<Locale, Dictionary> = { en, id };

export const LOCALES: { code: Locale; label: string }[] = [
  { code: 'en', label: 'EN' },
  { code: 'id', label: 'ID' }
];

const STORAGE_KEY = 'cacalm_locale';

function detectInitialLocale(): Locale {
  try {
    if (typeof localStorage !== 'undefined') {
      const stored = localStorage.getItem(STORAGE_KEY);
      if (stored === 'en' || stored === 'id') return stored;
    }
  } catch {
    // Private mode / storage blocked
  }
  return 'en';
}

const initial = typeof window === 'undefined' ? 'en' : detectInitialLocale();
let currentLocale = $state<Locale>(initial);

try {
  setCauiLocale(initial);
} catch {
  // Headless environment
}

export function getLocale(): Locale {
  return currentLocale;
}

export function setLocale(next: Locale) {
  currentLocale = next;
  try {
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem(STORAGE_KEY, next);
    }
  } catch {
    // Storage error
  }
  if (typeof document !== 'undefined') {
    document.documentElement.lang = next;
  }
  try {
    setCauiLocale(next);
  } catch {
    // Ignore
  }
}

function resolve(dict: unknown, path: string): unknown {
  return path.split('.').reduce<unknown>((node, key) => {
    if (node && typeof node === 'object' && key in node) {
      return (node as Record<string, unknown>)[key];
    }
    return undefined;
  }, dict);
}

function interpolate(template: string, vars?: Record<string, string | number>): string {
  if (!vars) return template;
  return template.replace(/\{\{(\w+)\}\}/g, (match, name) => {
    const val = vars[name];
    return val === undefined ? match : String(val);
  });
}

export function t(key: string, vars?: Record<string, string | number>): string {
  const primary = resolve(DICTIONARIES[currentLocale], key);
  if (typeof primary === 'string') {
    return interpolate(primary, vars);
  }

  const fallback = resolve(DICTIONARIES.en, key);
  if (typeof fallback === 'string') {
    return interpolate(fallback, vars);
  }

  return key;
}
