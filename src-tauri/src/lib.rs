#[cfg(not(target_os = "android"))]
use serde::Serialize;
#[cfg(not(target_os = "android"))]
use std::{
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};
#[cfg(target_os = "android")]
use tauri::Manager;
#[cfg(not(target_os = "android"))]
use tauri::{AppHandle, Emitter, Manager, State};
use url::Url;

#[cfg(not(target_os = "android"))]
mod desktop_install;

#[cfg(not(target_os = "android"))]
struct ActiveJob {
    id: String,
    cancelled: Arc<AtomicBool>,
}

#[cfg(not(target_os = "android"))]
#[derive(Default)]
struct DownloadState(Mutex<Option<ActiveJob>>);

#[cfg(not(target_os = "android"))]
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DownloadEvent {
    id: String,
    status: &'static str,
    message: String,
    percent: Option<f64>,
}

fn supported_url(input: &str) -> Result<(), String> {
    let url = Url::parse(input).map_err(|_| "Enter a valid link.".to_owned())?;
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
    {
        return Err("Use an HTTPS link without credentials or a custom port.".into());
    }
    let host = url.host_str().unwrap_or_default();
    const HOSTS: &[&str] = &[
        "x.com",
        "twitter.com",
        "instagram.com",
        "facebook.com",
        "fb.watch",
        "reddit.com",
        "redd.it",
        "tiktok.com",
        "youtube.com",
        "youtu.be",
    ];
    if !HOSTS
        .iter()
        .any(|domain| host == *domain || host.ends_with(&format!(".{domain}")))
    {
        return Err("This site is not supported yet.".into());
    }
    Ok(())
}

#[cfg(not(target_os = "android"))]
fn emit(
    app: &AppHandle,
    id: &str,
    status: &'static str,
    message: impl Into<String>,
    percent: Option<f64>,
) {
    let _ = app.emit(
        "download-event",
        DownloadEvent {
            id: id.to_owned(),
            status,
            message: message.into(),
            percent,
        },
    );
}

#[cfg(not(target_os = "android"))]
fn bundled_tool(name: &str) -> Result<PathBuf, String> {
    let executable = std::env::current_exe()
        .map_err(|_| "Could not locate the application executable.".to_owned())?;
    let file = format!("{name}{}", if cfg!(windows) { ".exe" } else { "" });
    let path = executable
        .parent()
        .ok_or("Could not locate the bundled download tools.")?
        .join(file);
    if !path.is_file() {
        return Err(format!("Bundled {name} is missing. Reinstall Framegrab."));
    }
    Ok(path)
}

#[cfg(not(target_os = "android"))]
#[tauri::command]
fn start_download(
    id: String,
    url: String,
    directory: String,
    state: State<'_, DownloadState>,
    app: AppHandle,
) -> Result<String, String> {
    let url = url.trim().to_owned();
    supported_url(&url)?;
    if !Path::new(&directory).is_dir() {
        return Err("Choose an existing download folder.".into());
    }

    if id.len() != 36 || !id.chars().all(|c| c.is_ascii_hexdigit() || c == '-') {
        return Err("Invalid download identifier.".into());
    }
    let cancelled = Arc::new(AtomicBool::new(false));
    {
        let mut active = state.0.lock().map_err(|_| "Download state unavailable.")?;
        if active.is_some() {
            return Err("Wait for the current download to finish.".into());
        }
        *active = Some(ActiveJob {
            id: id.clone(),
            cancelled: cancelled.clone(),
        });
    }

    let job_id = id.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let (status, message, percent) = run_download(&app, &job_id, &url, &directory, &cancelled);
        if let Ok(mut active) = app.state::<DownloadState>().0.lock() {
            if active.as_ref().is_some_and(|job| job.id == job_id) {
                *active = None;
            }
        }
        emit(&app, &job_id, status, message, percent);
    });
    Ok(id)
}

#[cfg(not(target_os = "android"))]
#[tauri::command]
fn cancel_download(id: String, state: State<'_, DownloadState>) -> Result<(), String> {
    let active = state.0.lock().map_err(|_| "Download state unavailable.")?;
    match active.as_ref() {
        Some(job) if job.id == id => {
            job.cancelled.store(true, Ordering::Relaxed);
            Ok(())
        }
        _ => Err("This download is no longer active.".into()),
    }
}

