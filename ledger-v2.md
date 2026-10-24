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

## 2026-10-24 — Sistem Lisensi Pro (Free vs Pro) & Tauri IPC Commands

Implementasi model lisensi Pro offline-first untuk CACalm terinspirasi dari pola CATerm/CAFramework, membedakan batasan Free Tier vs Pro Tier:
- **Free Tier**: 4 channel ambient dasar (`rain`, `waves`, `wind`, `white_noise`), Pomodoro timer standar, maksimal 2 preset.
- **Pro Tier** ($10/bulan atau Rp 199.000 seumur hidup): Akses semua 8+ channel (`birds`, `fireplace`, `cafe`, `stream`, `binaural_beats`, dsb.), Spatial 3D Audio Pan (slider kiri/kanan), Unlimited presets, dan custom sound import tanpa batas.

### Perubahan Arsitektur & Kode Rust

1. **`crates/calm-core/src/pro.rs` (Modul Baru)**
   - Struktur `ProStatus` (`is_pro: bool`, `plan: String`, `license_key: Option<String>`, `expires_at: Option<i64>`, `features: Vec<String>`).
   - Helper methods: `is_channel_allowed()`, `max_presets()`, `can_use_spatial_pan()`, `can_import_custom_sounds()`, `has_feature()`.
   - Validasi lisensi offline berbasis kriptografi HMAC-SHA256 (`verify_license`) dengan dukungan:
     - Kunci berformat terstruktur: `CACALM-<PLAN>-<CLIENT_ID>-<EXPIRY>-<SIG_HEX>` / `CALM-...`.
     - Kunci token titik: `CACALM.<PAYLOAD_HEX>.<SIG_HEX>`.
     - Verifikasi timestamp kedaluwarsa (`expires_at`) untuk paket subscription vs lifetime.
     - Kunci pre-configured founder lifetime: `FOUNDER_LIFETIME_KEY`.
   - Generator kunci lisensi: `generate_license_key()`.
   - Penyimpanan cache offline di `settings/pro_cache.json` di dalam direktori data aplikasi (`calm_core::paths::app_data_dir()`), dengan fallback ke `pro_cache.json`.
   - Siklus hidup lisensi: `activate_license()`, `deactivate_license()`, dan `get_pro_status()`.

2. **`crates/calm-core/src/lib.rs` & `Cargo.toml`**
   - Registrasi modul `pub mod pro;`.
   - Re-export `ProStatus`, `verify_license`, `get_pro_status`, `activate_license`, `deactivate_license`.
   - Menambahkan dependensi `hmac`, `sha2`, dan `hex` di workspace dan `calm-core`.

3. **`crates/calm-app/src/commands.rs` & `src/lib.rs`**
   - Penambahan 3 Tauri IPC command:
     - `get_pro_status() -> Result<ProStatus, String>`
     - `activate_pro_license(license_key: String) -> Result<ProStatus, String>`
     - `deactivate_pro_license() -> Result<(), String>`
   - Pendaftaran ketiga command di `invoke_handler` Tauri builder di `lib.rs` dan re-export di `calm_app`.

### Unit Test
- `pro::tests`:
  - `test_default_free_tier`: memastikan default Free tier hanya membuka 4 channel dasar, batas 2 preset, tanpa 3D pan.
  - `test_valid_lifetime_license`: validasi kunci lifetime offline berhasil mengaktifkan Pro tanpa batas expiry.
  - `test_valid_subscription_license`: validasi kunci subscription berdurasi aktif.
  - `test_expired_subscription_license`: penolakan kunci yang sudah melewati timestamp kedaluwarsa.
  - `test_tampered_signature_fails`: deteksi modifikasi tanda tangan kriptografis HMAC.
  - `test_tampered_payload_fails`: deteksi manipulasi payload lisensi tanpa re-signing.
  - `test_invalid_key_formats`: penolakan format kunci yang rusak atau tidak lengkap.
  - `test_preconfigured_founder_key`: verifikasi kunci founder bawaan.
  - `test_cache_activation_and_deactivation_lifecycle`: aktivasi cache disk, pembacaan status, dan deaktivasi bersih.
