use crate::common::{MajorVersion, MinorVersion, SectorShift, SectorType};
use crate::fat::Fat;
use binrw::{binrw, BinRead, BinWrite};
use std::cmp::PartialEq;
use std::fmt::{Display, Formatter};
use crate::difat::{AllEntryDifat, Difat};

#[binrw]
#[brw(little)]
#[brw(magic(0xE11AB1A1E011CFD0u64))]
#[derive(Debug, Clone)]
pub struct Header {
    // Header Signature (8 bytes): Identification signature for the compound file structure, and MUST be set to the value 0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1.
    // pub signature: [u8; 8],

    // Header CLSID (16 bytes): Reserved and unused class ID that MUST be set to all zeroes (CLSID_NULL).
    // pub clsid: [u8; 16],

    // Minor Version (2 bytes): Version number for nonbreaking changes. This field SHOULD be set to 0x003E if the major version field is either 0x0003 or 0x0004.
    #[brw(magic(0x0000u128))]
    pub minor_version: MinorVersion,

    //Major Version (2 bytes): Version number for breaking changes. This field MUST be set to either 0x0003 (version 3) or 0x0004 (version 4).
    pub major_version: MajorVersion,

    // Byte Order (2 bytes): This field MUST be set to 0xFFFE. This field is a byte order mark for all integer fields, specifying little-endian byte order.
    // pub byte_order: u16,

    // Sector Shift (2 bytes): This field MUST be set to 0x0009, or 0x000c, depending on the Major Version field. This field specifies the sector size of the compound file as a power of 2.
    //
    // If Major Version is 3, the Sector Shift MUST be 0x0009, specifying a sector size of 512 bytes.
    //
    // If Major Version is 4, the Sector Shift MUST be 0x000C, specifying a sector size of 4096 bytes.
    #[brw(magic(0xFFFEu16))]
    pub sector_shift: u16,

    // Mini Sector Shift (2 bytes): This field MUST be set to 0x0006. This field specifies the sector size of the Mini Stream as a power of 2. The sector size of the Mini Stream MUST be 64 bytes.
    pub mini_sector_shift: u16,

    // Reserved (6 bytes): This field MUST be set to all zeroes.
    // pub reserved: [u8; 6],

    //Number of Directory Sectors (4 bytes): This integer field contains the count of the number of directory sectors in the compound file.
    //
    // If Major Version is 3, the Number of Directory Sectors MUST be zero. This field is not supported for version 3 compound files.
    #[brw(magic(b"\0\0\0\0\0\0"))]
    pub number_of_directory_sectors: u32,

    //Number of FAT Sectors (4 bytes): This integer field contains the count of the number of FAT sectors in the compound file.
    pub number_of_fat_sectors: u32,

    // First Directory Sector Location (4 bytes): This integer field contains the starting sector number for the directory stream.
    pub first_directory_sector_location: SectorType,

    // Transaction Signature Number (4 bytes): This integer field MAY contain a sequence number that is incremented every time the compound file is saved by an implementation that supports file transactions. This is the field that MUST be set to all zeroes if file transactions are not implemented.<1>
    pub transaction_signature_number: u32,
    // Mini Stream Cutoff Size (4 bytes): This integer field MUST be set to 0x00001000. This field specifies the maximum size of a user-defined data stream that is allocated from the mini FAT and mini stream, and that cutoff is 4,096 bytes. Any user-defined data stream that is greater than or equal to this cutoff size must be allocated as normal sectors from the FAT.
    pub mini_stream_cutoff_size: u32,

    // First Mini FAT Sector Location (4 bytes): This integer field contains the starting sector number for the mini FAT.
    pub first_mini_fat_sector_location: SectorType,
    // Number of Mini FAT Sectors (4 bytes): This integer field contains the count of the number of mini FAT sectors in the compound file.
    pub number_of_mini_fat_sectors: u32,
    // First DIFAT Sector Location (4 bytes): This integer field contains the starting sector number for the DIFAT.
    pub first_difat_sector_location: SectorType,
    // Number of DIFAT Sectors (4 bytes): This integer field contains the count of the number of DIFAT sectors in the compound file.
    pub number_of_difat_sectors: u32,
    // DIFAT (436 bytes): This array of 32-bit integer fields contains the first 109 FAT sector locations of the compound file.
    // For version 4 compound files, the header size (512 bytes) is less than the sector size (4,096 bytes), so the remaining part of the header (3,584 bytes) MUST be filled with all zeroes.
    // pub difat_entries: AllEntryDifat
}

impl Display for Header{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "version: {:?}", &self.major_version)
    }
}