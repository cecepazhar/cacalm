<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { isTauri } from '$lib/env';
  import { invoke } from '@tauri-apps/api/core';
  import { ThemeSwitcher, LanguageSwitcher, Badge, Button, Modal } from '@cecepazhar/caui';
  import Logo from '$lib/components/Logo.svelte';
  import { initTheme } from '$lib/stores/theme.svelte';
  import { getLocale, setLocale, t } from '$lib/i18n/index.svelte';
  import {
    getPro,
    togglePlan,
    upgradeToPro,
    openUpgradeModal,
    closeUpgradeModal
  } from '$lib/stores/pro.svelte';

  let { children } = $props();

  const pro = getPro();
  let isMaximized = $state(false);

  async function startDragging(e: MouseEvent) {
    if (e.button !== 0) return;
    const target = e.target as HTMLElement | null;
    if (target?.closest('button, input, textarea, a, select, [role="button"], .no-drag')) return;
    try {
      await invoke('window_start_dragging');
    } catch {
      try {
        if (typeof window !== 'undefined') {
          const { getCurrentWindow } = await import('@tauri-apps/api/window');
          await getCurrentWindow().startDragging();
        }
      } catch (err) {
        console.warn('Failed to start dragging window:', err);
      }
    }
  }

  async function toggleMaximize() {
    try {
      await invoke('window_maximize');
      isMaximized = !isMaximized;
    } catch {
      try {
        const { getCurrentWindow } = await import('@tauri-apps/api/window');
        const win = getCurrentWindow();
        if (await win.isMaximized()) {
          await win.unmaximize();
          isMaximized = false;
        } else {
          await win.maximize();
          isMaximized = true;
        }
      } catch (err) {
        console.warn('Failed to toggle maximize:', err);
      }
    }
  }

  async function minimizeWindow() {
    try {
      await invoke('window_minimize');
    } catch {
      try {
        const { getCurrentWindow } = await import('@tauri-apps/api/window');
        await getCurrentWindow().minimize();
      } catch {
        void invoke('minimize_to_tray');
      }
    }
  }

  async function closeWindow() {
    try {
      await invoke('window_close');
    } catch {
      window.close();
    }
  }

  onMount(() => {
    initTheme();
    if (isTauri()) {
      invoke<boolean>('window_is_maximized')
        .then((m) => {
          isMaximized = m;
        })
        .catch(() => {});
    }
  });

  const navItems = $derived([
    { label: t('nav.mixer'), icon: '🎛️', href: '/' },
    { label: t('nav.library'), icon: '🎵', href: '/library' },
    { label: t('nav.settings'), icon: '⚙️', href: '/settings' }
  ]);
</script>

