<script lang="ts">
  import { onMount } from 'svelte';
  import { LanguageSwitcher, ThemeSwitcher } from '@cecepazhar/caui';
  import Icon from '$lib/components/Icon.svelte';
  import { showToast } from '$lib/toast';
  import { isTauri } from '$lib/env';
  import { t, getLocale, setLocale } from '$lib/i18n/index.svelte';
  import { getTimerPrefs, setChime } from '$lib/stores/timer.svelte';

  interface AppSettings {
    tray_enabled: boolean;
    shortcuts_enabled: boolean;
    minimize_to_tray: boolean;
    shortcut_timer_toggle: string;
    shortcut_open_app: string;
  }

  const prefs = getTimerPrefs();

  let settings = $state<AppSettings>({
    tray_enabled: true,
    shortcuts_enabled: true,
    minimize_to_tray: true,
    shortcut_timer_toggle: 'Ctrl+Alt+C',
    shortcut_open_app: 'Ctrl+Alt+L',
  });
  let isLoaded = $state(false);

  onMount(() => {
    let unlisten: (() => void) | undefined;
    (async () => {
      if (!isTauri()) {
        isLoaded = true;
        return;
      }
      const { invoke } = await import('@tauri-apps/api/core');
      const { listen } = await import('@tauri-apps/api/event');
      try {
        settings = await invoke<AppSettings>('get_settings');
      } catch (e) {
        console.error('Failed to load settings:', e);
      } finally {
        isLoaded = true;
      }
      unlisten = await listen<{ tray_enabled: boolean; shortcuts_enabled: boolean }>('settings-updated', (event) => {
        settings.tray_enabled = event.payload.tray_enabled;
        settings.shortcuts_enabled = event.payload.shortcuts_enabled;
      });
    })();
    return () => unlisten?.();
  });

  async function saveSettings() {
    if (!isTauri()) return;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('update_settings', { newSettings: settings });
      showToast(t('settings.saveSuccess'), 'success');
    } catch (e) {
      console.error('Failed to save settings:', e);
      showToast(t('settings.saveFailed'), 'error');
    }
  }

  async function toggleTray() {
    settings.tray_enabled = !settings.tray_enabled;
    settings.minimize_to_tray = settings.tray_enabled;
    await saveSettings();
  }

  async function toggleShortcuts() {
    settings.shortcuts_enabled = !settings.shortcuts_enabled;
    await saveSettings();
  }

  function validateShortcut(value: string): boolean {
    const parts = value
      .split('+')
      .map((p) => p.trim())
      .filter(Boolean);
    if (parts.length < 2) return false;
    const mods = ['ctrl', 'control', 'alt', 'shift', 'cmd', 'cmdorcontrol', 'super', 'meta'];
    return parts.some((p) => mods.includes(p.toLowerCase())) && parts.some((p) => !mods.includes(p.toLowerCase()));
  }

  async function commitShortcut(value: string) {
    if (!validateShortcut(value)) {
      showToast(t('settings.invalidShortcut'), 'error');
      return;
    }
    await saveSettings();
  }
</script>

{#snippet toggle(on: boolean, onclick: () => void, label: string)}
  <button
    type="button"
    role="switch"
    aria-checked={on}
    aria-label={label}
    {onclick}
    class="relative w-11 h-6 rounded-full transition-colors shrink-0 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--ca-brand)] {on
      ? 'bg-[var(--ca-brand)]'
      : 'bg-[var(--ca-surface-subtle)]'}"
  >
    <span class="absolute top-0.5 left-0.5 w-5 h-5 rounded-full bg-white shadow transition-transform {on ? 'translate-x-5' : ''}"></span>
  </button>
{/snippet}