- `commands::tests::test_pro_commands_contract`: verifikasi kontrak command Tauri `get_pro_status`, `activate_pro_license`, dan `deactivate_pro_license`.

### Verifikasi
- `cargo test --workspace` → Lolos 100% (37 passed: 10 calm-app + 27 calm-core).
- `cargo clippy --workspace --all-targets` → Bersih (0 warning, 0 error).

## 2026-10-24 — Overhaul Frontend dengan @cecepazhar/caui, Light/Dark Mode, i18n EN/ID, ProGate & Live Audio IPC

Overhaul komprehensif frontend CACalm (`frontend/`) sesuai standar ekosistem CA Design System (`@cecepazhar/caui`):

### 1. Instalasi & Integrasi CAUI Design System
- Memasang paket `@cecepazhar/caui` (v1.3.0) via `pnpm add file:../../caui`.
- Mengkonfigurasi `src/app.css` dengan token baku `--ca-*` untuk mode Light (`:root`) dan Dark (`html.dark`), menyelaraskan warna aksen brand `--ca-brand: #10b981;` (Emerald/Calm theme) dan `--ca-brand-glow`.
- Menyertakan direktif `@source "../../caui/src/**/*.{svelte,ts}";` untuk pemindaian utilitas Tailwind CSS v4.

### 2. Eliminasi Double Header & Unified Navigation
- Menghapus double header redundan antara layout dan page.
- Membangun titlebar terpadu di `src/routes/+layout.svelte` menggunakan drag-region native desktop (`data-tauri-drag-region`), logo produk `🌿 CACalm`, versi `v0.1.1`, navigation tab strip (Mixer, Library, Settings), switchers, dan tombol kontrol jendela (minimize ke tray & close).
- Mengganti elemen `<slot />` lama ke standar Svelte 5 runes: `let { children } = $props();` dan `{@render children?.()}`.

### 3. Light / Dark Theme Store
- Membuat store reaktif `src/lib/stores/theme.svelte.ts`:
  - Mengelola `ThemeName` ('light' | 'dark') dengan persistensi ke `localStorage` (`cacalm_theme`).
  - Sinkronisasi toggle class `.dark` dan atribut `data-theme` pada tag `<html>`.
  - Terintegrasi dengan komponen `ThemeSwitcher` dari CAUI di navbar dan halaman Settings.

### 4. Sistem i18n Reaktif dengan Paritas Penuh EN/ID
- Membuat struktur i18n di `src/lib/i18n/`:
  - `locales/en.ts`: Kamus bahasa Inggris lengkap mencakup namespace `app`, `nav`, `timer`, `mixer`, `channels`, `presets`, `pro`, `library`, `settings`, `dialogs`.
  - `locales/id.ts`: Kamus bahasa Indonesia yang diikat dengan TypeScript type `Dictionary` dari `en.ts` sehingga kelengkapan kunci dijamin 100% saat kompilasi.
  - `index.svelte.ts`: Mesin i18n reaktif Svelte 5 dengan helper `t(key, vars)`, fallback multilapis, persistensi `cacalm_locale`, dan sinkronisasi dua arah dengan `setLocale`/`getLocale` dari `@cecepazhar/caui`.
- Integrasi `LanguageSwitcher` (compact) di header dan Settings.

### 5. Pro Badge & Komponen ProGate Eksklusif
- Membuat store `src/lib/stores/pro.svelte.ts` untuk melacak status Free vs Pro pengguna.
- Menampilkan Pro Badge interaktif (`FREE` / `PRO`) di header yang dapat diklik untuk membuka modal upgrade atau beralih mode.
- Memasang komponen `ProGate` dari CAUI pada channel eksklusif Pro:
  - 🌲 Burung Hutan (`ch-birds`)
  - ☕ Kedai Kopi (`ch-cafe`)
  - 🔥 Api Unggun (`ch-fire`)
  - 🏞️ Aliran Sungai (`ch-stream`)
  - 🧠 Binaural Beats (`ch-binaural`)
  - 🎧 Spatial Pan (`ch-spatial`)
