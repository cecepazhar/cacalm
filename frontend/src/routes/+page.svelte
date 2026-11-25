<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import SoundCard from '$lib/components/SoundCard.svelte';
  import BreathGuide from '$lib/components/BreathGuide.svelte';
  import { t, getLocale } from '$lib/i18n/index.svelte';
  import { CATEGORIES, SCENES, SOUNDS, SOUND_BY_ID, loc, type CategoryId } from '$lib/audio/catalog';
  import {
    getMixer,
    applyScene,
    applyUserMix,
    deleteUserMix,
    saveUserMix,
    clearMix,
  } from '$lib/stores/mixer.svelte';

  const mixer = getMixer();

  type Tab = 'all' | CategoryId;
  const TAB_KEY = 'cacalm_tab';
  let tab = $state<Tab>(readTab());
  let savingMix = $state(false);
  let mixName = $state('');

  function readTab(): Tab {
    try {
      return (localStorage.getItem(TAB_KEY) as Tab) || 'all';
    } catch {
      return 'all';
    }
  }

  function selectTab(next: Tab) {
    tab = next;
    try {
      localStorage.setItem(TAB_KEY, next);
    } catch {
      /* ignore */
    }
  }

  const hour = new Date().getHours();
  const period = hour < 11 ? 'morning' : hour < 15 ? 'afternoon' : hour < 19 ? 'evening' : 'night';

  const visible = $derived(tab === 'all' ? SOUNDS : SOUNDS.filter((s) => s.category === tab));
  const activeCategory = $derived(CATEGORIES.find((c) => c.id === tab));

  const breathId = $derived(
    mixer.playing ? mixer.selected.find((id) => SOUND_BY_ID.get(id)?.breath && !mixer.loading[id]) : undefined
  );

  function submitMix(e: Event) {
    e.preventDefault();
    saveUserMix(mixName || `Mix ${mixer.userMixes.length + 1}`);
    mixName = '';
    savingMix = false;
  }

  const tabBtn = (active: boolean) =>
    `flex items-center gap-1.5 rounded-full px-3.5 py-1.5 text-xs font-medium whitespace-nowrap transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--ca-brand)] ${
      active
        ? 'bg-[var(--ca-text-primary)] text-[var(--ca-canvas)]'
        : 'text-[var(--ca-text-secondary)] hover:text-[var(--ca-text-primary)] hover:bg-[var(--ca-surface-subtle)]/70'
    }`;
</script>

