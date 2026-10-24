<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { Badge, LanguageSwitcher, ThemeSwitcher } from '@cecepazhar/caui';
  import { showToast } from '$lib/toast';
  import { t, getLocale, setLocale } from '$lib/i18n/index.svelte';

  interface AppSettings {
    tray_enabled: boolean;
    shortcuts_enabled: boolean;
    minimize_to_tray: boolean;
    shortcut_timer_toggle: string;
    shortcut_open_app: string;
  }

  let settings = $state<AppSettings>({
    tray_enabled: true,
    shortcuts_enabled: true,
    minimize_to_tray: true,
    shortcut_timer_toggle: 'Ctrl+Alt+C',
    shortcut_open_app: 'Ctrl+Alt+L'
  });

  let isLoaded = $state(false);

  onMount(async () => {
    try {
      settings = await invoke<AppSettings>('get_settings');
    } catch (e) {
      console.error('Failed to load settings:', e);
      showToast(t('settings.loading'), 'error');
    } finally {
      isLoaded = true;
    }

    const unlisten = await listen<{ tray_enabled: boolean; shortcuts_enabled: boolean }>(
      'settings-updated',
      (event) => {
        settings.tray_enabled = event.payload.tray_enabled;
        settings.shortcuts_enabled = event.payload.shortcuts_enabled;
      }
    );

    onDestroy(unlisten);
  });

  async function saveSettings() {
    try {
      await invoke('update_settings', { newSettings: settings });
      showToast(t('settings.saveSuccess'), 'success');
    } catch (e) {
      console.error('Failed to save settings:', e);
      showToast(t('settings.saveFailed'), 'error');
    }
  }

  async function toggleTray() {
    settings.tray_enabled = !settings.tray_enabled;
    await saveSettings();
  }

  async function toggleShortcuts() {
    settings.shortcuts_enabled = !settings.shortcuts_enabled;
    await saveSettings();
  }

  function validateShortcut(value: string): boolean {
    if (!isLoaded) return true;
    const parts = value
      .split('+')
      .map((p) => p.trim())
      .filter((p) => p.length > 0);

    if (parts.length < 2) return false;

    const validModifiers = [
      'ctrl', 'control', 'alt', 'shift', 'cmd',
      'cmdorcontrol', 'super', 'meta'
    ];
    const hasModifier = parts.some((p) => validModifiers.includes(p.toLowerCase()));
    const hasKey = parts.some(
      (p) => !validModifiers.includes(p.toLowerCase()) && p.length > 0
    );

    return hasModifier && hasKey;
  }
</script>

