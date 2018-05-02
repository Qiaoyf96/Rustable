use std::ffi::OsStr;
use std::char::decode_utf16;
use std::borrow::Cow;
use std::io;

use traits;
use util::VecExt;
use vfat::{VFat, Shared, File, Cluster, Entry};
use vfat::{Metadata, Attributes, Timestamp, Time, Date};

#[derive(Debug)]
pub struct Dir {
    // FIXME: Fill me in.
    // pub first_cluster: Cluster,
    // pub vfat: Shared<VFat>,
    // pub meta: Metadata,
    // pub short_file_name: String,
    // pub lfn: String,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatRegularDirEntry {
    // FIXME: Fill me in.
    // file_name: [ u8; 8 ],
    // file_extension: [ u8; 3 ],
    // pub meta: Metadata,
    // size_file: u32,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatLfnDirEntry {
    // FIXME: Fill me in.
    // sequence_num: u8,
    // name_chars_0: [ u8; 10 ],
    // attrib: Attributes,
    // entry_type: u8,
    // checksum_dos_file_name: u8,
    // name_chars_1: [ u8; 12 ],
    // signature: u16,
    // name_chars_2: [ u8; 4 ],
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatUnknownDirEntry {
    // FIXME: Fill me in.
    // _data_0: [ u8; 11 ],
    // attrib: Attributes,
    // _data_1: [ u8; 20 ],
}

pub union VFatDirEntry {
    unknown: VFatUnknownDirEntry,
    regular: VFatRegularDirEntry,
    long_filename: VFatLfnDirEntry,
}

// impl Dir {
//     /// Finds the entry named `name` in `self` and returns it. Comparison is
//     /// case-insensitive.
//     ///
//     /// # Errors
//     ///
//     /// If no entry with name `name` exists in `self`, an error of `NotFound` is
//     /// returned.
//     ///
//     /// If `name` contains invalid UTF-8 characters, an error of `InvalidInput`
//     /// is returned.
//     // pub fn find<P: AsRef<OsStr>>(&self, name: P) -> io::Result<Entry> {
        
//     // }
// }

// FIXME: Implement `trait::Dir` for `Dir`.
