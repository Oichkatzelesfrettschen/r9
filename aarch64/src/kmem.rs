use crate::param::KZERO;
use port::mem::{PhysAddr, PhysRange, VirtAddr};

// These map to definitions in kernel.ld
unsafe extern "C" {
    static eboottext: [u64; 0];
    static text: [u64; 0];
    static etext: [u64; 0];
    static rodata: [u64; 0];
    static erodata: [u64; 0];
    static data: [u64; 0];
    static edata: [u64; 0];
    static bss: [u64; 0];
    static ebss: [u64; 0];
    static end: [u64; 0];
    static early_pagetables: [u64; 0];
    static eearly_pagetables: [u64; 0];
}

fn base_addr() -> usize {
    KZERO
}

fn eboottext_addr() -> usize {
    unsafe { eboottext.as_ptr().addr() }
}

fn text_addr() -> usize {
    unsafe { text.as_ptr().addr() }
}

fn etext_addr() -> usize {
    unsafe { etext.as_ptr().addr() }
}

fn rodata_addr() -> usize {
    unsafe { rodata.as_ptr().addr() }
}

fn erodata_addr() -> usize {
    unsafe { erodata.as_ptr().addr() }
}

fn data_addr() -> usize {
    unsafe { data.as_ptr().addr() }
}

fn edata_addr() -> usize {
    unsafe { edata.as_ptr().addr() }
}

fn bss_addr() -> usize {
    unsafe { bss.as_ptr().addr() }
}

fn ebss_addr() -> usize {
    unsafe { ebss.as_ptr().addr() }
}

fn end_addr() -> usize {
    unsafe { end.as_ptr().addr() }
}

fn early_pagetables_addr() -> usize {
    unsafe { early_pagetables.as_ptr().addr() }
}

fn eearly_pagetables_addr() -> usize {
    unsafe { eearly_pagetables.as_ptr().addr() }
}

pub fn boottext_range() -> PhysRange {
    PhysRange(from_virt_to_physaddr(VirtAddr::new(base_addr()))..from_virt_to_physaddr(VirtAddr::new(eboottext_addr())))
}

pub fn text_range() -> PhysRange {
    PhysRange(from_virt_to_physaddr(VirtAddr::new(text_addr()))..from_virt_to_physaddr(VirtAddr::new(etext_addr())))
}

pub fn rodata_range() -> PhysRange {
    PhysRange(from_virt_to_physaddr(VirtAddr::new(rodata_addr()))..from_virt_to_physaddr(VirtAddr::new(erodata_addr())))
}

pub fn data_range() -> PhysRange {
    PhysRange(from_virt_to_physaddr(VirtAddr::new(data_addr()))..from_virt_to_physaddr(VirtAddr::new(edata_addr())))
}

pub fn bss_range() -> PhysRange {
    PhysRange(from_virt_to_physaddr(VirtAddr::new(bss_addr()))..from_virt_to_physaddr(VirtAddr::new(ebss_addr())))
}

pub fn total_kernel_range() -> PhysRange {
    PhysRange(from_virt_to_physaddr(VirtAddr::new(base_addr()))..from_virt_to_physaddr(VirtAddr::new(end_addr())))
}

/// Transform the physical address to a virtual address, under the assumption that
/// the virtual address is the physical address offset from KZERO.
pub const fn physaddr_as_ptr_mut_offset_from_kzero<T>(pa: PhysAddr) -> *mut T {
    (pa.addr() as usize).wrapping_add(KZERO) as *mut T
}

/// Given a virtual address, return the physical address.  Makes a massive assumption
/// that the code is mapped offset to KZERO, so should be used with extreme care.
pub fn from_virt_to_physaddr(va: VirtAddr) -> PhysAddr {
    debug_assert!(va.addr() >= KZERO, "from_virt_to_physaddr: va {:?} must be >= KZERO ({})", va, KZERO);
    PhysAddr::new((va.addr() - KZERO) as u64)
}

/// Given an address, return the physical address.  Makes a massive assumption
/// that the code is mapped offset to KZERO, so should be used with extreme care.
pub fn from_ptr_to_physaddr_offset_from_kzero<T>(a: *const T) -> PhysAddr {
    from_virt_to_physaddr(VirtAddr::new(a.addr()))
}

pub fn early_pages_range() -> PhysRange {
    PhysRange::new(
        from_virt_to_physaddr(VirtAddr::new(early_pagetables_addr())),
        from_virt_to_physaddr(VirtAddr::new(eearly_pagetables_addr())),
    )
}
