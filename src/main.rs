#![no_std]
#![no_main]

mod interrupts;
mod key;
mod logo;
mod pic;
mod pit;
mod vga;
mod print;

use core::panic::PanicInfo;
use core::sync::atomic::Ordering;

#[no_mangle]
pub extern "C" fn kernel_main(_mb: u32) -> ! {
    unsafe {
        key::init();
        pic::init();
        interrupts::init();
        pit::init(50000);
    }

    vga::clear();
    vga::disable_cursor();
    vga::draw_logo(0, 2);
    vga::print_line(9, 0, b"Type something:");

    // Включаем прерывания — теперь IRQ0 (PIT) и IRQ1 (клавиатура) пойдут к нам.
    interrupts::enable();

    let mut row = 11usize;
    let mut col = 0usize;
    let mut last_ticks = 0u64;
	println!("goot");
    loop {
        // --- 1. Обновить счётчик тиков, если он изменился ---
        let t = pit::TICKS.load(Ordering::Relaxed);
        if t != last_ticks {
            last_ticks = t;
            let mut buf = [b'0'; 8];
            let mut v = t;
            for i in (0..8).rev() {
                buf[i] = b'0' + (v % 10) as u8;
                v /= 10;
            }
            vga::print_line(20, 0, b"ticks: ");
            vga::print_line(20, 7, &buf);
        }
		
        // --- 2. Обработать ввод, если есть ---
        if let Some(ch) = key::try_get() {
            match ch {
                0x08 => {
                    if col > 0 {
                        col -= 1;
                        vga::put_at(row, col, b' ');
                    }
                }
                b'\n' => {
                    col = 0;
                    row += 1;
                    if row >= 20 { row = 11; }
                }
                _ => {
                    vga::put_at(row, col, ch);
                    col += 1;
                    if col >= vga::WIDTH {
                        col = 0;
                        row += 1;
                        if row >= 20 { row = 11; }
                    }
                }
            }
        } else {
            // Нет ввода и тик не изменился — спим до следующего IRQ.
            unsafe { core::arch::asm!("hlt", options(nomem, nostack)); }
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        unsafe { core::arch::asm!("hlt", options(nomem, nostack)); }
    }
}
