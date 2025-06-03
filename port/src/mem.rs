use crate::fdt::RegBlock;
use core::{
    cmp::{max, min},
    fmt,
    iter::{Step, StepBy},
    ops::{self, Range},
};

pub const PAGE_SIZE_4K: usize = 4 << 10;
pub const PAGE_SIZE_2M: usize = 2 << 20;
pub const PAGE_SIZE_1G: usize = 1 << 30;

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
#[repr(transparent)]
pub struct VirtAddr(pub usize);

impl VirtAddr {
    pub const fn new(value: usize) -> Self {
        VirtAddr(value)
    }

    pub const fn addr(&self) -> usize {
        self.0
    }
}

impl ops::Add<usize> for VirtAddr {
    type Output = VirtAddr;

    fn add(self, offset: usize) -> VirtAddr {
        VirtAddr(self.0 + offset)
    }
}

impl ops::Sub<usize> for VirtAddr {
    type Output = VirtAddr;

    fn sub(self, offset: usize) -> VirtAddr {
        VirtAddr(self.0 - offset)
    }
}

impl fmt::Debug for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VirtAddr({:#016x})", self.0)?;
        Ok(())
    }
}

pub struct VirtRange(pub Range<VirtAddr>);

impl VirtRange {
    pub fn with_len(start: VirtAddr, len: usize) -> Self {
        Self(start..start + len)
    }

    pub fn offset_addr(&self, offset: usize) -> Option<VirtAddr> {
        let addr = self.0.start + offset;
        if self.0.contains(&addr) { Some(addr) } else { None }
    }

    pub fn start(&self) -> VirtAddr {
        self.0.start
    }

    pub fn end(&self) -> VirtAddr {
        self.0.end
    }
}

impl From<&RegBlock> for VirtRange {
    fn from(r: &RegBlock) -> Self {
        let start = VirtAddr(r.addr as usize);
        let end = start + r.len.unwrap_or(0) as usize;
        VirtRange(start..end)
    }
}

impl fmt::Display for VirtRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#018x}..{:#018x}", self.0.start.addr(), self.0.end.addr())
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
#[repr(transparent)]
pub struct PhysAddr(pub u64);

impl PhysAddr {
    pub const fn new(value: u64) -> Self {
        PhysAddr(value)
    }

    pub const fn addr(&self) -> u64 {
        self.0
    }

    pub const fn round_up(&self, step: u64) -> PhysAddr {
        assert!(step.is_power_of_two());
        PhysAddr((self.0 + step - 1) & !(step - 1))
    }

    pub const fn round_down(&self, step: u64) -> PhysAddr {
        assert!(step.is_power_of_two());
        PhysAddr(self.0 & !(step - 1))
    }

    pub const fn is_multiple_of(&self, n: u64) -> bool {
        self.0.is_multiple_of(n)
    }
}

impl ops::Add<u64> for PhysAddr {
    type Output = PhysAddr;

    fn add(self, offset: u64) -> PhysAddr {
        PhysAddr(self.0 + offset)
    }
}

impl Step for PhysAddr {
    fn steps_between(&startpa: &Self, &endpa: &Self) -> (usize, Option<usize>) {
        if startpa.0 <= endpa.0 {
            if let Some(diff) = endpa.0.checked_sub(startpa.0) {
                if let Ok(diff) = usize::try_from(diff) {
                    return (diff, Some(diff));
                }
            }
        }
        (0, None)
    }

    fn forward_checked(startpa: Self, count: usize) -> Option<Self> {
        startpa.0.checked_add(count as u64).map(PhysAddr)
    }

    fn backward_checked(startpa: Self, count: usize) -> Option<Self> {
        startpa.0.checked_sub(count as u64).map(PhysAddr)
    }
}

impl fmt::Debug for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PhysAddr({:#016x})", self.0)?;
        Ok(())
    }
}

pub struct PhysRange(pub Range<PhysAddr>);

impl PhysRange {
    pub fn new(start: PhysAddr, end: PhysAddr) -> Self {
        Self(start..end)
    }

    pub fn with_end(start: u64, end: u64) -> Self {
        Self(PhysAddr(start)..PhysAddr(end))
    }

    pub fn with_len(start: u64, len: usize) -> Self {
        Self(PhysAddr(start)..PhysAddr(start + len as u64))
    }

    pub fn with_pa_len(start: PhysAddr, len: usize) -> Self {
        Self(start..PhysAddr(start.0 + len as u64))
    }

    #[allow(dead_code)]
    pub fn offset_addr(&self, offset: u64) -> Option<PhysAddr> {
        let addr = self.0.start + offset;
        if self.0.contains(&addr) { Some(addr) } else { None }
    }

    pub fn start(&self) -> PhysAddr {
        self.0.start
    }

    pub fn end(&self) -> PhysAddr {
        self.0.end
    }

    pub fn size(&self) -> usize {
        (self.0.end.addr() - self.0.start.addr()) as usize
    }

    pub fn step_by_rounded(&self, step_size: usize) -> StepBy<Range<PhysAddr>> {
        let startpa = self.start().round_down(step_size as u64);
        let endpa = self.end().round_up(step_size as u64);
        (startpa..endpa).step_by(step_size)
    }

    pub fn add(&self, other: &PhysRange) -> Self {
        Self(min(self.0.start, other.0.start)..max(self.0.end, other.0.end))
    }
}

