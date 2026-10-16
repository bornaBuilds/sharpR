# sharpR

Local desktop tool for preparing images and videos for the web: batch-convert images to WebP / AVIF / JPEG / PNG, crop, rotate, skew, flip and resize single images, and compress videos to MP4 or WebM with poster frames. Everything runs on your machine; no file ever leaves it.

Built with [Tauri 2](https://tauri.app) (Rust) and Svelte 5 + Tailwind CSS 4.

## Features

**Convert tab**
- Add images with a file picker, a folder picker (optionally recursive) or by dropping files/folders onto the window.
- Output formats: WebP (default), AVIF, JPEG, PNG, with a quality slider for lossy formats and a lossless toggle for WebP.
- Optional "max dimension" that downscales the longest side (never upscales).
- Parallel conversion with per-image status, size savings and a cancel button.
- Save one image, all images as a ZIP, or all images into a folder. Name collisions get `-1`, `-2` suffixes.
- EXIF orientation is applied; all other metadata is stripped from the output.

**Crop & resize tab**
- Free or fixed aspect ratios (1:1, 16:9, 9:16, 4:3, 3:4, 3:2, 2:3, 21:9, 4:5, custom).
- Rotate in 45° steps or type an exact angle, skew X/Y in 5° steps (up to ±45°), flip horizontally/vertically, zoom.
- Output size fields linked to the crop ratio, with an upscaling warning.
- The preview is a CSS transform; the final pixels are produced in Rust on the full-resolution original.
- Save the result or send it to the converter list.

**Compress videos tab**
- Add videos (MP4, MOV, MKV, WebM, AVI, WMV, FLV, MPEG, 3GP, OGV, MTS/M2TS) with the pickers or by dropping files/folders.
- Output MP4 (H.264 + AAC, default) or WebM (VP9 + Opus), with quality presets (High / Balanced / Small / Smallest), an optional resolution cap (1080p / 720p / 480p, never upscales) and a "Remove audio" toggle.
- One video at a time with per-video progress and a cancel button. Phone rotation is applied, HDR (HLG/PQ) footage is tone-mapped to SDR, and metadata is stripped.
- If re-encoding an already web-friendly file (H.264 MP4 or VP8/VP9/AV1 WebM) saves less than 5% and no downscale was requested, the original streams are kept instead ("Already optimized").
- Eight poster candidates from the first seconds of each video, at the compressed video's size, shown in a sidebar; save the best one as WebP, AVIF, JPEG or PNG.
- Save one video or all of them into a folder.

## Development

Prerequisites: [Rust](https://rustup.rs) (stable), Node 24, pnpm 11, and the platform prerequisites from the [Tauri guide](https://v2.tauri.app/start/prerequisites/) (Visual Studio Build Tools with C++ on Windows, Xcode Command Line Tools on macOS, `libwebkit2gtk-4.1-dev` and friends on Linux). A C compiler is required because libwebp is compiled from source.

```bash
pnpm install
pnpm ffmpeg:fetch
pnpm tauri dev
```

`pnpm ffmpeg:fetch` downloads the pinned FFmpeg build for your platform (checksum-verified, see `scripts/ffmpeg-manifest.json`) into `src-tauri/binaries/`. It is bundled as the `sharpr-ffmpeg` sidecar, and every `cargo`/`tauri` build fails until it is there. `pnpm ffmpeg:fetch --system` copies the FFmpeg on your `PATH` instead, for quick local development only.

Useful scripts:

| Command | What it does |
|---|---|
| `pnpm ffmpeg:fetch` | Install the FFmpeg sidecar (once, and after manifest updates) |
| `pnpm tauri dev` | Run the desktop app with hot reload |
| `pnpm tauri build` | Build the installer for the current platform |
| `pnpm lint` / `pnpm fix` | Biome lint + format (check / write) |
| `pnpm check` | svelte-check type checking |
| `pnpm test` | Frontend unit tests (vitest) |
| `cargo test --manifest-path src-tauri/Cargo.toml` | Rust unit and pipeline tests (video tests run against the sidecar) |

## Architecture

- `src/` — Svelte 5 frontend. Stores live in `src/lib/stores/*.svelte.ts`; typed Tauri command wrappers in `src/lib/tauri/`.
- `src-tauri/src/commands/` — Tauri commands: file intake, conversion, saving, cropping, video intake/compression/saving.
- `src-tauri/src/video/` — video pipeline on top of the FFmpeg sidecar: probe parsing (no ffprobe needed), FFmpeg argument builders, the process runner with progress and cancellation, poster frames, and the per-session scratch folder for encoded videos.
- `src-tauri/src/imaging/` — pure image code: decoding with EXIF orientation, encoders (`webp`, `ravif`, `jpeg-encoder`, `oxipng`), thumbnails, and the crop transform.
- `src/lib/crop/transform.ts` and `src-tauri/src/imaging/transform.rs` implement the same matrix math and share test fixtures, so the on-screen preview and the rendered output always agree.

## Notes and limitations

- AVIF is supported as an output only. Reading AVIF would require the dav1d C library.
- Animated GIF/WebP inputs use their first frame.
- ICC profiles are not colour-managed; wide-gamut sources are re-encoded as-is.
- AVIF encoding runs without rav1e's assembly kernels by default (no NASM needed). Install NASM and build with `--features avif-asm` (`cargo build --features avif-asm` in `src-tauri`) for faster AVIF encoding.
- Release builds are unsigned. On macOS, right-click → Open the app the first time (or run `xattr -cr sharpR.app`).

## Releases

Pushing a `v*` tag runs the GitHub Actions workflow in `.github/workflows/release.yml`, which builds Windows (NSIS), macOS (Apple Silicon and Intel DMG) and Linux (AppImage + DEB) installers and attaches them to a draft release.

For testing on real devices without releasing, `.github/workflows/test-build.yml` builds unsigned Windows and macOS (Apple Silicon and Intel, ad-hoc signed) installers and attaches them to the workflow run as artifacts for 14 days. Push a branch named `test-build/<anything>`, or start it with **Actions → Test build → Run workflow** once the workflow is on the default branch.

## License

MIT. The bundled Manrope font is licensed under the SIL Open Font License (`src/assets/fonts/OFL.txt`).

## Third-party software

The installers bundle FFmpeg (GPL-3.0-or-later, built with x264, libvpx, Opus, dav1d and zimg), which sharpR runs as a separate program. The license text and build details ship with the app in `src-tauri/licenses/` (`FFmpeg-LICENSE.txt`, `THIRD-PARTY-NOTICES.md`).
