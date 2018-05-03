use std::ffi::OsStr;
use std::char::decode_utf16;
use std::borrow::Cow;
use std::io;

use traits;
use util::{VecExt, Unused};
use vfat::{VFat, Shared, File, Cluster, Entry};
use vfat::{Metadata, Attributes, Timestamp, Time, Date};

#[derive(Debug)]
pub struct Dir {
    // FIXME: Fill me in.
    start_cluster: Cluster,
    vfat: Shared<VFat> 
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatRegularDirEntry {
    // FIXME: Fill me in.
    filename: [u8; 8],
    extension: [u8; 3],
    attributes: Attributes,
    reserved: Unused<u8>,
    creation_time_subsecond: Unused<u8>,
    created: Timestamp,
    accessed: Date,
    cluster_high: u16,
    modified: Timestamp,
    cluster_low: u16,
    file_size: u32
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatLfnDirEntry {
    // FIXME: Fill me in.
    sequence_number: u8,
    name_1: [u16; 5],
    attributes: u8,
    unused_1: Unused<u8>,
    checksum: u8,
    name_2: [u16; 6],
    unused_2: Unused<u16>,
    name_3: [u16; 2],
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatUnknownDirEntry {
    // FIXME: Fill me in.
    entry_info: u8,
    unknown: Unused<[u8; 10]>,
    attributes: u8,
    unknown_2: Unused<[u8; 20]>,
}

pub union VFatDirEntry {
    unknown: VFatUnknownDirEntry,
    regular: VFatRegularDirEntry,
    long_filename: VFatLfnDirEntry,
}

pub struct DirIterator {
    data: Vec<VFatDirEntry>,
    offset: usize,
    vfat: Shared<VFat>,
}

impl VFatRegularDirEntry {
    pub fn filename(&self) -> String {
        let name = VFatRegularDirEntry::fat_string(&self.filename);

        if !self.is_dir() {
            let extension = VFatRegularDirEntry::fat_string(&self.extension);

            if !extension.is_empty() {
                let mut full_name = name.into_owned();
                full_name.push('.');
                full_name.push_str(&extension);
                return full_name;
            }
        }

        name.into_owned()
    }

    pub fn fat_string<'a>(buf: &'a [u8]) -> Cow<'a, str> {
        let mut end = 0;
        for i in 0..buf.len() {
            // A file name may be terminated early using 0x00 or 0x20 characters.
            if buf[i] == 0x00 || buf[i] == 0x20 {
                break
            }

            end += 1;
        }

        String::from_utf8_lossy(&buf[..end])
    }

    pub fn is_dir(&self) -> bool {
        self.attributes.directory()
    }

    pub fn cluster(&self) -> Cluster {
        return Cluster::from((self.cluster_high as u32) << 16
                                | self.cluster_low as u32)
    }
}

impl VFatLfnDirEntry {
    pub fn sequence_number(&self) -> usize {
        let result = self.sequence_number & 0b11111;
        assert!(result != 0);
        result as usize
    }

    pub fn append_name(&self, buf: &mut Vec<u16>) {
        let start = buf.len();
        buf.extend_from_slice(&self.name_1);
        buf.extend_from_slice(&self.name_2);
        buf.extend_from_slice(&self.name_3);

        // A file name may be terminated early using 0x00 or 0xFF characters. *from PDF
        for i in start..buf.len() {
            if buf[i] == 0x0000 || buf[i] == 0x00FF {
                buf.resize(i, 0);
                return;
            }
        }
    }

}

impl VFatUnknownDirEntry {
    const FLAG_END: u8 = 0x00;
    const FLAG_UNUSED: u8 = 0xE5;
    const FLAG_LFN: u8 = 0x0F;

    pub fn is_end(&self) -> bool {
        self.entry_info == VFatUnknownDirEntry::FLAG_END
    }

    pub fn is_unused(&self) -> bool {
        self.entry_info == VFatUnknownDirEntry::FLAG_UNUSED
    }

