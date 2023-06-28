use noto_sans_mono_bitmap::{get_raster_width, FontWeight, RasterHeight};

/// Constants for the usage of the [`noto_sans_mono_bitmap`] crate.
pub mod font_constants {
    use super::*;

    /// Height of each char raster. The font size is ~0.84% of this. Thus, this is the line height that
    /// enables multiple characters to be side-by-side and appear optically in one line in a natural way.
    pub const CHAR_RASTER_HEIGHT: RasterHeight = RasterHeight::Size16;

    /// The width of each single symbol of the mono space font.
    pub const CHAR_RASTER_WIDTH: usize = get_raster_width(FontWeight::Regular, CHAR_RASTER_HEIGHT);

    /// Backup character if a desired symbol is not available by the font.
    /// The '�' character requires the feature "unicode-specials".
    pub const BACKUP_CHAR: char = '�';

    pub const FONT_WEIGHT: FontWeight = FontWeight::Regular;
}
