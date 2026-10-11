//! IDT и обработчики прерываний.

use core::arch::global_asm;
use core::mem::size_of;

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct IdtEntry {
    offset_low:  u16,
    selector:    u16,
    ist:         u8,
    type_attr:   u8,
    offset_mid:  u16,
    offset_high: u32,
    reserved:    u32,
}

impl IdtEntry {
    pub const fn missing() -> Self {
        Self { offset_low: 0, selector: 0, ist: 0, type_attr: 0,
               offset_mid: 0, offset_high: 0, reserved: 0 }
    }
    pub fn new(handler: u64, selector: u16) -> Self {
        Self {
            offset_low:  (handler & 0xFFFF) as u16,
            selector,
            ist: 0,
            type_attr: 0x8E,
            offset_mid: ((handler >> 16) & 0xFFFF) as u16,
            offset_high: (handler >> 32) as u32,
            reserved: 0,
        }
    }
}

#[repr(C, packed)]
struct IdtPointer {
    limit: u16,
    base:  u64,
}

static mut IDT: [IdtEntry; 256] = [IdtEntry::missing(); 256];

#[repr(C)]
pub struct InterruptFrame {
    pub r15: u64, pub r14: u64, pub r13: u64, pub r12: u64,
    pub r11: u64, pub r10: u64, pub r9:  u64, pub r8:  u64,
    pub rbp: u64, pub rdi: u64, pub rsi: u64, pub rdx: u64,
    pub rcx: u64, pub rbx: u64, pub rax: u64,
    pub int_no:   u64,
    pub err_code: u64,
    pub rip:    u64,
    pub cs:     u64,
    pub rflags: u64,
    pub rsp:    u64,
    pub ss:     u64,
}

global_asm!(r#"
.macro ISR_NOERR num
.global isr\num
isr\num:
    push 0
    push \num
    jmp isr_common
.endm

.macro ISR_ERR num
.global isr\num
isr\num:
    push \num
    jmp isr_common
.endm

.macro IRQ num, irqno
.global irq\num
irq\num:
    push 0
    push \irqno
    jmp isr_common
.endm

ISR_NOERR 0
ISR_NOERR 1
ISR_NOERR 2
ISR_NOERR 3
ISR_NOERR 4
ISR_NOERR 5
ISR_NOERR 6
ISR_NOERR 7
ISR_ERR   8
ISR_NOERR 9
ISR_ERR   10
ISR_ERR   11
ISR_ERR   12
ISR_ERR   13
ISR_ERR   14
ISR_NOERR 15
ISR_NOERR 16
ISR_ERR   17
ISR_NOERR 18
ISR_NOERR 19
ISR_NOERR 20
ISR_ERR   21
ISR_NOERR 22
ISR_NOERR 23
ISR_NOERR 24
ISR_NOERR 25
ISR_NOERR 26
ISR_NOERR 27
ISR_NOERR 28
ISR_ERR   29
ISR_ERR   30
ISR_NOERR 31

IRQ 0, 32
IRQ 1, 33
IRQ 2, 34
IRQ 3, 35
IRQ 4, 36
IRQ 5, 37
IRQ 6, 38
IRQ 7, 39
IRQ 8, 40
IRQ 9, 41
IRQ 10, 42
IRQ 11, 43
IRQ 12, 44
IRQ 13, 45
IRQ 14, 46
IRQ 15, 47

isr_common:
    push rax
    push rbx
    push rcx
    push rdx
    push rsi
    push rdi
    push rbp
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15

    mov rdi, rsp
    call interrupt_dispatch

    pop r15
    pop r14
    pop r13
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop rbp
    pop rdi
    pop rsi
    pop rdx
    pop rcx
    pop rbx
    pop rax

    add rsp, 16
    iretq
"#);

extern "C" {
    fn isr0();  fn isr1();  fn isr2();  fn isr3();
    fn isr4();  fn isr5();  fn isr6();  fn isr7();
    fn isr8();  fn isr9();  fn isr10(); fn isr11();
    fn isr12(); fn isr13(); fn isr14(); fn isr15();
    fn isr16(); fn isr17(); fn isr18(); fn isr19();
    fn isr20(); fn isr21(); fn isr22(); fn isr23();
    fn isr24(); fn isr25(); fn isr26(); fn isr27();
    fn isr28(); fn isr29(); fn isr30(); fn isr31();
    fn irq0();  fn irq1();  fn irq2();  fn irq3();
    fn irq4();  fn irq5();  fn irq6();  fn irq7();
    fn irq8();  fn irq9();  fn irq10(); fn irq11();
    fn irq12(); fn irq13(); fn irq14(); fn irq15();
}

static HANDLERS: [unsafe extern "C" fn(); 48] = [
    isr0, isr1, isr2, isr3, isr4, isr5, isr6, isr7,
    isr8, isr9, isr10, isr11, isr12, isr13, isr14, isr15,
    isr16, isr17, isr18, isr19, isr20, isr21, isr22, isr23,
    isr24, isr25, isr26, isr27, isr28, isr29, isr30, isr31,
    irq0, irq1, irq2, irq3, irq4, irq5, irq6, irq7,
    irq8, irq9, irq10, irq11, irq12, irq13, irq14, irq15,
];

pub unsafe fn init() {
    for i in 0..48 {
        IDT[i] = IdtEntry::new(HANDLERS[i] as u64, 0x08);
    }
    let ptr = IdtPointer {
        limit: (size_of::<IdtEntry>() * 256 - 1) as u16,
        base:  &raw const IDT as *const _ as u64,
    };
    core::arch::asm!("lidt [{}]", in(reg) &ptr, options(nostack));
}
#[no_mangle]
extern "C" fn interrupt_dispatch(frame: *mut InterruptFrame) {
    let f = unsafe { &mut *frame };
    let n = f.int_no as u8;

    match n {
        0..=31  => exception(n, f),
        32..=47 => {
            let irq = n - 32;
            irq_handler(irq);
            unsafe {
                if irq >= 8 { crate::pic::eoi_slave(); }
                crate::pic::eoi_master();
            }
        }
        _ => {}
    }
}

fn exception(n: u8, f: &InterruptFrame) {
    crate::vga::print(b"EXC ");
    crate::vga::print_hex(n as u64);
    crate::vga::print(b" RIP=");
    crate::vga::print_hex(f.rip);
    crate::vga::print(b"\n");
    loop { unsafe { core::arch::asm!("hlt"); } }
}

fn irq_handler(irq: u8) {
    match irq {
        0 => crate::pit::tick(),
        1 => crate::key::on_irq(),
        _ => {}
    }
}

pub fn enable() {
    unsafe { core::arch::asm!("sti", options(nomem, nostack)); }
}

