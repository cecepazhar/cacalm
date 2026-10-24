<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { showToast } from '$lib/toast';

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
      showToast('Gagal memuat pengaturan.', 'error');
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
      showToast('Pengaturan disimpan.', 'success');
    } catch (e) {
      console.error('Failed to save settings:', e);
      showToast('Gagal menyimpan pengaturan.', 'error');
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

<div class="flex flex-col h-full overflow-y-auto p-6 space-y-4 bg-[#0A0A0C] text-[#EDEDED] max-w-7xl mx-auto w-full">
  <!-- Header -->
  <div class="flex items-center justify-between border-b border-[#272732] pb-4">
    <div>
      <h1 class="text-lg font-bold text-white flex items-center gap-2">
        ⚙️ Pengaturan
        <span class="text-[10px] font-mono text-neutral-500 bg-[#18181F] px-2 py-0.5 rounded border border-[#272732]">
          {isLoaded ? 'Dimuat' : 'Memuat...'}
        </span>
      </h1>
      <p class="text-xs text-neutral-400">Konfigurasi tray icon, shortcut global, dan perilaku aplikasi.</p>
    </div>
  </div>

  <!-- Toggle Tray -->
  <div class="bg-[#111827] border border-[#272732] rounded-xl p-5 space-y-4 shadow-sm hover:border-teal-500/40 transition-colors">
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-3">
        <div class="w-10 h-10 rounded-lg bg-[#18181F] flex items-center justify-center text-teal-400 text-lg">
          🖥️
        </div>
        <div>
          <h3 class="text-xs font-bold text-white">Tray Icon</h3>
          <span class="text-[9px] text-neutral-500 font-mono uppercase">tampilkan di system tray</span>
        </div>
      </div>

      <button
        onclick={toggleTray}
        class={`w-11 h-6 p-0.5 rounded-full flex items-center transition-colors cursor-pointer ${
          settings.tray_enabled
            ? 'bg-teal-500 justify-end'
            : 'bg-[#18181F] border border-[#272732] justify-start'
        }`}
        title={settings.tray_enabled ? 'Nonaktifkan tray' : 'Aktifkan tray'}
      >
        <span
          class={`w-5 h-5 rounded-full bg-white transition-transform ${
            settings.tray_enabled ? 'translate-x-5' : 'translate-x-0'
          }`}
        ></span>
      </button>
    </div>
    <p class="text-[11px] text-neutral-400">
      {settings.tray_enabled
        ? 'Tray icon aktif. Klik kiri untuk menu cepat, klik dua kali untuk membuka aplikasi.'
        : 'Tray icon disembunyikan dari system tray.'}
    </p>
  </div>

  <!-- Keyboard Shortcuts -->
  <div class="bg-[#111827] border border-[#272732] rounded-xl p-5 space-y-4 shadow-sm hover:border-teal-500/40 transition-colors">
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-3">
        <div class="w-10 h-10 rounded-lg bg-[#18181F] flex items-center justify-center text-teal-400 text-lg">
          ⌨️
        </div>
        <div>
          <h3 class="text-xs font-bold text-white">Shortcut Global</h3>
          <span class="text-[9px] text-neutral-500 font-mono uppercase">berlaku saat aplikasi terbuka</span>
        </div>
      </div>

      <button
        onclick={toggleShortcuts}
        class={`w-11 h-6 p-0.5 rounded-full flex items-center transition-colors cursor-pointer ${
          settings.shortcuts_enabled
            ? 'bg-teal-500 justify-end'
            : 'bg-[#18181F] border border-[#272732] justify-start'
        }`}
        title={settings.shortcuts_enabled ? 'Nonaktifkan shortcut' : 'Aktifkan shortcut'}
      >
        <span
          class={`w-5 h-5 rounded-full bg-white transition-transform ${
            settings.shortcuts_enabled ? 'translate-x-5' : 'translate-x-0'
          }`}
        ></span>
      </button>
    </div>

    {#if settings.shortcuts_enabled}
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4 pt-2 border-t border-[#1f2937]">
        <!-- Timer Toggle -->
        <div class="space-y-2">
          <label for="shortcut-timer" class="text-[10px] font-bold uppercase tracking-wider text-neutral-400 block">
            Mulai/Jeda Timer
          </label>
          <input
            id="shortcut-timer"
            type="text"
            bind:value={settings.shortcut_timer_toggle}
            class="w-full bg-[#18181F] border border-[#272732] rounded-lg px-3 py-2 text-xs font-mono text-white focus:outline-none focus:ring-2 focus:ring-teal-500"
            placeholder="Ctrl+Alt+C"
            oninput={() => {
              if (!validateShortcut(settings.shortcut_timer_toggle)) {
                showToast('Format shortcut tidak valid. Gunakan kombinasi seperti Ctrl+Alt+C', 'error');
              }
            }}
            onblur={saveSettings}
          />
          <p class="text-[10px] text-neutral-500">
            Gunakan modifier + kombinasi, contoh: Ctrl+Alt+C, Cmd+Shift+P
          </p>
        </div>

        <!-- Open App -->
        <div class="space-y-2">
          <label for="shortcut-open" class="text-[10px] font-bold uppercase tracking-wider text-neutral-400 block">
            Buka Aplikasi
          </label>
          <input
            id="shortcut-open"
            type="text"
            bind:value={settings.shortcut_open_app}
            class="w-full bg-[#18181F] border border-[#272732] rounded-lg px-3 py-2 text-xs font-mono text-white focus:outline-none focus:ring-2 focus:ring-teal-500"
            placeholder="Ctrl+Alt+L"
            oninput={() => {
              if (!validateShortcut(settings.shortcut_open_app)) {
                showToast('Format shortcut tidak valid. Gunakan kombinasi seperti Ctrl+Alt+L', 'error');
              }
            }}
            onblur={saveSettings}
          />
          <p class="text-[10px] text-neutral-500">
            Membuka jendela utama dan memfokuskan aplikasi.
          </p>
        </div>
      </div>
    {:else}
      <p class="text-[11px] text-neutral-500 pt-2 border-t border-[#1f2937]">
        Shortcut global dinonaktifkan. Aktivkan toggle di atas untuk mengatur.
      </p>
    {/if}
  </div>

  <!-- Minimize to Tray -->
  <div class="bg-[#111827] border border-[#272732] rounded-xl p-5 space-y-4 shadow-sm hover:border-teal-500/40 transition-colors">
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-3">
        <div class="w-10 h-10 rounded-lg bg-[#18181F] flex items-center justify-center text-teal-400 text-lg">
          📥
        </div>
        <div>
          <h3 class="text-xs font-bold text-white">Minimize to Tray</h3>
          <span class="text-[9px] text-neutral-500 font-mono uppercase">
            tutup jendela = sembunyikan ke tray
          </span>
        </div>
      </div>

      <button
        onclick={() => {
          settings.minimize_to_tray = !settings.minimize_to_tray;
          saveSettings();
        }}
        class={`w-11 h-6 p-0.5 rounded-full flex items-center transition-colors cursor-pointer ${
          settings.minimize_to_tray
            ? 'bg-teal-500 justify-end'
            : 'bg-[#18181F] border border-[#272732] justify-start'
        }`}
        title={settings.minimize_to_tray ? 'Matikan minimize ke tray' : 'Aktifkan minimize ke tray'}
      >
        <span
          class={`w-5 h-5 rounded-full bg-white transition-transform ${
            settings.minimize_to_tray ? 'translate-x-5' : 'translate-x-0'
          }`}
        ></span>
      </button>
    </div>
    <p class="text-[11px] text-neutral-400">
      {settings.minimize_to_tray
        ? 'Menutup jendela aplikasi tidak akan mengakhiri proses. Aplikasi tetap berjalan di system tray.'
        : 'Menutup jendela akan sepenuhnya menutup aplikasi.'}
    </p>
  </div>
</div>
