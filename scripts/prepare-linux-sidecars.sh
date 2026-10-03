#!/bin/sh
set -eu

if [ "$(uname -s)" != Linux ] || [ "$(uname -m)" != x86_64 ]; then
  echo "This preparation script supports Linux x86_64 only." >&2
  exit 1
fi

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cache="$root/.sidecar-downloads"
dest="$root/src-tauri/binaries"
triple=x86_64-unknown-linux-gnu
yt_version=2026.08.19
ff_version=7.0.2
deno_version=2.9.6
yt_sha=58162f9bfdc27458ea47bfcb311cf47028f17d8154a8bf7d689861d46399230a
ff_sha=abda8d77ce8309141f83ab8edf0596834087c52467f6badf376a6a2a4c87cf67
deno_sha=394f07f4da2bebe6ce6f1e7ce0fa16429b29b08c35e3fac3fe25972676dff4b2

mkdir -p "$cache" "$dest"

download() {
  url=$1
  output=$2
  sha=$3
  if [ ! -f "$output" ]; then
    curl -fL --retry 3 --silent --show-error -o "$output.part" "$url"
    mv "$output.part" "$output"
  fi
  printf '%s  %s\n' "$sha" "$output" | sha256sum --check --status || {
    echo "Checksum mismatch for $output. Remove that cached file and retry." >&2
    exit 1
  }
}

download "https://github.com/yt-dlp/yt-dlp/releases/download/$yt_version/yt-dlp_linux" \
  "$cache/yt-dlp_linux" "$yt_sha"
download "https://johnvansickle.com/ffmpeg/releases/ffmpeg-$ff_version-amd64-static.tar.xz" \
  "$cache/ffmpeg-$ff_version-amd64-static.tar.xz" "$ff_sha"
download "https://github.com/denoland/deno/releases/download/v$deno_version/deno-x86_64-unknown-linux-gnu.zip" \
  "$cache/deno-x86_64-unknown-linux-gnu.zip" "$deno_sha"

install -m 755 "$cache/yt-dlp_linux" "$dest/yt-dlp-$triple"
for tool in ffmpeg ffprobe; do
  tar -xJOf "$cache/ffmpeg-$ff_version-amd64-static.tar.xz" \
    "ffmpeg-$ff_version-amd64-static/$tool" > "$dest/$tool-$triple"
  chmod 755 "$dest/$tool-$triple"
done
tar -xJOf "$cache/ffmpeg-$ff_version-amd64-static.tar.xz" \
  "ffmpeg-$ff_version-amd64-static/GPLv3.txt" > "$dest/FFmpeg-GPLv3.txt"
cp "$root/src-tauri/licenses/FFmpeg-LICENSE.txt" "$dest/FFmpeg-LICENSE.txt"
unzip -p "$cache/deno-x86_64-unknown-linux-gnu.zip" deno > "$dest/deno-$triple"
chmod 755 "$dest/deno-$triple"

echo "Prepared verified yt-dlp $yt_version, static FFmpeg $ff_version, and Deno $deno_version sidecars."
