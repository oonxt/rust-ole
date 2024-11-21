use crate::common::{OleResult, SectorType};
use binrw::{binrw, BinRead, BinWrite};
use std::fmt::{Display, Formatter};

/// directory sector
/// https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/a94d7445-c4be-49cd-b6b9-2f4abc663817

const MAX_REG_SID: u32 = 0xFFFFFFFA;
const NO_STREAM: u32 = 0xFFFFFFFF;

#[derive(Debug, Clone, BinRead, BinWrite)]
#[brw(little)]
#[brw(import(entry_count: u16))]
pub struct Directory {
    #[br(count = entry_count)]
    pub entries: Vec<Entry>,
}

#[derive(Debug, Clone, BinRead, BinWrite)]
#[brw(little)]
pub struct Entry {
    // Directory Entry Name (64 bytes): This field MUST contain a Unicode string for the storage or stream name encoded in UTF-16. The name MUST be terminated with a UTF-16 terminating null character. Thus, storage and stream names are limited to 32 UTF-16 code points, including the terminating null character. When locating an object in the compound file except for the root storage, the directory entry name is compared by using a special case-insensitive uppercase mapping, described in Red-Black Tree. The following characters are illegal and MUST NOT be part of the name: '/', '\', ':', '!'.
    pub name: [u8; 64],
    // Directory Entry Name Length (2 bytes): This field MUST match the length of the Directory Entry Name Unicode string in bytes. The length MUST be a multiple of 2 and include the terminating null character in the count. This length MUST NOT exceed 64, the maximum size of the Directory Entry Name field.
    pub name_length: u16,
    // Object Type (1 byte): This field MUST be 0x00, 0x01, 0x02, or 0x05, depending on the actual type of object. All other values are not valid.
    pub object_type: ObjectType,
    //Color Flag (1 byte): This field MUST be 0x00 (red) or 0x01 (black). All other values are not valid.
    pub color: Color,
    //Left Sibling ID (4 bytes): This field contains the stream ID of the left sibling. If there is no left sibling, the field MUST be set to NOSTREAM (0xFFFFFFFF).
    pub left_sibling_id: SectorType,
    //Right Sibling ID (4 bytes): This field contains the stream ID of the right sibling. If there is no right sibling, the field MUST be set to NOSTREAM (0xFFFFFFFF).
    pub right_sibling_id: SectorType,
    //Child ID (4 bytes): This field contains the stream ID of a child object. If there is no child object, including all entries for stream objects, the field MUST be set to NOSTREAM (0xFFFFFFFF).
    pub child_id: SectorType,
    //CLSID (16 bytes): This field contains an object class GUID, if this entry is for a storage object or root storage object. For a stream object, this field MUST be set to all zeroes. A value containing all zeroes in a storage or root storage directory entry is valid, and indicates that no object class is associated with the storage. If an implementation of the file format enables applications to create storage objects without explicitly setting an object class GUID, it MUST write all zeroes by default. If this value is not all zeroes, the object class GUID can be used as a parameter to start applications.
    pub clsid: [u8; 16],
    //State Bits (4 bytes): This field contains the user-defined flags if this entry is for a storage object or root storage object. For a stream object, this field SHOULD be set to all zeroes because many implementations provide no way for applications to retrieve state bits from a stream object. If an implementation of the file format enables applications to create storage objects without explicitly setting state bits, it MUST write all zeroes by default.
    pub state_bits: u32,
    //Creation Time (8 bytes): This field contains the creation time for a storage object, or all zeroes to indicate that the creation time of the storage object was not recorded. The Windows FILETIME structure is used to represent this field in UTC. For a stream object, this field MUST be all zeroes. For a root storage object, this field MUST be all zeroes, and the creation time is retrieved or set on the compound file itself.
    pub creation_time: u64,
    //Modified Time (8 bytes): This field contains the modification time for a storage object, or all zeroes to indicate that the modified time of the storage object was not recorded. The Windows FILETIME structure is used to represent this field in UTC. For a stream object, this field MUST be all zeroes. For a root storage object, this field MAY<2> be set to all zeroes, and the modified time is retrieved or set on the compound file itself.
    pub modified_time: u64,
    //Starting Sector Location (4 bytes): This field contains the first sector location if this is a stream object. For a root storage object, this field MUST contain the first sector of the mini stream, if the mini stream exists. For a storage object, this field MUST be set to all zeroes.
    pub starting_sector_location: SectorType,
    //Stream Size (8 bytes): This 64-bit integer field contains the size of the user-defined data if this is a stream object. For a root storage object, this field contains the size of the mini stream. For a storage object, this field MUST be set to all zeroes.
    // For a version 3 compound file 512-byte sector size, the value of this field MUST be less than or equal to 0x80000000. (Equivalently, this requirement can be stated: the size of a stream or of the mini stream in a version 3 compound file MUST be less than or equal to 2 gigabytes (GB).) Note that as a consequence of this requirement, the most significant 32 bits of this field MUST be zero in a version 3 compound file. However, implementers should be aware that some older implementations did not initialize the most significant 32 bits of this field, and these bits might therefore be nonzero in files that are otherwise valid version 3 compound files. Although this document does not normatively specify parser behavior, it is recommended that parsers ignore the most significant 32 bits of this field in version 3 compound files, treating it as if its value were zero, unless there is a specific reason to do otherwise (for example, a parser whose purpose is to verify the correctness of a compound file).
    pub stream_size: u64,

    #[brw(ignore)]
    pub chain: Option<Vec<SectorType>>,
}

impl Display for Entry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "name: {},\ttype: {},\tcolor: {},\tsize: {}\n", self.name(), self.object_type, self.color, self.stream_size)?;
        write!(f, "left sibling: {},\tright_sibling: {},\t", self.left_sibling_id, self.right_sibling_id)?;
        match &self.chain {
            Some(c) => write!(f, "chain: {:?}", c.iter().map(|v| v.to_string()).collect::<Vec<String>>()),
            None => write!(f, "chain: []")
        }
    }
}

impl Entry {
    pub fn name(&self) -> String {
        self.name.iter().enumerate().filter_map(|(i, v): (usize, &u8)| {
            if v != &0 && i % 2 == 0 {
                Some(*v as char)
            } else {
                None
            }
        }).collect::<String>()
    }

    pub fn parse(&mut self) {}

    pub fn append_chain(&mut self, sector: Vec<SectorType>) {
        if self.chain.is_some() {
            self.chain.as_mut().unwrap().extend(sector);
        } else {
            self.chain = Some(sector);
        }
    }
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub enum ObjectType {
    #[brw(magic(0x00u8))]
    Unknown,
    #[brw(magic(0x01u8))]
    Storage,
    #[brw(magic(0x02u8))]
    Stream,
    #[brw(magic(0x05u8))]
    RootStorage,
}

impl Display for ObjectType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectType::Unknown => write!(f, "unknown"),
            ObjectType::Storage => write!(f, "storage"),
            ObjectType::Stream => write!(f, "stream"),
            ObjectType::RootStorage => write!(f, "root storage")
        }
    }
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub enum Color {
    #[brw(magic(0x00u8))]
    Red,
    #[brw(magic(0x01u8))]
    Black,
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Color::Red => write!(f, "red"),
            Color::Black => write!(f, "black")
        }
    }
}