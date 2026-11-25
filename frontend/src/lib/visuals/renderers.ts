// Lightweight Canvas 2D renderers for Immersive Mode.
//
// Performance rules (see notes T1.11): static scenery is painted once into an offscreen
// background; each frame only draws small particles from pre-rendered sprites; no canvas
// filters / blur; the host caps the frame rate (30 fps) and stops drawing when idle.

import { Rng } from '$lib/audio/dsp';

export type RendererId = 'night' | 'nap' | 'forest' | 'hearth' | 'romantic';

/** Per-renderer budget: internal resolution (fraction of the screen) and frame rate. */
export interface Quality {
  scale: number;
  fps: number;
}

export interface Renderer {
  quality: Quality;
  resize(w: number, h: number): void;
  /** t = seconds since start, dt = seconds since last frame. */
  frame(ctx: CanvasRenderingContext2D, t: number, dt: number): void;
  /** Optional live parameters (e.g. nap "wake" brightness 0..1). */
  set?(key: string, value: number): void;
}

type Stop = [number, string];

function canvas(w: number, h: number): [HTMLCanvasElement, CanvasRenderingContext2D] {
  const c = document.createElement('canvas');
  c.width = Math.max(1, Math.round(w));
  c.height = Math.max(1, Math.round(h));
  return [c, c.getContext('2d')!];
}

/** Soft round glow sprite. */
function glow(radius: number, stops: Stop[]): HTMLCanvasElement {
  const [c, g] = canvas(radius * 2, radius * 2);
  const grad = g.createRadialGradient(radius, radius, 0, radius, radius, radius);
  for (const [o, col] of stops) grad.addColorStop(o, col);
  g.fillStyle = grad;
  g.fillRect(0, 0, radius * 2, radius * 2);
  return c;
}

function vgrad(g: CanvasRenderingContext2D, w: number, h: number, stops: Stop[]) {
  const grad = g.createLinearGradient(0, 0, 0, h);
  for (const [o, col] of stops) grad.addColorStop(o, col);
  g.fillStyle = grad;
  g.fillRect(0, 0, w, h);
}

/** A row of layered pine silhouettes (stacked triangles + trunk) along the base line. */
function pines(g: CanvasRenderingContext2D, w: number, h: number, base: number, height: number, count: number, color: string, rng: Rng) {
  g.fillStyle = color;
  // Ground below the treeline.
  g.fillRect(0, base - 2, w, h - base + 2);
  const step = w / count;
  for (let i = -1; i <= count + 1; i++) {
    const x = i * step + rng.range(-step * 0.3, step * 0.3);
    const th = height * rng.range(0.6, 1.15);
    const tw = th * rng.range(0.34, 0.44);
    g.fillRect(x - tw * 0.05, base - th * 0.18, tw * 0.1, th * 0.2);
    const tiers = 4;
    for (let k = 0; k < tiers; k++) {
      const apex = base - th + (k * th * 0.2);
      const bottom = apex + th * (0.32 + k * 0.04);
      const half = tw * (0.22 + k * 0.1);
      g.beginPath();
      g.moveTo(x, apex);
      g.lineTo(x + half, bottom);
      g.lineTo(x + half * 0.35, bottom - th * 0.03);
      g.lineTo(x - half * 0.35, bottom - th * 0.03);
      g.lineTo(x - half, bottom);
      g.closePath();
      g.fill();
    }
  }
}

