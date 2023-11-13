#![no_std]

extern crate alloc;
extern crate ndless_handler;

use ndless::prelude::*;

use ndless::input::{any_key_pressed, iter_keys, wait_key_pressed, Key};
// use ndless::msg::{msg_2b, msg_3b, Button};

use ndless_sdl::nsdl::{Font, FontOptions};

use ndless_sdl::video::Surface;

// const WIDTH:

/// Display the text in full screen, respecting line wrap and supporting scrolling.
/// Screen can not be modified after creation, and blocks the executor until enter/esc is pressed.
pub fn display(text: &str) {
    // screen setup
    let screen = ndless_sdl::init_default().expect("failed to set video mode");
    let font = Font::new(FontOptions::Thin, 255, 255, 255);
    font.monospaced(true);

    // 240px, each letter 6px wide
    // let width = 40;

    const WIDTH: usize = 40;
    const HEIGHT: usize = 26;
    // ^@$«ßäöü/\;"§*#|
    // ^@$     /\;" *#|

    //     let text = "^@$«ßäöü/\\;\"§*#|Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vivamus quis lacus mattis, bibendum tellus eu, ultrices nisi. Maecenas a magna volutpat, mattis mauris et molestie orci. Morbi vulputate tempor lacus, a dapibus nulla rhoncus ac. Donec malesuada fringilla odio, ut consequat magna sodales ac. Integer a bibendum nisi, at auctor leo. Morbi consequat urna sed justo viverra, at lacinia odio eleifend. Nulla facilisi. Pellentesque habitant morbi tristique senectus et netus et malesuada fames ac turpis egestas. Maecenas egestas leo vel lacus hendrerit laoreet. Suspendisse vitae eleifend ipsum. Nunc massa lacus, malesuada nec vestibulum non, auctor in erat. Aliquam vulputate rhoncus volutpat. Nullam vitae viverra dolor. Ut tincidunt diam vitae velit porta maximus. Aenean convallis sit amet dui et molestie.

    // Maecenas varius erat id ipsum gravida egestas. Morbi a ultrices ante. Proin luctus tortor sit amet placerat ultricies. Cras vitae faucibus libero. Nullam molestie, dui vitae consequat feugiat, lacus elit ultricies arcu, vitae egestas mauris odio nec ante. Donec scelerisque mattis lorem, in euismod eros lacinia vitae. Praesent laoreet bibendum magna, ut mollis sapien vulputate quis.

    // Mauris rutrum congue eros vitae facilisis. Nunc in mattis odio. Fusce eget rhoncus arcu, vel eleifend ex. Curabitur eu nunc id risus commodo interdum eu at nibh. Duis eleifend, metus in fringilla interdum, dui ex molestie lectus, a feugiat est dui sagittis quam. Curabitur at mollis leo, eu commodo ligula. Vivamus dapibus libero ut nulla volutpat aliquet. Donec elementum elit vel leo eleifend dapibus. In et diam odio. Aliquam ultricies egestas aliquet. Nam metus nisi, vulputate vitae purus at, viverra feugiat eros. Ut tempor sapien enim, id ultricies sem viverra vel. Fusce pharetra tempor finibus. Proin odio est, porta nec congue vitae, luctus sed ipsum. Sed sit amet est maximus enim hendrerit malesuada quis eget diam.";

    let lines = split_and_wrap_lines(text, WIDTH);

    let mut start_line = 0;
    let mut max_line;

    'display: loop {
        max_line = if start_line + HEIGHT > lines.len() {
            lines.len()
        } else {
            start_line + HEIGHT
        };

        clear_screen(&screen);

        for (i, line) in lines[start_line..max_line].iter().enumerate() {
            screen.draw_str(&font, line, 0, 9 * i as i32 + 3);
        }
        screen.flip();

        loop {
            if !any_key_pressed() {
                wait_key_pressed();
            }

            match iter_keys().next() {
                Some(Key::Esc) | Some(Key::Enter) | Some(Key::Key5) => break 'display,
                Some(Key::Down) | Some(Key::Key2) => {
                    if max_line < lines.len() {
                        start_line += 1;
                        break;
                    }
                }
                Some(Key::Up) | Some(Key::Key8) => {
                    if start_line > 0 {
                        start_line -= 1;
                        break;
                    }
                }
                _ => {}
            }
        }
    }
}

fn split_and_wrap_lines(text: &str, width: usize) -> Vec<String> {
    let mut lines = vec![];

    for line in text.split('\n') {
        let mut cursor = 0;
        let mut upto = 0;

        if line.is_empty() {
            lines.push(String::new())
        } else {
            while upto != line.len() {
                upto = if line.len() - cursor > width {
                    cursor + width
                } else {
                    line.len()
                };
                lines.push(line[cursor..upto].to_string());
                cursor = upto;
            }
        }
    }

    lines
}

fn clear_screen(screen: &Surface) {
    screen.fill_rect(
        Some(ndless_sdl::Rect {
            x: 0,
            y: 0,
            w: 320,
            h: 240,
        }),
        ndless_sdl::video::RGB(0, 0, 0),
    );
}
