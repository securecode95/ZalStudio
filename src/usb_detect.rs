use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, channel};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct UsbBlockDev {
    pub _name: String,
    pub dev_path: String,
    pub mountpoint: Option<PathBuf>,
    pub children: Vec<UsbBlockDev>,
}

#[derive(Debug, Clone)]
pub struct UsbSnapshot {
    pub devices: Vec<UsbBlockDev>,
}

#[derive(Debug, Clone)]
pub enum UsbResult {
    /// Device found and mounted — UI should switch to Importing
    Mounted(PathBuf),
    /// Scanned photo list from device — UI should show picker
    FoundPhotos(Vec<crate::wired_import::PhonePhoto>),
    /// Copy complete — UI should switch to Gallery
    Imported(usize),
    /// Progress update during copy (current, total, current_file_name)
    Progress {
        current: usize,
        total: usize,
        file_name: String,
    },
    /// Something failed
    Error(String),
}

// ---------------------------------------------------------------------------
// Parse lsblk JSON output
// ---------------------------------------------------------------------------

#[derive(serde::Deserialize)]
struct LsblkChild {
    name: String,
    #[serde(rename = "type")]
    _kind: String,
    _tran: Option<String>,
    mountpoint: Option<String>,
}

#[derive(serde::Deserialize)]
struct LsblkDevice {
    name: String,
    #[serde(rename = "type")]
    _kind: String,
    tran: Option<String>,
    mountpoint: Option<String>,
    children: Option<Vec<LsblkChild>>,
}

#[derive(serde::Deserialize)]
struct LsblkRoot {
    blockdevices: Vec<LsblkDevice>,
}

/// Capture a snapshot of all USB block devices from lsblk.
pub fn usb_snapshot() -> UsbSnapshot {
    let mut devices = Vec::new();

    if let Ok(output) = std::process::Command::new("lsblk")
        .args(["-o", "NAME,TYPE,TRAN,MOUNTPOINT", "-J"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Ok(root) = serde_json::from_str::<LsblkRoot>(&stdout) {
                for dev in root.blockdevices {
                    if dev.tran.as_deref() != Some("usb") {
                        continue;
                    }
                    if dev._kind != "disk" {
                        continue;
                    }

                    let mut children = Vec::new();
                    if let Some(c) = dev.children {
                        for child in c {
                            children.push(UsbBlockDev {
                                dev_path: format!("/dev/{}", child.name),
                                _name: child.name,
                                mountpoint: child.mountpoint.as_deref().map(PathBuf::from),
                                children: Vec::new(),
                            });
                        }
                    }

                    devices.push(UsbBlockDev {
                        dev_path: format!("/dev/{}", dev.name),
                        _name: dev.name,
                        mountpoint: dev.mountpoint.as_deref().map(PathBuf::from),
                        children,
                    });
                }
            }
        }
    }

    // Fallback: if lsblk found nothing, try /dev/disk/by-id/usb-*
    if devices.is_empty() {
        if let Ok(entries) = std::fs::read_dir("/dev/disk/by-id") {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("usb-")
                    && !name.ends_with("-part1")
                    && !name.ends_with("-part2")
                    && !name.ends_with("-part3")
                    && !name.ends_with("-part4")
                {
                    if let Ok(target) = std::fs::canonicalize(entry.path()) {
                        let dev_name = target
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default();
                        if !dev_name.is_empty()
                            && !devices
                                .iter()
                                .any(|d| d.dev_path == target.to_string_lossy())
                        {
                            devices.push(UsbBlockDev {
                                dev_path: target.to_string_lossy().to_string(),
                                _name: dev_name,
                                mountpoint: None,
                                children: Vec::new(),
                            });
                        }
                    }
                }
            }
        }
    }

    UsbSnapshot { devices }
}

/// Collect every mountpoint (disk or partition) present in the snapshot.
pub fn usb_mountpoints(snapshot: &UsbSnapshot) -> Vec<PathBuf> {
    let mut mounts = Vec::new();
    for dev in &snapshot.devices {
        if let Some(ref m) = dev.mountpoint {
            mounts.push(m.clone());
        }
        for child in &dev.children {
            if let Some(ref m) = child.mountpoint {
                mounts.push(m.clone());
            }
        }
    }
    mounts
}

