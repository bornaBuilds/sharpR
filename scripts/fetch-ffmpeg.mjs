// Installs the FFmpeg sidecar that tauri-build expects at
// src-tauri/binaries/sharpr-ffmpeg-<target-triple>[.exe].
//
//   node scripts/fetch-ffmpeg.mjs [--system] [--force] [triple ...]
//
// The host triple is always installed (cargo test builds for the host); extra
// triples cover cross-target bundles such as x86_64-apple-darwin on an arm64
// runner. Builds are pinned in scripts/ffmpeg-manifest.json and verified by
// SHA-256. --system copies the ffmpeg found on PATH instead, for quick local
// development only (releases must use the pinned builds).

import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  chmodSync,
  copyFileSync,
  createWriteStream,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { Readable, Transform } from "node:stream";
import { pipeline } from "node:stream/promises";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const BIN_DIR = join(ROOT, "src-tauri", "binaries");
const MANIFEST = JSON.parse(
  readFileSync(join(ROOT, "scripts", "ffmpeg-manifest.json"), "utf8"),
);
const HOST_LINE = /^host: (\S+)$/m;
const LINE_BREAK = /\r?\n/;
const WHITESPACE = /\s+/;
const EXECUTABLE_MODE = 0o755;
const SYSTEM_STAMP = "system";

// Everything sharpR's video pipeline relies on (see src-tauri/src/video/args.rs).
const REQUIRED = {
  encoders: ["libx264", "libvpx-vp9", "libopus", "aac", "png"],
  decoders: ["h264", "hevc", "vp9", "libdav1d"],
  filters: ["scale", "fps", "zscale", "tonemap"],
};

function log(message) {
  process.stdout.write(`[ffmpeg] ${message}\n`);
}

function hostTriple() {
  const output = execFileSync("rustc", ["-vV"], { encoding: "utf8" });
  const match = HOST_LINE.exec(output);
  if (!match) {
    throw new Error("Could not read the host triple from `rustc -vV`");
  }
  return match[1];
}

function destFor(triple) {
  const ext = triple.includes("windows") ? ".exe" : "";
  return join(BIN_DIR, `sharpr-ffmpeg-${triple}${ext}`);
}

function isCurrent(dest, stamp) {
  const stampFile = `${dest}.source`;
  return (
    existsSync(dest) &&
    existsSync(stampFile) &&
    readFileSync(stampFile, "utf8").trim() === stamp
  );
}

function install(binary, dest, stamp) {
  mkdirSync(BIN_DIR, { recursive: true });
  copyFileSync(binary, dest);
  chmodSync(dest, EXECUTABLE_MODE);
  writeFileSync(`${dest}.source`, `${stamp}\n`);
  log(`installed ${dest}`);
}

async function download(url, file) {
  const response = await fetch(url);
  if (!(response.ok && response.body)) {
    throw new Error(`Download failed (${response.status}): ${url}`);
  }
  const hash = createHash("sha256");
  const tap = new Transform({
    transform(chunk, _encoding, done) {
      hash.update(chunk);
      done(null, chunk);
    },
  });
  await pipeline(Readable.fromWeb(response.body), tap, createWriteStream(file));
  return hash.digest("hex");
}

function extract(archive, member, outDir) {
  // GNU tar has no zip support; Windows' bsdtar handles both formats.
  if (archive.endsWith(".zip") && process.platform !== "win32") {
    execFileSync("unzip", ["-o", "-q", archive, member, "-d", outDir]);
  } else {
    execFileSync("tar", ["-xf", archive, "-C", outDir, member]);
  }
  return join(outDir, member);
}

async function fetchPinned(triple, force) {
  const entry = MANIFEST.targets[triple];
  if (!entry) {
    throw new Error(`No FFmpeg build is pinned for ${triple}`);
  }
  const dest = destFor(triple);
  if (!force && isCurrent(dest, entry.sha256)) {
    log(`${triple}: up to date (${entry.version})`);
    return dest;
  }
  const work = mkdtempSync(join(tmpdir(), "sharpr-ffmpeg-"));
  try {
    const archive = join(work, entry.url.split("/").at(-1));
    log(`${triple}: downloading ${entry.version} from ${entry.source}`);
    const digest = await download(entry.url, archive);
    if (digest !== entry.sha256) {
      throw new Error(`${triple}: checksum mismatch (got ${digest})`);
    }
    install(extract(archive, entry.member, work), dest, entry.sha256);
    return dest;
  } finally {
    rmSync(work, { recursive: true, force: true });
  }
}

function copySystem(triple) {
  const finder = process.platform === "win32" ? "where" : "which";
  const found = execFileSync(finder, ["ffmpeg"], { encoding: "utf8" });
  const source = found.split(LINE_BREAK)[0].trim();
  const dest = destFor(triple);
  log(`${triple}: using system ffmpeg at ${source} (development only)`);
  install(source, dest, SYSTEM_STAMP);
  return dest;
}

function verify(binary) {
  const missing = [];
  for (const [kind, names] of Object.entries(REQUIRED)) {
    const listing = execFileSync(binary, ["-hide_banner", `-${kind}`], {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
    });
    const words = new Set(listing.split(WHITESPACE));
    for (const name of names) {
      if (!words.has(name)) {
        missing.push(`${kind}: ${name}`);
      }
    }
  }
  if (missing.length > 0) {
    throw new Error(`${binary} lacks ${missing.join(", ")}`);
  }
  log(`verified ${binary}`);
}

async function installTriple(triple, host, flags) {
  const useSystem = flags.has("--system") && triple === host;
  const dest = useSystem
    ? copySystem(triple)
    : await fetchPinned(triple, flags.has("--force"));
  if (triple === host) {
    verify(dest);
  }
}

async function main() {
  const args = process.argv.slice(2).filter(Boolean);
  const flags = new Set(args.filter((arg) => arg.startsWith("--")));
  const host = hostTriple();
  const triples = new Set([
    host,
    ...args.filter((arg) => !arg.startsWith("--")),
  ]);
  await Promise.all(
    [...triples].map((triple) => installTriple(triple, host, flags)),
  );
}

main().catch((error) => {
  process.stderr.write(`[ffmpeg] ${error.message}\n`);
  process.exitCode = 1;
});
