<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import Logo from '$lib/components/Logo.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { SOUNDS } from '$lib/audio/catalog';
  import { APP_VERSION, BRAND, openExternal } from '$lib/brand';

  const features = [
    { icon: 'leaf', title: 'about.offlineTitle', desc: 'about.offlineDesc' },
    { icon: 'heart', title: 'about.privacyTitle', desc: 'about.privacyDesc' },
    { icon: 'grid', title: 'about.platformsTitle', desc: 'about.platformsDesc' },
  ];
</script>

<div class="calm-page">
  <section class="relative overflow-hidden rounded-3xl border border-[var(--ca-border)]/60 bg-[var(--ca-surface)]">
    <div class="calm-aurora" aria-hidden="true"></div>
    <div class="relative p-7 sm:p-10 flex flex-col sm:flex-row items-center sm:items-start gap-6 text-center sm:text-left">
      <div class="relative shrink-0">
        <span class="absolute inset-0 rounded-3xl bg-[var(--ca-brand)] calm-breathe-ring" aria-hidden="true"></span>
        <div class="relative w-24 h-24 rounded-3xl bg-[var(--ca-surface)] border border-[var(--ca-border)]/60 flex items-center justify-center shadow-lg">
          <Logo size={60} mode="brand" />
        </div>
      </div>
      <div class="space-y-2">
        <div class="flex flex-wrap items-baseline justify-center sm:justify-start gap-2">
          <h1 class="text-3xl font-semibold tracking-tight">CACalm</h1>
          <span class="text-xs font-mono text-[var(--ca-text-muted)]">{t('about.version')} {APP_VERSION}</span>
        </div>
        <p class="text-sm text-[var(--ca-text-secondary)]">{t('about.subtitle')}</p>
        <div class="flex flex-wrap justify-center sm:justify-start gap-2 pt-1">
          <span class="text-[11px] font-medium px-2.5 py-1 rounded-full bg-[var(--ca-brand)]/12 text-[var(--ca-brand)]">{t('app.free')}</span>
          <span class="text-[11px] font-medium px-2.5 py-1 rounded-full bg-[var(--ca-surface-subtle)]/80 text-[var(--ca-text-secondary)]">
            {t('about.soundsCount', { count: SOUNDS.length })}
          </span>
          <span class="text-[11px] font-medium px-2.5 py-1 rounded-full bg-[var(--ca-surface-subtle)]/80 text-[var(--ca-text-secondary)]">Windows · macOS · Linux</span>
        </div>
      </div>
    </div>
  </section>

    <section class="grid sm:grid-cols-3 gap-5">
    {#each features as f (f.title)}
      <div class="calm-card p-5 space-y-2">
        <span class="w-9 h-9 rounded-xl flex items-center justify-center bg-[var(--ca-brand)]/10 text-[var(--ca-brand)]">
          <Icon name={f.icon} size={17} />
        </span>
        <div class="text-sm font-semibold">{t(f.title)}</div>
        <p class="text-xs leading-relaxed text-[var(--ca-text-muted)]">{t(f.desc)}</p>
      </div>
    {/each}
  </section>

  <div class="grid gap-5 xl:grid-cols-2">
  <section class="calm-card p-5 flex flex-col sm:flex-row sm:items-center gap-4">
    <div class="flex-1">
      <div class="calm-label">{t('about.madeBy')}</div>
      <div class="text-lg font-semibold mt-1">{BRAND.author} · {BRAND.company}</div>
      <div class="text-xs text-[var(--ca-text-muted)] mt-0.5">© {BRAND.year} {BRAND.authorFull} — {BRAND.company}</div>
      <div class="text-xs text-[var(--ca-text-muted)]">{t('about.license')}</div>
    </div>
    <div class="flex gap-2">
      <button
        type="button"
        onclick={() => openExternal(BRAND.repoUrl)}
        class="inline-flex items-center gap-2 rounded-full px-4 py-2 text-sm border border-[var(--ca-border)] hover:bg-[var(--ca-surface-subtle)]/60"
      >
        <Icon name="github" size={16} />
        {t('about.repo')}
      </button>
    </div>
  </section>



  <section class="rounded-2xl border border-rose-500/25 bg-rose-500/5 p-5 flex flex-col sm:flex-row sm:items-center gap-4">
    <span class="w-10 h-10 rounded-xl flex items-center justify-center bg-rose-500/15 text-rose-500 shrink-0">
      <Icon name="heart" size={18} />
    </span>
    <p class="flex-1 text-sm text-[var(--ca-text-secondary)]">{t('about.supportDesc')}</p>
    <button
      type="button"
      onclick={() => openExternal(BRAND.supportUrl)}
      class="inline-flex items-center justify-center gap-2 rounded-full px-4 py-2 text-sm font-semibold bg-rose-500 text-white hover:brightness-110"
    >
      <Icon name="heart" size={15} />
      {t('about.support')}
    </button>
  </section>
  </div>
</div>
