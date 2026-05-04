//! Windows MTP backend using PowerShell + MediaDevices .NET library.
//! MediaDevices talks DIRECTLY to WPD (Windows Portable Device) API —
//! the same fast API Windows 7 used, bypassing the slow Shell.Application wrapper.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

/// Path to the bundled MediaDevices.dll (must exist next to the .exe or in assets/)
const MEDIA_DEVICES_DLL: &str = r"assets\MediaDevices.dll";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PsPhoto {
    pub name: String,
    /// Format: "DeviceFriendlyName|\Internal storage\DCIM\Camera\IMG_001.jpg"
    pub path: String,
    pub size: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PsFolder {
    pub name: String,
    /// Format: "DeviceFriendlyName|\Internal storage\DCIM\Camera"
    pub full_path: String,
    pub item_count: u32,
}

#[derive(Debug, Clone)]
pub enum PsCommand {
    ListFolders,
    ListPhotos { folder_path: String },
    Download { photos: Vec<PsPhoto>, dest_dir: PathBuf },
}

#[derive(Debug, Clone)]
pub enum PsResult {
    Folders(Vec<PsFolder>),
    Photos(Vec<PsPhoto>),
    Downloaded { count: usize },
    Error(String),
}

fn debug_log(msg: &str) {
    let log_path = PathBuf::from(r"C:\temp\zalstudio_mtp_debug.log");
    if let Some(parent) = log_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let mut file = match OpenOptions::new().create(true).append(true).open(&log_path) {
        Ok(f) => f,
        Err(_) => return,
    };
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let _ = writeln!(file, "[{}] {}", timestamp, msg);
}

/// Spawn a worker that talks to the phone via PowerShell + MediaDevices (WPD).
pub fn spawn_powershell_worker() -> (Sender<PsCommand>, Receiver<PsResult>) {
    let (cmd_tx, cmd_rx) = channel::<PsCommand>();
    let (res_tx, res_rx) = channel::<PsResult>();

    thread::spawn(move || {
        debug_log("=== PowerShell WPD worker started ===");
        loop {
            let Ok(cmd) = cmd_rx.recv() else {
                debug_log("Worker: command channel closed, exiting");
                break;
            };
            debug_log(&format!("Worker received command: {:?}", cmd));

            match cmd {
                PsCommand::ListFolders => {
                    let out_file = r"C:\temp\zalstudio_mtp_folders.txt";
                    let dll_path = resolve_media_devices_dll();
                    let script = format!(
                        r#"
$ErrorActionPreference = "Stop"
Add-Type -Path "{dll}"

$imageExts = @('.jpg','.jpeg','.png','.gif','.bmp','.webp','.heic','.heif','.tiff','.tif')

# Clear old output file
if (Test-Path "{out}") {{ Remove-Item "{out}" -Force }}

$devices = [MediaDevices.MediaDevice]::GetDevices()
if (-not $devices) {{
    Add-Content -Path "{out}" -Value "ERROR|No devices found|0" -Encoding UTF8
    exit
}}

function Count-Images($device, $folderPath) {{
    $count = 0
    try {{
        $files = $device.GetFiles($folderPath)
        foreach ($f in $files) {{
            $ext = [System.IO.Path]::GetExtension($f).ToLower()
            if ($imageExts -contains $ext) {{ $count++ }}
        }}
    }} catch {{ }}
    return $count
}}

function Find-Dcim($device, $dirPath) {{
    $results = @()
    try {{
        $subDirs = $device.GetDirectories($dirPath)
        foreach ($sub in $subDirs) {{
            $name = [System.IO.Path]::GetFileName($sub)
            if ($name -eq 'DCIM') {{
                $results += $sub
            }} else {{
                $results += Find-Dcim $device $sub
            }}
        }}
    }} catch {{ }}
    return $results
}}

foreach ($dev in $devices) {{
    try {{
        $dev.Connect()
        $rootDir = $dev.GetRootDirectory().FullName
        $dcimPaths = Find-Dcim $dev $rootDir

        foreach ($dcim in $dcimPaths) {{
            try {{
                $subDirs = $dev.GetDirectories($dcim)
                foreach ($sub in $subDirs) {{
                    $subName = [System.IO.Path]::GetFileName($sub)
                    $imgCount = Count-Images $dev $sub
                    if ($imgCount -gt 0) {{
                        # Store path as "FriendlyName|\Internal storage\DCIM\Camera"
                        $storedPath = $dev.FriendlyName + "|" + $sub
                        $line = $subName + "|" + $storedPath + "|" + $imgCount
                        Add-Content -Path "{out}" -Value $line -Encoding UTF8
                    }}
                }}
            }} catch {{ }}
        }}

        $dev.Disconnect()
    }} catch {{
        $msg = $_.Exception.Message
        Add-Content -Path "{out}" -Value ("ERROR|" + $msg + "|0") -Encoding UTF8
    }}
}}
"#,
                        dll = dll_path.to_string_lossy().replace('\\', "\\\\"),
                        out = out_file
                    );

                    debug_log("Running LIST FOLDERS via WPD...");
                    let start = std::time::Instant::now();
                    if let Err(e) = run_powershell_script(&script) {
                        debug_log(&format!("LIST FOLDERS failed: {}", e));
                        let _ = res_tx.send(PsResult::Error(e));
                        continue;
                    }
                    debug_log(&format!("LIST FOLDERS finished in {:?}", start.elapsed()));

                    let output = match std::fs::read_to_string(out_file) {
                        Ok(s) => s,
                        Err(e) => {
                            debug_log(&format!("Could not read folders file: {}", e));
                            let _ = res_tx.send(PsResult::Error(
                                "Kunde inte lasa mappar fran telefon.".into(),
                            ));
                            continue;
                        }
                    };

                    let folders = match parse_folder_list(&output) {
                        Ok(f) => f,
                        Err(e) => {
                            debug_log(&format!("FOLDER parse error: {}", e));
                            let _ = res_tx.send(PsResult::Error(format!(
                                "Kunde inte tolka mappar: {}", e
                            )));
                            continue;
                        }
                    };

                    debug_log(&format!("LIST FOLDERS parsed {} folders", folders.len()));
                    if folders.is_empty() {
                        let _ = res_tx.send(PsResult::Error(
                            "Telefon hittad men ingen DCIM-mapp hittades.".into(),
                        ));
                    } else {
                        let _ = res_tx.send(PsResult::Folders(folders));
                    }
                }

                PsCommand::ListPhotos { folder_path } => {
                    let out_file = r"C:\temp\zalstudio_mtp_output.txt";
                    let dll_path = resolve_media_devices_dll();

                    // folder_path format: "FriendlyName|\Internal storage\DCIM\Camera"
                    let script = format!(
                        r#"
$ErrorActionPreference = "Stop"
Add-Type -Path "{dll}"

$imageExts = @('.jpg','.jpeg','.png','.gif','.bmp','.webp','.heic','.heif','.tiff','.tif')

# Clear old output file
if (Test-Path "{out}") {{ Remove-Item "{out}" -Force }}

# Parse the combined path: "FriendlyName|\Internal storage\DCIM\Camera"
$combinedPath = "{path}"
$pipeIndex = $combinedPath.IndexOf("|")
if ($pipeIndex -lt 0) {{
    Add-Content -Path "{out}" -Value "ERROR|Invalid path format|0" -Encoding UTF8
    exit
}}
$deviceName = $combinedPath.Substring(0, $pipeIndex)
$folderPath = $combinedPath.Substring($pipeIndex + 1)

$devices = [MediaDevices.MediaDevice]::GetDevices()
$dev = $null
foreach ($d in $devices) {{
    if ($d.FriendlyName -eq $deviceName) {{ $dev = $d; break }}
}}
if (-not $dev) {{
    Add-Content -Path "{out}" -Value "ERROR|Device not found|0" -Encoding UTF8
    exit
}}

try {{
    $dev.Connect()

    if (-not $dev.DirectoryExists($folderPath)) {{
        Add-Content -Path "{out}" -Value "ERROR|Folder not found|0" -Encoding UTF8
        $dev.Disconnect()
        exit
    }}

    $files = $dev.GetFiles($folderPath)
    $count = 0
    $max = 500
    foreach ($f in $files) {{
        if ($count -ge $max) {{ break }}
        $ext = [System.IO.Path]::GetExtension($f).ToLower()
        if ($imageExts -contains $ext) {{
            try {{
                $info = $dev.GetFileInfo($f)
                $size = $info.Length
                $name = $info.Name
                # Return path in same format: "FriendlyName|\full\path"
                $storedPath = $deviceName + "|" + $f
                $line = $name + "|" + $storedPath + "|" + $size
                Add-Content -Path "{out}" -Value $line -Encoding UTF8
                $count++
            }} catch {{ }}
        }}
    }}

    $dev.Disconnect()
}} catch {{
    $msg = $_.Exception.Message
    Add-Content -Path "{out}" -Value ("ERROR|" + $msg + "|0") -Encoding UTF8
}}
"#,
                        dll = dll_path.to_string_lossy().replace('\\', "\\\\"),
                        out = out_file,
                        path = folder_path
                    );

                    debug_log(&format!("Running LIST PHOTOS via WPD for: {}", folder_path));
                    let start = std::time::Instant::now();
                    if let Err(e) = run_powershell_script(&script) {
                        debug_log(&format!("LIST PHOTOS failed: {}", e));
                        let _ = res_tx.send(PsResult::Error(e));
                        continue;
                    }
                    debug_log(&format!("LIST PHOTOS finished in {:?}", start.elapsed()));

                    let output = match std::fs::read_to_string(out_file) {
                        Ok(s) => s,
                        Err(e) => {
                            debug_log(&format!("Could not read output file: {}", e));
                            let _ = res_tx.send(PsResult::Error(
                                "Kunde inte lasa bilder fran mapp.".into(),
                            ));
                            continue;
                        }
                    };

                    if output.trim().starts_with("ERROR|") {
                        let err = output.trim().strip_prefix("ERROR|").unwrap_or("Unknown error");
                        let _ = res_tx.send(PsResult::Error(format!("Mappfel: {}", err)));
                        continue;
                    }

                    let photos = match parse_photo_list(&output) {
                        Ok(p) => p,
                        Err(e) => {
                            debug_log(&format!("PHOTO parse error: {}", e));
                            let _ = res_tx.send(PsResult::Error(format!(
                                "Kunde inte tolka bilder: {}", e
                            )));
                            continue;
                        }
                    };

                    debug_log(&format!("LIST PHOTOS parsed {} photos", photos.len()));
                    if photos.is_empty() {
                        let _ = res_tx.send(PsResult::Error(
                            "Mappen hittades men inneholl inga bilder.".into(),
                        ));
                    } else {
                        let _ = res_tx.send(PsResult::Photos(photos));
                    }
                }

                PsCommand::Download { photos: to_download, dest_dir } => {
                    let dest_str = dest_dir.to_string_lossy();
                    let files_json = match serde_json::to_string(&to_download) {
                        Ok(j) => j,
                        Err(e) => {
                            let _ = res_tx.send(PsResult::Error(format!(
                                "Kunde inte forbereda kopiering: {}", e
                            )));
                            continue;
                        }
                    };

                    debug_log(&format!("DOWNLOAD: {} files to {}", to_download.len(), dest_str));
                    let dll_path = resolve_media_devices_dll();

                    let script = format!(
                        r#"
$ErrorActionPreference = "Stop"
Add-Type -Path "{dll}"

$destPath = "{dest}"
$filesJson = @'
{files}
'@

[System.IO.Directory]::CreateDirectory($destPath) | Out-Null

$count = 0
$photos = $filesJson | ConvertFrom-Json

# Group photos by device name so we connect once per device
$byDevice = @{{}}
foreach ($photo in $photos) {{
    $combinedPath = $photo.path
    $pipeIndex = $combinedPath.IndexOf("|")
    if ($pipeIndex -lt 0) {{ continue }}
    $deviceName = $combinedPath.Substring(0, $pipeIndex)
    $filePath = $combinedPath.Substring($pipeIndex + 1)
    if (-not $byDevice.ContainsKey($deviceName)) {{
        $byDevice[$deviceName] = @()
    }}
    $byDevice[$deviceName] += @{{ Name = $photo.name; Path = $filePath }}
}}

$devices = [MediaDevices.MediaDevice]::GetDevices()

foreach ($deviceName in $byDevice.Keys) {{
    $dev = $null
    foreach ($d in $devices) {{
        if ($d.FriendlyName -eq $deviceName) {{ $dev = $d; break }}
    }}
    if (-not $dev) {{ continue }}

    try {{
        $dev.Connect()
        foreach ($file in $byDevice[$deviceName]) {{
            try {{
                $src = $file.Path
                $destFile = Join-Path $destPath $file.Name
                $stream = [System.IO.File]::OpenWrite($destFile)
                try {{
                    $dev.DownloadFile($src, $stream)
                    $count++
                }} finally {{
                    $stream.Dispose()
                }}
            }} catch {{
                # If we failed to write, try removing partial file
                try {{ Remove-Item (Join-Path $destPath $file.Name) -Force -ErrorAction SilentlyContinue }} catch {{ }}
            }}
        }}
        $dev.Disconnect()
    }} catch {{ }}
}}

Write-Output "COPIED:$count"
"#,
                        dll = dll_path.to_string_lossy().replace('\\', "\\\\"),
                        dest = dest_str.replace('"', "\"\""),
                        files = files_json
                    );

                    let output = match run_powershell_script(&script) {
                        Ok(o) => o,
                        Err(e) => {
                            debug_log(&format!("DOWNLOAD failed: {}", e));
                            let _ = res_tx.send(PsResult::Error(e));
                            continue;
                        }
                    };

                    let count = output
                        .trim()
                        .strip_prefix("COPIED:")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);
                    debug_log(&format!("DOWNLOAD copied {} files", count));
                    let _ = res_tx.send(PsResult::Downloaded { count });
                }
            }
        }
    });

    (cmd_tx, res_rx)
}

