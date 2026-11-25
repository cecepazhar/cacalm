/// <reference lib="webworker" />
// Renders procedural soundscapes off the UI thread.
import { renderSound } from './generators';

interface Req {
  reqId: number;
  id: string;
  sr: number;
}

self.onmessage = (e: MessageEvent<Req>) => {
  const { reqId, id, sr } = e.data;
  try {
    const [L, R] = renderSound(id, sr);
    (self as unknown as DedicatedWorkerGlobalScope).postMessage({ reqId, L, R }, [L.buffer, R.buffer]);
  } catch (err) {
    (self as unknown as DedicatedWorkerGlobalScope).postMessage({ reqId, error: String(err) });
  }
};
