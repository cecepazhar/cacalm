# CACalm

**Free calm & focus soundscapes for Windows, macOS and Linux.**
Made by **Cecep Azhar · Fathforce**.

CACalm is a free, offline calm app. Layer nature sounds, steady noise, binaural
beats and breathing guides into your own mix, then set a sleep timer or pair it
with a Pomodoro focus timer.

## Features

- **37 soundscapes in 5 categories**, all generated procedurally on your device
  (no recordings to download, no internet needed):
  - **Nature**: rain, thunderstorm, ocean waves, stream, wind, forest birds, crickets, frogs, tropical forest, cave drips, snowy wind
  - **Sleep**: brown, pink, white and green noise, fan, airplane cabin, night train, heartbeat, rain on roof, delta 2 Hz
  - **Focus**: coffee shop, typing, quiet library, ticking clock, beta 14 Hz, gamma 40 Hz, isochronic 10 Hz
  - **Relax**: fireplace, alpha 10 Hz, 4-7-8 breathing, box breathing, 8D ambient
  - **Meditate**: singing bowl, wind chimes, 432 Hz drone, theta 6 Hz
- **Scenes**: one-tap mixes such as Deep Sleep, Deep Focus, Rainy Café, Calm Shore, Breathe and Meditation
- **Mixer**: per-sound volume, balance, mute and solo; save your own mixes
- **Breathing guide**: an on-screen count that stays in sync with the breathing sounds
- **Sleep timer**: 15–90 minutes with a gentle 30 second fade-out
- **Focus timer**: Pomodoro cycles with a soft chime between phases
- **My Sounds**: import your own MP3, WAV, FLAC or OGG files and layer them in
- **Background playback**: closing the window keeps CACalm playing in the system tray
- **No account, no ads, no telemetry.** English and Bahasa Indonesia, light and dark themes

## Download

Grab the installer for your OS from
[Releases](https://github.com/cecepazhar/cacalm/releases):

| OS      | File |
|---------|------|
| Windows | `CACalm_x.y.z_x64-setup.exe` |
| macOS   | `CACalm_x.y.z_aarch64.dmg` (Apple Silicon) or `_x64.dmg` (Intel) |
| Linux   | `.deb` (Debian/Ubuntu) or `.AppImage` (any distro) |

## Build from source

Prerequisites: Rust stable, Node.js 20+, and the
[`caui`](https://github.com/cecepazhar/caui) repo checked out next to this one
(`../caui`). On Linux, also install the Tauri system dependencies
(`libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`).

```bash
cd frontend
npm install
npm run build
cd ..
frontend/node_modules/.bin/tauri build            # all bundles for this OS
frontend/node_modules/.bin/tauri build --bundles nsis   # Windows installer only
```

Installers for all three platforms are built by the
[`Release` workflow](.github/workflows/release.yml) when a `v*` tag is pushed.

## Architecture

- `frontend/`: SvelteKit + Tailwind UI. All audio runs in the Web Audio API.
  `src/lib/audio/generators.ts` renders seamless procedural loops in a Web Worker,
  and `engine.ts` mixes them with click-free fades and a master limiter.
- `crates/calm-core`: Pomodoro timer, sound library (SQLite) and presets.
- `crates/calm-app`: Tauri shell (tray, global shortcuts, window controls, file import).

## License

MIT © 2026 Cecep Saeful Azhar Hidayat — Fathforce
