# Framegrab

A desktop and Android video downloader built with Tauri 2, TypeScript, Rust, and `yt-dlp`. Desktop bundles FFmpeg and Deno; the Android preview does not.

## Download and install

Download the latest release from [GitHub Releases](https://github.com/Loedn/framegrab/releases):

| System | Download | Install |
| --- | --- | --- |
| Windows x64 | `.exe` (NSIS) | Run the installer. |
| macOS Apple Silicon | `aarch64.dmg` | Open the disk image and drag Framegrab to Applications. |
| macOS Intel | `x64.dmg` | Open the disk image and drag Framegrab to Applications. |
| Linux x64 | `.AppImage` | Make it executable and run it; optionally click **Add to applications** in the app. |
| Android arm64 (Android 10+) | `.apk` | Open the APK on your phone and allow installation from your browser or file manager when prompted. |

The Windows and macOS installers are currently **unsigned previews**. Windows SmartScreen and macOS Gatekeeper may warn or block them. Signing/notarization requires platform certificates that are not configured in this repository. The Android APK is signed with a separate stable preview key, not a Play Store certificate.

## Try it on Arch Linux

Run the built AppImage:

```sh
./src-tauri/target/release/bundle/appimage/Framegrab_0.3.1_amd64.AppImage
```

**No separate `yt-dlp`, FFmpeg, ffprobe, or Deno installation is needed.** Verified standalone copies are inside the AppImage and the app calls them by their bundled paths, not through your system `PATH`. The AppImage was built and launched on x86_64 Arch. If your system cannot mount AppImages without FUSE, use `APPIMAGE_EXTRACT_AND_RUN=1` before the command above. Pick a folder in the app, paste a public video link, and add it to the queue. Downloads run one at a time, with progress and cancellation.

To add Framegrab to your applications menu, open the AppImage and click **Add to applications** at the bottom of the left sidebar. This copies the AppImage to `~/.local/opt/framegrab/Framegrab.AppImage` and adds a launcher and icon under your user application directories. It does not need `sudo`, move your original AppImage, or touch your downloaded videos. Your desktop may take a moment to show the new menu item. Click **Remove from apps** in Framegrab to remove only the managed copy, launcher, and icon. You can still run the original AppImage afterward.

The first release accepts HTTPS links from YouTube, Instagram, TikTok, X/Twitter, Reddit, and Facebook. Support depends on `yt-dlp` and the source site; an accepted link is not a guarantee it can be downloaded. It does not support private/login-required posts, DRM, playlists, or browser cookie imports. Only save content you have permission to download.

## Android preview

Framegrab downloads directly on the device and saves finished videos in **Movies/Framegrab**, visible in the gallery or Files app. It does not send links or videos to a Framegrab server. The current APK is arm64 only and requires Android 10 or later. An X direct MP4 was downloaded in an Android 15 x86_64 emulator with both audio and video and saved through MediaStore; the arm64 APK builds, but has not been tested on a physical phone.

Android includes Python 3.13 and pinned `yt-dlp` 2026.08.19. It selects a direct MP4 containing both video and audio. No FFmpeg, ffprobe, or Deno is included. Many YouTube, Reddit, and other links provide separate audio/video streams and **cannot** be downloaded by this preview. Some direct MP4 links, including an X link in the emulator test, work. Private posts, login/cookie-required content, DRM, playlists, formats requiring a JavaScript runtime, and site extractors that change may fail. Do not assume all six displayed platforms work on Android. Jobs run only while the app is open and disappear on restart.

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

The Windows NSIS and macOS DMG installers are built on their native GitHub Actions runners. A push of a `v*` tag builds all four desktop installers and the Android APK, then publishes a prerelease only after every build succeeds. Platform-specific bundled executables are pinned by SHA-256. FFmpeg license text and third-party source information are included in each desktop bundle. macOS and Windows installers cannot be run or fully tested on this Arch host; the release workflow smoke-tests the tools on their native runners.

Android builds need JDK 21, Android SDK 36, NDK 27.2.12479018, Python 3.13, Rust's `aarch64-linux-android` target, and Node.js. To build an unsigned local release APK:

```sh
npm run tauri -- android build --apk --target aarch64 --ci
```

Before publishing, configure the repository secrets `ANDROID_KEYSTORE_BASE64` (base64-encoded keystore with key alias `framegrab`) and `ANDROID_KEYSTORE_PASSWORD`. Back up the keystore securely: losing it prevents future APKs from updating existing installations. Never add the key or password to the repository.

## Current limitations

- Jobs live in memory and disappear when the app closes. The selected folder is remembered locally.
- Cancelling a download can leave a partial file in the destination folder.
- Live downloads depend on network access and source-site availability; the automated checks cover URL validation and builds, not successful downloads from every platform.
