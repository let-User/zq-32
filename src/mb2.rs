//! Парсер Multiboot2 info — ищем только framebuffer tag.

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FramebufferInfo {
    pub addr:   u64,
    pub pitch:  u32,
    pub width:  u32,
    pub height: u32,
    pub bpp:    u8,
}

pub unsafe fn find_framebuffer(info_addr: u64) -> Option<FramebufferInfo> {
    if info_addr == 0 { return None; }

    let base = info_addr as *const u8;
    let total_size = core::ptr::read_unaligned(base as *const u32) as usize;

    let mut ptr = base.add(8);
    let end = base.add(total_size);

    while ptr < end {
        let tag_type = core::ptr::read_unaligned(ptr as *const u32);
        let tag_size = core::ptr::read_unaligned(ptr.add(4) as *const u32) as usize;

        if tag_type == 0 { break; }
        if tag_type == 8 {
            let addr   = core::ptr::read_unaligned(ptr.add(8)  as *const u64);
            let pitch  = core::ptr::read_unaligned(ptr.add(16) as *const u32);
            let width  = core::ptr::read_unaligned(ptr.add(20) as *const u32);
            let height = core::ptr::read_unaligned(ptr.add(24) as *const u32);
            let bpp    = core::ptr::read_unaligned(ptr.add(28) as *const u8);
            return Some(FramebufferInfo { addr, pitch, width, height, bpp });
        }

        ptr = ptr.add((tag_size + 7) & !7);
    }
    None
}
