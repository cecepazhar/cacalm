// Drives the boot splash defined inline in app.html (same pattern as CAPost). The splash
// stays up while CACalm warms up — at least MIN_MS so it never flickers, never more than
// MAX_MS (app.html also enforces a hard cap, so a stuck promise can't keep anyone waiting).

import { engine } from '$lib/audio/engine';
import { getMixer } from '$lib/stores/mixer.svelte';
import { t } from '$lib/i18n/index.svelte';

const MIN_MS = 900;
const MAX_MS = 3500;

type BootApi = { start: number; step: (text: string, pct?: number) => void; done: () => void };
const api = (): BootApi | undefined =>
  typeof window !== 'undefined' ? (window as unknown as { __cacalmBoot?: BootApi }).__cacalmBoot : undefined;

const sleep = (ms: number) => new Promise<void>((r) => setTimeout(r, ms));
// rAF never fires while the window is still hidden, so cap the wait.
const frame = () =>
  Promise.race([
    new Promise<void>((r) => requestAnimationFrame(() => requestAnimationFrame(() => r()))),
    sleep(120),
  ]);

async function warmUp(step: BootApi['step']) {
  step(t('boot.prefs'), 25);
  await frame();

  // Render the restored mix now, so pressing Play is instant.
  const mixer = getMixer();
  const ids = mixer.selected.slice(0, 4);
  if (ids.length) {
    step(t('boot.sounds'), 45);
    let done = 0;
    await Promise.all(
      ids.map((id) =>
        engine.warm([id]).then(() => {
          done += 1;
          step(t('boot.sounds'), 45 + Math.round((40 * done) / ids.length));
        })
      )
    );
  }

  step(t('boot.fonts'), 88);
  if (document.fonts?.ready) await Promise.race([document.fonts.ready.then(() => undefined), sleep(300)]);

  step(t('boot.ready'), 96);
  await frame(); // let the real UI paint underneath before the splash fades
}

let finished = false;

export async function finishBoot(): Promise<void> {
  const boot = api();
  if (!boot || finished) return;
  finished = true;
  await Promise.race([warmUp(boot.step), sleep(MAX_MS)]);
  const elapsed = performance.now() - boot.start;
  if (elapsed < MIN_MS) await sleep(MIN_MS - elapsed);
  boot.done();
}
