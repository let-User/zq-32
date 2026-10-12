#![no_std]
#![no_main]

use core::ffi::c_void;

// ============================================================
//  EFI-структуры (минимально необходимое)
// ============================================================

#[repr(C)]
pub struct EfiTableHeader {
    pub signature:   u64,
    pub revision:    u32,
    pub header_size: u32,
    pub crc32:       u32,
    pub reserved:    u32,
}

#[repr(C)]
pub struct EfiSystemTable {
    pub hdr: EfiTableHeader,
    pub firmware_vendor:      *const u16,
    pub firmware_revision:    u32,
    pub console_in_handle:    *mut c_void,
    pub con_in:               *mut c_void,
    pub console_out_handle:   *mut c_void,
    pub con_out:              *mut c_void,
    pub standard_error_handle:*mut c_void,
    pub std_err:              *mut c_void,
    pub runtime_services:     *mut c_void,
    pub boot_services:        *mut c_void,
}

#[repr(C)]
pub struct EfiGuid {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

const GOP_GUID: EfiGuid = EfiGuid {
    data1: 0x9042A9DE,
    data2: 0x23DC,
    data3: 0x4A38,
    data4: [0x96, 0xFB, 0x7A, 0xDE, 0xD0, 0x80, 0x51, 0x6A],
};

// Boot Services — нужен только locate_protocol по смещению.
// Чтобы не расписывать все поля, объявляем как opaque-структуру и
// вычисляем указатель на нужный слот.
#[repr(C)]
pub struct EfiBootServices {
    pub hdr: EfiTableHeader,
    // ... много полей ...
    // locate_protocol — на фиксированном смещении 320 (0x140) в UEFI 2.x
}

// ============================================================
//  GOP (Graphics Output Protocol)
// ============================================================

#[repr(C)]
pub struct EfiPixelBitmask {
    pub red: u32,
    pub green: u32,
    pub blue: u32,
    pub reserved: u32,
}

#[repr(C)]
pub struct EfiGopModeInfo {
    pub version:                   u32,
    pub horizontal_resolution:     u32,
    pub vertical_resolution:       u32,
    pub pixel_format:              u32,
    pub pixel_information:         EfiPixelBitmask,
    pub pixels_per_scan_line:      u32,
}

#[repr(C)]
pub struct EfiGopMode {
    pub max_mode:           u32,
    pub mode:               u32,
    pub info:               *mut EfiGopModeInfo,
    pub size_of_info:       usize,
    pub frame_buffer_base:  u64,
    pub frame_buffer_size:  usize,
}

#[repr(C)]
pub struct EfiGraphicsOutputProtocol {
    pub query_mode: *mut c_void,
    pub set_mode:   *mut c_void,
    pub blt:        *mut c_void,
    pub mode:       *mut EfiGopMode,
}

// ============================================================
//  Точка входа
// ============================================================

#[no_mangle]
pub extern "efiapi" fn efi_main(
    _image: *mut c_void,
    system_table: *mut EfiSystemTable,
) -> usize {
    unsafe {
        let st = &*system_table;

        // locate_protocol — на смещении 320 (0x140) от начала BootServices.
        // Это зафиксировано в UEFI spec (все версии 2.x).
        let bs_ptr = st.boot_services as *const u8;
        let locate_protocol: extern "efiapi" fn(
            *const EfiGuid,
            *mut c_void,
            *mut *mut c_void,
        ) -> usize = core::mem::transmute(*(bs_ptr.add(320) as *const usize));

        // Найти GOP
        let mut gop_ptr: *mut c_void = core::ptr::null_mut();
        let status = locate_protocol(&GOP_GUID, core::ptr::null_mut(), &mut gop_ptr);
        if status != 0 || gop_ptr.is_null() {
            // Не нашли — вечно висим
            loop { core::arch::asm!("hlt", options(nomem, nostack)); }
        }

        let gop  = &*(gop_ptr as *const EfiGraphicsOutputProtocol);
        let mode = &*gop.mode;
        let info = &*mode.info;

        let fb_addr = mode.frame_buffer_base;
        let width   = info.horizontal_resolution as usize;
        let height  = info.vertical_resolution as usize;
        let stride  = info.pixels_per_scan_line as usize;

        // Залить экран красным
        let fb = fb_addr as *mut u32;
        for y in 0..height {
            for x in 0..width {
                core::ptr::write_volatile(fb.add(y * stride + x), 0x00FF0000);
            }
        }

        // Зависнуть
        loop { core::arch::asm!("hlt", options(nomem, nostack)); }
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop { unsafe { core::arch::asm!("hlt", options(nomem, nostack)); } }
}
