# CACalm Development Ledger

## 2026-10-24 — F1 Audio Engine & Default Feature (P0)

Perbaikan blocker P0 dari `PRD_AUDIT.md`: default feature `calm-core`
sebelumnya `stub-audio` (aplikasi diam saat di-build, tidak ada back-end
audio). Kini `default = ["audio"]` agar playback nyata (`rodio`/`cpal` via
ALSA) aktif out-of-box.

### Dependency Sistem

Tambahkan dependensi sistem (dev headers audio) di laptop Fedora 44 ini
agar `cargo build` tidak gagal saat mengompilasi `alsa-sys` / `cpal`:

| Package | Versi | Alasan |
|---|---|---|
| `alsa-lib-devel` | `1.2.16.1-1.fc44` | Header ALSA (`/usr/include/alsa/*.h`) + symlink `libasound.so` yang dibutuhkan `alsa-sys` & `cpal`. Tanpa ini, `cargo build --release` gagal di step build `alsa-sys`. |

**Catatan**: `alsa-lib` (runtime) sudah ter-install sejak awal. Hanya
`alsa-lib-devel` yang hilang. Install via `sudo dnf install alsa-lib-devel`.
Setelah install, `PKG_CONFIG_PATH=/usr/lib64/pkgconfig` cukup untuk
`cargo build`. (Jika build dari user non-root di `~/.local`, lihat catatan
di bawah tentang `PKG_CONFIG_PATH`.)

Untuk sesi agent non-root (tanpa `sudo`), file header & `.pc` yang sama
dicopi ke `~/.local` agar tidak memblock build:
- `~/.local/include/alsa/` (dari `usr/include/alsa/`)
- `~/.local/lib/libasound.so` (symlink ke `/usr/lib64/libasound.so.2.0.0`)
- `~/.local/lib/pkgconfig/alsa.pc` (prefix di-remap ke `~/.local`,
  `Libs` menunjuk `/usr/lib64`)

Lingkungan build untuk sesi agent:
```sh
export PKG_CONFIG_PATH=~/.local/lib/pkgconfig
export C_INCLUDE_PATH=~/.local/include
export CPLUS_INCLUDE_PATH=~/.local/include
export LIBRARY_PATH=~/.local/lib:/usr/lib64
export LD_LIBRARY_PATH=~/.local/lib:/usr/lib64
```

### Perubahan Kode Rust

| File | Perubahan |
|---|---|
| `crates/calm-core/Cargo.toml` | `default = ["stub-audio"]` → `default = ["audio"]`. `stub-audio` tetap tersedia sebagai fallback opt-in. |
| `crates/calm-app/Cargo.toml` | Kalahkan `default-features = false` pada `calm-core`; tambah feature `image-png` pada `tauri` (dibutuhkan `Image::from_bytes` untuk ikon tray). |
| `crates/calm-app/src/lib.rs` | `builder` dibuat `mut` tidak perlu; import `tauri::Manager` untuk `state::<AppState>()` di `setup` & `on_window_event`. |
| `crates/calm-app/src/commands.rs` | Tambah 6 command baru F6/F7: `get_settings`, `update_settings`, `toggle_tray`, `toggle_shortcuts`, `set_shortcuts_enabled`, `set_tray_enabled`, plus `minimize_to_tray`. Perbaiki `register_global_shortcuts(&app, &settings)` (`&AppSettings`, bukan `MutexGuard`). |
| `crates/calm-app/src/settings.rs` | `AppSettings` (tray/shortcuts/minimize) + `load_settings()` / `save_settings()` di `~/.local/share/cacalm/settings.json`. Test `test_json_serialization_roundtrip` di-restrukturisasi agar lolos clippy `field_reassign_with_default`. |
| `crates/calm-app/src/shortcuts.rs` | `ShortcutModel` + `register_global_shortcuts` / `unregister_global_shortcuts` (plugin `tauri-plugin-global-shortcut`). String→`&str` fix untuk `on_shortcut` (`ShortcutWrapper` tidak `From<&String>`). |
| `crates/calm-app/src/tray.rs` | `setup_tray`, `set_tray_visible`, `update_tray_tooltip` + `TrayState` pure state-machine (label dinamis Start/Pause, tooltip format `phase mm:ss (status)`). Ikon dari `default_window_icon()` fallback `Image::from_bytes` `32x32.png`. |
| `crates/calm-core/src/audio/audio_impl.rs` | `AudioEngine::new()` kini membuat `OutputStream` di thread dedicated ("cacalm-audio-stream") karena `rodio::OutputStream` `!Send + !Sync` di Linux/ALSA; `AudioEngine` memegang `OutputStreamHandle` + `mpsc::Sender` drop + `JoinHandle`, menjadikan `AudioEngine` sendiri `Send + Sync` (syarat `tauri::State`). Perbaiki `open_channel`: `Decoder` tidak di-reassign (`Decoder` dan `Repeat` tipe beda), gunakan `sink.append(decoder.repeat_infinite())` bila `loop_enabled`. Clippy: 4x `collapsible_if` di-merger ke let-chains. |
| `crates/calm-core/src/audio/audio_stub.rs` | Hapus import `parking_lot::Mutex` & `std::collections::HashMap` yang tak terpakai. |
| `crates/calm-core/src/preset/mod.rs` | Hapus import `std::collections::HashMap` yang tak terpakai. Clippy: `map_or(false, …)` → `is_some_and(…)`, 2x `collapsible_if` di-merger. |
| `crates/calm-core/src/paths.rs` | Clippy: `std::io::Error::new(ErrorKind::Other, …)` → `std::io::Error::other(…)`. |