#[cfg(target_os = "android")]
struct AndroidMedia(tauri::plugin::PluginHandle<tauri::Wry>);

#[cfg(target_os = "android")]
fn android_media_plugin() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri::plugin::Builder::new("media")
        .setup(|app, api| {
            let handle = api.register_android_plugin("app.framegrab.mobile", "MediaPlugin")?;
            app.manage(AndroidMedia(handle));
            Ok(())
        })
        .build()
}

#[cfg(target_os = "android")]
#[tauri::command]
fn start_download(
    id: String,
    url: String,
    directory: String,
    media: tauri::State<'_, AndroidMedia>,
) -> Result<String, String> {
    let _ = directory;
    supported_url(&url)?;
    if !id.is_ascii() || id.len() != 36 || !id.chars().all(|c| c.is_ascii_hexdigit() || c == '-') {
        return Err("Invalid download identifier.".into());
    }
    media
        .0
        .run_mobile_plugin::<serde_json::Value>(
            "startDownload",
            serde_json::json!({ "id": id, "url": url }),
        )
        .map_err(|error| error.to_string())?;
    Ok(id)
}

#[cfg(target_os = "android")]
#[tauri::command]
fn cancel_download(id: String, media: tauri::State<'_, AndroidMedia>) -> Result<(), String> {
    media
        .0
        .run_mobile_plugin::<serde_json::Value>("cancelDownload", serde_json::json!({ "id": id }))
        .map_err(|error| error.to_string())?;
    Ok(())
}

#[cfg(not(target_os = "android"))]
#[tauri::command]
fn desktop_install_status() -> desktop_install::InstallStatus {
    desktop_install::status()
}

#[cfg(not(target_os = "android"))]
#[tauri::command]
fn desktop_install_app() -> Result<(), String> {
    desktop_install::install()
}

#[cfg(not(target_os = "android"))]
#[tauri::command]
fn desktop_uninstall_app() -> Result<(), String> {
    desktop_install::uninstall()
}

