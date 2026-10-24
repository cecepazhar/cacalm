<script lang="ts">
  import { onMount } from 'svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import { invoke } from '@tauri-apps/api/core';
  import { Button, Badge } from '@cecepazhar/caui';
  import { showToast } from '$lib/toast';
  import { t } from '$lib/i18n/index.svelte';

  interface SoundEntry {
    id: string;
    name: string;
    category: string;
    duration_secs: number;
    file_name: string;
  }

  let sounds = $state<SoundEntry[]>([]);
  let isLoading = $state(false);

  onMount(async () => {
    await loadSounds();
  });

  async function loadSounds() {
    isLoading = true;
    try {
      sounds = await invoke<SoundEntry[]>('list_sounds');
    } catch (e) {
      console.error('Failed to load library:', e);
    } finally {
      isLoading = false;
    }
  }

  async function handleImport() {
    try {
      const path = await open({
        multiple: true,
        filters: [
          { name: 'Audio Files', extensions: ['mp3', 'wav', 'flac', 'ogg', 'opus', 'm4a'] }
        ]
      });

      if (path) {
        const paths = Array.isArray(path) ? path : [path];
        for (const p of paths) {
          const fileName = p.split('/').pop() || p.split('\\').pop() || 'unknown';
          const cleanName = fileName.replace(/\.[^/.]+$/, '');
          try {
            await invoke('import_sound', {
              sourcePath: p,
              name: cleanName,
              category: 'imported'
            });
          } catch (e: unknown) {
            console.error(`Failed to import ${fileName}:`, e);
          }
        }
        await loadSounds();
        showToast(t('library.importSuccess'), 'success');
      }
    } catch (e) {
      console.error('Import dialog failed:', e);
    }
  }

  async function handleDelete(id: string) {
    if (confirm(t('library.deleteConfirm'))) {
      try {
        await invoke('delete_sound', { id });
        await loadSounds();
        showToast(t('library.deleteSuccess'), 'info');
      } catch (e: unknown) {
        const msg = (e as { message?: string })?.message || t('library.deleteFailed');
        showToast(msg, 'error');
      }
    }
  }
</script>

<div class="flex flex-col h-full overflow-y-auto p-6 space-y-4 bg-[var(--ca-canvas)] text-[var(--ca-text-primary)] max-w-7xl mx-auto w-full">
  <!-- Header -->
  <div class="flex items-center justify-between border-b border-[var(--ca-border)] pb-4">
    <div>
      <h1 class="text-base font-bold text-[var(--ca-text-primary)] flex items-center gap-2">
        🎵 {t('library.title')}
        <Badge variant="neutral" size="xs">
          {t('library.fileCount', { count: sounds.length })}
        </Badge>
      </h1>
      <p class="text-xs text-[var(--ca-text-muted)] mt-0.5">
        {t('library.subtitle')}
      </p>
    </div>

    <Button
      variant="brand"
      size="sm"
      onclick={handleImport}
      disabled={isLoading}
      class="font-semibold"
    >
      {t('library.addAudio')}
    </Button>
  </div>

  <!-- Sounds Grid -->
  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 pt-2">
    {#each sounds as sound (sound.id)}
      <div class="bg-[var(--ca-surface)] border border-[var(--ca-border)] rounded-xl p-4 space-y-3 shadow-xs hover:border-[var(--ca-brand)]/40 transition-colors">
        <div class="flex items-start justify-between">
          <div class="flex items-center gap-3">
            <div class="w-9 h-9 rounded-lg bg-[var(--ca-brand)]/10 text-[var(--ca-brand)] flex items-center justify-center text-lg border border-[var(--ca-brand)]/20">
              🎵
            </div>
            <div>
              <h3 class="text-xs font-bold text-[var(--ca-text-primary)] truncate max-w-[140px]">{sound.name}</h3>
              <span class="text-[9px] text-[var(--ca-text-muted)] font-mono uppercase">{sound.category}</span>
            </div>
          </div>

          <button
            type="button"
            onclick={() => handleDelete(sound.id)}
            class="text-[var(--ca-text-muted)] hover:text-[var(--ca-danger)] transition-colors text-xs p-1 cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--ca-danger)] rounded"
            title={t('dialogs.delete')}
            aria-label={t('dialogs.delete')}
          >
            ✕
          </button>
        </div>

        <div class="bg-[var(--ca-surface-subtle)] p-2 rounded-lg text-[10px] font-mono text-[var(--ca-text-secondary)] flex items-center justify-between">
          <span>{sound.duration_secs > 0 ? `${Math.floor(sound.duration_secs / 60)}m ${Math.round(sound.duration_secs % 60)}s` : 'Continuous'}</span>
          <span class="truncate max-w-[140px]">{sound.file_name}</span>
        </div>
      </div>
    {:else}
      <div class="col-span-full p-10 text-center text-[var(--ca-text-muted)] text-xs bg-[var(--ca-surface)] border border-dashed border-[var(--ca-border)] rounded-xl">
        {t('library.empty')}
      </div>
    {/each}
  </div>
</div>
