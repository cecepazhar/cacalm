// Personal details shown in the UI (currently just the name used in the greeting).
const KEY = 'cacalm_name';

function read(): string {
  try {
    return (localStorage.getItem(KEY) ?? '').slice(0, 40);
  } catch {
    return '';
  }
}

const state = $state({ name: typeof window === 'undefined' ? '' : read() });

export function getProfile() {
  return state;
}

export function setName(name: string) {
  state.name = name.trim().slice(0, 40);
  try {
    if (state.name) localStorage.setItem(KEY, state.name);
    else localStorage.removeItem(KEY);
  } catch {
    /* ignore */
  }
}
