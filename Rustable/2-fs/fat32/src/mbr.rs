use std::{fmt, io, ptr};

use traits::BlockDevice;

#[repr(C, packed)]
#[derive(Copy, Clone)]
#[derive(Debug)]
pub struct CHS {
    head: u8,
    sector_cylinder:[u8; 2]
}

#[repr(C, packed)]
#[derive(Debug, Clone)]
pub struct PartitionEntry {
    boot_indicator: u8,
    starting_CHS: CHS,
    pub partition_type: u8,
    ending_CHS: CHS,
    pub relative_sector: u32,
    total_sectors_in_partition: u32,
}

/// The master boot record (MBR).
#[repr(C, packed)]
pub struct MasterBootRecord {
    bootstrap: [u8; 436],
    disk_id: [u8; 10],
    pub partition_table: [PartitionEntry; 4],
    signature: [u8; 2]
}

#[derive(Debug)]
pub enum Error {
    /// There was an I/O error while reading the MBR.
    Io(io::Error),
    /// Partiion `.0` (0-indexed) contains an invalid or unknown boot indicator.
    UnknownBootIndicator(u8),
    /// The MBR magic signature was invalid.
    BadSignature,
}

impl MasterBootRecord {
    /// Reads and returns the master boot record (MBR) from `device`.
    ///
    /// # Errors
    ///
    /// Returns `BadSignature` if the MBR contains an invalid magic signature.
    /// Returns `UnknownBootIndicator(n)` if partition `n` contains an invalid
    /// boot indicator. Returns `Io(err)` if the I/O error `err` occured while
    /// reading the MBR.
    pub fn from<T: BlockDevice>(mut device: T) -> Result<MasterBootRecord, Error> {
        let mut buf = vec![];
        let read = match device.read_all_sector(0, &mut buf) {
            Ok(read) => { read },
            Err(err) => { return Err(Error::Io(err))}
        };
        use std::slice;
        let mbr = unsafe { ptr::read( (&buf[0]) as *const u8 as *const MasterBootRecord ) };
        // let mbr = unsafe { slice::from_raw_parts((&buf[0]) as *const u8, 512) as MasterBootRecord};

        if mbr.signature != [0x55, 0xAA] {
            return Err(Error::BadSignature);
        }
        for i in 0..4 {
            match mbr.partition_table[i].boot_indicator {
                0x00 | 0x80 => {},
                _ => { 
                    return Err(Error::UnknownBootIndicator(i as u8)); }
            }
        }

        Ok(mbr)
    }
}

impl fmt::Debug for MasterBootRecord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct( "MasterBootRecord")
            .field("disk_id", & self.disk_id )
            .field("table_entry_0", & self.partition_table[0] )
            .field("table_entry_1", & self.partition_table[1] )
            .field("table_entry_2", & self.partition_table[2] )
            .field("table_entry_3", & self.partition_table[3] )
            .field("signature", & self.signature )
            .finish()
    }
}
