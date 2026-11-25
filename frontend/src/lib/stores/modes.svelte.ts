// Which Immersive Mode is open (null = none). Kept global so the layout can render
// the full-screen overlay above every page.
import { MODE_BY_ID, type ModeDef } from '$lib/modes/catalog';

const state = $state({ active: null as ModeDef | null });

export function getModes() {
  return state;
}

export function openMode(id: string) {
  state.active = MODE_BY_ID.get(id) ?? null;
}

export function closeMode() {
  state.active = null;
}