- Pengguna Free melihat kartu gated dengan ikon gembok dan tombol upgrade, sedangkan pengguna Pro memperoleh akses kontrol penuh.

### 6. Live Real Audio IPC Backend & Web Audio Fallback
- Mengimplementasikan 5 Tauri IPC audio commands di backend Rust (`crates/calm-app/src/commands.rs` & `src/lib.rs`):
  - `play_channel(channel_id, volume)`
  - `stop_channel(channel_id)`
  - `set_channel_volume(channel_id, volume)`
  - `set_master_volume(volume)`
  - `set_channel_pan(channel_id, pan)`
- Menambahkan sintesis audio prosedural tanpa ketergantungan file eksternal pada `crates/calm-core/src/audio/audio_impl.rs` (`AmbientSoundSource` mono 44.1kHz untuk rodio `SpatialSink`) sehingga audio berbunyi nyata di hardware lokal.
- Menambahkan mesin fallback Web Audio API di frontend (`src/lib/audio/webAudio.ts`) untuk memastikan audio tetap terdengar secara instan baik saat berjalan di Tauri desktop maupun di preview browser.
- Menghubungkan seluruh interaksi UI di `src/routes/+page.svelte` (toggle aktif channel, volume slider, pan slider, mute, solo, preset, dan master volume) langsung ke command audio tersebut.

### 7. Verifikasi Kualitas
- `pnpm run check` (svelte-check): **0 errors, 0 warnings**.
- `pnpm run build` (vite build): **Berhasil 100%**, static site di-generate ke `build/`.
- `cargo test --workspace`: **39 passed, 0 failed** (12 calm-app + 27 calm-core).

## 2026-10-24 — Audio Engine Backend, Procedural Ambient Generator & Tauri Audio Commands

Penyelesaian tuntas backend audio CACalm untuk memecahkan masalah aplikasi bisu (silent application) tanpa media audio lokal default:

### 1. Built-in Procedural Audio & Noise Generator (`crates/calm-core/src/audio/synth.rs`)
- **Noise Generator**:
  - White Noise: PRNG deterministik Xorshift64* tanpa dependensi luar.
  - Pink Noise: Filter 6-pole Paul Kellet (1/f equal energy per octave).
  - Brown Noise: Leaky integrator 1/f^2 (Brownian motion) untuk red noise hangat pereda tinnitus.
- **DSP Filter & Modulator**:
  - One-pole Low-Pass Filter (`LowPass`) & High-Pass Filter (`HighPass`) dengan clamping frekuensi aman.
  - Second-order Resonant Band-Pass Filter (`BandPass`) untuk resonansi desau angin dan gemericik air.
  - Circular Loop Smoothing (`apply_circular_crossfade`): Crossfade lingkaran 0.4–0.5s antara ujung dan awal buffer audio untuk mencegah suara klik/pop saat di-loop tanpa henti.
- **8 Channel Ambient Generator**:
  1. `generate_rain`: Pink noise bed terfilter + impuls acak tetesan hujan (Poisson arrival resonant drops).
  2. `generate_ocean_waves`: LFO sinusoidal lambat (~0.12 Hz) yang memodulasi cutoff dan gain ombak bergulir & buih surut.
  3. `generate_birds`: Desau dedaunan lembut + modulasi frekuensi (FM synthesis) kicauan burung berkicau periodik.
  4. `generate_cafe`: Formant-filtered noise menyerupai gumaman ruangan + dentingan cangkir keramik resolusi tinggi.
  5. `generate_fire`: Gemuruh hangat frekuensi rendah + letupan api tajam (Poisson spark crackles & wood pops).
  6. `generate_wind`: Sapuan bandpass resonan turbulen yang bergerak perlahan mengikuti LFO ganda.
  7. `generate_stream`: Tekstur air mengalir deras + gelembung air beresonansi mikro (babbling brook).
  8. `generate_brown_noise`: Brownian noise murni untuk fokus mendalam dan peredam bising ruangan.
