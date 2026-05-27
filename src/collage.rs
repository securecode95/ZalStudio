use image::{DynamicImage, RgbaImage, imageops};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollageLayout {
    Split2H, // 2 side by side
    Split2V, // 2 stacked vertically
    Grid4,   // 2x2 grid
    Grid3,   // 1 large left, 2 small right
}

impl CollageLayout {
    pub fn name(self) -> &'static str {
        match self {
            CollageLayout::Split2H => "2 bilder sida vid sida",
            CollageLayout::Split2V => "2 bilder ovanpå varandra",
            CollageLayout::Grid4 => "4 bilder rutnät",
            CollageLayout::Grid3 => "3 bilder mix",
        }
    }

    pub fn photo_count(self) -> usize {
        match self {
            CollageLayout::Split2H | CollageLayout::Split2V => 2,
            CollageLayout::Grid3 => 3,
            CollageLayout::Grid4 => 4,
        }
    }

    /// Return normalized slot rectangles (0-1 range) for preview drawing.
    pub fn slot_rects(self) -> Vec<(f32, f32, f32, f32)> {
        let m = 0.03; // 3% margin
        match self {
            CollageLayout::Split2H => {
                let w = (1.0 - m * 3.0) / 2.0;
                let h = 1.0 - m * 2.0;
                vec![(m, m, w, h), (m * 2.0 + w, m, w, h)]
            }
            CollageLayout::Split2V => {
                let w = 1.0 - m * 2.0;
                let h = (1.0 - m * 3.0) / 2.0;
                vec![(m, m, w, h), (m, m * 2.0 + h, w, h)]
            }
            CollageLayout::Grid4 => {
                let w = (1.0 - m * 3.0) / 2.0;
                let h = (1.0 - m * 3.0) / 2.0;
                vec![
                    (m, m, w, h),
                    (m * 2.0 + w, m, w, h),
                    (m, m * 2.0 + h, w, h),
                    (m * 2.0 + w, m * 2.0 + h, w, h),
                ]
            }
            CollageLayout::Grid3 => {
                let left_w = (1.0 - m * 3.0) * 2.0 / 3.0;
                let right_w = (1.0 - m * 3.0) / 3.0;
                let h = (1.0 - m * 3.0) / 2.0;
                vec![
                    (m, m, left_w, 1.0 - m * 2.0),
                    (m * 2.0 + left_w, m, right_w, h),
                    (m * 2.0 + left_w, m * 2.0 + h, right_w, h),
                ]
            }
        }
    }
}

/// All available layouts.
pub fn all_layouts() -> Vec<CollageLayout> {
    vec![
        CollageLayout::Split2H,
        CollageLayout::Split2V,
        CollageLayout::Grid3,
        CollageLayout::Grid4,
    ]
}

struct Slot {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

fn paper_size_to_pixels(size: &str, dpi: u32) -> (u32, u32) {
    let parts: Vec<&str> = size.split('x').collect();
    if parts.len() == 2 {
        let w_cm: f32 = parts[0].parse().unwrap_or(10.0);
        let h_cm: f32 = parts[1].parse().unwrap_or(15.0);
        let w = (w_cm / 2.54 * dpi as f32) as u32;
        let h = (h_cm / 2.54 * dpi as f32) as u32;
        return (w.max(1), h.max(1));
    }
    (1181, 1772) // default ~10x15 @ 300dpi
}

fn layout_pixel_slots(layout: CollageLayout, canvas_w: u32, canvas_h: u32) -> Vec<Slot> {
    let m = (canvas_w.min(canvas_h) as f32 * 0.03) as u32;
    match layout {
        CollageLayout::Split2H => {
            let slot_w = (canvas_w - m * 3) / 2;
            let slot_h = canvas_h - m * 2;
            vec![
                Slot {
                    x: m,
                    y: m,
                    w: slot_w,
                    h: slot_h,
                },
                Slot {
                    x: m * 2 + slot_w,
                    y: m,
                    w: slot_w,
                    h: slot_h,
                },
            ]
        }
        CollageLayout::Split2V => {
            let slot_w = canvas_w - m * 2;
            let slot_h = (canvas_h - m * 3) / 2;
            vec![
                Slot {
                    x: m,
                    y: m,
                    w: slot_w,
                    h: slot_h,
                },
                Slot {
                    x: m,
                    y: m * 2 + slot_h,
                    w: slot_w,
                    h: slot_h,
                },
            ]
        }
        CollageLayout::Grid4 => {
            let slot_w = (canvas_w - m * 3) / 2;
            let slot_h = (canvas_h - m * 3) / 2;
            vec![
                Slot {
                    x: m,
                    y: m,
                    w: slot_w,
                    h: slot_h,
                },
                Slot {
                    x: m * 2 + slot_w,
                    y: m,
                    w: slot_w,
                    h: slot_h,
                },
                Slot {
                    x: m,
                    y: m * 2 + slot_h,
                    w: slot_w,
                    h: slot_h,
                },
                Slot {
                    x: m * 2 + slot_w,
                    y: m * 2 + slot_h,
                    w: slot_w,
                    h: slot_h,
                },
            ]
        }
        CollageLayout::Grid3 => {
            let left_w = (canvas_w - m * 3) * 2 / 3;
            let right_w = (canvas_w - m * 3) / 3;
            let slot_h = (canvas_h - m * 3) / 2;
            vec![
                Slot {
                    x: m,
                    y: m,
                    w: left_w,
                    h: canvas_h - m * 2,
                },
                Slot {
                    x: m * 2 + left_w,
                    y: m,
                    w: right_w,
                    h: slot_h,
                },
                Slot {
                    x: m * 2 + left_w,
                    y: m * 2 + slot_h,
                    w: right_w,
                    h: slot_h,
                },
            ]
        }
    }
}

/// Render a collage to a JPEG file.
pub fn render_collage(
    layout: CollageLayout,
    photo_paths: &[&Path],
    output_path: &Path,
    paper_size: &str,
) -> Result<(), String> {
    let (canvas_w, canvas_h) = paper_size_to_pixels(paper_size, 300);
    let mut canvas = RgbaImage::new(canvas_w, canvas_h);

    // Fill white
    for pixel in canvas.pixels_mut() {
        *pixel = image::Rgba([255, 255, 255, 255]);
    }

    let slots = layout_pixel_slots(layout, canvas_w, canvas_h);

    for (i, path) in photo_paths.iter().enumerate() {
        if let Some(slot) = slots.get(i) {
            match image::open(path) {
                Ok(img) => {
                    let resized =
                        imageops::resize(&img, slot.w, slot.h, imageops::FilterType::Lanczos3);
                    imageops::overlay(&mut canvas, &resized, slot.x as i64, slot.y as i64);
                }
                Err(e) => eprintln!("[collage] Failed to open {}: {}", path.display(), e),
            }
        }
    }

    canvas
        .save(output_path)
        .map_err(|e| format!("Failed to save collage: {}", e))?;
    Ok(())
}
