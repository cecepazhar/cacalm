<script lang="ts">
  import Icon from './Icon.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import {
    getMixer,
    toggleSound,
    setVolume,
    setPan,
    toggleMute,
    toggleSolo,
  } from '$lib/stores/mixer.svelte';

  interface Props {
    id: string;
    name: string;
    desc?: string;
    icon: string;
    headphones?: boolean;
  }

  let { id, name, desc = '', icon, headphones = false }: Props = $props();

  const mixer = getMixer();
  let optionsOpen = $state(false);

  const selected = $derived(mixer.selected.includes(id));
  const ch = $derived(mixer.ch[id]);
  const volume = $derived(ch?.volume ?? 0.5);
  const pan = $derived(ch?.pan ?? 0);
  const muted = $derived(ch?.muted ?? false);
  const loading = $derived(!!mixer.loading[id]);
  const live = $derived(selected && mixer.playing);
  const soloed = $derived(mixer.solo === id);
  const silenced = $derived(muted || (mixer.solo !== null && !soloed));

  function panLabel(p: number) {
    if (Math.abs(p) < 0.08) return t('sounds.center');
    return `${p < 0 ? t('sounds.left') : t('sounds.right')} ${Math.round(Math.abs(p) * 100)}`;
  }
</script>

<svelte:window onclick={() => (optionsOpen = false)} />

<div
  class="group relative rounded-2xl border transition-all duration-300 {selected
    ? 'border-[var(--ca-brand)]/45 bg-[var(--calm-card-active)] shadow-[0_0_0_1px_var(--ca-brand-glow),0_8px_24px_-12px_var(--ca-brand-glow)]'
    : 'border-[var(--ca-border)]/70 bg-[var(--ca-surface)] hover:border-[var(--ca-border-hover)] hover:-translate-y-0.5'}"
>
  <button
    type="button"
    onclick={() => toggleSound(id)}
    class="w-full text-left p-4 pb-3 flex items-start gap-3 rounded-2xl focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--ca-brand)]"
    aria-pressed={selected}
    aria-label={name}
  >
    <span
      class="w-10 h-10 rounded-xl flex items-center justify-center shrink-0 transition-colors {selected
        ? 'bg-[var(--ca-brand)] text-white'
        : 'bg-[var(--ca-surface-subtle)]/80 text-[var(--ca-text-secondary)] group-hover:text-[var(--ca-brand)]'}"
    >
      {#if loading}
        <span class="w-4 h-4 rounded-full border-2 border-current/30 border-t-current calm-spin"></span>
      {:else}
        <Icon name={icon} size={20} />
      {/if}
    </span>
    <span class="min-w-0 flex-1 pt-0.5">
      <span class="flex items-center gap-1.5">
        <span class="text-[13px] font-semibold truncate">{name}</span>
        {#if headphones}
          <span title={t('sounds.headphones')} class="text-[var(--ca-text-muted)]"><Icon name="headphones" size={12} /></span>
        {/if}
      </span>
      <span class="block text-[11px] leading-snug text-[var(--ca-text-muted)] line-clamp-2 mt-0.5">
        {loading ? t('sounds.loading') : desc}
      </span>
    </span>
    {#if live && !silenced && !loading}
      <span class="calm-eq mt-1.5" aria-hidden="true"><span></span><span></span><span></span></span>
    {/if}
  </button>

  {#if selected}
    <div class="px-4 pb-3.5 flex items-center gap-2.5">
      <button
        type="button"
        onclick={() => toggleMute(id)}
        class="text-[var(--ca-text-muted)] hover:text-[var(--ca-text-primary)]"
        title={muted ? t('sounds.unmute') : t('sounds.mute')}
        aria-label={muted ? t('sounds.unmute') : t('sounds.mute')}
      >
        <Icon name={silenced ? 'volume-x' : 'volume'} size={15} />
      </button>
      <input
        type="range"
        min="0"
        max="1"
        step="0.01"
        value={volume}
        oninput={(e) => setVolume(id, Number((e.target as HTMLInputElement).value))}
        class="calm-range"
        style="--val:{Math.round(volume * 100)}"
        aria-label="{name} {t('sounds.volume')}"
      />
      <span class="w-7 text-right text-[10px] font-mono tabular-nums text-[var(--ca-text-muted)]">{Math.round(volume * 100)}</span>
      <div class="relative">
        <button
          type="button"
          onclick={(e) => {
            e.stopPropagation();
            optionsOpen = !optionsOpen;
          }}
          class="w-7 h-7 rounded-lg flex items-center justify-center text-[var(--ca-text-muted)] hover:text-[var(--ca-text-primary)] hover:bg-[var(--ca-surface-subtle)]"
          title={t('sounds.options')}
          aria-label={t('sounds.options')}
          aria-expanded={optionsOpen}
        >
          <Icon name="more" size={16} stroke={3} />
        </button>
        {#if optionsOpen}
          <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
          <div
            class="absolute right-0 top-9 z-30 w-56 rounded-2xl border border-[var(--ca-border)] bg-[var(--ca-surface)] shadow-xl p-3 space-y-3"
            onclick={(e) => e.stopPropagation()}
          >
            <div class="space-y-1">
              <div class="flex justify-between text-[11px] text-[var(--ca-text-muted)]">
                <span>{t('sounds.pan')}</span>
                <span class="font-mono">{panLabel(pan)}</span>
              </div>
              <input
                type="range"
                min="-1"
                max="1"
                step="0.05"
                value={pan}
                oninput={(e) => setPan(id, Number((e.target as HTMLInputElement).value))}
                ondblclick={() => setPan(id, 0)}
                class="calm-range"
                style="--val:{Math.round((pan + 1) * 50)}"
                aria-label="{name} {t('sounds.pan')}"
              />
            </div>
            <div class="grid grid-cols-2 gap-1.5 text-xs">
              <button
                type="button"
                onclick={() => toggleSolo(id)}
                class="rounded-lg py-1.5 border transition-colors {soloed
                  ? 'bg-[var(--ca-brand)] text-white border-transparent'
                  : 'border-[var(--ca-border)] hover:bg-[var(--ca-surface-subtle)]'}"
              >
                {soloed ? t('sounds.unsolo') : t('sounds.solo')}
              </button>
              <button
                type="button"
                onclick={() => {
                  optionsOpen = false;
                  toggleSound(id);
                }}
                class="rounded-lg py-1.5 border border-[var(--ca-border)] hover:bg-[var(--ca-danger)]/10 hover:text-[var(--ca-danger)] hover:border-[var(--ca-danger)]/40 transition-colors"
              >
                {t('sounds.remove')}
              </button>
            </div>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>
