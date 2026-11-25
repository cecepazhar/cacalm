<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { isTauri } from '$lib/env';
  import { invoke } from '@tauri-apps/api/core';
  import { ThemeSwitcher, LanguageSwitcher } from '@cecepazhar/caui';
  import Logo from '$lib/components/Logo.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import PlayerBar from '$lib/components/PlayerBar.svelte';
  import { initTheme } from '$lib/stores/theme.svelte';
  import { getLocale, setLocale, t } from '$lib/i18n/index.svelte';
  import { initMixer, togglePlay, toggleMasterMute } from '$lib/stores/mixer.svelte';
  import { initTimer, toggleTimer, resetTimer, skipPhase } from '$lib/stores/timer.svelte';
  import { APP_VERSION, BRAND, openExternal } from '$lib/brand';

  let { children } = $props();

  let isMaximized = $state(false);

  async function startDragging(e: MouseEvent) {
    if (e.button !== 0) return;
    const target = e.target as HTMLElement | null;
    if (target?.closest('button, input, textarea, a, select, [role="button"], .no-drag')) return;
    try {
      await invoke('window_start_dragging');
    } catch {
      /* browser preview */
    }
  }

  async function toggleMaximize() {
    try {
      await invoke('window_maximize');
      isMaximized = await invoke<boolean>('window_is_maximized');
    } catch {
      /* browser preview */
    }
  }

  async function minimizeWindow() {
    try {
      await invoke('window_minimize');
    } catch {
      /* browser preview */
    }
  }

  async function closeWindow() {
    try {
      await invoke('window_close');
    } catch {
      window.close();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    const el = e.target as HTMLElement | null;
    const typing = el?.closest('input[type="text"], textarea, [contenteditable="true"]');
    if (typing) return;
    const onFocusPage = page.url.pathname.startsWith('/focus');
    if (e.code === 'Space' && !el?.closest('button, input, a, [role="button"]')) {
      e.preventDefault();
      if (onFocusPage) void toggleTimer();
      else togglePlay();
    } else if ((e.ctrlKey || e.metaKey) && !e.shiftKey && e.key.toLowerCase() === 'm') {
      e.preventDefault();
      toggleMasterMute();
    } else if ((e.ctrlKey || e.metaKey) && !e.shiftKey && e.key.toLowerCase() === 'r' && onFocusPage) {
      e.preventDefault();
      void resetTimer();
    } else if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key.toLowerCase() === 'n') {
      e.preventDefault();
      void skipPhase();
    }
  }

  onMount(() => {
    initTheme();
    initMixer();
    initTimer();
    if (isTauri()) {
      invoke<boolean>('window_is_maximized')
        .then((m) => (isMaximized = m))
        .catch(() => {});
    }
    window.addEventListener('keydown', handleKeydown);
    return () => window.removeEventListener('keydown', handleKeydown);
  });

  const navItems = $derived([
    { label: t('nav.sounds'), icon: 'headphones', href: '/' },
    { label: t('nav.focus'), icon: 'timer', href: '/focus' },
    { label: t('nav.library'), icon: 'library', href: '/library' },
    { label: t('nav.settings'), icon: 'settings', href: '/settings' },
    { label: t('nav.about'), icon: 'info', href: '/about' },
  ]);

  function isActive(href: string) {
    return href === '/' ? page.url.pathname === '/' : page.url.pathname.startsWith(href);
  }

  const winBtn =
    'w-8 h-8 flex items-center justify-center rounded-lg text-[var(--ca-text-muted)] hover:text-[var(--ca-text-primary)] hover:bg-[var(--ca-surface-subtle)] transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--ca-brand)]';
</script>

