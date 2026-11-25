<script lang="ts">
  import Icon from './Icon.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { getMixer, cancelPowerAction } from '$lib/stores/mixer.svelte';

  const mixer = getMixer();
  const shutdown = $derived(mixer.powerCountdownAction === 'shutdown');
</script>

{#if mixer.powerCountdown !== null}
  <div class="fixed inset-0 z-[80] flex items-center justify-center bg-black/60" role="alertdialog" aria-modal="true" aria-labelledby="pc-title">
    <div class="w-[min(92vw,420px)] calm-card p-6 text-center space-y-4 shadow-2xl">
      <span class="mx-auto w-12 h-12 rounded-2xl flex items-center justify-center {shutdown ? 'bg-rose-500/15 text-rose-500' : 'bg-indigo-500/15 text-indigo-400'}">
        <Icon name={shutdown ? 'power' : 'moon'} size={22} />
      </span>
      <h2 id="pc-title" class="text-lg font-semibold">
        {shutdown ? t('sleep.countdownShutdown') : t('sleep.countdownSleep')}
      </h2>
      <p class="text-5xl font-light font-mono tabular-nums">{mixer.powerCountdown}</p>
      {#if shutdown}
        <p class="text-xs text-[var(--ca-text-muted)]">{t('sleep.saveWork')}</p>
      {/if}
      <!-- svelte-ignore a11y_autofocus -->
      <button
        type="button"
        autofocus
        onclick={cancelPowerAction}
        class="w-full rounded-xl py-3 font-semibold bg-[var(--ca-brand)] text-white hover:brightness-110"
      >
        {t('dialogs.cancel')}
      </button>
    </div>
  </div>
{/if}
