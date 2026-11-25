<script lang="ts">
  import { ICONS } from './icons';

  interface Props {
    name: string;
    size?: number;
    stroke?: number;
    class?: string;
  }

  let { name, size = 18, stroke = 1.75, class: cls = '' }: Props = $props();
</script>

<svg
  xmlns="http://www.w3.org/2000/svg"
  viewBox="0 0 24 24"
  width={size}
  height={size}
  fill="none"
  stroke="currentColor"
  stroke-width={stroke}
  stroke-linecap="round"
  stroke-linejoin="round"
  class="shrink-0 {cls}"
  aria-hidden="true"
>
  <path d={ICONS[name] ?? ICONS.music} />
</svg>
