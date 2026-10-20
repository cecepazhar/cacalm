<script lang="ts">
  import { onMount } from 'svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import { invoke } from '@tauri-apps/api/core';
  import { showToast } from '$lib/toast';

  interface SoundEntry {
    id: string;
    name: string;
    category: string;
    duration_secs: number;
    file_name: string;
  }

  let sounds = $state<SoundEntry[]>([]);
  let isLoading = $state(false);
  let isAddOpen = $state(false);

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
          } catch (e: any) {
            console.error(`Failed to import ${fileName}:`, e);
          }
        }
        await loadSounds();
        showToast('Import file audio berhasil.', 'success');
      }
    } catch (e) {
      console.error('Import dialog failed:', e);
    }
  }

  async function handleDelete(id: string) {
    if (confirm('Yakin ingin menghapus audio ini?')) {
      try {
        await invoke('delete_sound', { id });
        await loadSounds();
        showToast('Suara dihapus.', 'info');
      } catch (e: any) {
        showToast(e?.message || 'Gagal menghapus.', 'error');
      }
    }
  }
</script>

<div class="flex flex-col h-full overflow-y-auto p-6 space-y-4 bg-[#0A0A0C] text-[#EDEDED] max-w-7xl mx-auto w-full">
  <!-- Header -->
  <div class="flex items-center justify-between border-b border-[#272732] pb-4">
    <div>
      <h1 class="text-lg font-bold text-white flex items-center gap-2">
        🎵 Sound Library
        <span class="text-[10px] font-mono text-neutral-500 bg-[#18181F] px-2 py-0.5 rounded border border-[#272732]">
          {sounds.length} File
        </span>
      </h1>
      <p class="text-xs text-neutral-400">Punya perpustakaan custom audio MP3, WAV, FLAC, atau OGG</p>
    </div>

    <button
      onclick={handleImport}
      disabled={isLoading}
      class="px-3.5 py-1.5 text-xs font-bold text-black bg-teal-500 hover:bg-teal-400 rounded-lg transition-colors cursor-pointer disabled:opacity-50"
    >
      + Tambah Audio
    </button>
  </div>

  <!-- Sounds Grid -->
  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 pt-2">
    {#each sounds as sound (sound.id)}
      <div class="bg-[#111827] border border-[#272732] rounded-xl p-4 space-y-3 shadow-sm hover:border-teal-500/40 transition-colors">
        <div class="flex items-start justify-between">
          <div class="flex items-center gap-3">
            <div class="w-9 h-9 rounded-lg bg-[#18181F] flex items-center justify-center text-teal-400 text-lg">
              🎵
            </div>
            <div>
              <h3 class="text-xs font-bold text-white truncate max-w-[120px]">{sound.name}</h3>
              <span class="text-[9px] text-neutral-500 font-mono uppercase">{sound.category}</span>
            </div>
          </div>

          <button
            onclick={() => handleDelete(sound.id)}
            class="text-neutral-500 hover:text-rose-400 text-xs"
            title="Hapus Audio"
          >
            ✕
          </button>
        </div>

        <div class="bg-[#18181F] p-2 rounded-lg text-[10px] font-mono text-neutral-400 flex items-center justify-between">
          <span>Duration: {sound.duration_secs > 0 ? `${Math.floor(sound.duration_secs / 60)}m ${sound.duration_secs % 60}s` : 'Unknown'}</span>
          <span>{sound.file_name}</span>
        </div>
      </div>
    {:else}
      <div class="col-span-full p-8 text-center text-neutral-500 text-xs bg-[#111827] border border-[#272732] rounded-xl">
        No imported audio files yet. Click "+ Tambah Audio" to import MP3, WAV, FLAC, or OGG clips.
      </div>
    {/each}
  </div>
</div>
