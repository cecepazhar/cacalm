<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { isTauri } from '$lib/env';
  import { invoke } from '@tauri-apps/api/core';
  import { ThemeSwitcher, LanguageSwitcher, Badge, Button, Modal } from '@cecepazhar/caui';
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

  onMount(() => {
    initTheme();
  });

  const navItems = $derived([
    { label: t('nav.mixer'), icon: '🎛️', href: '/' },
    { label: t('nav.library'), icon: '🎵', href: '/library' },
    { label: t('nav.settings'), icon: '⚙️', href: '/settings' }
  ]);
</script>

<div class="h-screen w-full overflow-hidden flex flex-col bg-[var(--ca-canvas)] text-[var(--ca-text-primary)] select-none">
  <!-- Unified Top Titlebar / Navigation Header -->
  <header
    data-tauri-drag-region
    class="flex items-center justify-between px-4 py-2 border-b border-[var(--ca-border)] bg-[var(--ca-surface)]/90 backdrop-blur-md z-30 transition-colors"
  >
    <!-- Left: Brand Logo & Pro Badge -->
    <div class="flex items-center gap-2.5 no-drag">
      <a href="/" class="flex items-center gap-2 group cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--ca-brand)] rounded-lg px-1 py-0.5">
        <span class="w-7 h-7 rounded-lg bg-[var(--ca-brand)]/15 border border-[var(--ca-brand)]/35 flex items-center justify-center text-sm shadow-xs transition-transform group-hover:scale-105">
          🌿
        </span>
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

    <!-- Center: Navigation Tabs -->
    <nav class="flex items-center gap-1 no-drag" aria-label="Main Navigation">
      {#each navItems as item (item.href)}
        {@const active = page.url.pathname === item.href}
        <a
          href={item.href}
          class={`px-3 py-1.5 text-xs font-semibold rounded-lg flex items-center gap-1.5 transition-all cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--ca-brand)] ${
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

    <!-- Right: Quick Controls, Theme, Language & Window Actions -->
    <div class="flex items-center gap-2 no-drag">
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
        <div class="h-4 w-px bg-[var(--ca-border)] mx-0.5" aria-hidden="true"></div>
        <button
          type="button"
          onclick={() => invoke('minimize_to_tray')}
          class="w-7 h-7 flex items-center justify-center rounded-lg bg-[var(--ca-surface-subtle)] border border-[var(--ca-border)] text-[var(--ca-text-secondary)] hover:text-[var(--ca-text-primary)] hover:bg-[var(--ca-surface-elevated)] transition-colors cursor-pointer text-xs focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--ca-brand)]"
          title={t('app.minimizeToTray')}
          aria-label={t('app.minimizeToTray')}
        >
          —
        </button>
        <button
          type="button"
          onclick={() => window.close()}
          class="w-7 h-7 flex items-center justify-center rounded-lg bg-[var(--ca-surface-subtle)] border border-[var(--ca-border)] text-[var(--ca-text-secondary)] hover:text-[var(--ca-danger)] hover:bg-[var(--ca-surface-elevated)] transition-colors cursor-pointer text-xs focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--ca-danger)]"
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
