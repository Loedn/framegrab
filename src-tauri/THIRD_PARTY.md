# Bundled download tools

- yt-dlp 2026.08.19, official Linux standalone release:
  https://github.com/yt-dlp/yt-dlp/releases/tag/2026.08.19
  Source and license information: https://github.com/yt-dlp/yt-dlp/tree/2026.08.19
- FFmpeg and ffprobe 7.0.2, static GPLv3 builds by John Van Sickle:
  https://johnvansickle.com/ffmpeg/
  License text: `FFmpeg-GPLv3.txt` in the bundle.
  Corresponding source and build information are available from the builder's site.
- Deno 2.9.6, official x86_64 Linux release:
  https://github.com/denoland/deno/releases/tag/v2.9.6
  Source and license information: https://github.com/denoland/deno/tree/v2.9.6

These tools are separate executables. The build preparation script downloads
versioned release assets and verifies their SHA-256 checksums before bundling.
