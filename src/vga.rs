//! VGA-текст 80x25.

use core::ptr;

pub const WIDTH:  usize = 80;
pub const HEIGHT: usize = 25;
pub const ATTR:   u8    = 0x0F;

const VGA: *mut u8 = 0xB8000 as *mut u8;

pub fn clear() {
    for i in 0..(WIDTH * HEIGHT) {
        unsafe {
            ptr::write_volatile(VGA.add(i * 2),     b' ');
            ptr::write_volatile(VGA.add(i * 2 + 1), ATTR);
        }
    }
}

pub fn put_at(row: usize, col: usize, ch: u8) {
    if row >= HEIGHT || col >= WIDTH { return; }
    let off = (row * WIDTH + col) * 2;
    unsafe {
        ptr::write_volatile(VGA.add(off),     ch);
        ptr::write_volatile(VGA.add(off + 1), ATTR);
    }
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

pub fn disable_cursor() {
    unsafe {
        core::arch::asm!("out dx, al", in("dx") 0x3D4u16, in("al") 0x0Au8,
                         options(nomem, nostack, preserves_flags));
        core::arch::asm!("out dx, al", in("dx") 0x3D5u16, in("al") 0x20u8,
                         options(nomem, nostack, preserves_flags));
    }
}

/// Простая отладочная печать (не трогает основной вывод).
static mut DBG_CURSOR: usize = 0;

pub fn print(bytes: &[u8]) {
    unsafe {
        for &b in bytes {
            let off = DBG_CURSOR * 2;
            if off + 1 < WIDTH * HEIGHT * 2 {
                ptr::write_volatile(VGA.add(off),     b);
                ptr::write_volatile(VGA.add(off + 1), ATTR);
            }
            DBG_CURSOR += 1;
            if DBG_CURSOR >= WIDTH * HEIGHT { DBG_CURSOR = 0; }
        }
    }
}

pub fn print_hex(v: u64) {
    let mut buf = [b'0'; 18];
    buf[0] = b'0';
    buf[1] = b'x';
    for i in 0..16 {
        let nib = (v >> ((15 - i) * 4)) & 0xF;
        buf[2 + i] = if nib < 10 { b'0' + nib as u8 } else { b'a' + (nib - 10) as u8 };
    }
    print(&buf);
}
