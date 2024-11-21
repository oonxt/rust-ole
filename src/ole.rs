use crate::common::{get_sector_size, get_valid_entries, MajorVersion, OleError, OleResult, SectorType};
use crate::difat::{AllEntryDifat, Difat};
use crate::directory::{Directory, Entry, ObjectType};
use crate::fat::Fat;
use crate::header::Header;
use crate::mini_fat::MiniFat;
use binrw::BinRead;
use std::fmt::{Display, Formatter};
use std::fs;
use std::io::{Cursor, Read};
use std::slice::SliceIndex;

#[derive(Debug, Clone)]
pub struct Ole {
    pub header: Header,
    pub version: MajorVersion,
    pub difat: Vec<SectorType>,
    pub directory: Option<Vec<SectorType>>,
    pub mini_fat: Option<Vec<SectorType>>,
    pub fat: Option<Vec<SectorType>>,

    pub entries: Option<Vec<Entry>>,

    body: Vec<Vec<u8>>,
}


impl Display for Ole {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n", &self.header.to_string())?;
        self.entries.as_ref().unwrap()
            .iter()
            .for_each(|v| {
                write!(f, "{}\n", v.to_string()).unwrap();
            });

        Ok(())
    }
}
impl Ole {
    pub fn from_path(path: &str) -> OleResult<Self> {
        let buf = fs::read(path)?;

        let header = Header::read_le(&mut Cursor::new(&buf[..76]))?;
        let difat_entries = AllEntryDifat::read_le(&mut Cursor::new(&buf[76..512]))?;
        let mut relative_pos = 512usize;
        let version = header.major_version.clone();
        let sector_size = get_sector_size(&version);
        //skip all bytes between header and difat
        if version == MajorVersion::Version4 {
            let len = sector_size - 512;
            relative_pos = len * 8;
        }

        let body = buf[relative_pos..].chunks(sector_size).map(|v| v.to_vec()).collect::<Vec<Vec<u8>>>();

        Ok(Self {
            header,
            version,
            difat: get_valid_entries(&difat_entries.entries.to_vec()),
            body,
            fat: None,
            directory: None,
            mini_fat: None,
            entries: None,
        })
    }

    pub fn parse(&mut self) -> OleResult<()> {
        self.parse_difat()?;
        self.parse_fat()?;
        self.parse_mini_fat()?;
        self.parse_directory()
    }

    pub fn read(&self, entry: &Entry) -> OleResult<Vec<u8>> {
        let entry_size = entry.stream_size;

        if entry_size == 0 {
            return Err(OleError::InvalidEntrySize);
        }

        if entry_size < self.header.mini_stream_cutoff_size as u64 {
            self.get_mini_stream_data(&entry)
        } else {
            self.get_stream_data(&entry)
        }
    }

    fn parse_difat(&mut self) -> OleResult<()> {
        let count = get_sector_size(&self.version) / 4;
        let Header { first_difat_sector_location, .. } = &self.header;

        // if there are more difat sectors
        if let SectorType::RegularSect(idx) = first_difat_sector_location {
            let mut current_idx = *idx as usize;
            loop {
                let buf: &Vec<u8> = self.body.get(current_idx).ok_or(OleError::InvalidDifat)?;

                let Difat { entries, next } = Difat::read_le_args(&mut Cursor::new(&buf), (count as u16,))?;

                self.difat.extend(get_valid_entries(&entries));
                if let SectorType::RegularSect(v) = next {
                    current_idx = v as usize;
                } else {
                    break;
                }
            }
        }
        Ok(())
    }

    fn parse_fat(&mut self) -> OleResult<()> {
        let count = get_sector_size(&self.version) / 4;
        let Header { number_of_fat_sectors, .. } = &self.header;

        if *number_of_fat_sectors as usize != self.difat.len() {
            return Err(OleError::InvalidDifat);
        }

        for sector in &self.difat {
            if let SectorType::RegularSect(idx) = sector {
                let buf: &Vec<u8> = self.body.get(*idx as usize).ok_or(OleError::InvalidEntryIndex)?;
                let fat = Fat::read_le_args(&mut Cursor::new(&buf), (count as u16,))?;
                if self.fat.is_some() {
                    self.fat.as_mut().unwrap().extend(fat.entries);
                } else {
                    self.fat = Some(fat.entries);
                }
            }
        }

        Ok(())
    }

    fn parse_mini_fat(&mut self) -> OleResult<()> {
        let count = get_sector_size(&self.version) / 4;
        let Header { first_mini_fat_sector_location, .. } = &self.header;

        if let SectorType::RegularSect(_) = first_mini_fat_sector_location {
            for sector in self.get_fat_chain(first_mini_fat_sector_location) {
                if let SectorType::RegularSect(v) = sector {
                    let buf: &Vec<u8> = self.body.get(v as usize).ok_or(OleError::InvalidEntryIndex)?;
                    let mini_fat = MiniFat::read_le_args(&mut Cursor::new(&buf), (count as u16,))?;
                    if self.mini_fat.is_some() {
                        self.mini_fat.as_mut().unwrap().extend(mini_fat.entries);
                    } else {
                        self.mini_fat = Some(mini_fat.entries);
                    }
                }
            }
        }
        Ok(())
    }