impl fmt::Display for PhysRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#018x}..{:#018x}", self.0.start.addr(), self.0.end.addr())
    }
}

impl From<&RegBlock> for PhysRange {
    fn from(r: &RegBlock) -> Self {
        let start = PhysAddr(r.addr);
        let end = start + r.len.unwrap_or(0);
        PhysRange(start..end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virtaddr_ops() {
        let va1 = VirtAddr::new(0x1000);
        assert_eq!(va1.addr(), 0x1000);
        let va2 = va1 + 0x100;
        assert_eq!(va2.addr(), 0x1100);
        let va3 = va2 - 0x100;
        assert_eq!(va3.addr(), 0x1000);
        assert_eq!(va1, va3);
    }

    #[test]
    fn virtrange_ops() {
        let start_va = VirtAddr::new(0x1000);
        let range = VirtRange::with_len(start_va, 0x100);
        assert_eq!(range.start(), start_va);
        assert_eq!(range.end(), VirtAddr::new(0x1100));

        assert_eq!(range.offset_addr(0x0), Some(VirtAddr::new(0x1000)));
        assert_eq!(range.offset_addr(0x80), Some(VirtAddr::new(0x1080)));
        assert_eq!(range.offset_addr(0xff), Some(VirtAddr::new(0x10ff))); // Contained
        assert_eq!(range.offset_addr(0x100), None); // Exclusive end

        let reg_block = RegBlock { addr: 0x2000, len: Some(0x200) };
        let vr_from_reg = VirtRange::from(&reg_block);
        assert_eq!(vr_from_reg.start(), VirtAddr::new(0x2000));
        assert_eq!(vr_from_reg.end(), VirtAddr::new(0x2200));
    }

    #[test]
    fn physaddr_ops() {
        let pa1 = PhysAddr::new(0x1000);
        assert_eq!(pa1.addr(), 0x1000);
        let pa2 = pa1 + 0x100;
        assert_eq!(pa2.addr(), 0x1100);

        assert!(PhysAddr::new(0x2000).is_multiple_of(0x1000));
        assert!(!PhysAddr::new(0x2001).is_multiple_of(0x1000));

        assert_eq!(PhysAddr::new(0x1234).round_up(0x100), PhysAddr::new(0x1300));
        assert_eq!(PhysAddr::new(0x1200).round_up(0x100), PhysAddr::new(0x1200));
        assert_eq!(PhysAddr::new(0x1234).round_down(0x100), PhysAddr::new(0x1200));
        assert_eq!(PhysAddr::new(0x1200).round_down(0x100), PhysAddr::new(0x1200));
    }

    #[test]
    fn physrange_ops() {
        let r1 = PhysRange::with_end(0x1000, 0x2000);
        assert_eq!(r1.start(), PhysAddr::new(0x1000));
        assert_eq!(r1.end(), PhysAddr::new(0x2000));
        assert_eq!(r1.size(), 0x1000);

        let r2 = PhysRange::with_len(0x3000, 0x100);
        assert_eq!(r2.start(), PhysAddr::new(0x3000));
        assert_eq!(r2.end(), PhysAddr::new(0x3100));
        assert_eq!(r2.size(), 0x100);

        let r_start_pa = PhysAddr::new(0x4000);
        let r3 = PhysRange::with_pa_len(r_start_pa, 0x200);
        assert_eq!(r3.start(), r_start_pa);
        assert_eq!(r3.end(), PhysAddr::new(0x4200));

        let r_combined = r1.add(&r2); // (0x1000..0x2000) + (0x3000..0x3100) -> (0x1000..0x3100)
        assert_eq!(r_combined.start(), PhysAddr::new(0x1000));
        assert_eq!(r_combined.end(), PhysAddr::new(0x3100));

        let r_overlapping = PhysRange::with_end(0x1500, 0x2500);
        let r_combined_overlap = r1.add(&r_overlapping); // (0x1000..0x2000) + (0x1500..0x2500) -> (0x1000..0x2500)
        assert_eq!(r_combined_overlap.start(), PhysAddr::new(0x1000));
        assert_eq!(r_combined_overlap.end(), PhysAddr::new(0x2500));
    }

    #[test]
    fn physaddr_step() {
        let range = PhysRange(PhysAddr::new(4096)..PhysAddr::new(4096 * 3));
        let pas = range.step_by_rounded(PAGE_SIZE_4K).collect::<Vec<PhysAddr>>();
        assert_eq!(pas, [PhysAddr::new(4096), PhysAddr::new(4096 * 2)]);
    }

    #[test]
    fn physaddr_step_rounds_up_and_down() {
        // Start should round down to 8192
        // End should round up to 16384
        let range = PhysRange(PhysAddr::new(9000)..PhysAddr::new(5000 * 3));
        let pas = range.step_by_rounded(PAGE_SIZE_4K).collect::<Vec<PhysAddr>>();
        assert_eq!(pas, [PhysAddr::new(4096 * 2), PhysAddr::new(4096 * 3)]);
    }

    #[test]
    fn physaddr_step_2m() {
        let range =
            PhysRange(PhysAddr::new(0x3f000000)..PhysAddr::new(0x3f000000 + 4 * 1024 * 1024));
        let pas = range.step_by_rounded(PAGE_SIZE_2M).collect::<Vec<PhysAddr>>();
        assert_eq!(pas, [PhysAddr::new(0x3f000000), PhysAddr::new(0x3f000000 + 2 * 1024 * 1024)]);
    }
}
