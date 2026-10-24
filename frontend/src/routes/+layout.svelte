<script lang="ts">
  import '../app.css';
  import { page } from '$app/state';
  import { isTauri } from '$lib/env';
  import { invoke } from '@tauri-apps/api/core';

  const navItems = [
    { label: 'Mixer', icon: '🎛️', href: '/' },
    { label: 'Library', icon: '🎵', href: '/library' },
    { label: 'Settings', icon: '⚙️', href: '/settings' }
  ];
</script>

<div class="h-screen w-full overflow-hidden flex flex-col">
  {#if isTauri()}
    <nav class="flex items-center justify-between px-6 py-2 bg-[#0D0D10] border-b border-[#272732] text-[#EDEDED]">
      <div class="flex items-center gap-4">
        <div class="flex items-center gap-2 text-white">
          <span class="text-lg">🌿</span>
          <span class="font-bold text-sm tracking-tight">CACalm</span>
          <span class="text-[10px] font-mono px-1.5 py-0.5 rounded bg-[#18181F] text-neutral-400 border border-[#272732]">v0.1.1</span>
        </div>
      </div>

      <div class="flex items-center gap-1">
        {#each navItems as item}
          <a
            href={item.href}
            class={`px-3 py-1.5 text-xs font-semibold rounded-lg flex items-center gap-2 transition-colors cursor-pointer ${
              page.url.pathname === item.href
                ? 'bg-teal-500/10 text-teal-300 border border-teal-500/40'
                : 'text-neutral-400 hover:text-white hover:bg-[#18181F]'
            }`}
          >
            <span class="text-sm">{item.icon}</span>
            <span>{item.label}</span>
          </a>
        {/each}
      </div>

      <div class="flex items-center gap-2">
        <button
          onclick={() => invoke('minimize_to_tray')}
          class="w-7 h-7 flex items-center justify-center rounded-lg bg-[#18181F] border border-[#272732] text-neutral-400 hover:text-white hover:bg-[#202028] transition-colors cursor-pointer text-xs"
          title="Minimize ke Tray (jendela tetap terbuka di background)"
        >
          —
        </button>
        <button
          onclick={() => window.close()}
          class="w-7 h-7 flex items-center justify-center rounded-lg bg-[#18181F] border border-[#272732] text-neutral-400 hover:text-rose-400 hover:bg-[#202028] transition-colors cursor-pointer text-xs"
          title="Tutup Jendela"
        >
          ✕
        </button>
      </div>
    </nav>
  {/if}

  <div class="flex-1 overflow-y-auto">
    <slot />
  </div>
</div>
