#![no_std]

pub mod types;
use crate::types::{CharData, Direction, ScreenInfo, ViewerConfig};

extern crate alloc;
extern crate ndless_handler;

use alloc::collections::BTreeMap;
use ndless::prelude::*;

use ndless::input::{any_key_pressed, iter_keys, wait_key_pressed, Key};
#[cfg(feature = "benchmark")]
use ndless::timer::{get_ticks, TICKS_PER_MILLISECOND};
use ndless_freetype::Face;
use ndless_sdl::Rect;

use ndless_sdl::video::{Color, Surface};
use unicode_segmentation::UnicodeSegmentation;

/// Display the text in full screen, respecting line wrap and supporting scrolling.
/// Screen can not be modified after creation, and blocks the executor until enter/esc is pressed.
/// * `text` - any input text. Manual linebreaks will be respected, and overfull lines will be broken up automatically.
///          - If this results in more lines than fit the screen at the given font size, scrolling will be enabled.
/// * `config` - (optional) custom font setup.
///     - `face` - any `.ttf` file in bytes, preferably monospaced.
///     - `font_size` - comparable to text size in most text editors
///     - `char_width` — how much pixels a char should be allowed to take up horizontally
///     - `char_width` — how much pixels a char should be allowed to take up vertically
/// # Example 1 - default font
/// ```
/// ndless_text_viewer::display("Hello, world!", None);
/// ```
/// # Example 2 - custom font
/// ```
/// use ndless_text_viewer::types::ViewerConfig;
///
/// let font_config = Some(ViewerConfig {
///     face: include_bytes!("assets/font.ttf").as_slice(),
///     font_size: 16_isize,
///     char_width: 6_usize,
///     char_height: 11_usize,
///     white_mode: true,
/// });
/// ndless_text_viewer::display("Hello, world!", font_config);
/// ```
pub fn display(text: &str, config: Option<ViewerConfig>) {
    let (face, font_size, char_width, char_height) = match config {
        Some(config) => (
            config.face,
            config.font_size,
            config.char_width,
            config.char_height,
        ),
        None => (
            include_bytes!("assets/VeraMono-Bold.ttf").as_slice(),
            16_isize,
            6_usize,
            11_usize,
        ),
    };

    // screen setup
    let screen = ndless_sdl::init_default().unwrap();
    let lib = ndless_freetype::Library::init().unwrap();

    let face = lib.new_static_face(face, 0).unwrap();
    face.set_char_size(font_size * 64, 0, 50, 0).unwrap();

    // save every rendered bitmap to buffer
    // using too many characters might exhaust RAM?
    let mut char_cache: BTreeMap<usize, CharData> = BTreeMap::new();
    // let mut char_cache: Vec<(usize, CharData)> = Vec::new();

    let screen_info = ScreenInfo {
        char_width,
        char_height,
        max_cols: 320 / char_width,
        max_lines: 240 / char_height,
    };

    let input_lines = split_and_wrap_lines(text, screen_info.max_cols);

    // clear_screen(&screen);
    screen.clear();

    let mut lines_scrolled_down = 0;

    #[cfg(feature = "benchmark")]
    let mut ticks_taken = 0;
    #[cfg(feature = "benchmark")]
    let mut rendered_lines = 0;

    render_text(
        &screen,
        &screen_info,
        &face,
        &mut char_cache,
        &input_lines,
        lines_scrolled_down,
        None,
        #[cfg(feature = "benchmark")]
        &mut ticks_taken,
    );

    loop {
        if !any_key_pressed() {
            wait_key_pressed();
        }
        #[cfg(feature = "benchmark")]
        match iter_keys().next() {
            Some(Key::Esc) | Some(Key::Enter) | Some(Key::Key5) => break,
            Some(Key::Down) | Some(Key::Key2) => {
                if input_lines.len() - lines_scrolled_down > screen_info.max_lines {
                    rendered_lines += 1;
                    lines_scrolled_down += 1;
                    shift_fb(&screen, screen_info.char_height, Direction::Up);
                    render_text(
                        &screen,
                        &screen_info,
                        &face,
                        &mut char_cache,
                        &input_lines,
                        lines_scrolled_down,
                        Some(screen_info.max_lines - 1),
                        &mut ticks_taken,
                    );
                }
            }
            Some(Key::Up) | Some(Key::Key8) => {
                if lines_scrolled_down != 0 {
                    rendered_lines += 1;
                    lines_scrolled_down -= 1;
                    shift_fb(&screen, screen_info.char_height, Direction::Down);
                    render_text(
                        &screen,
                        &screen_info,
                        &face,
                        &mut char_cache,
                        &input_lines,
                        lines_scrolled_down,
                        Some(0),
                        &mut ticks_taken,
                    );
                }
            }
            Some(Key::Space) => {
                screen.draw_str(
                    &ndless_sdl::nsdl::Font::new(ndless_sdl::nsdl::FontOptions::VGA, 255, 0, 0),
                    &format!(
                        "avg: {:.2}ms",
                        (ticks_taken as f64 / rendered_lines as f64) / TICKS_PER_MILLISECOND as f64
                    ),
                    0,
                    0,
                );
                screen.flip();
            }
            _ => {}
        }
        #[cfg(not(feature = "benchmark"))]
        match iter_keys().next() {
            Some(Key::Esc) | Some(Key::Enter) | Some(Key::Key5) => break,
            Some(Key::Down) | Some(Key::Key2) => {
                if input_lines.len() - lines_scrolled_down > screen_info.max_lines {
                    lines_scrolled_down += 1;
                    shift_fb(&screen, screen_info.char_height, Direction::Up);
                    render_text(
                        &screen,
                        &screen_info,
                        &face,
                        &mut char_cache,
                        &input_lines,
                        lines_scrolled_down,
                        Some(screen_info.max_lines - 1),
                    );
                }
            }
            Some(Key::Up) | Some(Key::Key8) => {
                if lines_scrolled_down != 0 {
                    lines_scrolled_down -= 1;
                    shift_fb(&screen, screen_info.char_height, Direction::Down);
                    render_text(
                        &screen,
                        &screen_info,
                        &face,
                        &mut char_cache,
                        &input_lines,
                        lines_scrolled_down,
                        Some(0),
                    );
                }
            }
            _ => {}
        }
    }
    ndless_sdl::quit();
}