// ---------------------------------------------------------------------------
// Night sky — "Tidur Malam"
// ---------------------------------------------------------------------------
function night(): Renderer {
  let w = 0, h = 0;
  let bg: HTMLCanvasElement | null = null;
  let stars: { x: number; y: number; r: number; a: number; s: number; p: number; big: boolean }[] = [];
  const starSprite = glow(6, [[0, 'rgba(255,255,255,1)'], [0.25, 'rgba(220,230,255,.55)'], [1, 'rgba(180,200,255,0)']]);
  let shoot = { t0: 6, x: 0, y: 0, vx: 0, vy: 0 };
  const rng = new Rng(7);
  return {
    quality: { scale: 0.75, fps: 20 },
    resize(W, H) {
      w = W; h = H;
      const [c, g] = canvas(w, h);
      vgrad(g, w, h, [[0, '#02040d'], [0.55, '#0a1230'], [1, '#18204a']]);
      // Faint milky way band.
      g.save();
      g.translate(w * 0.45, h * 0.35);
      g.rotate(-0.5);
      g.scale(1, 0.22);
      const mw = g.createRadialGradient(0, 0, 0, 0, 0, w * 0.6);
      mw.addColorStop(0, 'rgba(140,150,210,.10)');
      mw.addColorStop(1, 'rgba(140,150,210,0)');
      g.fillStyle = mw;
      g.fillRect(-w, -w, w * 2, w * 2);
      g.restore();
      // Moon with glow.
      const mr = Math.min(w, h) * 0.045, mx = w * 0.8, my = h * 0.2;
      g.drawImage(glow(mr * 6, [[0, 'rgba(255,248,220,.22)'], [1, 'rgba(255,248,220,0)']]), mx - mr * 6, my - mr * 6);
      g.fillStyle = '#f4f1e4';
      g.beginPath(); g.arc(mx, my, mr, 0, Math.PI * 2); g.fill();
      g.fillStyle = 'rgba(200,195,175,.35)';
      for (const [dx, dy, rr] of [[-0.3, -0.2, 0.22], [0.25, 0.1, 0.16], [-0.05, 0.35, 0.12]]) {
        g.beginPath(); g.arc(mx + dx * mr, my + dy * mr, rr * mr, 0, Math.PI * 2); g.fill();
      }
      // Hills silhouette.
      g.fillStyle = '#03050c';
      g.beginPath(); g.moveTo(0, h);
      for (let x = 0; x <= w; x += w / 40) g.lineTo(x, h * 0.86 - Math.sin(x / w * 5.3) * h * 0.03 - Math.sin(x / w * 13) * h * 0.012);
      g.lineTo(w, h); g.closePath(); g.fill();
      bg = c;
      const n = Math.min(320, Math.round((w * h) / 6500));
      const r2 = new Rng(11);
      stars = Array.from({ length: n }, () => {
        const big = r2.next() < 0.07;
        return { x: r2.next() * w, y: r2.next() * h * 0.85, r: big ? r2.range(1.2, 2) : r2.range(0.4, 1.1), a: r2.range(0.35, 1), s: r2.range(0.3, 1.4), p: r2.next() * 6.28, big };
      });
    },
    frame(ctx, t) {
      if (!bg) return;
      ctx.drawImage(bg, 0, 0);
      for (const s of stars) {
        const a = s.a * (0.6 + 0.4 * Math.sin(t * s.s + s.p));
        ctx.globalAlpha = a;
        if (s.big) ctx.drawImage(starSprite, s.x - 6, s.y - 6);
        else { ctx.fillStyle = '#e8eeff'; ctx.fillRect(s.x, s.y, s.r, s.r); }
      }
      ctx.globalAlpha = 1;
      // Occasional shooting star.
      if (t > shoot.t0 + 1.2) {
        shoot = { t0: t + rng.range(8, 22), x: rng.range(0.1, 0.7) * w, y: rng.range(0.05, 0.35) * h, vx: rng.range(500, 800), vy: rng.range(160, 260) };
      }
      const st = t - shoot.t0;
      if (st >= 0 && st < 1.1) {
        const x = shoot.x + shoot.vx * st, y = shoot.y + shoot.vy * st;
        const fade = st < 0.2 ? st / 0.2 : 1 - (st - 0.2) / 0.9;
        const g = ctx.createLinearGradient(x, y, x - shoot.vx * 0.18, y - shoot.vy * 0.18);
        g.addColorStop(0, `rgba(255,255,255,${0.9 * fade})`);
        g.addColorStop(1, 'rgba(255,255,255,0)');
        ctx.strokeStyle = g; ctx.lineWidth = 1.6;
        ctx.beginPath(); ctx.moveTo(x, y); ctx.lineTo(x - shoot.vx * 0.18, y - shoot.vy * 0.18); ctx.stroke();
      }
    },
  };
}

