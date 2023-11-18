use alloc::vec::Vec;

/// Information about a custom TTF-Font
///
/// * `face` - any `.ttf` file in bytes, preferably monospaced.
/// * `font_size` - comparable to text size in most text editors
/// * `char_width` — how much pixels a char should be allowed to take up horizontally
/// * `char_width` — how much pixels a char should be allowed to take up vertically
/// # Example
/// ```
/// let font_config = (include_bytes!("assets/VeraMono-Bold.ttf").as_slice(), 16_usize, 6_usize, 11_usize);
/// ```
pub struct ViewerConfig {
    pub face: &'static [u8],
    pub font_size: isize,
    pub char_width: usize,
    pub char_height: usize,
    pub white_mode: bool,
}

pub struct ScreenInfo {
    pub char_width: usize,
    pub char_height: usize,
    pub max_cols: usize,
    pub max_lines: usize,
}

#[derive(Clone)]
pub struct CharData {
    pub bm_buffer: Vec<u8>,
    pub bm_left: usize,
    pub bm_top: usize,
    pub bm_w: usize,
    pub bm_h: usize,
}

pub enum Direction {
    Up,
    Down,
}