{#snippet header(icon: string, title: string, desc: string)}
  <div class="flex items-center gap-3">
    <span class="w-10 h-10 rounded-xl flex items-center justify-center bg-[var(--ca-brand)]/10 text-[var(--ca-brand)]">
      <Icon name={icon} size={18} />
    </span>
    <div>
      <h2 class="text-sm font-semibold">{title}</h2>
      <p class="text-xs text-[var(--ca-text-muted)]">{desc}</p>
    </div>
  </div>
{/snippet}

<div class="max-w-3xl mx-auto w-full px-4 sm:px-6 py-6 space-y-5">
  <div>
    <h1 class="text-xl font-semibold tracking-tight">{t('settings.title')}</h1>
    <p class="text-sm text-[var(--ca-text-muted)] mt-1">{t('settings.subtitle')}</p>
  </div>

  <!-- Appearance -->
  <section class="rounded-2xl border border-[var(--ca-border)]/70 bg-[var(--ca-surface)] p-5 space-y-4">
    {@render header('sparkles', t('settings.appearanceTitle'), t('settings.appearanceDesc'))}
    <div class="grid sm:grid-cols-2 gap-3">
      <div class="flex items-center justify-between rounded-xl bg-[var(--ca-surface-subtle)]/50 px-4 py-3">
        <span class="text-sm">{t('settings.theme')}</span>
        <ThemeSwitcher showAccentPicker={false} size="sm" />
      </div>
      <div class="flex items-center justify-between rounded-xl bg-[var(--ca-surface-subtle)]/50 px-4 py-3">
        <span class="text-sm">{t('settings.language')}</span>
        <LanguageSwitcher compact value={getLocale()} onchange={(code) => setLocale(code as 'en' | 'id')} size="sm" />
      </div>
    </div>
  </section>

  <!-- Sound -->
  <section class="rounded-2xl border border-[var(--ca-border)]/70 bg-[var(--ca-surface)] p-5 space-y-4">
    {@render header('bell', t('settings.soundTitle'), t('settings.soundDesc'))}
    <div class="flex items-center justify-between gap-4 rounded-xl bg-[var(--ca-surface-subtle)]/50 px-4 py-3">
      <span class="text-sm">{t('settings.chime')}</span>
      {@render toggle(prefs.chime, () => setChime(!prefs.chime), t('settings.chime'))}
    </div>
  </section>

  {#if isTauri()}
    <!-- Background playback -->
    <section class="rounded-2xl border border-[var(--ca-border)]/70 bg-[var(--ca-surface)] p-5 space-y-4">
      <div class="flex items-center justify-between gap-4">
        {@render header('moon', t('settings.trayTitle'), t('settings.trayDesc'))}
        {@render toggle(settings.tray_enabled, toggleTray, t('settings.enableTray'))}
      </div>
      <p class="text-xs text-[var(--ca-text-secondary)]">
        {settings.tray_enabled ? t('settings.trayOn') : t('settings.trayOff')}
      </p>
    </section>

    <!-- Shortcuts -->
    <section class="rounded-2xl border border-[var(--ca-border)]/70 bg-[var(--ca-surface)] p-5 space-y-4">
      <div class="flex items-center justify-between gap-4">
        {@render header('keyboard', t('settings.shortcutsTitle'), t('settings.shortcutsDesc'))}
        {@render toggle(settings.shortcuts_enabled, toggleShortcuts, t('settings.enableShortcuts'))}
      </div>
      {#if settings.shortcuts_enabled && isLoaded}
        <div class="grid sm:grid-cols-2 gap-3">
          <label class="space-y-1.5 block">
            <span class="text-xs text-[var(--ca-text-secondary)]">{t('settings.timerToggleShortcut')}</span>
            <input
              type="text"
              bind:value={settings.shortcut_timer_toggle}
              onblur={() => commitShortcut(settings.shortcut_timer_toggle)}
              class="w-full rounded-xl bg-[var(--ca-surface-subtle)]/50 border border-[var(--ca-border)] px-3 py-2 text-sm font-mono focus:outline-none focus:ring-2 focus:ring-[var(--ca-brand)] select-text"
              placeholder="Ctrl+Alt+C"
            />
          </label>
          <label class="space-y-1.5 block">
            <span class="text-xs text-[var(--ca-text-secondary)]">{t('settings.openAppShortcut')}</span>
            <input
              type="text"
              bind:value={settings.shortcut_open_app}
              onblur={() => commitShortcut(settings.shortcut_open_app)}
              class="w-full rounded-xl bg-[var(--ca-surface-subtle)]/50 border border-[var(--ca-border)] px-3 py-2 text-sm font-mono focus:outline-none focus:ring-2 focus:ring-[var(--ca-brand)] select-text"
              placeholder="Ctrl+Alt+L"
            />
          </label>
        </div>
        <p class="text-[11px] text-[var(--ca-text-muted)]">{t('settings.shortcutExample')}</p>
      {/if}
    </section>
  {/if}
</div>
