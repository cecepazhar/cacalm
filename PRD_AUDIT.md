# CACalm PRD Audit — Vault `CACalm/00 PRD.md` (F1..F8) vs Implementasi

> Audit: 2026-10-11 · Base: `/home/cecepazhar/Product/cacalm` · Metode: statis
> Catatan: git init baru pagi ini (2026-10-11); status = fresh scaffold.

## Hasil: 2/8 F (25%) · Design Compliance: PARTIAL (dark custom, belum CADS token)

### Peta Fitur

| F | Judul | Status | Bukti |
|---|---|---|---|
| F1 | Audio engine multi-channel (8 channel, mix, loop, gain) | ✅ KODE, ⚠️ DEFAULT-CF | `audio/audio_impl.rs` (rodio multi-Sink); **feature-gated `stub-audio` = default** → out-of-box tidak bersuara |
| F2 | Channel mixer UI (volume/pan/mute/solo) | ❌ | Placeholder card di `+page.svelte` |
| F3 | Import & library (drag-drop, metadata SQLite) | ⚠️ | `library/mod.rs` SoundLibrary metadata; tanpa UI import |
| F4 | Preset system (save/load, bundled) | ✅ | `preset/mod.rs` (JSON, 5 bundled) |
| F5 | Pomodoro timer (FOCUS/SHORT/LONG, injectable clock) | ✅ | `timer/mod.rs` |
| F6 | Tray control (minimize-to-tray, quick controls) | ✅ | `tray.rs` (TrayIconBuilder, menu Start/Pause, Open, Exit, tooltip) |
| F7 | Keyboard shortcuts (global/local) | ✅ | `shortcuts.rs` (tauri-plugin-global-shortcut, Ctrl+Alt+C/L, settings toggle) |
| F8 | CAFramework integration (vault, prefs, i18n, CADS) | ❌ | Belum ter-integrasi |

**Frontend = 3 file svelte**: `+layout.svelte`, `+page.svelte`, `PageHeader.svelte`. Scaffold, bukan produk.
**Task list Vault (`01 Dev Task List.md`)**: semua item `[ ]` unchecked (T0.1..T7.6, QA1-3) — belum ada fase selesai.

### Design compliance
- Field: `src/app.css` pakai hex hardcoded (`#0A0A0C` dll), bukan token CADS (`--bg-app`, `--ca-brand`).
- Tidak impor CAUI. Tidak ada focus-visible/aria eksplisit. i18n belum ada.
- → **PARTIAL**: UI sangat minimal, jadi belum melanggar banyak, tapi belum menerapkan CADS.

### Rekomendasi
1. P0 (Decision): Proyek umur 1 hari — tentukan lanjut (fase 1: audio real + mixer UI) atau hold.
2. P1: Aktifkan default feature audio (hindari `stub-audio`) agar app berbunyi out-of-box.
3. P1: Migrasi `app.css` ke token CADS + impor CAUI untuk PageHeader/controls.
4. P2: Core timer/preset sudah bersih & testable → prioritaskan UI (T5.1-T5.4 mixer, library modal).