/// Collect device paths of unmounted USB partitions / whole disks.
pub fn unmounted_usb_partitions(snapshot: &UsbSnapshot) -> Vec<String> {
    let mut parts = Vec::new();
    for dev in &snapshot.devices {
        if dev.children.is_empty() {
            if dev.mountpoint.is_none() {
                parts.push(dev.dev_path.clone());
            }
        } else {
            for child in &dev.children {
                if child.mountpoint.is_none() {
                    parts.push(child.dev_path.clone());
                }
            }
        }
    }
    parts
}

/// Auto-mount a block device using udisks2 (no root required on most desktops).
/// Never uses pkexec — avoids password prompts on kiosk machines.
pub fn auto_mount(dev: &str) -> Result<PathBuf, String> {
    let output = std::process::Command::new("udisksctl")
        .args(["mount", "--block-device", dev, "--no-user-interaction"])
        .output()
        .map_err(|e| format!("udisksctl failed: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        if let Some(at_pos) = stdout.find(" at ") {
            let mp = stdout[at_pos + 4..]
                .trim()
                .trim_end_matches('.')
                .to_string();
            if !mp.is_empty() {
                return Ok(PathBuf::from(mp));
            }
        }
        let line = stdout.trim().trim_end_matches('.');
        if !line.is_empty() {
            return Ok(PathBuf::from(line));
        }
        return Err("Kunde inte tolka monteringspunkt från udisksctl".into());
    }

    // Do NOT fall back to pkexec — it shows a GUI password dialog which
    // freezes the kiosk UI and requires superuser interaction.
    Err(format!(
        "Kunde inte montera {} via udisksctl: {}",
        dev,
        stderr.trim()
    ))
}

/// Mountpoints that appeared between old and new snapshot.
pub fn new_mountpoints(old: &UsbSnapshot, new: &UsbSnapshot) -> Vec<PathBuf> {
    let old_set: HashSet<String> = usb_mountpoints(old)
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    usb_mountpoints(new)
        .into_iter()
        .filter(|m| !old_set.contains(&m.to_string_lossy().to_string()))
        .collect()
}

/// Unmounted partitions that appeared between old and new snapshot.
pub fn new_unmounted_partitions(old: &UsbSnapshot, new: &UsbSnapshot) -> Vec<String> {
    let old_set: HashSet<String> = unmounted_usb_partitions(old).into_iter().collect();
    unmounted_usb_partitions(new)
        .into_iter()
        .filter(|d| !old_set.contains(d))
        .collect()
}

// ---------------------------------------------------------------------------
// Background worker
// ---------------------------------------------------------------------------