<div class="h-screen w-full overflow-hidden flex flex-col bg-[var(--ca-canvas)] text-[var(--ca-text-primary)] select-none">
  <!-- Title bar -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <header
    data-tauri-drag-region
    onmousedown={startDragging}
    ondblclick={(e) => {
      const target = e.target as HTMLElement | null;
      if (!target?.closest('button, input, textarea, a, select, [role="button"], .no-drag')) void toggleMaximize();
    }}
    class="h-11 shrink-0 flex items-center justify-between pl-4 pr-2 border-b border-[var(--ca-border)]/70 bg-[var(--ca-surface)]/80 backdrop-blur-md z-30"
  >
    <div class="flex items-center gap-2.5" data-tauri-drag-region>
      <Logo size={22} mode="brand" />
      <span class="font-semibold text-[13px] tracking-tight" data-tauri-drag-region>CACalm</span>
      <span
        class="hidden sm:inline text-[10px] font-medium px-2 py-0.5 rounded-full bg-[var(--ca-brand)]/12 text-[var(--ca-brand)] border border-[var(--ca-brand)]/25"
        data-tauri-drag-region
      >
        {t('app.free')}
      </span>
    </div>

    <div class="flex-1 h-full" data-tauri-drag-region></div>

    <div class="flex items-center gap-1 no-drag">
      <LanguageSwitcher compact value={getLocale()} onchange={(code) => setLocale(code as 'en' | 'id')} size="sm" />
      <ThemeSwitcher showAccentPicker={false} size="sm" />
      {#if isTauri()}
        <div class="h-4 w-px bg-[var(--ca-border)] mx-1.5" aria-hidden="true"></div>
        <button type="button" onclick={minimizeWindow} class={winBtn} title={t('app.minimize')} aria-label={t('app.minimize')}>
          <Icon name="minus" size={15} />
        </button>
        <button
          type="button"
          onclick={toggleMaximize}
          class={winBtn}
          title={isMaximized ? t('app.restore') : t('app.maximize')}
          aria-label={isMaximized ? t('app.restore') : t('app.maximize')}
        >
          <Icon name={isMaximized ? 'restore' : 'maximize'} size={13} />
        </button>
        <button
          type="button"
          onclick={closeWindow}
          class="{winBtn} hover:!bg-[var(--ca-danger)] hover:!text-white"
          title={t('app.close')}
          aria-label={t('app.close')}
        >
          <Icon name="x" size={15} />
        </button>
      {/if}
    </div>
  </header>

  <div class="flex-1 min-h-0 flex">
    <!-- Sidebar -->
    <aside
      class="w-[68px] lg:w-56 shrink-0 flex flex-col justify-between border-r border-[var(--ca-border)]/70 bg-[var(--ca-surface)]/50 py-3 px-2.5"
    >
      <nav class="flex flex-col gap-1" aria-label="Main">
        {#each navItems as item (item.href)}
          {@const active = isActive(item.href)}
          <a
            href={item.href}
            class="group flex items-center gap-3 rounded-xl px-3 py-2.5 text-[13px] font-medium transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--ca-brand)] {active
              ? 'bg-[var(--ca-brand)]/12 text-[var(--ca-brand)]'
              : 'text-[var(--ca-text-secondary)] hover:text-[var(--ca-text-primary)] hover:bg-[var(--ca-surface-subtle)]/70'}"
            aria-current={active ? 'page' : undefined}
            title={item.label}
          >
            <Icon name={item.icon} size={18} />
            <span class="hidden lg:inline">{item.label}</span>
          </a>
        {/each}
      </nav>

      <!-- Branding -->
      <div class="hidden lg:block rounded-2xl border border-[var(--ca-border)]/70 bg-[var(--ca-surface)] p-3.5 space-y-2">
        <div class="flex items-center gap-2">
          <Logo size={18} mode="brand" />
          <span class="text-xs font-semibold">CACalm</span>
          <span class="text-[10px] text-[var(--ca-text-muted)] font-mono">v{APP_VERSION}</span>
        </div>
        <p class="text-[11px] leading-snug text-[var(--ca-text-muted)]">{t('app.by')}</p>
        <button
          type="button"
          onclick={() => openExternal(BRAND.supportUrl)}
          class="w-full flex items-center justify-center gap-1.5 rounded-lg py-1.5 text-[11px] font-medium text-[var(--ca-text-secondary)] border border-[var(--ca-border)] hover:text-rose-500 hover:border-rose-500/40 transition-colors"
        >
          <Icon name="heart" size={13} />
          {t('about.support')}
        </button>
      </div>
      <button
        type="button"
        class="lg:hidden flex justify-center py-2 text-[var(--ca-text-muted)] hover:text-[var(--ca-brand)]"
        title="CACalm v{APP_VERSION} — {t('app.by')}"
        onclick={() => goto('/about')}
      >
        <Logo size={20} mode="brand" />
      </button>
    </aside>

    <!-- Content -->
    <main class="flex-1 min-w-0 overflow-y-auto">
      {@render children?.()}
    </main>
  </div>

  <PlayerBar />
</div>