// ---------------------------------------------------------------------------
// Soft daylight — "Tidur Siang" (power nap)
// ---------------------------------------------------------------------------
function nap(): Renderer {
  let w = 0, h = 0, wake = 0;
  let bg: HTMLCanvasElement | null = null;
  let clouds: { img: HTMLCanvasElement; x: number; y: number; v: number }[] = [];
  let sun: HTMLCanvasElement | null = null;
  const makeCloud = (cw: number, rng: Rng) => {
    const [c, g] = canvas(cw, cw * 0.5);
    for (let i = 0; i < 7; i++) {
      const r = cw * rng.range(0.12, 0.22);
      const x = cw * rng.range(0.2, 0.8), y = cw * 0.5 * rng.range(0.35, 0.65);
      const gr = g.createRadialGradient(x, y, 0, x, y, r);
      gr.addColorStop(0, 'rgba(255,255,255,.55)');
      gr.addColorStop(1, 'rgba(255,255,255,0)');
      g.fillStyle = gr; g.fillRect(x - r, y - r, r * 2, r * 2);
    }
    return c;
  };
  return {
    quality: { scale: 0.5, fps: 15 },
    set(k, v) { if (k === 'wake') wake = v; },
    resize(W, H) {
      w = W; h = H;
      const [c, g] = canvas(w, h);
      vgrad(g, w, h, [[0, '#c9d6f2'], [0.55, '#efd6e4'], [1, '#fbe3d3']]);
      bg = c;
      sun = glow(Math.min(w, h) * 0.45, [[0, 'rgba(255,244,214,.9)'], [0.2, 'rgba(255,236,200,.45)'], [1, 'rgba(255,236,200,0)']]);
      const rng = new Rng(5);
      clouds = Array.from({ length: 6 }, (_, i) => ({ img: makeCloud(w * rng.range(0.25, 0.4), rng), x: rng.next() * w, y: h * (0.1 + i * 0.12), v: rng.range(4, 10) }));
    },
    frame(ctx, t, dt) {
      if (!bg || !sun) return;
      ctx.drawImage(bg, 0, 0);
      const s = 1 + 0.04 * Math.sin(t * 0.25) + wake * 0.5;
      const sw = sun.width * s;
      ctx.globalAlpha = 0.85 + wake * 0.15;
      ctx.drawImage(sun, w * 0.22 - sw / 2, h * 0.28 - sw / 2, sw, sw);
      ctx.globalAlpha = 1;
      for (const c of clouds) {
        c.x += c.v * dt;
        if (c.x > w) c.x = -c.img.width;
        ctx.drawImage(c.img, c.x, c.y);
      }
      if (wake > 0) {
        ctx.fillStyle = `rgba(255,246,225,${wake * 0.35})`;
        ctx.fillRect(0, 0, w, h);
      }
    },
  };
}

// ---------------------------------------------------------------------------
// Dark forest — "Hutan Gelap"
// ---------------------------------------------------------------------------
function forest(): Renderer {
  let w = 0, h = 0;
  let far: HTMLCanvasElement | null = null, mid: HTMLCanvasElement | null = null, near: HTMLCanvasElement | null = null, fog: HTMLCanvasElement | null = null;
  let flies: { x: number; y: number; p: number; s: number; ax: number; ay: number }[] = [];
  const fly = glow(10, [[0, 'rgba(236,252,170,1)'], [0.25, 'rgba(190,242,100,.6)'], [1, 'rgba(163,230,53,0)']]);
  return {
    quality: { scale: 0.6, fps: 20 },
    resize(W, H) {
      w = W; h = H;
      const rng = new Rng(3);
      let [c, g] = canvas(w, h);
      vgrad(g, w, h, [[0, '#010507'], [0.5, '#04130f'], [1, '#082019']]);
      g.drawImage(glow(Math.min(w, h) * 0.35, [[0, 'rgba(170,220,200,.12)'], [1, 'rgba(170,220,200,0)']]), w * 0.6, -h * 0.1);
      pines(g, w, h, h * 0.72, h * 0.32, 26, '#061612', rng);
      far = c;
      [c, g] = canvas(w, h);
      pines(g, w, h, h * 0.84, h * 0.42, 16, '#030d0a', rng);
      mid = c;
      [c, g] = canvas(w, h);
      pines(g, w, h, h * 1.02, h * 0.62, 8, '#010504', rng);
      near = c;
      [c, g] = canvas(w * 2, h * 0.35);
      for (let i = 0; i < 10; i++) {
        const x = (i / 10) * w * 2 + rng.range(0, w * 0.2), y = h * 0.35 * rng.range(0.3, 0.7), r = w * rng.range(0.15, 0.3);
        g.save(); g.translate(x, y); g.scale(1, 0.25);
        const gr = g.createRadialGradient(0, 0, 0, 0, 0, r);
        gr.addColorStop(0, 'rgba(160,200,190,.10)'); gr.addColorStop(1, 'rgba(160,200,190,0)');
        g.fillStyle = gr; g.fillRect(-r, -r, r * 2, r * 2); g.restore();
      }
      fog = c;
      const r2 = new Rng(9);
      flies = Array.from({ length: 34 }, () => ({ x: r2.next() * w, y: h * r2.range(0.45, 0.95), p: r2.next() * 6.28, s: r2.range(0.4, 1.1), ax: r2.range(20, 60), ay: r2.range(10, 30) }));
    },
    frame(ctx, t) {
      if (!far || !mid || !near || !fog) return;
      ctx.drawImage(far, 0, 0);
      const off = (t * 8) % w;
      ctx.drawImage(fog, -off, h * 0.55);
      ctx.drawImage(mid, 0, 0);
      ctx.drawImage(fog, -((off * 1.6) % w), h * 0.68);
      ctx.globalCompositeOperation = 'lighter';
      for (const f of flies) {
        const x = f.x + Math.sin(t * 0.3 * f.s + f.p) * f.ax;
        const y = f.y + Math.sin(t * 0.21 * f.s + f.p * 1.7) * f.ay;
        const a = Math.max(0, Math.sin(t * f.s * 1.3 + f.p)) ** 2;
        if (a < 0.03) continue;
        ctx.globalAlpha = a;
        ctx.drawImage(fly, x - 10, y - 10);
      }
      ctx.globalAlpha = 1;
      ctx.globalCompositeOperation = 'source-over';
      ctx.drawImage(near, 0, 0);
    },
  };
}

