<script lang="ts">
  import { onMount } from 'svelte';
  import Icon from '$lib/components/Icon.svelte';
  import { showToast } from '$lib/toast';
  import { t } from '$lib/i18n/index.svelte';
  import { isTauri } from '$lib/env';
  import { engine } from '$lib/audio/engine';
  import { getMixer, refreshCustomSounds, toggleSound } from '$lib/stores/mixer.svelte';

  interface SoundEntry {
    id: string;
    name: string;
    category: string;
    file_name: string;
    imported_at: number;
  }

  const mixer = getMixer();
  let sounds = $state<SoundEntry[]>([]);
  let busy = $state(false);

  onMount(() => {
    void load();
  });

  async function load() {
    if (!isTauri()) return;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const all = await invoke<SoundEntry[]>('list_sounds');
      sounds = all.filter((s) => s.category === 'imported');
    } catch (e) {
      console.error('Failed to load library:', e);
    }
  }

  async function handleImport() {
    if (!isTauri()) {
      showToast(t('library.desktopOnly'));
      return;
    }
    busy = true;
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const { invoke } = await import('@tauri-apps/api/core');
      const picked = await open({
        multiple: true,
        filters: [{ name: 'Audio', extensions: ['mp3', 'wav', 'flac', 'ogg', 'opus', 'm4a'] }],
      });
      if (!picked) return;
      const paths = Array.isArray(picked) ? picked : [picked];
      for (const p of paths) {
        const fileName = p.split(/[\\/]/).pop() || 'audio';
        try {
          await invoke('import_sound', { sourcePath: p, name: fileName.replace(/\.[^/.]+$/, ''), category: 'imported' });
        } catch (e) {
          console.error(`Failed to import ${fileName}:`, e);
        }
      }
      await load();
      await refreshCustomSounds();
      showToast(t('library.importSuccess'), 'success');
    } catch (e) {
      console.error('Import failed:', e);
    } finally {
      busy = false;
    }
  }

  async function handleDelete(id: string) {
    if (!confirm(t('library.deleteConfirm'))) return;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      engine.stop(`custom:${id}`, 0.3);
      await invoke('delete_sound', { id });
      await load();
      await refreshCustomSounds();
      showToast(t('library.deleteSuccess'));
    } catch (e: unknown) {
      showToast((e as { message?: string })?.message || t('library.deleteFailed'), 'error');
    }
  }
</script>

<div class="max-w-5xl mx-auto w-full px-4 sm:px-6 py-6 space-y-6">
  <div class="flex flex-wrap items-end justify-between gap-4">
    <div>
      <h1 class="text-xl font-semibold tracking-tight">{t('library.title')}</h1>
      <p class="text-sm text-[var(--ca-text-muted)] mt-1">{t('library.subtitle')}</p>
    </div>
    <button
      type="button"
      onclick={handleImport}
      disabled={busy}
      class="inline-flex items-center gap-2 rounded-full px-4 py-2 bg-[var(--ca-brand)] text-white text-sm font-semibold shadow-sm hover:brightness-110 disabled:opacity-50"
    >
      <Icon name="upload" size={16} />
      {t('library.addAudio')}
    </button>
  </div>

  {#if sounds.length}
    <div class="text-[11px] uppercase tracking-wider font-semibold text-[var(--ca-text-muted)]">
      {t('library.fileCount', { count: sounds.length })}
    </div>
    <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
      {#each sounds as s (s.id)}
        {@const cid = `custom:${s.id}`}
        {@const inMix = mixer.selected.includes(cid)}
        <div class="flex items-center gap-3 rounded-2xl border border-[var(--ca-border)]/70 bg-[var(--ca-surface)] p-3.5">
          <span class="w-10 h-10 rounded-xl flex items-center justify-center bg-[var(--ca-surface-subtle)]/80 text-[var(--ca-brand)]">
            {#if mixer.loading[cid]}
              <span class="w-4 h-4 rounded-full border-2 border-current/30 border-t-current calm-spin"></span>
            {:else}
              <Icon name="music" size={18} />
            {/if}
          </span>
          <div class="min-w-0 flex-1">
            <div class="text-[13px] font-semibold truncate">{s.name}</div>
            <div class="text-[11px] text-[var(--ca-text-muted)] truncate font-mono">{s.file_name}</div>
          </div>
          <button
            type="button"
            onclick={() => toggleSound(cid)}
            class="inline-flex items-center gap-1.5 rounded-full px-3 py-1.5 text-xs font-medium border transition-colors {inMix
              ? 'bg-[var(--ca-brand)] text-white border-transparent'
              : 'border-[var(--ca-border)] text-[var(--ca-text-secondary)] hover:text-[var(--ca-brand)] hover:border-[var(--ca-brand)]/50'}"
          >
            <Icon name={inMix ? 'check' : 'plus'} size={13} />
            {inMix ? t('library.inMix') : t('library.addToMix')}
          </button>
          <button
            type="button"
            onclick={() => handleDelete(s.id)}
            class="w-8 h-8 rounded-lg flex items-center justify-center text-[var(--ca-text-muted)] hover:text-[var(--ca-danger)] hover:bg-[var(--ca-danger)]/10"
            title={t('dialogs.delete')}
            aria-label="{t('dialogs.delete')} {s.name}"
          >
            <Icon name="trash" size={15} />
          </button>
        </div>
      {/each}
    </div>
  {:else}
    <button
      type="button"
      onclick={handleImport}
      class="w-full rounded-3xl border-2 border-dashed border-[var(--ca-border)] p-12 text-center text-sm text-[var(--ca-text-muted)] hover:text-[var(--ca-brand)] hover:border-[var(--ca-brand)]/50 transition-colors"
    >
      <Icon name="upload" size={28} class="mx-auto mb-3" />
      {isTauri() ? t('library.empty') : t('library.desktopOnly')}
    </button>
  {/if}
</div>