### Verifikasi

Semua gate lulus di sesi ini (Fedora 44, Rust stable):

| Gate | Hasil |
|---|---|
| `cargo build --release` | Sukses (exit 0), `target/release/calm-app` + `libcalm_app_lib.{a,so,rlib}` dibangun. 0 error. |
| `cargo test --workspace` | 20 passed, 0 failed (9 `calm-app` + 11 `calm-core`). |
| `cargo clippy --workspace --all-targets` | 0 error, 0 warning. |
| `cargo build --release` (ulang pasca clippy fix) | Sukses (exit 0), 0 error, 0 warning. |

### Sisa Gap

- [ ] `default-features = false` dihapus dari `calm-app → calm-core`; jika
      ada build khusus yang ingin memaksa `stub-audio`, gunakan
      `cargo build --no-default-features --features stub-audio` di level
      workspace, bukan di `calm-app`.
- [ ] `TrayIconBuilder::icon` hanya dipakai saat `default_window_icon()`
      `None`; untuk Tauri 2 di Linux, `default_window_icon()` jarang
      mengembalikan `Some` — pastikan ikon tray terlihat saat
      `default_window_icon()` = `None` (fallback `32x32.png` sudah
      diaktifkan via feature `image-png`).
- [ ] Global shortcuts diuji hanya untuk registrasi; perilaku actual saat
      tombol ditekan belum diuji end-to-end di Linux (ALSA tidak tersedia
      di sandbox agent — `AudioEngine::new()` akan gagal `NoDevice` dan
      `AppState.engine` akan menyimpan `Err`, UI tetap berfungsi).

## 2026-10-24 — F6 & F7: Tray Control & Keyboard Shortcuts

### Keputusan Desain

1. **Modul terpisah di `calm-app`**
   - `settings.rs` — struktur `AppSettings` + load/save JSON
   - `tray.rs` — `TrayState`, `TrayAction`, `setup_tray()`, `set_tray_visible()`
   - `shortcuts.rs` — `ShortcutModel`, validasi string shortcut, `register_global_shortcuts()`, `unregister_global_shortcuts()`
   - `commands.rs` — IPC commands baru + `toggle_timer_state()`, `open_main_window()`

2. **Tray icon**
   - `TrayIconBuilder::with_id("cacalm-tray")`
   - Menu: "Start/Pause Focus", "Open App", "Exit"
   - Klik kiri ikon tray → buka jendela utama
   - Tray tooltip dinamis menunjukkan fase & sisa waktu timer

3. **Shortcut global (cross-platform)**
   - `Ctrl+Alt+C` = toggle timer (pause/resume)
   - `Ctrl+Alt+L` = open app window
   - Bisa dikonfigurasi dari halaman Settings; default `AppSettings::default()`

4. **Persistensi**
   - `settings.json` di app data dir (`calm_core::paths::app_data_dir()`)
   - `AppState.settings` = `Mutex<AppSettings>` — dibaca saat `run()` start

5. **Behavior jendela**
   - `minimize_to_tray` (default ON): close window hanya hide, tidak exit
   - Frontend topbar tombol `minimize_to_tray` → hide window

6. **Toggling**
   - Halaman `/settings` di frontend: toggle tray, toggle shortcuts, edit shortcut string
   - `update_settings()` memaksa re-register shortcuts jika enabled status berubah
   - Tray visibility langsung di-set via `set_tray_visible()`

### Unit Test
- `settings::tests` — 3 test: default, JSON roundtrip, partial JSON fill default
- `tray::tests` — 3 test: `TrayAction::from_id`, `TrayState` labels & tooltip, visibility toggle
- `shortcuts::tests` — 3 test: validasi string, resolve action when enabled, resolve when disabled

### Verifikasi
- `cargo clippy --workspace --tests` → 0 warning, 0 error
- `cargo test --workspace` → 20 test passed (11 calm-core + 9 calm-app)
- `npm --prefix frontend run check` → 0 error
- `npm --prefix frontend run build` → berhasil