// ---------------------------------------------------------------------------
// Hearth fire — "Api Tungku"
// ---------------------------------------------------------------------------
function hearth(): Renderer {
  let w = 0, h = 0, cx = 0, base = 0, fw = 0;
  let bg: HTMLCanvasElement | null = null;
  let room: HTMLCanvasElement | null = null;
  const sprites = [
    glow(26, [[0, 'rgba(255,250,220,.95)'], [0.4, 'rgba(255,214,120,.5)'], [1, 'rgba(255,170,60,0)']]),
    glow(26, [[0, 'rgba(255,190,90,.8)'], [0.45, 'rgba(255,120,40,.35)'], [1, 'rgba(230,80,20,0)']]),
    glow(26, [[0, 'rgba(230,80,30,.6)'], [0.5, 'rgba(160,40,15,.25)'], [1, 'rgba(120,20,10,0)']]),
  ];
  const ember = glow(4, [[0, 'rgba(255,200,120,1)'], [1, 'rgba(255,120,40,0)']]);
  const rng = new Rng(21);
  type P = { x: number; y: number; vx: number; vy: number; life: number; age: number; s: number };
  const flames: P[] = [];
  const embers: P[] = [];
  const spawnFlame = (): P => ({ x: cx + rng.range(-fw / 2, fw / 2) * rng.next(), y: base, vx: rng.range(-10, 10), vy: -rng.range(90, 170), life: rng.range(0.5, 1.1), age: 0, s: rng.range(0.6, 1.2) });
  const spawnEmber = (): P => ({ x: cx + rng.range(-fw / 2, fw / 2), y: base - 10, vx: rng.range(-15, 15), vy: -rng.range(25, 70), life: rng.range(2, 4.5), age: 0, s: 1 });
  return {
    quality: { scale: 0.6, fps: 30 },
    resize(W, H) {
      w = W; h = H; cx = w / 2; base = h * 0.78; fw = Math.min(w * 0.22, 360);
      const [c, g] = canvas(w, h);
      g.fillStyle = '#060302'; g.fillRect(0, 0, w, h);
      // Stone hearth arch.
      g.fillStyle = '#120a07';
      const aw = fw * 2.1, ah = h * 0.5;
      g.fillRect(cx - aw / 2 - 30, base - ah - 20, aw + 60, ah + h * 0.22);
      g.fillStyle = '#020101';
      g.beginPath();
      g.moveTo(cx - aw / 2, base + 30); g.lineTo(cx - aw / 2, base - ah * 0.65);
      g.quadraticCurveTo(cx, base - ah * 1.15, cx + aw / 2, base - ah * 0.65);
      g.lineTo(cx + aw / 2, base + 30); g.closePath(); g.fill();
      // Logs.
      g.fillStyle = '#1d0f08';
      for (const [dx, rot] of [[-0.18, 0.12], [0.18, -0.12], [0, 0]] as const) {
        g.save(); g.translate(cx + dx * fw, base + 12); g.rotate(rot);
        g.beginPath(); g.ellipse(0, 0, fw * 0.55, 13, 0, 0, Math.PI * 2); g.fill(); g.restore();
      }
      bg = c;
      room = glow(Math.max(w, h) * 0.55, [[0, 'rgba(255,140,50,.28)'], [0.4, 'rgba(200,90,30,.12)'], [1, 'rgba(120,50,20,0)']]);
      flames.length = 0; embers.length = 0;
    },
    frame(ctx, t, dt) {
      if (!bg || !room) return;
      ctx.drawImage(bg, 0, 0);
      const flick = 0.75 + 0.15 * Math.sin(t * 7.3) + 0.1 * Math.sin(t * 13.1 + 1);
      ctx.globalCompositeOperation = 'lighter';
      ctx.globalAlpha = flick;
      ctx.drawImage(room, cx - room.width / 2, base - room.height / 2);
      ctx.globalAlpha = 1;
      while (flames.length < 130) flames.push(spawnFlame());
      for (let i = 0; i < flames.length; i++) {
        const p = flames[i];
        p.age += dt;
        if (p.age > p.life) { flames[i] = spawnFlame(); continue; }
        const k = p.age / p.life;
        p.x += (p.vx + Math.sin(t * 3 + p.y * 0.05) * 18) * dt + (cx - p.x) * 0.9 * dt;
        p.y += p.vy * dt;
        const spr = sprites[k < 0.35 ? 0 : k < 0.7 ? 1 : 2];
        const size = 40 * p.s * (1 - k * 0.65);
        ctx.globalAlpha = (1 - k) * 0.85;
        ctx.drawImage(spr, p.x - size / 2, p.y - size / 2, size, size);
      }
      if (embers.length < 22 && rng.next() < dt * 6) embers.push(spawnEmber());
      for (let i = embers.length - 1; i >= 0; i--) {
        const e = embers[i];
        e.age += dt;
        if (e.age > e.life) { embers.splice(i, 1); continue; }
        e.x += (e.vx + Math.sin(t * 2 + i) * 10) * dt;
        e.y += e.vy * dt;
        ctx.globalAlpha = (1 - e.age / e.life) * (0.6 + 0.4 * Math.sin(t * 9 + i));
        ctx.drawImage(ember, e.x - 4, e.y - 4);
      }
      ctx.globalAlpha = 1;
      ctx.globalCompositeOperation = 'source-over';
    },
  };
}

