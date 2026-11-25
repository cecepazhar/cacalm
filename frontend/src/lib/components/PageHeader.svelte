<script lang="ts">
  import type { Snippet } from 'svelte';
  import Icon from './Icon.svelte';

  interface Props {
    title: string;
    subtitle?: string;
    icon?: string;
    actions?: Snippet;
  }

  let { title, subtitle = '', icon, actions }: Props = $props();
</script>

<header class="flex flex-wrap items-end justify-between gap-4">
  <div class="flex items-center gap-3.5 min-w-0">
    {#if icon}
      <span class="w-11 h-11 shrink-0 rounded-2xl flex items-center justify-center bg-[var(--ca-brand)]/12 text-[var(--ca-brand)]">
        <Icon name={icon} size={20} />
      </span>
    {/if}
    <div class="min-w-0">
      <h1 class="text-[22px] font-semibold tracking-tight leading-tight">{title}</h1>
      {#if subtitle}
        <p class="text-sm text-[var(--ca-text-muted)] mt-1">{subtitle}</p>
      {/if}
    </div>
  </div>
  {#if actions}
    <div class="flex items-center gap-2">{@render actions()}</div>
  {/if}
</header>
