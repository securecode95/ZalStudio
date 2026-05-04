use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use mtp_rs::mtp::MtpDevice;
use mtp_rs::ptp::{ObjectFormatCode, ObjectHandle, StorageId};

const IMAGE_EXTS: &[&str] = &["jpg", "jpeg", "png", "gif", "bmp", "webp", "tiff", "tif", "heic", "heif"];

#[derive(Debug, Clone)]
pub struct MtpPhoto {
    pub handle: u32,
    pub filename: String,
    pub size: u64,
    pub storage_id: u32,
}

#[derive(Debug, Clone)]
pub enum MtpCommand {
    ListPhotos,
    Download { photos: Vec<MtpPhoto>, dest_dir: PathBuf },
}

#[derive(Debug, Clone)]
pub enum MtpResult {
    Photos(Vec<MtpPhoto>),
    Downloaded { count: usize },
    Error(String),
}

/// Spawn an MTP worker thread that talks to Android devices directly.
/// Returns a sender for commands and a receiver for results.
pub fn spawn_mtp_worker() -> (Sender<MtpCommand>, Receiver<MtpResult>) {
    let (cmd_tx, cmd_rx) = channel::<MtpCommand>();
    let (res_tx, res_rx) = channel::<MtpResult>();

    thread::spawn(move || {
        futures::executor::block_on(async {
            let mut device: Option<MtpDevice> = None;
            let mut _photos: Vec<MtpPhoto> = Vec::new();

            loop {
                let Ok(cmd) = cmd_rx.recv() else {
                    break;
                };

                match cmd {
                    MtpCommand::ListPhotos => {
                        // Try to open device if not already open
                        if device.is_none() {
                            match MtpDevice::open_first().await {
                                Ok(d) => {
                                    device = Some(d);
                                }
                                Err(e) => {
                                    #[cfg(windows)]
                                    let msg = format!(
                                        "Ingen MTP-enhet hittad. \
På Windows krävs WinUSB-drivrutin (installera med Zadig). \
Fel: {}",
                                        e
                                    );
                                    #[cfg(not(windows))]
                                    let msg = format!("Ingen MTP-enhet hittad: {}", e);
                                    let _ = res_tx.send(MtpResult::Error(msg));
                                    continue;
                                }
                            }
                        }

                        let Some(dev) = device.as_ref() else {
                            let _ = res_tx.send(MtpResult::Error(
                                "Ingen MTP-enhet ansluten".into(),
                            ));
                            continue;
                        };

                        let mut found = Vec::new();
                        let storages = match dev.storages().await {
                            Ok(s) => s,
                            Err(e) => {
                                let _ = res_tx.send(MtpResult::Error(format!(
                                    "MTP-lagring: {}",
                                    e
                                )));
                                device = None; // Reset connection
                                continue;
                            }
                        };

                        for storage in storages {
                            let sid = storage.id().0;
                            // Use recursive listing for each storage
                            match storage.list_objects_recursive(None).await {
                                Ok(objects) => {
                                    for obj in objects {
                                        if obj.is_folder() {
                                            continue;
                                        }
                                        if is_image_format(obj.format, &obj.filename) {
                                            found.push(MtpPhoto {
                                                handle: obj.handle.0,
                                                filename: obj.filename.clone(),
                                                size: obj.size,
                                                storage_id: sid,
                                            });
                                        }
                                    }
                                }
                                Err(_) => continue,
                            }
                        }

                        _photos = found.clone();
                        let _ = res_tx.send(MtpResult::Photos(found));
                    }

                    MtpCommand::Download { photos: to_download, dest_dir } => {
                        let Some(dev) = device.as_ref() else {
                            let _ = res_tx.send(MtpResult::Error(
                                "Ingen MTP-enhet ansluten".into(),
                            ));
                            continue;
                        };

                        let mut count = 0;
                        for photo in to_download {
                            let storage = match dev.storage(StorageId(photo.storage_id)).await {
                                Ok(s) => s,
                                Err(_) => continue,
                            };

                            let dest_path = dest_dir.join(&photo.filename);
                            match storage.download(ObjectHandle(photo.handle)).await {
                                Ok(data) => {
                                    if std::fs::write(&dest_path, &data).is_ok() {
                                        count += 1;
                                    }
                                }
                                Err(_) => continue,
                            }
                        }

                        let _ = res_tx.send(MtpResult::Downloaded { count });
                    }
                }
            }
        });
    });

    (cmd_tx, res_rx)
}

fn is_image_format(format: ObjectFormatCode, filename: &str) -> bool {
    match format {
        ObjectFormatCode::Jpeg
        | ObjectFormatCode::Tiff
        | ObjectFormatCode::Gif
        | ObjectFormatCode::Bmp
        | ObjectFormatCode::Png
        | ObjectFormatCode::Pict => true,
        _ => {
            let ext = filename.to_lowercase();
            IMAGE_EXTS.iter().any(|e| ext.ends_with(e))
        }
    }
}

/// Check if any MTP device is connected (synchronous, lightweight)
pub fn mtp_device_available() -> bool {
    match MtpDevice::list_devices() {
        Ok(devs) => !devs.is_empty(),
        Err(_) => false,
    }
}
