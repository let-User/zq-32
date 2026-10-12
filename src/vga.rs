//! Framebuffer-«текст» — рисуем шрифтом 8x16 в линейный буфер.

use core::ptr;
use crate::mb2::FramebufferInfo;

const FONT: &[u8; 4096] = include_bytes!("../fonts/font8x16.bin");
const CHAR_W: usize = 8;
const CHAR_H: usize = 16;

const SCALE:  usize = 3;   // было 2 — буквы станут крупнее
pub const WIDTH:  usize = 80;   // совпадает с шириной 1920/(8*3)
pub const HEIGHT: usize = 22;   // 1080/(16*3) = 22.5, берём 22
const FG: u32 = 0xFFFFFF;
const BG: u32 = 0x000000;

static mut FB: Option<FramebufferInfo> = None;

pub unsafe fn init(info: FramebufferInfo) {
    FB = Some(info);
}

#[inline]
fn fb() -> &'static FramebufferInfo {
    unsafe { FB.as_ref().unwrap() }
}

#[inline]
fn put_pixel(x: usize, y: usize, color: u32) {
    let f = fb();
    if x >= f.width as usize || y >= f.height as usize { return; }
    let bpp_bytes = (f.bpp as usize) / 8;
    let offset = y * f.pitch as usize + x * bpp_bytes;
    unsafe {
        let p = (f.addr as *mut u8).add(offset) as *mut u32;
        ptr::write_volatile(p, color);
    }
}

fn draw_char(row: usize, col: usize, ch: u8, fg: u32, bg: u32) {
    let glyph = &FONT[(ch as usize) * CHAR_H .. (ch as usize + 1) * CHAR_H];
    let px0 = col * CHAR_W * SCALE;
    let py0 = row * CHAR_H * SCALE;

    for gy in 0..CHAR_H {
        let bits = glyph[gy];
        for gx in 0..CHAR_W {
            let on = (bits >> (7 - gx)) & 1 != 0;
            let color = if on { fg } else { bg };
            for sy in 0..SCALE {
                for sx in 0..SCALE {
                    put_pixel(px0 + gx * SCALE + sx, py0 + gy * SCALE + sy, color);
                }
            }
        }
    }
}

pub fn clear() {
    let f = fb();
    let bpp_bytes = (f.bpp as usize) / 8;
    unsafe {
        for y in 0..f.height as usize {
            for x in 0..f.width as usize {
                let off = y * f.pitch as usize + x * bpp_bytes;
                let p = (f.addr as *mut u8).add(off) as *mut u32;
                ptr::write_volatile(p, BG);
            }
        }
    }
}

pub fn put_at(row: usize, col: usize, ch: u8) {
    draw_char(row, col, ch, FG, BG);
}

pub fn print_line(row: usize, col: usize, bytes: &[u8]) {
    for (i, &b) in bytes.iter().enumerate() {
        if col + i >= WIDTH { break; }
        put_at(row, col + i, b);
    }
}

pub fn draw_logo(row: usize, col: usize) {
    for (i, line) in crate::logo::LOGO.iter().enumerate() {
        for (j, &b) in line.iter().enumerate() {
            put_at(row + i, col + j, b);
        }
    }
}

pub fn disable_cursor() {}

pub fn print(_bytes: &[u8]) {}
pub fn print_hex(_v: u64) {}
