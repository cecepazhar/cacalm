// Backup & restore of every CACalm setting (one JSON file).
// Included: all `cacalm_*` local settings (mixes, volumes, name, theme, language, tab,
// chime, sleep action) and the desktop settings (tray, shortcuts).
// Not included: imported audio files (they can be large) — re-import them after restoring.

import { isTauri } from '$lib/env';
import { APP_VERSION } from '$lib/brand';

const PREFIX = 'cacalm_';
const FORMAT = 'cacalm-backup';

export interface Backup {
  format: typeof FORMAT;
  version: 1;
  appVersion: string;
  exportedAt: string;
  local: Record<string, string>;
  settings?: unknown;
}

async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<T>(cmd, args);
}

export async function buildBackup(): Promise<Backup> {
  const local: Record<string, string> = {};
  for (let i = 0; i < localStorage.length; i++) {
    const k = localStorage.key(i);
    if (k && k.startsWith(PREFIX)) local[k] = localStorage.getItem(k) ?? '';
  }
  let settings: unknown;
  if (isTauri()) settings = await invoke('get_settings').catch(() => undefined);
  return { format: FORMAT, version: 1, appVersion: APP_VERSION, exportedAt: new Date().toISOString(), local, settings };
}

export function parseBackup(text: string): Backup {
  const data = JSON.parse(text) as Partial<Backup>;
  if (data?.format !== FORMAT || data.version !== 1 || typeof data.local !== 'object' || !data.local) {
    throw new Error('not a CACalm backup');
  }
  for (const [k, v] of Object.entries(data.local)) {
    if (!k.startsWith(PREFIX) || typeof v !== 'string') throw new Error('invalid entry: ' + k);
  }
  return data as Backup;
}

function fileName() {
  return `CACalm-backup-${new Date().toISOString().slice(0, 10)}.json`;
}

/** Asks where to save and writes the backup. Returns false if cancelled. */
export async function exportBackup(): Promise<boolean> {
  const text = JSON.stringify(await buildBackup(), null, 2);
  if (isTauri()) {
    const { save } = await import('@tauri-apps/plugin-dialog');
    const path = await save({ defaultPath: fileName(), filters: [{ name: 'CACalm backup', extensions: ['json'] }] });
    if (!path) return false;
    await invoke('backup_write', { path, contents: text });
    return true;
  }
  const url = URL.createObjectURL(new Blob([text], { type: 'application/json' }));
  const a = Object.assign(document.createElement('a'), { href: url, download: fileName() });
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
  return true;
}

/** Lets the user pick a backup file and returns its parsed contents (null if cancelled). */
export async function pickBackup(): Promise<Backup | null> {
  let text: string | null = null;
  if (isTauri()) {
    const { open } = await import('@tauri-apps/plugin-dialog');
    const path = await open({ multiple: false, filters: [{ name: 'CACalm backup', extensions: ['json'] }] });
    if (!path || Array.isArray(path)) return null;
    text = await invoke<string>('backup_read', { path });
  } else {
    text = await new Promise<string | null>((resolve) => {
      const input = Object.assign(document.createElement('input'), { type: 'file', accept: '.json,application/json' });
      input.onchange = () => (input.files?.[0] ? input.files[0].text().then(resolve) : resolve(null));
      input.click();
    });
  }
  return text ? parseBackup(text) : null;
}

/** Replaces all CACalm settings with the backup, then reloads the app. */
export async function restoreBackup(b: Backup) {
  const old: string[] = [];
  for (let i = 0; i < localStorage.length; i++) {
    const k = localStorage.key(i);
    if (k && k.startsWith(PREFIX)) old.push(k);
  }
  for (const k of old) localStorage.removeItem(k);
  for (const [k, v] of Object.entries(b.local)) localStorage.setItem(k, v);
  if (isTauri() && b.settings && typeof b.settings === 'object') {
    await invoke('update_settings', { newSettings: b.settings }).catch(() => {});
  }
  location.reload();
}
