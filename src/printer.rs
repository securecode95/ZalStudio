use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::config::PrinterOptions;

#[derive(Debug, Clone)]
pub struct PrintJob {
    pub id: usize,
    pub photo_path: String,
    pub media_size: String,
    pub copies: u32,
    pub status: JobStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum JobStatus {
    Queued,
    Printing,
    Done,
    Failed(String),
}

#[derive(Clone)]
pub struct Printer {
    name: String,
    jobs: Arc<Mutex<Vec<PrintJob>>>,
    counter: Arc<Mutex<usize>>,
}

impl Printer {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            jobs: Arc::new(Mutex::new(Vec::new())),
            counter: Arc::new(Mutex::new(0)),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn queue_job(
        &mut self,
        photo_path: &Path,
        media_size: &str,
        copies: u32,
        options: PrinterOptions,
    ) -> usize {
        let mut counter = self.counter.lock().unwrap();
        *counter += 1;
        let id = *counter;
        drop(counter);

        let job = PrintJob {
            id,
            photo_path: photo_path.to_string_lossy().to_string(),
            media_size: media_size.to_string(),
            copies,
            status: JobStatus::Queued,
        };

        {
            let mut jobs = self.jobs.lock().unwrap();
            jobs.push(job.clone());
        }

        let name = self.name.clone();
        let jobs = self.jobs.clone();
        thread::spawn(move || {
            {
                let mut jobs = jobs.lock().unwrap();
                if let Some(j) = jobs.iter_mut().find(|j| j.id == id) {
                    j.status = JobStatus::Printing;
                }
            }

            let result = Self::print_via_lp(
                &name,
                &job.photo_path,
                &job.media_size,
                job.copies,
                &options,
            );

            {
                let mut jobs = jobs.lock().unwrap();
                if let Some(j) = jobs.iter_mut().find(|j| j.id == id) {
                    j.status = match result {
                        Ok(_) => JobStatus::Done,
                        Err(e) => JobStatus::Failed(e),
                    };
                }
            }
        });

        id
    }

    #[allow(dead_code)]
    pub fn jobs(&self) -> Vec<PrintJob> {
        self.jobs.lock().unwrap().clone()
    }

    #[allow(dead_code)]
    pub fn clear_completed(&self) {
        let mut jobs = self.jobs.lock().unwrap();
        jobs.retain(|j| !matches!(j.status, JobStatus::Done | JobStatus::Failed(_)));
    }

    fn print_via_lp(
        printer_name: &str,
        image_path: &str,
        media_size: &str,
        copies: u32,
        opts: &PrinterOptions,
    ) -> Result<(), String> {
        let mut cmd = Command::new("lp");
        cmd.arg("-d").arg(printer_name)
            .arg("-o").arg(format!("media={}", media_size))
            .arg("-o").arg("StpColorPrecision=Best")
            .arg("-o").arg("StpPrintSpeed=SuperFine")
            .arg("-o").arg("StpImageType=Photo")
            .arg("-o").arg("ColorModel=RGB")
            .arg("-o").arg(format!("StpColorCorrection={}", opts.color_correction))
            .arg("-o").arg(format!("StpBrightness={}", opts.brightness))
            .arg("-o").arg(format!("StpContrast={}", opts.contrast))
            .arg("-o").arg(format!("StpSaturation={}", opts.saturation))
            .arg("-o").arg(format!("StpGamma={}", opts.gamma))
            .arg("-o").arg(format!("StpCyanGamma={}", opts.cyan_gamma))
            .arg("-o").arg(format!("StpMagentaGamma={}", opts.magenta_gamma))
            .arg("-o").arg(format!("StpYellowGamma={}", opts.yellow_gamma))
            .arg("-o").arg(format!("StpCyanBalance={}", opts.cyan_balance))
            .arg("-o").arg(format!("StpMagentaBalance={}", opts.magenta_balance))
            .arg("-o").arg(format!("StpYellowBalance={}", opts.yellow_balance))
            .arg("-n").arg(copies.to_string());
        cmd.arg(image_path);

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to execute lp: {}", e))?;

        if !output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "lp failed (exit {}): stdout='{}' stderr='{}'",
                output.status.code().unwrap_or(-1),
                stdout.trim(),
                stderr.trim()
            ));
        }

        Ok(())
    }
}
