import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { createReadStream, createWriteStream, existsSync, mkdirSync, copyFileSync, chmodSync, readFileSync, renameSync, writeFileSync } from "node:fs";
import { platform, arch } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { pipeline } from "node:stream/promises";
import { Readable } from "node:stream";
import { unzipSync } from "fflate";

const root = dirname(dirname(fileURLToPath(import.meta.url)));
const cache = join(root, ".sidecar-downloads");
const binaries = join(root, "src-tauri", "binaries");

const target = process.argv[2] ?? `${platform()}-${arch()}`;
if (target === "linux-x64") {
  execFileSync("sh", [join(root, "scripts", "prepare-linux-sidecars.sh")], { stdio: "inherit" });
  process.exit(0);
}

const releases = {
  "darwin-arm64": {
    triple: "aarch64-apple-darwin",
    yt: ["yt-dlp_macos", "0f192b7ec147ab6288885d6351d9ab67367640029b4377576ef46dd79cf7b202"],
    ffmpeg: ["ffmpeg-darwin-arm64", "9f865039102a1139c7057d7f21ddaacd106d602fa3af1f99b70f43d520439b8c"],
    ffprobe: ["ffprobe-darwin-arm64", "05a26b32c32115785d48b01601e104712bbc6c2b1d363b9cf44c42232684e25e"],
    deno: ["deno-aarch64-apple-darwin.zip", "213a2f304f04d3c9cb5220669afad138f60a5aab1fe80962abdeb8f35807a472"],
  },
  "darwin-x64": {
    triple: "x86_64-apple-darwin",
    yt: ["yt-dlp_macos", "0f192b7ec147ab6288885d6351d9ab67367640029b4377576ef46dd79cf7b202"],
    ffmpeg: ["ffmpeg-darwin-x64", "4a4a968b98859588e98500ae25973d80a5ca5eed0724222b9f76360dcb72a001"],
    ffprobe: ["ffprobe-darwin-x64", "ce5414269f0efa1e88b5e23b57f801d5b9a40be554716544936e0332b4601a62"],
    deno: ["deno-x86_64-apple-darwin.zip", "7d4524b82bcc557fe020a1a5b56956ed42b992ae5b28026e8ad5d17329533f5f"],
  },
  "win32-x64": {
    triple: "x86_64-pc-windows-msvc",
    yt: ["yt-dlp.exe", "66674953fe251b89f4d08c5f0e35e0728679bd67ab3d7d05c0562af101dd3e7a"],
    ffmpeg: ["ffmpeg-win32-x64", "4b3a7a7c41a4064a8738536da6a304371c1fdc567c599ccb768bebd49ccdb9fb"],
    ffprobe: ["ffprobe-win32-x64", "c75d3ea9d6ab334ee2e90f15232af80aeac24d8a3185cd6ba19680acb8f43732"],
    deno: ["deno-x86_64-pc-windows-msvc.zip", "15e5300b0ba3c3695a7621d90160a746ec9e710228cee639afa9d580f6e3cd11"],
  },
};

const current = releases[target];
if (!current) throw new Error(`Unsupported build platform: ${target}`);

mkdirSync(cache, { recursive: true });
mkdirSync(binaries, { recursive: true });

async function sha256(path) {
  const hash = createHash("sha256");
  for await (const chunk of createReadStream(path)) hash.update(chunk);
  return hash.digest("hex");
}

async function download(name, expected, url) {
  const destination = join(cache, name);
  if (!existsSync(destination)) {
    const response = await fetch(url);
    if (!response.ok || !response.body) throw new Error(`Download failed for ${name}: HTTP ${response.status}`);
    const partial = `${destination}.part`;
    await pipeline(Readable.fromWeb(response.body), createWriteStream(partial, { flags: "w" }));
    const actual = await sha256(partial);
    if (actual !== expected) throw new Error(`Checksum mismatch for ${name}; refusing to use it`);
    renameSync(partial, destination);
  }
  if (await sha256(destination) !== expected) throw new Error(`Checksum mismatch for cached ${name}; remove the cached file and retry`);
  return destination;
}

const ytUrl = `https://github.com/yt-dlp/yt-dlp/releases/download/2026.08.19/${current.yt[0]}`;
const ffUrl = (name) => `https://github.com/descriptinc/ffmpeg-ffprobe-static/releases/download/b6.1.2-rc.1/${name}`;
const denoUrl = `https://github.com/denoland/deno/releases/download/v2.9.6/${current.deno[0]}`;
const assets = await Promise.all([
  download(...current.yt, ytUrl),
  download(...current.ffmpeg, ffUrl(current.ffmpeg[0])),
  download(...current.ffprobe, ffUrl(current.ffprobe[0])),
  download(...current.deno, denoUrl),
]);

const isWindows = target.startsWith("win32-");
const suffix = isWindows ? ".exe" : "";
for (const [index, name] of ["yt-dlp", "ffmpeg", "ffprobe"].entries()) {
  const destination = join(binaries, `${name}-${current.triple}${suffix}`);
  copyFileSync(assets[index], destination);
  if (!isWindows) chmodSync(destination, 0o755);
}

const zip = unzipSync(readFileSync(assets[3]));
const deno = zip[isWindows ? "deno.exe" : "deno"];
if (!deno) throw new Error("Deno release archive does not contain the expected executable");
const denoPath = join(binaries, `deno-${current.triple}${suffix}`);
writeFileSync(denoPath, deno);
if (!isWindows) chmodSync(denoPath, 0o755);

// Shipped with each platform, alongside the pinned binary provenance in THIRD_PARTY.md.
copyFileSync(join(root, "src-tauri", "licenses", "GPLv3.txt"), join(binaries, "FFmpeg-GPLv3.txt"));
copyFileSync(join(root, "src-tauri", "licenses", "FFmpeg-LICENSE.txt"), join(binaries, "FFmpeg-LICENSE.txt"));
console.log(`Prepared verified ${target} downloader sidecars.`);
