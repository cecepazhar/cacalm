<script lang="ts">
  interface Props {
    title: string;
    description?: string;
  }

  let { title, description }: Props = $props();
</script>

<header class="pb-5 border-b border-[#272732]">
  <h1 class="text-xl font-bold text-white tracking-wide">{title}</h1>
  {#if description}
    <p class="text-sm text-neutral-400 mt-0.5">{description}</p>
  {/if}
</header>
