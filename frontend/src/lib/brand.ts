declare const __APP_VERSION__: string;

export const APP_VERSION: string = typeof __APP_VERSION__ === 'string' ? __APP_VERSION__ : '0.0.0';

export const BRAND = {
  product: 'CACalm',
  author: 'Cecep Azhar',
  authorFull: 'Cecep Saeful Azhar Hidayat',
  company: 'Fathforce',
  year: 2026,
  repoUrl: 'https://github.com/cecepazhar/cacalm',
  profileUrl: 'https://github.com/cecepazhar',
  supportUrl: 'https://github.com/sponsors/cecepazhar',
} as const;

/** Opens an author-owned link in the system browser (desktop) or a new tab (web). */
export async function openExternal(url: string) {
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window) {
      await invoke('open_external', { url });
      return;
    }
  } catch {
    /* fall back to window.open */
  }
  window.open(url, '_blank', 'noopener');
}