/// Try to find MediaDevices.dll next to the executable, then in assets/, then fallback.
fn resolve_media_devices_dll() -> PathBuf {
    // 1. Try next to the executable
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let next_to_exe = exe_dir.join("MediaDevices.dll");
            if next_to_exe.exists() {
                return next_to_exe;
            }
        }
    }
    // 2. Try assets/ relative to working directory
    let assets_path = PathBuf::from(MEDIA_DEVICES_DLL);
    if assets_path.exists() {
        return assets_path;
    }
    // 3. Fallback to just the filename and hope PowerShell finds it
    PathBuf::from("MediaDevices.dll")
}

fn run_powershell_script(script: &str) -> Result<String, String> {
    let temp_file = std::env::temp_dir().join("zalstudio_mtp_script.ps1");
    if let Err(e) = std::fs::write(&temp_file, script) {
        return Err(format!("Kunde inte skapa temporar skriptfil: {}", e));
    }

    let temp_path = temp_file.to_string_lossy();
    debug_log(&format!("Running PowerShell script: {}", temp_path));

    let output = Command::new("powershell.exe")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", &temp_path])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("PowerShell kunde inte startas: {}", e))?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.trim().is_empty() {
        debug_log(&format!("PowerShell stderr: {}", stderr.trim()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn parse_photo_list(output: &str) -> Result<Vec<PsPhoto>, String> {
    let mut photos = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 3 && parts[0] != "ERROR" {
            let size = parts[2].parse::<u64>().unwrap_or(0);
            photos.push(PsPhoto {
                name: parts[0].to_string(),
                path: parts[1].to_string(),
                size,
            });
        }
    }
    Ok(photos)
}

fn parse_folder_list(output: &str) -> Result<Vec<PsFolder>, String> {
    let mut folders = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 3 && parts[0] != "ERROR" {
            let count = parts[2].parse::<u32>().unwrap_or(0);
            folders.push(PsFolder {
                name: parts[0].to_string(),
                full_path: parts[1].to_string(),
                item_count: count,
            });
        }
    }
    Ok(folders)
}
