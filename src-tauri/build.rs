fn main() {
    let attributes = tauri_build::Attributes::new().plugin(
        "media",
        tauri_build::InlinedPlugin::new()
            .commands(&["register_listener", "remove_listener"])
            .default_permission(tauri_build::DefaultPermissionRule::AllowAllCommands),
    );
    tauri_build::try_build(attributes).expect("failed to build Tauri application");
}
