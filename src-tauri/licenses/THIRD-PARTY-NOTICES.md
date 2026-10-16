# Third-party software in sharpR

sharpR itself is MIT licensed. The installers also contain the following
third-party software.

## FFmpeg

The video tab drives a bundled FFmpeg executable (`sharpr-ffmpeg`). sharpR
runs it as a separate program and talks to it only through command-line
arguments and pipes; the two are not linked together.

FFmpeg is free software licensed under the GNU General Public License,
version 3 or later (see `FFmpeg-LICENSE.txt`). The bundled builds include,
among others, x264 (GPL-2.0-or-later), libvpx (BSD-3-Clause), Opus
(BSD-3-Clause), dav1d (BSD-2-Clause), zimg (WTFPL) and SVT-AV1 (BSD-3-Clause).

| Platform | FFmpeg version | Build |
|---|---|---|
| Windows x64, Linux x64 | n9.0.2-28-g669f5ab083 | BtbN/FFmpeg-Builds, `gpl` variant: https://github.com/BtbN/FFmpeg-Builds |
| macOS (Apple Silicon, Intel) | 9.0.2 | Martin Riedl: https://ffmpeg.martin-riedl.de |

The exact download URLs and SHA-256 checksums are pinned in
`scripts/ffmpeg-manifest.json` in the sharpR repository
(https://github.com/bornaBuilds/sharpR). The corresponding FFmpeg source code
is available from https://ffmpeg.org/download.html and from the build
projects above, which publish their build scripts and library versions.

## Manrope

The interface font, Manrope, is licensed under the SIL Open Font License 1.1.
