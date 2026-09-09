# sharpR

Local desktop tool for preparing images for the web: batch-convert to WebP / AVIF / JPEG / PNG, and crop, rotate, skew, flip and resize single images. Everything runs on your machine; no image ever leaves it.

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

## Development

Prerequisites: [Rust](https://rustup.rs) (stable), Node 24, pnpm 11, and the platform prerequisites from the [Tauri guide](https://v2.tauri.app/start/prerequisites/) (Visual Studio Build Tools with C++ on Windows, Xcode Command Line Tools on macOS, `libwebkit2gtk-4.1-dev` and friends on Linux). A C compiler is required because libwebp is compiled from source.

```bash
pnpm install
pnpm tauri dev
```

Useful scripts:

| Command | What it does |
|---|---|
| `pnpm tauri dev` | Run the desktop app with hot reload |
| `pnpm tauri build` | Build the installer for the current platform |
| `pnpm lint` / `pnpm fix` | Biome lint + format (check / write) |
| `pnpm check` | svelte-check type checking |
| `pnpm test` | Frontend unit tests (vitest) |
| `cargo test --manifest-path src-tauri/Cargo.toml` | Rust unit and pipeline tests |

## Architecture

- `src/` — Svelte 5 frontend. Stores live in `src/lib/stores/*.svelte.ts`; typed Tauri command wrappers in `src/lib/tauri/`.
- `src-tauri/src/commands/` — Tauri commands: file intake, conversion, saving, cropping.
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

## License

MIT. The bundled Manrope font is licensed under the SIL Open Font License (`src/assets/fonts/OFL.txt`).
