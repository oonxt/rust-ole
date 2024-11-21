use std::fmt::{Display, Formatter};
use binrw::{binrw, BinRead, BinWrite};
use thiserror::Error;

pub const MAX_REG_SECT: u32 = 0xFFFFFFFA;
pub const NOT_APPLICABLE: u32 = 0xFFFFFFFB;
pub const DIF_SECT: u32 = 0xFFFFFFFC;
pub const FAT_SECT: u32 = 0xFFFFFFFD;
pub const END_OF_CHAIN: u32 = 0xFFFFFFFE;
pub const FREE_SECT: u32 = 0xFFFFFFFF;

/// REGSECT 0x00000000 - 0xFFFFFFF9 Regular sector number.
///
/// MAXREGSECT 0xFFFFFFFA Maximum regular sector number.
///
/// Not applicable 0xFFFFFFFB Reserved for future use.
///
/// DIFSECT 0xFFFFFFFC Specifies a DIFAT sector in the FAT.
///
/// FATSECT 0xFFFFFFFD Specifies a FAT sector in the FAT.
///
/// ENDOFCHAIN 0xFFFFFFFE End of a linked chain of sectors.
///
/// FREESECT 0xFFFFFFFF Specifies an unallocated sector in the FAT, Mini FAT, or DIFAT.
#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum SectorType {
    #[brw(magic(0xFFFFFFFAu32))]
    MaxRegSect,
    #[brw(magic(0xFFFFFFFBu32))]
    NotApplicable,
    #[brw(magic(0xFFFFFFFCu32))]
    DifSect,
    #[brw(magic(0xFFFFFFFDu32))]
    FatSect,
    #[default]
    #[brw(magic(0xFFFFFFFEu32))]
    EndOfChain,
    #[brw(magic(0xFFFFFFFFu32))]
    FreeSect,
    RegularSect(u32),
}

impl Display for SectorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SectorType::MaxRegSect => write!(f, "MaxRegSect"),
            SectorType::NotApplicable => write!(f, "NotApplicable"),
            SectorType::DifSect => write!(f, "DifatSect"),
            SectorType::FatSect => write!(f, "FatSect"),
            SectorType::EndOfChain => write!(f, "EndOfChain"),
            SectorType::FreeSect => write!(f, "FreeSect"),
            SectorType::RegularSect(v) => write!(f, "{}", v)
        }
    }
}


#[binrw]
#[brw(little)]
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum MinorVersion {
    #[brw(magic(0x003Eu16))]
    MainVersion,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Eq, PartialEq, Clone, Default)]
pub enum MajorVersion {
    #[brw(magic(0x0003u16))]
    #[default]
    Version3,
    #[brw(magic(0x0004u16))]
    Version4,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Eq, PartialEq, Copy, Clone, Default)]
pub enum SectorShift {
    #[brw(magic(0x0009u16))]
    #[default]
    Shift9,
    #[brw(magic(0x000Cu16))]
    Shift12,
}
#[derive(Debug, Error)]
pub enum OleError {
    #[error("Invalid File Format")]
    InvalidFileFormat,
    #[error("Invalid Difat")]
    InvalidDifat,
    #[error("Parse Error")]
    IoError(#[from] std::io::Error),
    #[error("Parse Error")]
    ParseError(#[from] binrw::Error),
    #[error("Invalid Entry Index")]
    InvalidEntryIndex,
    #[error("Invalid Entry Size")]
    InvalidEntrySize,
    #[error("Invalid Entry Chain")]
    InvalidEntryChain,
}

pub type OleResult<T> = Result<T, OleError>;
pub fn get_valid_entries(entries: &Vec<SectorType>) -> Vec<SectorType> {
    let count = entries.len();
    let mut result = Vec::with_capacity(count);
    for entry in entries {
        if entry == &SectorType::EndOfChain {
            break;
        } else if let SectorType::RegularSect(v) = entry {
            result.push(entry.clone())
        }
    }
    result
}


pub fn get_sector_size(version: &MajorVersion) -> usize {
    if version == &MajorVersion::Version3 { 512 } else { 4096 }
}