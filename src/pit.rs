//! PIT 8253/8254 — таймер, канал 0, режим 3.

use core::arch::asm;
use core::sync::atomic::{AtomicU64, Ordering};

const PIT_CH0:  u16 = 0x40;
const PIT_CMD:  u16 = 0x43;
const BASE_HZ:  u32 = 1_193_182;

pub static TICKS: AtomicU64 = AtomicU64::new(0);

#[inline]
unsafe fn outb(port: u16, val: u8) {
    asm!("out dx, al", in("dx") port, in("al") val,
         options(nomem, nostack, preserves_flags));
}

pub unsafe fn init(freq: u32) {
    let divisor = BASE_HZ / freq;
    outb(PIT_CMD, 0x36);
    outb(PIT_CH0, (divisor & 0xFF) as u8);
    outb(PIT_CH0, ((divisor >> 8) & 0xFF) as u8);
}

pub fn tick() {
    TICKS.fetch_add(1, Ordering::Relaxed);
}