<div class="h-screen w-full overflow-hidden flex flex-col bg-[var(--ca-canvas)] text-[var(--ca-text-primary)] select-none">
  <!-- Unified Top Titlebar / Navigation Header -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <header
    data-tauri-drag-region
    onmousedown={startDragging}
    ondblclick={(e) => {
      const target = e.target as HTMLElement | null;
      if (!target?.closest('button, input, textarea, a, select, [role="button"], .no-drag')) {
        void toggleMaximize();
      }
    }}
    class="flex items-center justify-between px-3.5 py-1.5 border-b border-[var(--ca-border)] bg-[var(--ca-surface)]/90 backdrop-blur-md z-30 transition-colors select-none cursor-default shrink-0"
  >
    <!-- Left: Brand Logo & Pro Badge -->
    <div class="flex items-center gap-2.5" data-tauri-drag-region>
      <a
        href="/"
        class="flex items-center gap-2 group cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--ca-brand)] rounded-lg px-1 py-0.5 no-drag"
        title="CACalm — Cecep Azhar Sovereign Acoustic Workspace"
      >
        <Logo size={22} mode="brand" />
        <div class="flex items-baseline gap-1.5">
          <span class="font-bold text-sm tracking-tight text-[var(--ca-text-primary)]">CACalm</span>
          <span class="text-[10px] font-mono px-1.5 py-0.2 rounded bg-[var(--ca-surface-subtle)] text-[var(--ca-text-muted)] border border-[var(--ca-border)]">
            v0.1.1
          </span>
        </div>
      </a>

      <!-- Pro Badge (interactive toggle / upgrade trigger) -->
      <button
        type="button"
        onclick={() => {
          if (pro.isPro) {
            togglePlan();
          } else {
            openUpgradeModal();
          }
        }}
        class="transition-transform active:scale-95 cursor-pointer no-drag focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--ca-brand)] rounded-md"
        title={pro.isPro ? t('pro.statusPro') + ' (' + t('pro.switchPlan') + ')' : t('pro.upgradeTitle')}
      >
        {#if pro.isPro}
          <Badge variant="brand" size="xs">
            <span class="font-bold">★ {t('pro.badgePro')}</span>
          </Badge>
        {:else}
          <Badge variant="secondary" size="xs">
            <span class="text-[var(--ca-text-muted)] font-semibold">{t('pro.badgeFree')}</span>
          </Badge>
        {/if}
      </button>
    </div>

    <!-- Draggable flexible gap between left and center -->
    <div class="flex-1 h-full min-w-[12px] cursor-default" data-tauri-drag-region></div>

    <!-- Center: Navigation Tabs -->
    <nav class="flex items-center gap-1 no-drag" aria-label="Main Navigation">
      {#each navItems as item (item.href)}
        {@const active = page.url.pathname === item.href}
        <a
          href={item.href}
          class={`px-3 py-1 text-xs font-semibold rounded-lg flex items-center gap-1.5 transition-all cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--ca-brand)] ${
            active
              ? 'bg-[var(--ca-brand)]/15 text-[var(--ca-brand)] border border-[var(--ca-brand)]/35 shadow-xs'
              : 'text-[var(--ca-text-secondary)] hover:text-[var(--ca-text-primary)] hover:bg-[var(--ca-surface-subtle)] border border-transparent'
          }`}
          aria-current={active ? 'page' : undefined}
        >
          <span class="text-xs">{item.icon}</span>
          <span>{item.label}</span>
        </a>
      {/each}
    </nav>

    <!-- Draggable flexible gap between center and right -->
    <div class="flex-1 h-full min-w-[12px] cursor-default" data-tauri-drag-region></div>

    <!-- Right: Quick Controls, Theme, Language & Frameless Window Controls -->
    <div class="flex items-center gap-1.5 no-drag">
      <!-- Language Switcher -->
      <LanguageSwitcher
        compact
        value={getLocale()}
        onchange={(code) => setLocale(code as 'en' | 'id')}
        size="sm"
      />

      <!-- Theme Switcher (sun/moon) -->
      <ThemeSwitcher showAccentPicker={false} size="sm" />

      {#if isTauri()}
        <div class="h-4 w-px bg-[var(--ca-border)] mx-1" aria-hidden="true"></div>

        <!-- Minimize -->
        <button
          type="button"
          onclick={minimizeWindow}
          class="w-7 h-7 flex items-center justify-center rounded-lg bg-[var(--ca-surface-subtle)] border border-[var(--ca-border)] text-[var(--ca-text-secondary)] hover:text-[var(--ca-text-primary)] hover:bg-[var(--ca-surface-elevated)] transition-colors cursor-pointer text-xs focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--ca-brand)]"
          title={t('app.minimize')}
          aria-label={t('app.minimize')}
        >
          —
        </button>

        <!-- Maximize / Restore -->
        <button
          type="button"
          onclick={toggleMaximize}
          class="w-7 h-7 flex items-center justify-center rounded-lg bg-[var(--ca-surface-subtle)] border border-[var(--ca-border)] text-[var(--ca-text-secondary)] hover:text-[var(--ca-text-primary)] hover:bg-[var(--ca-surface-elevated)] transition-colors cursor-pointer text-xs focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--ca-brand)]"
          title={isMaximized ? t('app.restore') : t('app.maximize')}
          aria-label={isMaximized ? t('app.restore') : t('app.maximize')}
        >
          {#if isMaximized}
            <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
              <rect x="4" y="8" width="11" height="11" rx="1.5" />
              <path d="M8 4h11a1 1 0 011 1v11" stroke-linecap="round" />
            </svg>
          {:else}
            <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
              <rect x="4" y="4" width="16" height="16" rx="2" />
            </svg>
          {/if}
        </button>

        <!-- Close -->
        <button
          type="button"
          onclick={closeWindow}
          class="w-7 h-7 flex items-center justify-center rounded-lg bg-[var(--ca-surface-subtle)] border border-[var(--ca-border)] text-[var(--ca-text-secondary)] hover:text-white hover:bg-[var(--ca-danger,#ef4444)] hover:border-[var(--ca-danger,#ef4444)] transition-colors cursor-pointer text-xs focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--ca-danger,#ef4444)]"
          title={t('app.close')}
          aria-label={t('app.close')}
        >
          ✕
        </button>
      {/if}
    </div>
  </header>

  <!-- Main Content Viewport -->
  <main class="flex-1 overflow-y-auto bg-[var(--ca-canvas)] text-[var(--ca-text-primary)]">
    {@render children?.()}
  </main>

  <!-- Pro Upgrade Modal -->
  <Modal
    bind:open={pro.showUpgradeModal}
    title={t('pro.upgradeTitle')}
    description={t('pro.upgradeDesc')}
    onclose={closeUpgradeModal}
    size="md"
  >
    <div class="space-y-4 py-2">
      <div class="rounded-xl border border-[var(--ca-brand)]/30 bg-[var(--ca-brand)]/5 p-4 space-y-2">
        <h4 class="text-xs font-bold text-[var(--ca-brand)] uppercase tracking-wider">
          {t('pro.unlockChannels')}
        </h4>
        <ul class="text-xs space-y-1.5 text-[var(--ca-text-secondary)]">
          <li class="flex items-center gap-2">
            <span class="text-[var(--ca-brand)] font-bold">✓</span>
            <span>🌲 {t('channels.birds')} & ☕ {t('channels.cafe')}</span>
          </li>
          <li class="flex items-center gap-2">
            <span class="text-[var(--ca-brand)] font-bold">✓</span>
            <span>🔥 {t('channels.fire')} & 🏞️ {t('channels.stream')}</span>
          </li>
          <li class="flex items-center gap-2">
            <span class="text-[var(--ca-brand)] font-bold">✓</span>
            <span>🧠 {t('channels.binaural')} & 🎧 {t('channels.spatial')}</span>
          </li>
          <li class="flex items-center gap-2">
            <span class="text-[var(--ca-brand)] font-bold">✓</span>
            <span>{t('pro.lifetimeAccess')}</span>
          </li>
        </ul>
      </div>

      <div class="flex items-center justify-between pt-2">
        <span class="text-[11px] text-[var(--ca-text-muted)]">
          {t('pro.trialNotice')}
        </span>
        <div class="flex items-center gap-2">
          <Button variant="outline" size="sm" onclick={closeUpgradeModal}>
            {t('dialogs.cancel')}
          </Button>
          <Button variant="brand" size="sm" onclick={upgradeToPro}>
            {t('pro.upgradeButton')}
          </Button>
        </div>
      </div>
    </div>
  </Modal>
</div>
