use binrw::{BinRead, BinWrite};
use crate::common::SectorType;

/// mini fat sector
/// https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/c5d235f7-b73c-4ec5-bf8d-5c08306cd023

pub const MINI_FAT_SECTOR_SIZE: u16 = 64;

#[derive(Debug, Clone, BinRead, BinWrite)]
#[brw(little)]
#[brw(import(entry_count: u16))]
pub struct MiniFat {
    #[br(count = entry_count)]
    pub entries: Vec<SectorType>,
}