/// Spawn a background thread that waits for a USB device, auto-mounts if needed,
/// imports photos, and sends the result back.
///
/// The thread exits as soon as it either successfully imports, hits an error,
/// or the receiver is dropped (user cancelled).
pub fn spawn_usb_worker(dest_dir: PathBuf) -> Receiver<UsbResult> {
    let (tx, rx) = channel();

    thread::spawn(move || {
        eprintln!("[USB-WORKER] Started");
        let initial = usb_snapshot();
        eprintln!(
            "[USB-WORKER] Initial mounts: {:?}",
            usb_mountpoints(&initial)
        );
        eprintln!(
            "[USB-WORKER] Initial unmounted: {:?}",
            unmounted_usb_partitions(&initial)
        );

        // 1. Already mounted?
        let mounts = usb_mountpoints(&initial);
        if !mounts.is_empty() {
            eprintln!(
                "[USB-WORKER] Found already-mounted device at {:?}",
                mounts[0]
            );
            match scan_and_report(&mounts[0], &tx) {
                ImportOutcome::Done => {
                    eprintln!("[USB-WORKER] Scan done, exiting");
                    return;
                }
                ImportOutcome::Retry => {
                    eprintln!("[USB-WORKER] No photos on mounted device, keeping polling");
                }
            }
        }

        // 2. Already present but unmounted?
        let unmounted = unmounted_usb_partitions(&initial);
        for part in &unmounted {
            eprintln!("[USB-WORKER] Trying to auto-mount {}", part);
            match auto_mount(part) {
                Ok(mp) => {
                    eprintln!("[USB-WORKER] Mounted {} at {:?}", part, mp);
                    thread::sleep(Duration::from_millis(500)); // let fs settle
                    match scan_and_report(&mp, &tx) {
                        ImportOutcome::Done => {
                            eprintln!("[USB-WORKER] Scan done after mount, exiting");
                            return;
                        }
                        ImportOutcome::Retry => {
                            eprintln!("[USB-WORKER] No photos after mount, keeping polling");
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[USB-WORKER] Auto-mount failed for {}: {}", part, e);
                }
            }
        }

        // 3. Poll for new plug-in events
        let mut last = initial;
        loop {
            thread::sleep(Duration::from_secs(1));

            let current = usb_snapshot();

            // New mount appeared (e.g. desktop auto-mounted it)
            let new_mounts = new_mountpoints(&last, &current);
            if !new_mounts.is_empty() {
                eprintln!("[USB-WORKER] New mount detected: {:?}", new_mounts[0]);
                match scan_and_report(&new_mounts[0], &tx) {
                    ImportOutcome::Done => {
                        eprintln!("[USB-WORKER] Scan done from new mount, exiting");
                        return;
                    }
                    ImportOutcome::Retry => {
                        eprintln!("[USB-WORKER] No photos on new mount, keeping polling");
                    }
                }
            }

            // New unmounted partition appeared — try to mount it ourselves
            let new_parts = new_unmounted_partitions(&last, &current);
            for part in &new_parts {
                eprintln!("[USB-WORKER] New unmounted partition: {}", part);
                match auto_mount(part) {
                    Ok(mp) => {
                        eprintln!("[USB-WORKER] Mounted new partition at {:?}", mp);
                        thread::sleep(Duration::from_millis(500));
                        match scan_and_report(&mp, &tx) {
                            ImportOutcome::Done => {
                                eprintln!("[USB-WORKER] Scan done after new mount, exiting");
                                return;
                            }
                            ImportOutcome::Retry => {
                                eprintln!(
                                    "[USB-WORKER] No photos after new mount, keeping polling"
                                );
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "[USB-WORKER] Auto-mount failed for new partition {}: {}",
                            part, e
                        );
                    }
                }
            }

            last = current;
        }
    });

    rx
}

enum ImportOutcome {
    Done,
    Retry,
}

fn scan_and_report(
    source: &std::path::Path,
    tx: &std::sync::mpsc::Sender<UsbResult>,
) -> ImportOutcome {
    let photos = crate::wired_import::list_phone_photos(source);
    if !photos.is_empty() {
        let _ = tx.send(UsbResult::FoundPhotos(photos));
        ImportOutcome::Done
    } else {
        ImportOutcome::Retry
    }
}

fn import_and_report(
    source: &std::path::Path,
    dest: &std::path::Path,
    tx: &std::sync::mpsc::Sender<UsbResult>,
) -> ImportOutcome {
    // Tell the UI we found the device and are about to copy
    let _ = tx.send(UsbResult::Mounted(source.to_path_buf()));

    let progress = |current: usize, total: usize, file_name: &str| {
        let _ = tx.send(UsbResult::Progress {
            current,
            total,
            file_name: file_name.to_string(),
        });
    };

    match crate::import::import_from_storage(source, dest, Some(&progress)) {
        Ok(count) => {
            let _ = tx.send(UsbResult::Imported(count));
            ImportOutcome::Done
        }
        Err(e) => {
            // If no photos found, keep polling — the user might plug in a different device.
            // Other errors are reported so the UI can show a message.
            if e.contains("Inga bilder")
                || e.contains("No photos")
                || e.contains("Inga USB-enheter")
            {
                ImportOutcome::Retry
            } else {
                let _ = tx.send(UsbResult::Error(e));
                ImportOutcome::Done
            }
        }
    }
}