// ---------------------------------------------------------------------------
// Candlelight & bokeh — "Romantis"
// ---------------------------------------------------------------------------
function romantic(): Renderer {
  let w = 0, h = 0;
  let bg: HTMLCanvasElement | null = null;
  let bokeh: { img: HTMLCanvasElement; x: number; y: number; vx: number; vy: number; p: number; s: number }[] = [];
  let candles: { x: number; top: number }[] = [];
  let drops: { x: number; y: number; v: number; len: number }[] = [];
  const flame = (() => {
    const [c, g] = canvas(24, 56);
    const gr = g.createRadialGradient(12, 40, 1, 12, 34, 26);
    gr.addColorStop(0, 'rgba(255,255,235,1)');
    gr.addColorStop(0.35, 'rgba(255,214,130,.95)');
    gr.addColorStop(0.7, 'rgba(255,140,60,.55)');
    gr.addColorStop(1, 'rgba(255,100,40,0)');
    g.fillStyle = gr;
    g.beginPath();
    g.moveTo(12, 2);
    g.bezierCurveTo(22, 22, 22, 40, 12, 54);
    g.bezierCurveTo(2, 40, 2, 22, 12, 2);
    g.fill();
    return c;
  })();
  const halo = glow(110, [[0, 'rgba(255,190,110,.35)'], [0.5, 'rgba(255,150,80,.12)'], [1, 'rgba(255,120,60,0)']]);
  const rng = new Rng(13);
  return {
    quality: { scale: 0.5, fps: 20 },
    resize(W, H) {
      w = W; h = H;
      const [c, g] = canvas(w, h);
      vgrad(g, w, h, [[0, '#0c0408'], [0.6, '#22091a'], [1, '#2e0f1f']]);
      // Table edge.
      g.fillStyle = '#120509';
      g.fillRect(0, h * 0.86, w, h * 0.14);
      candles = [
        { x: w * 0.38, top: h * 0.62 },
        { x: w * 0.5, top: h * 0.56 },
        { x: w * 0.62, top: h * 0.66 },
      ];
      for (const cd of candles) {
        const cw = Math.min(w, h) * 0.05;
        const gr = g.createLinearGradient(cd.x - cw / 2, 0, cd.x + cw / 2, 0);
        gr.addColorStop(0, '#d8c3a8'); gr.addColorStop(0.5, '#f6ead8'); gr.addColorStop(1, '#bba489');
        g.fillStyle = gr;
        g.fillRect(cd.x - cw / 2, cd.top, cw, h * 0.86 - cd.top);
        g.fillStyle = '#2a1a10';
        g.fillRect(cd.x - 1, cd.top - 8, 2, 8);
      }
      bg = c;
      const tints: Stop[][] = [
        [[0, 'rgba(255,196,120,.32)'], [0.75, 'rgba(255,196,120,.18)'], [1, 'rgba(255,196,120,0)']],
        [[0, 'rgba(255,140,170,.28)'], [0.75, 'rgba(255,140,170,.14)'], [1, 'rgba(255,140,170,0)']],
        [[0, 'rgba(255,230,170,.26)'], [0.75, 'rgba(255,230,170,.12)'], [1, 'rgba(255,230,170,0)']],
      ];
      bokeh = Array.from({ length: 26 }, () => {
        const r = rng.range(18, 70);
        return { img: glow(r, rng.pick(tints)), x: rng.next() * w, y: rng.next() * h * 0.8, vx: rng.range(-6, 6), vy: rng.range(-8, -2), p: rng.next() * 6.28, s: rng.range(0.2, 0.6) };
      });
      drops = Array.from({ length: 14 }, () => ({ x: rng.next() * w, y: rng.next() * h, v: rng.range(10, 35), len: rng.range(18, 50) }));
    },
    frame(ctx, t, dt) {
      if (!bg) return;
      ctx.drawImage(bg, 0, 0);
      ctx.globalCompositeOperation = 'lighter';
      for (const b of bokeh) {
        b.x += b.vx * dt; b.y += b.vy * dt;
        if (b.y < -b.img.height) { b.y = h; b.x = Math.random() * w; }
        if (b.x < -b.img.width) b.x = w; else if (b.x > w) b.x = -b.img.width;
        ctx.globalAlpha = 0.55 + 0.45 * Math.sin(t * b.s + b.p);
        ctx.drawImage(b.img, b.x, b.y);
      }
      for (let i = 0; i < candles.length; i++) {
        const cd = candles[i];
        const f = 0.85 + 0.1 * Math.sin(t * 8.1 + i * 2) + 0.05 * Math.sin(t * 17.3 + i);
        ctx.globalAlpha = f;
        ctx.drawImage(halo, cd.x - 110, cd.top - 130);
        const sway = Math.sin(t * 2.3 + i) * 1.5;
        const fh = 46 * f;
        ctx.globalAlpha = 1;
        ctx.drawImage(flame, cd.x - 9 + sway, cd.top - 8 - fh, 18, fh);
      }
      ctx.globalCompositeOperation = 'source-over';
      // Rain sliding down the window.
      ctx.strokeStyle = 'rgba(255,220,230,.12)';
      ctx.lineWidth = 1.2;
      for (const d of drops) {
        d.y += d.v * dt;
        if (d.y > h) { d.y = -d.len; d.x = Math.random() * w; }
        ctx.beginPath(); ctx.moveTo(d.x, d.y - d.len); ctx.lineTo(d.x, d.y); ctx.stroke();
      }
      ctx.globalAlpha = 1;
    },
  };
}

export function createRenderer(id: RendererId): Renderer {
  switch (id) {
    case 'night': return night();
    case 'nap': return nap();
    case 'forest': return forest();
    case 'hearth': return hearth();
    case 'romantic': return romantic();
  }
}