    fn parse_directory(&mut self) -> OleResult<()> {
        let count = if self.version == MajorVersion::Version3 { 4 } else { 32 };

        let Header { first_directory_sector_location, mini_stream_cutoff_size, .. } = &self.header;

        if let SectorType::RegularSect(_) = first_directory_sector_location {
            let directories = self.get_fat_chain(first_directory_sector_location);
            let entries = directories.iter().flat_map(|directory| {
                if let SectorType::RegularSect(v) = directory {
                    let buf = self.body.get(*v as usize);
                    if buf.is_none() {
                        return vec![];
                    }
                    let buf = buf.unwrap();
                    let directory = match Directory::read_le_args(&mut Cursor::new(&buf), (count as u16,)) {
                        Ok(directory) => directory,
                        Err(err) => {
                            println!("Error: {}", err);
                            return vec![];
                        }
                    };

                    directory.entries.into_iter().map(|mut entry| {
                        let Entry { starting_sector_location, object_type, stream_size, .. } = &entry;
                        match object_type {
                            ObjectType::Stream => {
                                if *stream_size < *mini_stream_cutoff_size as u64 {
                                    let chain = self.get_mini_fat_chain(starting_sector_location);
                                    entry.append_chain(chain);
                                } else {
                                    let chain = self.get_fat_chain(starting_sector_location);
                                    entry.append_chain(chain);
                                }
                            }
                            ObjectType::RootStorage => {
                                let chain = self.get_fat_chain(starting_sector_location);
                                entry.append_chain(chain);
                            }
                            _ => {}
                        }
                        entry
                    }).collect::<Vec<Entry>>()
                } else {
                    vec![]
                }
            }).collect::<Vec<Entry>>();

            self.entries = Some(entries);
        }

        Ok(())
    }

    fn get_fat_chain(&self, index: &SectorType) -> Vec<SectorType> {
        let mut cur = index;
        let mut result = vec![];
        while let SectorType::RegularSect(v) = cur {
            result.push(SectorType::RegularSect(*v));
            cur = &self.fat.as_ref().unwrap()[*v as usize];
        }
        result
    }

    fn get_mini_fat_chain(&self, index: &SectorType) -> Vec<SectorType> {
        let mut cur = index;
        let mut result = vec![];
        while let SectorType::RegularSect(v) = cur {
            result.push(SectorType::RegularSect(*v));
            cur = &self.mini_fat.as_ref().unwrap()[*v as usize];
        }
        result
    }

    /// mini stream data sector chain is stored in root entry
    /// and because it's size is 64 bytes, so we should map the index in chain to a real sector index
    fn get_mini_stream_data(&self, entry: &Entry) -> OleResult<Vec<u8>> {
        let mini_sector_size = self.header.mini_sector_shift as usize;
        let sector_size = get_sector_size(&self.version);

        let count = sector_size / mini_sector_size;

        let size = entry.stream_size as usize;
        let chain = entry.chain.as_ref().ok_or(OleError::InvalidEntryChain)?;

        let mini_stream_chain = self.entries.as_ref().ok_or(OleError::InvalidEntryChain)?[0]
            .chain.as_ref().ok_or(OleError::InvalidEntryChain)?;

        let mut total_read: usize = 0;
        let mut data = vec![];
        for item in chain {
            match item {
                SectorType::RegularSect(idx) => {
                    let sector_cur = &mini_stream_chain[*idx as usize / count];
                    if let SectorType::RegularSect(v) = sector_cur {
                        let cur = *v as usize;
                        let buf: &Vec<u8> = self.body.get(cur).ok_or(OleError::InvalidEntryIndex)?;
                        let start = cur * mini_sector_size;
                        let end = start + std::cmp::min(mini_sector_size, size - total_read);
                        data.extend(&buf[start..end]);
                        total_read += end - start;
                    }
                }
                _ => {}
            }
        }

        Ok(data)
    }

    fn get_stream_data(&self, entry: &Entry) -> OleResult<Vec<u8>> {
        let size = entry.stream_size as usize;
        let sector_size = get_sector_size(&self.version);
        let chain = entry.chain.as_ref().ok_or(OleError::InvalidEntryChain)?;
        let mut total_read: usize = 0;
        let mut data = vec![];
        for item in chain {
            match item {
                SectorType::RegularSect(idx) => {
                    let cur = *idx as usize;
                    let buf: &Vec<u8> = self.body.get(cur).ok_or(OleError::InvalidEntryIndex)?;
                    let end = std::cmp::min(sector_size, size - total_read);
                    data.extend(&buf[0..end]);
                    total_read += end;
                }
                _ => {}
            }
        }

        Ok(data)
    }
}