mod constants;

use core::{
    fmt::{self, Write},
    ptr,
};

use bootloader_api::info::{FrameBufferInfo, PixelFormat};
use constants::font_constants;
use constants::font_constants::{BACKUP_CHAR, CHAR_RASTER_HEIGHT, FONT_WEIGHT};
use noto_sans_mono_bitmap::{get_raster, RasterizedChar};

/// Additional vertical space between lines
const LINE_SPACING: usize = 2;

/// Additional horizontal space between characters.
const LETTER_SPACING: usize = 0;

/// Padding from the border. Prevent that font is too close to border.
const BORDER_PADDING: usize = 1;

/// Returns the raster of the given char or the raster of [`font_constants::BACKUP_CHAR`].
fn get_char_raster(c: char) -> RasterizedChar {
    fn get(c: char) -> Option<RasterizedChar> {
        get_raster(c, FONT_WEIGHT, CHAR_RASTER_HEIGHT)
    }
    get(c).unwrap_or_else(|| get(BACKUP_CHAR).expect("Should get raster of backup char."))
}



/// Allows logging text to a pixel-based framebuffer.
#[derive(Debug)]
pub struct FrameBufferWriter<'a> {
    framebuffer: &'a mut [u8],
    info: FrameBufferInfo,
    x_pos: usize,
    y_pos: usize,
}

impl<'a> FrameBufferWriter<'a> {
    /// Creates a new logger that uses the given framebuffer.
    pub fn new(framebuffer: &'a mut [u8], info: FrameBufferInfo) -> Self {
        let mut logger = Self {
            framebuffer,
            info,
            x_pos: 0,
            y_pos: 0,
        };
        logger.clear();
        logger
    }

    fn newline(&mut self) {
        self.y_pos += font_constants::CHAR_RASTER_HEIGHT.val() + LINE_SPACING;
        self.carriage_return()
    }

    fn carriage_return(&mut self) {
        self.x_pos = BORDER_PADDING;
    }

    /// Erases all text on the screen. Resets `self.x_pos` and `self.y_pos`.
    pub fn clear(&mut self) {
        self.x_pos = BORDER_PADDING;
        self.y_pos = BORDER_PADDING;
        self.framebuffer.fill(0);
    }

    fn width(&self) -> usize {
        self.info.width
    }

    fn height(&self) -> usize {
        self.info.height
    }

    /// Sets the write position to the specified row and column.
    pub fn set_cursor(&mut self, row: usize, column: usize) {
        let max_row = self.height() / (font_constants::CHAR_RASTER_HEIGHT.val() + LINE_SPACING);
        let max_column = self.width() / (font_constants::CHAR_RASTER_WIDTH + LETTER_SPACING);
        self.y_pos = row * (font_constants::CHAR_RASTER_HEIGHT.val() + LINE_SPACING);
        self.x_pos = column * (font_constants::CHAR_RASTER_WIDTH + LETTER_SPACING);
        if self.y_pos >= self.height() {
            self.y_pos = (max_row - 1) * (font_constants::CHAR_RASTER_HEIGHT.val() + LINE_SPACING);
            self.clear();
        }
        if self.x_pos >= self.width() {
            self.x_pos = (max_column - 1) * (font_constants::CHAR_RASTER_WIDTH + LETTER_SPACING);
            self.newline();
        }
    }

    /// Writes a single char to the framebuffer. Takes care of special control characters, such as
    /// newlines and carriage returns.
    fn write_char(&mut self, c: char) {
        match c {
            '\n' => self.newline(),
            '\r' => self.carriage_return(),
            c => {
                let new_xpos = self.x_pos + font_constants::CHAR_RASTER_WIDTH;
                if new_xpos >= self.width() {
                    self.newline();
                }
                let new_ypos =
                    self.y_pos + font_constants::CHAR_RASTER_HEIGHT.val() + BORDER_PADDING;
                if new_ypos >= self.height() {
                    self.clear();
                }
                self.write_rendered_char(get_char_raster(c));
            }
        }
    }

    /// Prints a rendered char into the framebuffer.
    /// Updates `self.x_pos`.
    fn write_rendered_char(&mut self, rendered_char: RasterizedChar) {
        for (y, row) in rendered_char.raster().iter().enumerate() {
            for (x, byte) in row.iter().enumerate() {
                self.write_pixel(self.x_pos + x, self.y_pos + y, *byte);
            }
        }
        self.x_pos += rendered_char.width() + LETTER_SPACING;
    }

    fn write_pixel(&mut self, x: usize, y: usize, intensity: u8) {
        let pixel_offset = y * self.info.stride + x;
        let color = match self.info.pixel_format {
            PixelFormat::Rgb => [intensity, intensity, intensity / 2, 0],
            PixelFormat::Bgr => [intensity / 2, intensity, intensity, 0],
            PixelFormat::U8 => [if intensity > 200 { 0xf } else { 0 }, 0, 0, 0],
            other => {
                // set a supported (but invalid) pixel format before panicking to avoid a double
                // panic; it might not be readable though
                self.info.pixel_format = PixelFormat::Rgb;
                panic!("pixel format {:?} not supported in logger", other)
            }
        };
        let bytes_per_pixel = self.info.bytes_per_pixel;
        let byte_offset = pixel_offset * bytes_per_pixel;
        self.framebuffer[byte_offset..(byte_offset + bytes_per_pixel)]
            .copy_from_slice(&color[..bytes_per_pixel]);
        let _ = unsafe { ptr::read_volatile(&self.framebuffer[byte_offset]) };
    }
    pub fn backspace(&mut self) {
        if self.x_pos >= (BORDER_PADDING + font_constants::CHAR_RASTER_WIDTH) {
            self.x_pos -= font_constants::CHAR_RASTER_WIDTH + LETTER_SPACING;
            for y in self.y_pos..(self.y_pos + font_constants::CHAR_RASTER_HEIGHT.val()) {
                for x in (self.x_pos..(self.x_pos + font_constants::CHAR_RASTER_WIDTH)).rev() {
                    self.write_pixel(x, y, 0);
                }
            }
        }
    }
}


// Traits.
unsafe impl<'a> Send for FrameBufferWriter<'a> {}
unsafe impl<'a> Sync for FrameBufferWriter<'a> {}

impl<'a> fmt::Write for FrameBufferWriter<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }
        Ok(())
    }
}
