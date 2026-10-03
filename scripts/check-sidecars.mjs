import { execFileSync } from "node:child_process";
import { arch, platform } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const triples = {
  "linux-x64": "x86_64-unknown-linux-gnu",
  "darwin-arm64": "aarch64-apple-darwin",
  "darwin-x64": "x86_64-apple-darwin",
  "win32-x64": "x86_64-pc-windows-msvc",
};
const target = `${platform()}-${arch()}`;
const triple = triples[target];
if (!triple) throw new Error(`Unsupported build platform: ${target}`);
const binaries = join(dirname(dirname(fileURLToPath(import.meta.url))), "src-tauri", "binaries");
const suffix = platform() === "win32" ? ".exe" : "";

for (const tool of ["yt-dlp", "ffmpeg", "ffprobe", "deno"]) {
  const file = join(binaries, `${tool}-${triple}${suffix}`);
  const flag = tool === "ffmpeg" || tool === "ffprobe" ? "-version" : "--version";
  const output = execFileSync(file, [flag], { timeout: 30000, encoding: "utf8" });
  console.log(`${tool}: ${output.split(/\r?\n/, 1)[0]}`);
}