- **RIFF WAV Encoder**:
  - `encode_wav`: Encoding murni Rust ke format 16-bit PCM RIFF WAV standard (44.1 kHz, mono).
- **Default Sound Provisioning**:
  - `ensure_default_sounds(dir)`: Memeriksa dan mensintesis otomatis 8 file audio WAV ke direktori sounds lokal (`~/.local/share/cacalm/sounds/`) saat aplikasi dinyalakan jika belum ada.
  - Mengaktifkan feature `wav`, `vorbis`, dan `flac` pada crate `rodio` di `crates/calm-core/Cargo.toml`.

### 2. Multi-Channel Audio Engine & Spatial Audio Pan (`crates/calm-core/src/audio/audio_impl.rs`)
- `ChannelId(pub String)`: Abstraksi handle channel serbaguna yang kompatibel baik dengan UUID dinamis maupun string channel baku (`ch-rain`, `ch-waves`, dsb.).
- Integrasi `rodio::SpatialSink`:
  - Mendukung penempatan posisi suara 3D binaural / stereo panning.
  - `set_channel_pan(channel_id, pan)`: Mengatur posisi emitor spasial dari `-1.0` (kiri penuh) sampai `+1.0` (kanan penuh) dengan respons periodik 10ms.
- Fallback Ganda:
  - Membaca dan memainkan file WAV dari disk dengan decoding `Decoder::repeat_infinite()`.
  - Jika file tidak ditemukan di disk, fallback otomatis secara transparan ke in-memory procedural generator `AmbientSoundSource`.
- Method Audio Engine:
  - `play_channel(channel, name, source_path, volume)`
  - `stop_channel(channel)`
  - `set_channel_volume(channel, volume)`
  - `set_channel_pan(channel, pan)`
  - `set_master_volume(volume)`
  - `list_active_channel_ids()`

### 3. Tauri IPC Audio Commands (`crates/calm-app/src/commands.rs` & `src/lib.rs`)
- Menambahkan dan mengekspor 6 Tauri IPC commands:
  - `play_channel(channel_id: String, volume: f32) -> Result<(), String>`
  - `stop_channel(channel_id: String) -> Result<(), String>`
  - `set_channel_volume(channel_id: String, volume: f32) -> Result<(), String>`
  - `set_master_volume(volume: f32) -> Result<(), String>`
  - `set_channel_pan(channel_id: String, pan: f32) -> Result<(), String>`
  - `list_active_channels() -> Result<Vec<String>, String>`
- Resolusi path cerdas (`resolve_channel_audio_path`):
  - Memetakan channel default (`ch-rain`, `ch-waves`, dsb.) ke file sintetis di direktori data aplikasi.
  - Memeriksa file import kustom di `SoundLibrary` SQLite database.
  - Memeriksa path langsung bila diberikan path file absolut/relatif.
- Mendaftarkan seluruh command ke dalam `invoke_handler` di `calm_app_lib::run()`.
- Menambahkan auto-seeding default sounds di dalam hook `.setup()` aplikasi.

### 4. Unit Testing & Verifikasi Lengkap
- **`calm-core`**: 27 unit tests lulus (sintesis noise, filter DSP, encoding WAV, loop crossfade, audio engine workflow, sound library isolated storage & seeding).
- **`calm-app`**: 12 unit tests lulus (resolusi audio path mapping, operasi channel audio engine, kontrak pro status, settings, shortcuts, tray).
- **Semua Gates Terpenuhi**:
  - `cargo test --workspace` → **39 passed, 0 failed**.
  - `cargo clippy --workspace --all-targets -- -D warnings` → **0 warning, 0 error**.
  - `cargo build --release` → **Berhasil 100% (target/release/calm-app)**.
  - `npm --prefix frontend run check` → **0 error, 0 warning**.
  - `npm --prefix frontend run build` → **Berhasil 100% (frontend/build/)**.
