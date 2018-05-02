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
    start: Cluster,
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
