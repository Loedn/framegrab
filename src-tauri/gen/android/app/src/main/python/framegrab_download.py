"""On-device progressive MP4 downloads. No FFmpeg or JavaScript runtime is bundled."""

from pathlib import Path

import yt_dlp


class _Logger:
    def debug(self, message):
        pass

    def warning(self, message):
        pass

    def error(self, message):
        pass


def download(url, folder, sink):
    destination = Path(folder)
    destination.mkdir(parents=True, exist_ok=True)

    def progress(data):
        if sink.isCancelled():
            raise RuntimeError("Download cancelled")
        if data.get("status") == "downloading":
            total = data.get("total_bytes") or data.get("total_bytes_estimate")
            if total:
                sink.onProgress(min(100.0, data.get("downloaded_bytes", 0) * 100.0 / total))

    options = {
        # Direct MP4s from X may omit codec metadata while still containing both tracks.
        "format": "best[ext=mp4][vcodec!=?none][acodec!=?none]",
        "outtmpl": str(destination / "%(title).150B [%(id)s].%(ext)s"),
        "noplaylist": True,
        "overwrites": False,
        "quiet": True,
        "no_warnings": True,
        "logger": _Logger(),
        "progress_hooks": [progress],
    }
    with yt_dlp.YoutubeDL(options) as downloader:
        downloader.extract_info(url, download=True)

    if sink.isCancelled():
        raise RuntimeError("Download cancelled")
    files = [path for path in destination.iterdir() if path.is_file() and path.suffix.lower() == ".mp4"]
    if len(files) != 1:
        raise RuntimeError("No complete MP4 video was produced for this link.")
    return str(files[0])
