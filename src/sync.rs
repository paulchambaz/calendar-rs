use std::process::Command;

/// Sync calendars using vdirsyncer
pub fn sync_calendar(calendar_name: Option<String>) -> Result<(), String> {
    let mut vdirsyncer_command = Command::new("vdirsyncer");
    vdirsyncer_command.arg("sync");
    vdirsyncer_command.arg("--force-delete");

    if let Some(calendar) = &calendar_name {
        vdirsyncer_command.arg(calendar);
        println!("Syncing calendar '{}' with vdirsyncer", calendar);
    } else {
        println!("Syncing all calendars with vdirsyncer");
    }

    let output = vdirsyncer_command.output()
        .map_err(|e| format!("Failed to run vdirsyncer: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("vdirsyncer sync failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.is_empty() {
        println!("{}", stdout);
    }

    println!("Sync completed successfully");
    Ok(())
}

/// Check if vdirsyncer is available
pub fn check_vdirsyncer_available() -> bool {
    Command::new("vdirsyncer")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
