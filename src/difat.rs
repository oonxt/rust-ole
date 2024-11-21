use binrw::{BinRead, BinWrite};
use modular_bitfield::prelude::{B1, B127};
use crate::common::{SectorType};

/// difat sector
/// https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/0afa4e43-b18f-432a-9917-4f276eca7a73

#[derive(Debug, Clone, BinRead, BinWrite)]
#[brw(little)]
#[brw(import(entry_count: u16))]
pub struct Difat {
    #[br(count = entry_count)]
    pub entries: Vec<SectorType>,
    pub next: SectorType,
}
#[derive(Debug, Clone, BinRead, BinWrite)]
pub struct AllEntryDifat {
    pub entries: [SectorType; 109],
}
