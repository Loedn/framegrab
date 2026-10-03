use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallStatus {
    pub available: bool,
    pub installed: bool,
}

#[cfg(target_os = "linux")]
mod linux {
    use super::InstallStatus;
    use std::{
        env, fs,
        io::{self, Write},
        os::unix::fs::PermissionsExt,
        path::{Path, PathBuf},
    };

    const APP_NAME: &str = "Framegrab.AppImage";
    const DESKTOP_NAME: &str = "app.framegrab.desktop";
    const ICON_NAME: &str = "framegrab-appimage";
    const MARKER: &str = "framegrab-appimage-integration-v1\n";
    const ICON: &[u8] = include_bytes!("../icons/128x128@2x.png");

    struct Locations {
        install_dir: PathBuf,
        executable: PathBuf,
        marker: PathBuf,
        desktop: PathBuf,
        icon: PathBuf,
    }

    fn locations(home: &Path, data_home: &Path) -> Locations {
        let install_dir = home.join(".local/opt/framegrab");
        Locations {
            executable: install_dir.join(APP_NAME),
            marker: install_dir.join(".framegrab-installed"),
            install_dir,
            desktop: data_home.join("applications").join(DESKTOP_NAME),
            icon: data_home
                .join("icons/hicolor/256x256/apps")
                .join(format!("{ICON_NAME}.png")),
        }
    }

    fn paths() -> Result<Locations, String> {
        let home = env::var_os("HOME").ok_or("HOME is not set.")?;
        let home = PathBuf::from(home);
        if !home.is_absolute() {
            return Err("HOME must be an absolute path.".into());
        }
        let data_home = env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .unwrap_or_else(|| home.join(".local/share"));
        Ok(locations(&home, &data_home))
    }

    fn appimage_source() -> Result<PathBuf, String> {
        let source = env::var_os("APPIMAGE")
            .map(PathBuf::from)
            .ok_or("Run the AppImage to add it to your applications menu.")?;
        if !source.is_absolute() || !source.is_file() {
            return Err("The running AppImage could not be found.".into());
        }
        Ok(source)
    }

    fn escaped_exec(path: &Path) -> Result<String, String> {
        let text = path
            .to_str()
            .ok_or("The installation path must be valid UTF-8.")?;
        if text.contains(['\n', '\r']) {
            return Err("The installation path cannot contain a newline.".into());
        }
        let quoted = text
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('$', "\\$")
            .replace('`', "\\`")
            .replace('%', "%%");
        Ok(format!("\"{quoted}\""))
    }

    fn desktop_entry(path: &Path) -> Result<String, String> {
        Ok(format!(
            "[Desktop Entry]\nType=Application\nName=Framegrab\nComment=Download videos from social links\nExec={}\nIcon={ICON_NAME}\nTerminal=false\nCategories=AudioVideo;Network;\nX-Framegrab-AppImage=true\n",
            escaped_exec(path)?
        ))
    }

    fn owned(loc: &Locations) -> bool {
        fs::read_to_string(&loc.marker).is_ok_and(|text| text == MARKER)
            && fs::read_to_string(&loc.desktop).is_ok_and(|text| {
                desktop_entry(&loc.executable).is_ok_and(|expected| text == expected)
            })
            && loc.executable.is_file()
    }

    fn exists(path: &Path) -> bool {
        fs::symlink_metadata(path).is_ok()
    }

