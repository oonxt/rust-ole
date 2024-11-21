use binrw::{binrw, BinRead, BinWrite};
use crate::common::SectorType;

/// fat sector
/// https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/30e1013a-a0ff-4404-9ccf-d75d835ff404
#[derive(Debug, Clone, BinRead, BinWrite)]
#[brw(little)]
#[br(import(entry_count: u16))]
pub struct Fat {
    #[br(count = entry_count)]
    pub entries: Vec<SectorType>,
}

