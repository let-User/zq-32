//! PIC 8259A — перепрограммирование IRQ на векторы 32..47.

use core::arch::asm;

const PIC1_CMD:  u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_CMD:  u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

#[inline]
unsafe fn outb(port: u16, val: u8) {
    asm!("out dx, al", in("dx") port, in("al") val,
         options(nomem, nostack, preserves_flags));
}

#[inline]
unsafe fn io_wait() {
    outb(0x80, 0);
}

pub unsafe fn init() {
    outb(PIC1_CMD, 0x11); io_wait();
    outb(PIC2_CMD, 0x11); io_wait();
    outb(PIC1_DATA, 32);  io_wait();
    outb(PIC2_DATA, 40);  io_wait();
    outb(PIC1_DATA, 0x04); io_wait();
    outb(PIC2_DATA, 0x02); io_wait();
    outb(PIC1_DATA, 0x01); io_wait();
    outb(PIC2_DATA, 0x01); io_wait();

    // Разрешаем только IRQ0 (таймер) и IRQ1 (клавиатура).
    outb(PIC1_DATA, 0b1111_1100);
    outb(PIC2_DATA, 0xFF);
}

pub unsafe fn eoi_master() { outb(PIC1_CMD, 0x20); }
pub unsafe fn eoi_slave()  { outb(PIC2_CMD, 0x20); }