<div class="flex flex-col h-full overflow-y-auto p-6 space-y-4 bg-[var(--ca-canvas)] text-[var(--ca-text-primary)] max-w-7xl mx-auto w-full">
  <!-- Header -->
  <div class="flex items-center justify-between border-b border-[var(--ca-border)] pb-4">
    <div>
      <h1 class="text-base font-bold text-[var(--ca-text-primary)] flex items-center gap-2">
        ⚙️ {t('settings.title')}
        <Badge variant={isLoaded ? 'brand' : 'neutral'} size="xs">
          {isLoaded ? t('settings.loaded') : t('settings.loading')}
        </Badge>
      </h1>
      <p class="text-xs text-[var(--ca-text-muted)] mt-0.5">
        {t('settings.subtitle')}
      </p>
    </div>
  </div>

  <!-- Appearance & Language Card -->
  <div class="bg-[var(--ca-surface)] border border-[var(--ca-border)] rounded-xl p-5 space-y-4 shadow-xs">
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-3">
        <div class="w-10 h-10 rounded-lg bg-[var(--ca-brand)]/10 text-[var(--ca-brand)] flex items-center justify-center text-lg border border-[var(--ca-brand)]/20">
          🎨
        </div>
        <div>
          <h3 class="text-xs font-bold text-[var(--ca-text-primary)]">{t('settings.appearanceTitle')}</h3>
          <span class="text-[9px] text-[var(--ca-text-muted)] font-mono uppercase">{t('settings.appearanceDesc')}</span>
        </div>
      </div>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-4 pt-2 border-t border-[var(--ca-border)]">
      <!-- Theme Switcher -->
      <div class="flex items-center justify-between p-3 rounded-lg bg-[var(--ca-surface-subtle)]">
        <div>
          <span class="text-xs font-semibold text-[var(--ca-text-primary)] block">{t('settings.theme')}</span>
          <span class="text-[10px] text-[var(--ca-text-muted)]">Light / Dark</span>
        </div>
        <ThemeSwitcher showAccentPicker={false} size="sm" />
      </div>

      <!-- Language Switcher -->
      <div class="flex items-center justify-between p-3 rounded-lg bg-[var(--ca-surface-subtle)]">
        <div>
          <span class="text-xs font-semibold text-[var(--ca-text-primary)] block">{t('settings.language')}</span>
          <span class="text-[10px] text-[var(--ca-text-muted)]">EN / ID</span>
        </div>
        <LanguageSwitcher
          compact
          value={getLocale()}
          onchange={(code) => setLocale(code as 'en' | 'id')}
          size="sm"
        />
      </div>
    </div>
  </div>

  <!-- Toggle Tray Card -->
  <div class="bg-[var(--ca-surface)] border border-[var(--ca-border)] rounded-xl p-5 space-y-4 shadow-xs hover:border-[var(--ca-brand)]/40 transition-colors">
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-3">
        <div class="w-10 h-10 rounded-lg bg-[var(--ca-brand)]/10 text-[var(--ca-brand)] flex items-center justify-center text-lg border border-[var(--ca-brand)]/20">
          🖥️
        </div>
        <div>
          <h3 class="text-xs font-bold text-[var(--ca-text-primary)]">{t('settings.trayTitle')}</h3>
          <span class="text-[9px] text-[var(--ca-text-muted)] font-mono uppercase">{t('settings.trayDesc')}</span>
        </div>
      </div>

      <button
        type="button"
        onclick={toggleTray}
        class={`w-11 h-6 p-0.5 rounded-full flex items-center transition-colors cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--ca-brand)] ${
          settings.tray_enabled
            ? 'bg-[var(--ca-brand)] justify-end'
            : 'bg-[var(--ca-surface-subtle)] border border-[var(--ca-border)] justify-start'
        }`}
        title={settings.tray_enabled ? 'Disable Tray' : 'Enable Tray'}
        aria-label={t('settings.enableTray')}
      >
        <span
          class="w-5 h-5 rounded-full bg-white shadow-xs transition-transform"
        ></span>
      </button>
    </div>
    <p class="text-[11px] text-[var(--ca-text-secondary)]">
      {settings.tray_enabled
        ? 'Tray icon active. Left click for quick actions, double click to open CACalm.'
        : 'Tray icon hidden from system tray.'}
    </p>
  </div>

  <!-- Keyboard Shortcuts Card -->
  <div class="bg-[var(--ca-surface)] border border-[var(--ca-border)] rounded-xl p-5 space-y-4 shadow-xs hover:border-[var(--ca-brand)]/40 transition-colors">
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-3">
        <div class="w-10 h-10 rounded-lg bg-[var(--ca-brand)]/10 text-[var(--ca-brand)] flex items-center justify-center text-lg border border-[var(--ca-brand)]/20">
          ⌨️
        </div>
        <div>
          <h3 class="text-xs font-bold text-[var(--ca-text-primary)]">{t('settings.shortcutsTitle')}</h3>
          <span class="text-[9px] text-[var(--ca-text-muted)] font-mono uppercase">{t('settings.shortcutsDesc')}</span>
        </div>
      </div>

      <button
        type="button"
        onclick={toggleShortcuts}
        class={`w-11 h-6 p-0.5 rounded-full flex items-center transition-colors cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--ca-brand)] ${
          settings.shortcuts_enabled
            ? 'bg-[var(--ca-brand)] justify-end'
            : 'bg-[var(--ca-surface-subtle)] border border-[var(--ca-border)] justify-start'
        }`}
        title={settings.shortcuts_enabled ? 'Disable Shortcuts' : 'Enable Shortcuts'}
        aria-label={t('settings.enableShortcuts')}
      >
        <span
          class="w-5 h-5 rounded-full bg-white shadow-xs transition-transform"
        ></span>
      </button>
    </div>

    {#if settings.shortcuts_enabled}
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4 pt-2 border-t border-[var(--ca-border)]">
        <!-- Timer Toggle -->
        <div class="space-y-2">
          <label for="shortcut-timer" class="text-[10px] font-bold uppercase tracking-wider text-[var(--ca-text-secondary)] block">
            {t('settings.timerToggleShortcut')}
          </label>
          <input
            id="shortcut-timer"
            type="text"
            bind:value={settings.shortcut_timer_toggle}
            class="w-full bg-[var(--ca-surface-subtle)] border border-[var(--ca-border)] rounded-lg px-3 py-2 text-xs font-mono text-[var(--ca-text-primary)] focus:outline-none focus:ring-2 focus:ring-[var(--ca-brand)]"
            placeholder="Ctrl+Alt+C"
            oninput={() => {
              if (!validateShortcut(settings.shortcut_timer_toggle)) {
                showToast(t('settings.invalidShortcut'), 'error');
              }
            }}
            onblur={saveSettings}
          />
          <p class="text-[10px] text-[var(--ca-text-muted)]">
            E.g. Ctrl+Alt+C, Cmd+Shift+P
          </p>
        </div>

        <!-- Open App -->
        <div class="space-y-2">
          <label for="shortcut-open" class="text-[10px] font-bold uppercase tracking-wider text-[var(--ca-text-secondary)] block">
            {t('settings.openAppShortcut')}
          </label>
          <input
            id="shortcut-open"
            type="text"
            bind:value={settings.shortcut_open_app}
            class="w-full bg-[var(--ca-surface-subtle)] border border-[var(--ca-border)] rounded-lg px-3 py-2 text-xs font-mono text-[var(--ca-text-primary)] focus:outline-none focus:ring-2 focus:ring-[var(--ca-brand)]"
            placeholder="Ctrl+Alt+L"
            oninput={() => {
              if (!validateShortcut(settings.shortcut_open_app)) {
                showToast(t('settings.invalidShortcut'), 'error');
              }
            }}
            onblur={saveSettings}
          />
          <p class="text-[10px] text-[var(--ca-text-muted)]">
            Opens and focuses CACalm window.
          </p>
        </div>
      </div>
    {:else}
      <p class="text-[11px] text-[var(--ca-text-muted)] pt-2 border-t border-[var(--ca-border)]">
        Global shortcuts disabled.
      </p>
    {/if}
  </div>

  <!-- Minimize to Tray Card -->
  <div class="bg-[var(--ca-surface)] border border-[var(--ca-border)] rounded-xl p-5 space-y-4 shadow-xs hover:border-[var(--ca-brand)]/40 transition-colors">
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-3">
        <div class="w-10 h-10 rounded-lg bg-[var(--ca-brand)]/10 text-[var(--ca-brand)] flex items-center justify-center text-lg border border-[var(--ca-brand)]/20">
          📥
        </div>
        <div>
          <h3 class="text-xs font-bold text-[var(--ca-text-primary)]">{t('settings.minimizeToTray')}</h3>
          <span class="text-[9px] text-[var(--ca-text-muted)] font-mono uppercase">
            {t('settings.minimizeToTrayDesc')}
          </span>
        </div>
      </div>

      <button
        type="button"
        onclick={() => {
          settings.minimize_to_tray = !settings.minimize_to_tray;
          saveSettings();
        }}
        class={`w-11 h-6 p-0.5 rounded-full flex items-center transition-colors cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--ca-brand)] ${
          settings.minimize_to_tray
            ? 'bg-[var(--ca-brand)] justify-end'
            : 'bg-[var(--ca-surface-subtle)] border border-[var(--ca-border)] justify-start'
        }`}
        title={settings.minimize_to_tray ? 'Disable minimize to tray' : 'Enable minimize to tray'}
        aria-label={t('settings.minimizeToTray')}
      >
        <span
          class="w-5 h-5 rounded-full bg-white shadow-xs transition-transform"
        ></span>
      </button>
    </div>
    <p class="text-[11px] text-[var(--ca-text-secondary)]">
      {settings.minimize_to_tray
        ? 'Closing window keeps CACalm alive and playing in background.'
        : 'Closing window exits CACalm completely.'}
    </p>
  </div>
</div>