    fn write_new(path: &Path, bytes: &[u8]) -> io::Result<()> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)?;
        file.write_all(bytes)
    }

    pub fn status() -> InstallStatus {
        InstallStatus {
            available: appimage_source().is_ok(),
            installed: paths().is_ok_and(|loc| owned(&loc)),
        }
    }

    pub fn install() -> Result<(), String> {
        let source = appimage_source()?;
        let loc = paths()?;
        install_from(&source, &loc)
    }

    fn install_from(source: &Path, loc: &Locations) -> Result<(), String> {
        if owned(&loc) {
            return Err("Framegrab is already in your applications menu.".into());
        }
        if [&loc.install_dir, &loc.desktop, &loc.icon]
            .iter()
            .any(|path| exists(path))
        {
            return Err("An installation already uses Framegrab's application paths. No files were changed.".into());
        }
        let entry = desktop_entry(&loc.executable)?;
        fs::create_dir_all(loc.install_dir.parent().expect("install parent"))
            .map_err(|error| format!("Could not create the app folder: {error}"))?;
        fs::create_dir(&loc.install_dir)
            .map_err(|error| format!("Could not create the app folder: {error}"))?;

        // The launcher is written last, so the menu never points at a partial installation.
        let mut executable_created = false;
        let mut marker_created = false;
        let mut icon_created = false;
        let result = (|| -> io::Result<()> {
            let mut input = fs::File::open(source)?;
            let mut output = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&loc.executable)?;
            executable_created = true;
            io::copy(&mut input, &mut output)?;
            fs::set_permissions(&loc.executable, fs::Permissions::from_mode(0o755))?;
            write_new(&loc.marker, MARKER.as_bytes())?;
            marker_created = true;
            fs::create_dir_all(loc.icon.parent().expect("icon parent"))?;
            write_new(&loc.icon, ICON)?;
            icon_created = true;
            fs::create_dir_all(loc.desktop.parent().expect("desktop parent"))?;
            write_new(&loc.desktop, entry.as_bytes())?;
            Ok(())
        })();
        if let Err(error) = result {
            if icon_created {
                let _ = fs::remove_file(&loc.icon);
            }
            if marker_created {
                let _ = fs::remove_file(&loc.marker);
            }
            if executable_created {
                let _ = fs::remove_file(&loc.executable);
            }
            let _ = fs::remove_dir(&loc.install_dir);
            return Err(format!(
                "Could not add Framegrab to your applications: {error}"
            ));
        }
        Ok(())
    }

    pub fn uninstall() -> Result<(), String> {
        let loc = paths()?;
        uninstall_at(&loc)
    }

    fn uninstall_at(loc: &Locations) -> Result<(), String> {
        if !owned(&loc) {
            return Err("No Framegrab installation managed by this app was found.".into());
        }
        fs::remove_file(&loc.desktop)
            .map_err(|error| format!("Could not remove the menu entry: {error}"))?;
        if fs::read(&loc.icon).is_ok_and(|bytes| bytes == ICON) {
            fs::remove_file(&loc.icon)
                .map_err(|error| format!("Could not remove the icon: {error}"))?;
        }
        fs::remove_file(&loc.executable)
            .map_err(|error| format!("Could not remove the installed AppImage: {error}"))?;
        fs::remove_file(&loc.marker)
            .map_err(|error| format!("Could not remove the install marker: {error}"))?;
        let _ = fs::remove_dir(&loc.install_dir);
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::time::{SystemTime, UNIX_EPOCH};

        #[test]
        fn launcher_escapes_paths_and_has_required_fields() {
            let entry =
                desktop_entry(Path::new("/home/hello world/100%/$'\"/Framegrab.AppImage")).unwrap();
            assert!(entry.contains("Type=Application\nName=Framegrab\n"));
            assert!(entry.contains("Exec=\"/home/hello world/100%%/\\$'\\\"/Framegrab.AppImage\""));
            assert!(entry.contains("Icon=framegrab-appimage\nTerminal=false"));
        }

        #[test]
        fn refuses_line_breaks_in_launcher_path() {
            assert!(desktop_entry(Path::new("/home/bad\nname/Framegrab.AppImage")).is_err());
        }

        #[test]
        fn installation_paths_stay_under_user_directories() {
            let loc = locations(Path::new("/tmp/test-home"), Path::new("/tmp/test-data"));
            assert_eq!(
                loc.executable,
                Path::new("/tmp/test-home/.local/opt/framegrab/Framegrab.AppImage")
            );
            assert_eq!(
                loc.desktop,
                Path::new("/tmp/test-data/applications/app.framegrab.desktop")
            );
        }

        #[test]
        fn installs_and_removes_only_managed_files() {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let root = env::temp_dir().join(format!(
                "framegrab-install-test-{}-{nonce}",
                std::process::id()
            ));
            fs::create_dir(&root).unwrap();
            let source = root.join("source.AppImage");
            fs::write(&source, b"test appimage").unwrap();
            let loc = locations(&root.join("home"), &root.join("data"));
            install_from(&source, &loc).unwrap();
            assert!(owned(&loc));
            assert_eq!(fs::read(&loc.executable).unwrap(), b"test appimage");
            assert_eq!(fs::read(&loc.icon).unwrap(), ICON);
            if let Ok(output) = std::process::Command::new("desktop-file-validate")
                .arg(&loc.desktop)
                .output()
            {
                assert!(
                    output.status.success(),
                    "Invalid launcher: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            assert!(install_from(&source, &loc).is_err());
            let unrelated = root.join("unrelated.txt");
            fs::write(&unrelated, "keep").unwrap();
            uninstall_at(&loc).unwrap();
            assert!(!exists(&loc.executable));
            assert!(!exists(&loc.desktop));
            assert!(!exists(&loc.icon));
            assert_eq!(fs::read_to_string(unrelated).unwrap(), "keep");
            fs::remove_dir_all(root).unwrap();
        }

        #[test]
        fn refuses_to_replace_an_existing_launcher() {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let root = env::temp_dir().join(format!(
                "framegrab-collision-test-{}-{nonce}",
                std::process::id()
            ));
            fs::create_dir(&root).unwrap();
            let source = root.join("source.AppImage");
            fs::write(&source, b"test appimage").unwrap();
            let loc = locations(&root.join("home"), &root.join("data"));
            fs::create_dir_all(loc.desktop.parent().unwrap()).unwrap();
            fs::write(&loc.desktop, "someone else's launcher").unwrap();
            assert!(install_from(&source, &loc).is_err());
            assert_eq!(
                fs::read_to_string(&loc.desktop).unwrap(),
                "someone else's launcher"
            );
            assert!(!exists(&loc.executable));
            fs::remove_dir_all(root).unwrap();
        }
    }
}

#[cfg(target_os = "linux")]
pub use linux::{install, status, uninstall};

#[cfg(not(target_os = "linux"))]
pub fn status() -> InstallStatus {
    InstallStatus {
        available: false,
        installed: false,
    }
}

#[cfg(not(target_os = "linux"))]
pub fn install() -> Result<(), String> {
    Err("Application menu integration is only available on Linux.".into())
}

#[cfg(not(target_os = "linux"))]
pub fn uninstall() -> Result<(), String> {
    Err("Application menu integration is only available on Linux.".into())
}