fn render_text(
    screen: &Surface,
    screen_info: &ScreenInfo,
    face: &Face,
    // char_cache: &mut Vec<(usize, CharData)>,
    char_cache: &mut BTreeMap<usize, CharData>,
    input_lines: &[(String, bool)],
    lines_scrolled_down: usize,
    only_draw_line: Option<usize>,
    #[cfg(feature = "benchmark")] ticks_taken: &mut u32,
) {
    #[cfg(feature = "benchmark")]
    let start_ticks = get_ticks();

    let (input_line_range, mut draw_y) = match only_draw_line {
        Some(idx) => (
            lines_scrolled_down + idx..lines_scrolled_down + idx + 1,
            screen_info.char_height * (idx + 1),
        ),
        None => (
            0..match input_lines.len() - lines_scrolled_down {
                u if u < screen_info.max_lines => u,
                _ => screen_info.max_lines,
            },
            screen_info.char_height,
        ),
    };

    let mut draw_x: usize;

    for line in &input_lines[input_line_range] {
        draw_x = 0;

        for c in line.0.chars() {
            let c = c as usize;

            let char_data_temp: CharData;

            let char_data = match char_cache.get(&c) {
                Some(data) => data,
                None => {
                    face.load_char(c, ndless_freetype::face::LoadFlag::RENDER)
                        .unwrap();

                    let glyph = face.glyph();
                    let bm = glyph.bitmap();

                    char_data_temp = CharData {
                        bm_buffer: bm.buffer().to_vec(),
                        bm_left: glyph.bitmap_left() as usize,
                        bm_top: glyph.bitmap_top() as usize,
                        bm_w: bm.width() as usize,
                        bm_h: bm.rows() as usize,
                    };

                    char_cache.insert(c, char_data_temp.clone());

                    &char_data_temp
                }
            };

            let x = draw_x + char_data.bm_left;
            let y = draw_y - char_data.bm_top;
            let x_max = x + char_data.bm_w;
            let y_max = y + char_data.bm_h;

            for (row, x_scaled) in (x..x_max).enumerate() {
                for (col, y_scaled) in (y..y_max).enumerate() {
                    let alpha = char_data.bm_buffer[col * char_data.bm_w + row];
                    if alpha != 0 {
                        screen.fill_rect(
                            Some(Rect {
                                x: x_scaled as i16,
                                y: y_scaled as i16,
                                w: 1,
                                h: 1,
                            }),
                            match line.1 {
                                true => Color::RGB(alpha / 4, alpha, alpha / 5),
                                false => Color::RGB(alpha, alpha, alpha),
                            },
                        );
                    };
                }
            }
            draw_x += screen_info.char_width;
        }

        draw_y += screen_info.char_height;
    }

    screen.flip();

    #[cfg(feature = "benchmark")]
    if only_draw_line.is_some() {
        let elapsed_ticks = get_ticks() - start_ticks;
        *ticks_taken += elapsed_ticks
    };
}

fn shift_fb(screen: &Surface, char_height: usize, direction: Direction) {
    let h = 240 - char_height as u16;

    let (from_y, to_y, blank_y) = match direction {
        Direction::Up => (char_height as i16, 0_i16, 240 - char_height as i16),
        Direction::Down => (0_i16, char_height as i16, 0),
    };

    // shift framebuffer up or down by y
    screen.blit_rect(
        screen,
        Some(Rect {
            x: 0,
            y: from_y,
            w: 320,
            h,
        }),
        Some(Rect {
            x: 0,
            y: to_y,
            w: 320,
            h,
        }),
    );

    // blank first or last line
    screen.fill_rect(
        Some(Rect {
            x: 0,
            y: blank_y,
            w: 320,
            h: char_height as u16,
        }),
        Color::RGB(0, 0, 0),
    );
}

fn split_and_wrap_lines(text: &str, width: usize) -> Vec<(String, bool)> {
    let mut lines = vec![];

    for input_line in text.split('\n') {
        if input_line.is_empty() {
            lines.push((String::new(), false));
        } else {
            let mut color = false;

            let graphemes: Vec<&str>;

            if let Some(san) = input_line.strip_prefix("\x1b[32m") {
                if let Some(san) = san.strip_suffix("\x1b[0m") {
                    // only supports green color code from line start to end right now
                    graphemes = san.graphemes(true).collect();
                    color = true;
                } else {
                    graphemes = input_line.graphemes(true).collect();
                }
            } else {
                graphemes = input_line.graphemes(true).collect();
            }

            for subline in graphemes.chunks(width) {
                lines.push((subline.concat(), color))
            }
        }
    }

    // strip last newline if exists to match std behaviour
    if let Some(last) = lines.last() {
        if last.0.is_empty() {
            lines.pop();
        }
    }

    lines
}
