# Framegrab

A desktop-first video downloader built with Tauri 2, TypeScript, Rust, `yt-dlp`, FFmpeg, and Deno.

## Try it on Arch Linux

Run the built AppImage:

```sh
./src-tauri/target/release/bundle/appimage/Framegrab_0.1.1_amd64.AppImage
```

**No separate `yt-dlp`, FFmpeg, ffprobe, or Deno installation is needed.** Verified standalone copies are inside the AppImage and the app calls them by their bundled paths, not through your system `PATH`. The AppImage was built and launched on x86_64 Arch. If your system cannot mount AppImages without FUSE, use `APPIMAGE_EXTRACT_AND_RUN=1` before the command above. Pick a folder in the app, paste a public video link, and add it to the queue. Downloads run one at a time, with progress and cancellation.

To add Framegrab to your applications menu, open the AppImage and click **Add to applications** at the bottom of the left sidebar. This copies the AppImage to `~/.local/opt/framegrab/Framegrab.AppImage` and adds a launcher and icon under your user application directories. It does not need `sudo`, move your original AppImage, or touch your downloaded videos. Your desktop may take a moment to show the new menu item. Click **Remove from apps** in Framegrab to remove only the managed copy, launcher, and icon. You can still run the original AppImage afterward.

The first release accepts HTTPS links from YouTube, Instagram, TikTok, X/Twitter, Reddit, and Facebook. Support depends on `yt-dlp` and the source site; an accepted link is not a guarantee it can be downloaded. It does not support private/login-required posts, DRM, playlists, or browser cookie imports. Only save content you have permission to download.

## Develop

Build prerequisites: Node.js, npm, Rust, the [Tauri 2 Linux prerequisites](https://v2.tauri.app/start/prerequisites/#linux), and `curl`, `tar`, `unzip`, and `sha256sum`. The first build downloads version-pinned Linux x86_64 executables and verifies SHA-256 checksums; later builds use the local `.sidecar-downloads/` cache. The app itself does not need these build tools installed separately.

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

The UI and Rust backend are structured for Windows and macOS builds, but **only the Linux x86_64 AppImage has been built and smoke-tested so far**. Other platforms will need matching, verified sidecar binaries and build scripts. The FFmpeg static build is GPLv3; its license text and third-party source information are included in the bundle. Mobile is out of scope for this first version.

## Current limitations

- Jobs live in memory and disappear when the app closes. The selected folder is remembered locally.
- Cancelling a download can leave a partial file in the destination folder.
- Live downloads depend on network access and source-site availability; the automated checks cover URL validation and builds, not successful downloads from every platform.