#[cfg(not(target_os = "android"))]
fn run_download(
    app: &AppHandle,
    id: &str,
    url: &str,
    directory: &str,
    cancelled: &AtomicBool,
) -> (&'static str, String, Option<f64>) {
    if cancelled.load(Ordering::Relaxed) {
        return ("cancelled", "Download cancelled.".into(), None);
    }
    let yt_dlp = match bundled_tool("yt-dlp") {
        Ok(path) => path,
        Err(message) => return ("failed", message, None),
    };
    let ffmpeg = match bundled_tool("ffmpeg") {
        Ok(path) => path,
        Err(message) => return ("failed", message, None),
    };
    if let Err(message) = bundled_tool("ffprobe") {
        return ("failed", message, None);
    }
    let deno = match bundled_tool("deno") {
        Ok(path) => path,
        Err(message) => return ("failed", message, None),
    };
    let mut child = match Command::new(&yt_dlp)
        .arg("--ffmpeg-location")
        .arg(&ffmpeg)
        .arg("--no-js-runtimes")
        .arg("--js-runtimes")
        .arg(format!("deno:{}", deno.display()))
        .args([
            "--ignore-config",
            "--no-plugin-dirs",
            "--no-playlist",
            "--no-overwrites",
            "--no-simulate",
            "--windows-filenames",
            "--trim-filenames",
            "160",
            "--newline",
            "--progress-template",
            "download:FG_PROGRESS:%(progress._percent_str)s",
            "--print",
            "after_move:FG_FILE:%(filepath)s",
            "--paths",
            directory,
            "--output",
            "%(title).160B [%(id)s].%(ext)s",
            "--",
            url,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(_) => {
            return (
                "failed",
                "Could not start the bundled yt-dlp tool. Reinstall Framegrab.".into(),
                None,
            );
        }
    };

    emit(app, id, "downloading", "Fetching video…", None);
    let stdout = child.stdout.take().expect("piped stdout");
    let stderr = child.stderr.take().expect("piped stderr");
    let output_path = Arc::new(Mutex::new(None::<String>));
    let errors = Arc::new(Mutex::new(Vec::<String>::new()));
    let path_for_reader = output_path.clone();
    let app_for_reader = app.clone();
    let id_for_reader = id.to_owned();
    let out_reader = thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            if let Some(percent) = line.strip_prefix("FG_PROGRESS:") {
                let number = percent
                    .trim()
                    .trim_end_matches('%')
                    .trim()
                    .parse::<f64>()
                    .ok();
                emit(
                    &app_for_reader,
                    &id_for_reader,
                    "downloading",
                    "Downloading…",
                    number,
                );
            } else if let Some(path) = line.strip_prefix("FG_FILE:") {
                if let Ok(mut value) = path_for_reader.lock() {
                    *value = Some(path.to_owned());
                }
            } else if line.contains("[Merger]") || line.contains("[ExtractAudio]") {
                emit(
                    &app_for_reader,
                    &id_for_reader,
                    "processing",
                    "Finishing video…",
                    None,
                );
            }
        }
    });
    let errors_for_reader = errors.clone();
    let err_reader = thread::spawn(move || {
        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
            if let Ok(mut lines) = errors_for_reader.lock() {
                if line.starts_with("ERROR:") {
                    lines.push(line.chars().take(300).collect());
                    if lines.len() > 3 {
                        lines.remove(0);
                    }
                }
            }
        }
    });

    let result = loop {
        if cancelled.load(Ordering::Relaxed) {
            let _ = child.kill();
            break child.wait();
        }
        match child.try_wait() {
            Ok(Some(exit)) => break Ok(exit),
            Ok(None) => thread::sleep(Duration::from_millis(120)),
            Err(error) => break Err(error),
        }
    };
    let _ = out_reader.join();
    let _ = err_reader.join();
    if cancelled.load(Ordering::Relaxed) {
        ("cancelled", "Download cancelled.".into(), None)
    } else if result.is_ok_and(|status| status.success()) {
        let path = output_path.lock().ok().and_then(|value| value.clone());
        (
            "completed",
            path.unwrap_or_else(|| directory.to_owned()),
            Some(100.0),
        )
    } else {
        let message = errors
            .lock()
            .ok()
            .and_then(|lines| lines.last().cloned())
            .unwrap_or_else(|| {
                "Download failed. The link may be unavailable or require a login.".into()
            });
        ("failed", message, None)
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "android")]
    {
        tauri::Builder::default()
            .plugin(android_media_plugin())
            .invoke_handler(tauri::generate_handler![start_download, cancel_download])
            .run(tauri::generate_context!())
            .expect("error while running Framegrab");
    }
    #[cfg(not(target_os = "android"))]
    {
        tauri::Builder::default()
            .plugin(tauri_plugin_dialog::init())
            .manage(DownloadState::default())
            .invoke_handler(tauri::generate_handler![
                start_download,
                cancel_download,
                desktop_install_status,
                desktop_install_app,
                desktop_uninstall_app
            ])
            .run(tauri::generate_context!())
            .expect("error while running Framegrab");
    }
}

#[cfg(test)]
mod tests {
    use super::supported_url;

    #[test]
    fn accepts_supported_https_links() {
        for url in [
            "https://x.com/a/status/1",
            "https://www.instagram.com/reel/1/",
            "https://m.facebook.com/watch/?v=1",
            "https://v.redd.it/abc",
            "https://vm.tiktok.com/abc",
            "https://youtu.be/abc",
            "https://youtube.com/shorts/abc",
        ] {
            assert!(supported_url(url).is_ok(), "{url}");
        }
    }

    #[test]
    fn rejects_untrusted_or_malformed_links() {
        for url in [
            "file:///etc/passwd",
            "http://youtube.com/watch?v=abc",
            "https://youtube.com.evil.test/watch",
            "https://user:pass@youtube.com/watch",
            "https://youtube.com:8443/watch",
            "https://example.com/video",
            "--help",
        ] {
            assert!(supported_url(url).is_err(), "{url}");
        }
    }
}
