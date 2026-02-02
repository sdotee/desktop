use crate::error::{AppError, Result};
use cairo::{Context, PdfSurface};
use glib::prelude::*;
use glib::Bytes;
use gtk::gdk;
use image::{ImageBuffer, Rgba};
use qrcode::render::svg;
use qrcode::QrCode;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub struct QrGenerator;

impl QrGenerator {
    /// Generate a QR code and return it as a GDK Texture for display in GTK
    pub fn generate_texture(content: &str, size: u32) -> Result<gdk::Texture> {
        let code = QrCode::new(content.as_bytes())
            .map_err(|e| AppError::QrCode(format!("Failed to create QR code: {}", e)))?;

        let image = code
            .render::<image::Luma<u8>>()
            .min_dimensions(size, size)
            .build();

        // Convert to RGBA
        let width = image.width();
        let height = image.height();
        let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);

        for pixel in image.pixels() {
            let val = pixel.0[0];
            // RGBA format
            rgba_data.push(val); // R
            rgba_data.push(val); // G
            rgba_data.push(val); // B
            rgba_data.push(255); // A
        }

        let bytes = Bytes::from(&rgba_data);
        let texture = gdk::MemoryTexture::new(
            width as i32,
            height as i32,
            gdk::MemoryFormat::R8g8b8a8,
            &bytes,
            (width * 4) as usize,
        );

        Ok(texture.upcast())
    }

    /// Save QR code as PNG file
    pub fn save_png(content: &str, path: &Path, size: u32) -> Result<()> {
        let code = QrCode::new(content.as_bytes())
            .map_err(|e| AppError::QrCode(format!("Failed to create QR code: {}", e)))?;

        let image = code
            .render::<image::Luma<u8>>()
            .min_dimensions(size, size)
            .build();

        // Convert to RGBA
        let rgba_image: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::from_fn(image.width(), image.height(), |x, y| {
                let pixel = image.get_pixel(x, y);
                if pixel.0[0] == 0 {
                    Rgba([0, 0, 0, 255])
                } else {
                    Rgba([255, 255, 255, 255])
                }
            });

        rgba_image.save(path)?;
        Ok(())
    }

    /// Save QR code as SVG file
    pub fn save_svg(content: &str, path: &Path) -> Result<()> {
        let code = QrCode::new(content.as_bytes())
            .map_err(|e| AppError::QrCode(format!("Failed to create QR code: {}", e)))?;

        let svg_string = code
            .render()
            .min_dimensions(256, 256)
            .dark_color(svg::Color("#000000"))
            .light_color(svg::Color("#ffffff"))
            .build();

        let mut file = File::create(path)?;
        file.write_all(svg_string.as_bytes())?;
        Ok(())
    }

    /// Save QR code as PDF file
    pub fn save_pdf(content: &str, path: &Path, size: f64) -> Result<()> {
        let code = QrCode::new(content.as_bytes())
            .map_err(|e| AppError::QrCode(format!("Failed to create QR code: {}", e)))?;

        let image = code
            .render::<image::Luma<u8>>()
            .min_dimensions(256, 256)
            .build();

        let margin = 20.0;
        let page_size = size + margin * 2.0;

        let surface = PdfSurface::new(page_size, page_size, path)
            .map_err(|e| AppError::QrCode(format!("Failed to create PDF surface: {}", e)))?;

        let ctx = Context::new(&surface)
            .map_err(|e| AppError::QrCode(format!("Failed to create context: {}", e)))?;

        // Fill with white background
        ctx.set_source_rgb(1.0, 1.0, 1.0);
        ctx.paint()
            .map_err(|e| AppError::QrCode(format!("Failed to paint: {}", e)))?;

        // Calculate scale to fit QR code in desired size
        let scale = size / image.width() as f64;

        ctx.translate(margin, margin);
        ctx.scale(scale, scale);

        // Draw QR code pixels
        ctx.set_source_rgb(0.0, 0.0, 0.0);
        for (x, y, pixel) in image.enumerate_pixels() {
            if pixel.0[0] == 0 {
                // Black pixel
                ctx.rectangle(x as f64, y as f64, 1.0, 1.0);
            }
        }
        ctx.fill()
            .map_err(|e| AppError::QrCode(format!("Failed to fill: {}", e)))?;

        // Finish the surface to flush all pending drawing operations
        surface.finish();

        Ok(())
    }
}
