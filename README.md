# Framegrab

A desktop-first video downloader built with Tauri 2, TypeScript, Rust, `yt-dlp`, FFmpeg, and Deno.

## Download and install

Download the latest release from [GitHub Releases](https://github.com/Loedn/framegrab/releases):

| System | Download | Install |
| --- | --- | --- |
| Windows x64 | `.exe` (NSIS) | Run the installer. |
| macOS Apple Silicon | `aarch64.dmg` | Open the disk image and drag Framegrab to Applications. |
| macOS Intel | `x64.dmg` | Open the disk image and drag Framegrab to Applications. |
| Linux x64 | `.AppImage` | Make it executable and run it; optionally click **Add to applications** in the app. |

The Windows and macOS installers are currently **unsigned previews**. Windows SmartScreen and macOS Gatekeeper may warn or block them. Signing/notarization requires platform certificates that are not configured in this repository. There is no mobile installer yet.

## Try it on Arch Linux

Run the built AppImage:

```sh
./src-tauri/target/release/bundle/appimage/Framegrab_0.2.0_amd64.AppImage
```

**No separate `yt-dlp`, FFmpeg, ffprobe, or Deno installation is needed.** Verified standalone copies are inside the AppImage and the app calls them by their bundled paths, not through your system `PATH`. The AppImage was built and launched on x86_64 Arch. If your system cannot mount AppImages without FUSE, use `APPIMAGE_EXTRACT_AND_RUN=1` before the command above. Pick a folder in the app, paste a public video link, and add it to the queue. Downloads run one at a time, with progress and cancellation.

To add Framegrab to your applications menu, open the AppImage and click **Add to applications** at the bottom of the left sidebar. This copies the AppImage to `~/.local/opt/framegrab/Framegrab.AppImage` and adds a launcher and icon under your user application directories. It does not need `sudo`, move your original AppImage, or touch your downloaded videos. Your desktop may take a moment to show the new menu item. Click **Remove from apps** in Framegrab to remove only the managed copy, launcher, and icon. You can still run the original AppImage afterward.

The first release accepts HTTPS links from YouTube, Instagram, TikTok, X/Twitter, Reddit, and Facebook. Support depends on `yt-dlp` and the source site; an accepted link is not a guarantee it can be downloaded. It does not support private/login-required posts, DRM, playlists, or browser cookie imports. Only save content you have permission to download.

## Develop

Build prerequisites: Node.js, npm, Rust, and the [Tauri 2 platform prerequisites](https://v2.tauri.app/start/prerequisites/). The first build downloads version-pinned platform executables and verifies SHA-256 checksums; later builds use the local `.sidecar-downloads/` cache. Linux x64 also needs `curl`, `tar`, `unzip`, and `sha256sum` for its build script. The installed app itself does not need these build tools.

```sh
npm install
npm run tauri -- dev
```

Build an Arch-compatible AppImage:

```sh
npm run tauri -- build --bundles appimage
```

Checks:

```sh
npm run build
npm test
```

The Windows NSIS and macOS DMG installers are built on their native GitHub Actions runners. A push of a `v*` tag builds all four installers, then publishes a prerelease with all downloads only after every build succeeds. Platform-specific bundled executables are pinned by SHA-256. FFmpeg license text and third-party source information are included in each bundle. macOS and Windows installers cannot be run or fully tested on this Arch host; the release workflow smoke-tests the tools on their native runners.

## Current limitations

- Jobs live in memory and disappear when the app closes. The selected folder is remembered locally.
- Cancelling a download can leave a partial file in the destination folder.
- Live downloads depend on network access and source-site availability; the automated checks cover URL validation and builds, not successful downloads from every platform.