<div class="calm-page">
  <!-- Hero -->
  <section class="relative overflow-hidden rounded-3xl border border-[var(--ca-border)]/60 bg-[var(--ca-surface)]">
    <div class="calm-aurora {mixer.playing ? 'is-playing' : ''}" aria-hidden="true"></div>
    <div class="relative p-5 sm:p-7 space-y-5">
      <div class="flex flex-wrap items-start justify-between gap-4">
        <div>
          <h1 class="text-2xl sm:text-[28px] font-semibold tracking-tight">{t(`greeting.${period}`)}</h1>
          <p class="text-sm text-[var(--ca-text-secondary)] mt-1">{t(`greeting.${period}Sub`)}</p>
        </div>
        {#if breathId}
          <BreathGuide id={breathId} phases={SOUND_BY_ID.get(breathId)!.breath!} />
        {/if}
      </div>

      <div>
        <div class="calm-label mb-2">
          {t('sounds.scenes')}
        </div>
        <div class="flex gap-2.5 overflow-x-auto pb-1 -mx-1 px-1 min-[1500px]:grid min-[1500px]:grid-cols-8 min-[1500px]:overflow-visible">
          {#each SCENES as scene (scene.id)}
            {@const on = mixer.sceneId === scene.id}
            <button
              type="button"
              onclick={() => applyScene(scene)}
              class="shrink-0 w-44 min-[1500px]:w-auto text-left rounded-2xl p-3 border backdrop-blur-sm transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--ca-brand)] {on
                ? 'bg-[var(--ca-brand)] text-white border-transparent shadow-lg'
                : 'bg-[var(--ca-surface)]/70 border-[var(--ca-border)]/60 hover:bg-[var(--ca-surface)]'}"
            >
              <div class="flex items-center justify-between">
                <Icon name={scene.icon} size={18} class={on ? '' : 'text-[var(--ca-brand)]'} />
                {#if on && mixer.playing}
                  <span class="calm-eq [&>span]:!bg-white" aria-hidden="true"><span></span><span></span><span></span></span>
                {/if}
              </div>
              <div class="mt-2 text-[13px] font-semibold">{loc(scene.name, getLocale())}</div>
              <div class="text-[11px] leading-snug {on ? 'text-white/80' : 'text-[var(--ca-text-muted)]'}">
                {loc(scene.desc, getLocale())}
              </div>
            </button>
          {/each}
        </div>
      </div>
    </div>
  </section>

  <!-- My mixes -->
  {#if mixer.userMixes.length || mixer.selected.length}
    <section class="flex flex-wrap items-center gap-2">
      <span class="calm-label mr-1">
        {t('sounds.myMixes')}
      </span>
      {#each mixer.userMixes as m (m.id)}
        <span
          class="group inline-flex items-center rounded-full border text-xs transition-colors {mixer.sceneId === m.id
            ? 'border-[var(--ca-brand)]/50 bg-[var(--ca-brand)]/10 text-[var(--ca-brand)]'
            : 'border-[var(--ca-border)] text-[var(--ca-text-secondary)]'}"
        >
          <button type="button" class="pl-3 pr-1.5 py-1 hover:text-[var(--ca-text-primary)]" onclick={() => applyUserMix(m)}>
            {m.name}
          </button>
          <button
            type="button"
            class="pr-2 py-1 opacity-50 hover:opacity-100 hover:text-[var(--ca-danger)]"
            onclick={() => deleteUserMix(m.id)}
            aria-label="{t('dialogs.delete')} {m.name}"
          >
            <Icon name="x" size={12} />
          </button>
        </span>
      {/each}
      {#if mixer.selected.length}
        {#if savingMix}
          <form onsubmit={submitMix} class="inline-flex items-center gap-1">
            <!-- svelte-ignore a11y_autofocus -->
            <input
              type="text"
              bind:value={mixName}
              autofocus
              maxlength="32"
              placeholder={t('sounds.mixNamePlaceholder')}
              class="w-40 rounded-full bg-[var(--ca-surface)] border border-[var(--ca-border)] px-3 py-1 text-xs focus:outline-none focus:ring-2 focus:ring-[var(--ca-brand)] select-text"
              onkeydown={(e) => e.key === 'Escape' && (savingMix = false)}
            />
            <button type="submit" class="rounded-full p-1.5 bg-[var(--ca-brand)] text-white" aria-label={t('dialogs.save')}>
              <Icon name="check" size={13} />
            </button>
          </form>
        {:else}
          <button
            type="button"
            onclick={() => (savingMix = true)}
            class="inline-flex items-center gap-1.5 rounded-full border border-dashed border-[var(--ca-border)] px-3 py-1 text-xs text-[var(--ca-text-secondary)] hover:text-[var(--ca-brand)] hover:border-[var(--ca-brand)]/50"
          >
            <Icon name="save" size={13} />
            {t('sounds.saveMix')}
          </button>
        {/if}
        <button
          type="button"
          onclick={clearMix}
          class="ml-auto inline-flex items-center gap-1.5 rounded-full px-3 py-1 text-xs text-[var(--ca-text-muted)] hover:text-[var(--ca-danger)]"
        >
          <Icon name="trash" size={13} />
          {t('sounds.clear')}
        </button>
      {/if}
    </section>
  {/if}

  <!-- Category tabs -->
  <section class="space-y-4">
    <div class="flex items-center gap-1.5 overflow-x-auto pb-1">
      <button type="button" class={tabBtn(tab === 'all')} onclick={() => selectTab('all')}>
        <Icon name="grid" size={14} />
        {t('sounds.all')}
        <span class="opacity-60">{SOUNDS.length}</span>
      </button>
      {#each CATEGORIES as c (c.id)}
        <button type="button" class={tabBtn(tab === c.id)} onclick={() => selectTab(c.id)}>
          <Icon name={c.icon} size={14} />
          {loc(c.name, getLocale())}
        </button>
      {/each}
      <button type="button" class={tabBtn(tab === 'mine')} onclick={() => selectTab('mine')}>
        <Icon name="library" size={14} />
        {t('sounds.mine')}
        {#if mixer.custom.length}<span class="opacity-60">{mixer.custom.length}</span>{/if}
      </button>
    </div>

    {#if activeCategory}
      <p class="text-xs text-[var(--ca-text-muted)] -mt-1">{loc(activeCategory.desc, getLocale())}</p>
    {/if}

    {#if tab === 'mine'}
      {#if mixer.custom.length}
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5 min-[1800px]:grid-cols-6 gap-3">
          {#each mixer.custom as c (c.id)}
            <SoundCard id={c.id} name={c.name} icon="music" desc={t('library.title')} />
          {/each}
        </div>
      {:else}
        <a
          href="/library"
          class="block rounded-2xl border border-dashed border-[var(--ca-border)] p-10 text-center text-sm text-[var(--ca-text-muted)] hover:text-[var(--ca-brand)] hover:border-[var(--ca-brand)]/50"
        >
          <Icon name="upload" size={22} class="mx-auto mb-2" />
          {t('sounds.emptyMine')}
        </a>
      {/if}
    {:else}
      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5 min-[1800px]:grid-cols-6 gap-3">
        {#each visible as s (s.id)}
          <SoundCard
            id={s.id}
            name={loc(s.name, getLocale())}
            desc={loc(s.desc, getLocale())}
            icon={s.icon}
            headphones={s.headphones}
          />
        {/each}
      </div>
    {/if}
  </section>
</div>
