<script lang="ts">
  import { onMount } from 'svelte';
  import PageHeader from '$lib/components/PageHeader.svelte';

  let welcome = $state('CACalm');
  let version = 'v0.1.0';

  onMount(() => {
    console.log('[CACalm] App mounted');
  });
</script>

<div class="flex-1 flex flex-col h-full bg-[#0A0A0C] text-[#EDEDED] overflow-y-auto p-6 space-y-6 font-sans">
  <PageHeader
    title="CACalm"
    description="Offline ambient mixer & focus timer"
  />

  <div class="p-8 rounded-xl bg-[#111827] border border-[#272732] text-center space-y-3">
    <h1 class="text-3xl font-bold tracking-tight">Welcome to {welcome}</h1>
    <p class="text-neutral-400 text-sm">{version}</p>
    <p class="text-xs text-neutral-500">Multi-channel ambient mixer · Pomodoro focus timer · No account required</p>
  </div>

  <!-- Placeholder sections for Phase 1 components -->
  <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
    <div class="p-6 rounded-xl bg-[#111827] border border-[#272732]">
      <div class="text-xs font-mono uppercase text-neutral-500 mb-2">Channel Mixer</div>
      <div class="text-sm text-neutral-400">Coming in Phase 1</div>
    </div>
    <div class="p-6 rounded-xl bg-[#111827] border border-[#272732]">
      <div class="text-xs font-mono uppercase text-neutral-500 mb-2">Pomodoro Timer</div>
      <div class="text-sm text-neutral-400">Coming in Phase 2</div>
    </div>
  </div>
</div>