    pub fn is_LFN(&self) -> bool {
        self.attributes == VFatUnknownDirEntry::FLAG_LFN
    }
}

impl Dir {
    pub fn new(start_cluster: Cluster, vfat: Shared<VFat>) -> Dir {
        Dir { start_cluster, vfat }
    }

    /// Finds the entry named `name` in `self` and returns it. Comparison is
    /// case-insensitive.
    ///
    /// # Errors
    ///
    /// If no entry with name `name` exists in `self`, an error of `NotFound` is
    /// returned.
    ///
    /// If `name` contains invalid UTF-8 characters, an error of `InvalidInput`
    /// is returned.
    pub fn find<P: AsRef<OsStr>>(&self, name: P) -> io::Result<Entry> {
       use traits::{Dir, Entry};

        let name_str = name.as_ref().to_str().ok_or(
            io::Error::new(io::ErrorKind::InvalidInput, "Invalid UTF-8"))?;

        self.entries()?.find(|item| {
            item.name().eq_ignore_ascii_case(name_str)
        }).ok_or(io::Error::new(io::ErrorKind::NotFound, "Not found"))
    }
}

impl DirIterator {
    fn LFN_filename(LFN_entry: &mut Vec<&VFatLfnDirEntry>) -> String {
        LFN_entry.sort_by_key(|a| a.sequence_number());

        // LFN entry has 13 unicodes
        let mut name_string: Vec<u16> = Vec::with_capacity(13 * LFN_entry.len());
        for entry in LFN_entry.iter() {
            entry.append_name(&mut name_string);
        }

        String::from_utf16_lossy(name_string.as_slice())
    }

    pub fn create_entry(
        &self, 
        LFN_entry: &mut Vec<&VFatLfnDirEntry>,
        entry: VFatRegularDirEntry
    ) -> Entry {
        let name = if LFN_entry.is_empty() {
            entry.filename()
        } else {
            DirIterator::LFN_filename(LFN_entry)
        };

        let metadata = Metadata::new(
            entry.attributes,
            entry.created,
            Timestamp { time: Time::default(), date: entry.accessed },
            entry.modified
        );

        if entry.is_dir() {
            Entry::new_dir(name, metadata, Dir::new(entry.cluster(), self.vfat.clone()))
        } else {
            Entry::new_file(name, metadata, File::new(entry.cluster(), self.vfat.clone(), entry.file_size))
        }
    }
}

impl Iterator for DirIterator {
    type Item = Entry;

    fn next(&mut self) -> Option<Self::Item> {
        let mut LFN_entry: Vec<&VFatLfnDirEntry> = Vec::with_capacity(20);

        for offset in self.offset..self.data.len() {
            let dir_entry = &self.data[offset];

            let unknown_entry = unsafe{ dir_entry.unknown };
            if unknown_entry.is_end() {
                break;
            } 
            if unknown_entry.is_unused() {
                continue;
            }
            if unknown_entry.is_LFN() {
                // 长文件名目录项后面还会跟一个短文件名目录项，这个目录项记录了除文件名以外的这个文件的信息
                LFN_entry.push( unsafe{ &dir_entry.long_filename } );
            } else {
                self.offset = offset + 1;
                return Some(self.create_entry(&mut LFN_entry,
                                              unsafe { dir_entry.regular }));
            }
        }

        self.offset = self.data.len();
        None
    }
}



// FIXME: Implement `trait::Dir` for `Dir`.
impl traits::Dir for Dir {
    type Entry = Entry;

    /// An type that is an iterator over the entries in this directory.
    type Iter = DirIterator;

    /// Returns an interator over the entries in this directory.
    fn entries(&self) -> io::Result<Self::Iter> {
        let mut data = Vec::new();
        self.vfat.borrow_mut().read_chain(self.start_cluster, &mut data);

        Ok(DirIterator { data: unsafe { data.cast() }, 
                         offset: 0,
                         vfat: self.vfat.clone() })
    }
}