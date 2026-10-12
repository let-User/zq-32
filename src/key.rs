//! PS/2-клавиатура через IRQ1.

use core::arch::asm;

const KBD_DATA:   u16 = 0x60;
const KBD_STATUS: u16 = 0x64;
const BUF_SIZE: usize = 64;

static mut BUF:  [u8; BUF_SIZE] = [0; BUF_SIZE];
static mut HEAD: usize = 0;
static mut TAIL: usize = 0;

// отладочный вывод в правый нижний угол
static mut DBG_X: usize = 0;
static mut DBG_Y: usize = 22;

#[inline]
unsafe fn inb(port: u16) -> u8 {
    let v: u8;
    asm!("in al, dx", out("al") v, in("dx") port,
         options(nomem, nostack, preserves_flags));
    v
}

static ASCII: [u8; 58] = [
    0,  27, b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8',
    b'9', b'0', b'-', b'=', 0x08, 0x09,
    b'q', b'w', b'e', b'r', b't', b'y', b'u', b'i', b'o', b'p',
    b'[', b']', b'\n',
    0, b'a', b's', b'd', b'f', b'g', b'h', b'j', b'k', b'l',
    b';', b'\'', b'`', 0, b'\\',
    b'z', b'x', b'c', b'v', b'b', b'n', b'm',
    b',', b'.', b'/', 0, b'*', 0, b' ',
];

fn push(ch: u8) {
    unsafe {
        let next = (HEAD + 1) % BUF_SIZE;
        if next == TAIL { return; }
        BUF[HEAD] = ch;
        HEAD = next;
    }
}

pub unsafe fn init() {
    let mut guard = 10_000;
    while (inb(KBD_STATUS) & 0x01) != 0 && guard > 0 {
        let _ = inb(KBD_DATA);
        guard -= 1;
    }
}

pub static mut IRQ1_COUNT: u64 = 0;

pub fn on_irq() {
    unsafe { IRQ1_COUNT += 1; }

    let code = unsafe { inb(KBD_DATA) };

    // hex внизу (строка 16, если 800x600)
    unsafe {
        let hi = (code >> 4) & 0xF;
        let lo = code & 0xF;
        let to_hex = |n: u8| if n < 10 { b'0' + n } else { b'a' + (n - 10) };

        crate::vga::put_at(16, DBG_X,     to_hex(hi));
        crate::vga::put_at(16, DBG_X + 1, to_hex(lo));
        crate::vga::put_at(16, DBG_X + 2, b' ');

        DBG_X += 3;
        if DBG_X >= 45 { DBG_X = 0; }
    }

    if code & 0x80 != 0 { return; }
    let i = code as usize;
    if i >= ASCII.len() { return; }
    let c = ASCII[i];
    if c != 0 { push(c); }
}

pub fn try_get() -> Option<u8> {
    unsafe {
        if HEAD == TAIL { return None; }
        let c = BUF[TAIL];
        TAIL = (TAIL + 1) % BUF_SIZE;
        Some(c)
    }
}
