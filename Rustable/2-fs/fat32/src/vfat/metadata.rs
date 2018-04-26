use std::fmt;

use traits;

/// A date as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Date(u16);

/// Time as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Time(u16);

/// File attributes as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Attributes(u8);

/// A structure containing a date and time.
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
pub struct Timestamp {
    pub date: Date,
    pub time: Time
}

/// Metadata for a directory entry.
#[derive(Default, Debug, Clone)]
pub struct Metadata {
    attrib: Attributes,
    reserved: u8,
    create_time_tenth_second: u8,
    time_create: u16,
    date_create: u16,
    date_last_access: u16,
    first_cluster_num_h: u16,
    time_last_modify: u16,
    date_last_modify: u16,
    first_cluster_num_l: u16
}

// FIXME: Implement `traits::Timestamp` for `Timestamp`.
impl traits::Timestamp for Timestamp {
    /// The year is not offset. 2009 is 2009.
    fn year(&self) -> usize {
        (((self.date.0 >> 9)  as usize & ((1 << 7) - 1 )) + 1980) as usize
    }

    /// The calendar month, starting at 1 for January. Always in range [1, 12].
    ///
    /// January is 1, Feburary is 2, ..., December is 12.
    fn month(&self) -> u8 {
        ((self.date.0 >> 5) as u8 & (( 1 << 4 ) - 1)) as u8
    }

    /// The calendar day, starting at 1. Always in range [1, 31].
    fn day(&self) -> u8 {
        ((self.date.0 as u8) & ((1 << 5 ) - 1)) as u8
    }

    /// The 24-hour hour. Always in range [0, 24).
    fn hour(&self) -> u8 {
        ((self.time.0 >> 11) as u8 & ((1 << 5) - 1)) as u8
    }

    /// The minute. Always in range [0, 60).
    fn minute(&self) -> u8 {
        ((self.time.0 >> 5) as u8 & ((1 << 6) - 1)) as u8
    }

    /// The second. Always in range [0, 60).
    fn second(&self) -> u8 {
        ((self.time.0 as u8) & ((1 << 5) - 1) * 2) as u8
    }
}
// FIXME: Implement `traits::Metadata` for `Metadata`.
impl traits::Metadata for Metadata {
    type Timestamp = Timestamp;

    fn read_only(&self) -> bool {
        (self.attrib.0 & 0x01) > 0
    }

    /// Whether the entry should be "hidden" from directory traversals.
    fn hidden(&self) -> bool {
        (self.attrib.0 & 0x02) > 0 
    }

    // fn system(&self) -> bool {
    //     (self.attrib.0 & 0x04) > 0
    // }

    // fn volume_id(&self) -> bool {
    //     (self.attrib.0 & 0x08) > 0
    // }

    // fn is_lfn_entry(&self) -> bool {
    //     self.read_only() && self.hidden() && self.system() && self.volume_id()
    // }

    // fn is_directory(&self) -> bool {
    //     (self.attrib.0 & 0x10) > 0
    // }

    // fn is_archive(&self) -> bool {
    //     (self.attrib.0 & 0x20) > 0
    // }

    /// The timestamp when the entry was created.
    fn created(&self) -> Self::Timestamp {
        Self::Timestamp {
            time: Time( self.time_create ),
            date: Date( self.date_create ),
        }
    }

    /// The timestamp for the entry's last access.
    fn accessed(&self) -> Self::Timestamp {
        Self::Timestamp {
            time: Time( 0 ),
            date: Date( self.date_last_access ),
        }
    }

    /// The timestamp for the entry's last modification.
    fn modified(&self) -> Self::Timestamp {
        Self::Timestamp {
            time: Time( self.time_last_modify ),
            date: Date( self.date_last_modify ),
        }
    }

}
// FIXME: Implement `fmt::Display` (to your liking) for `Metadata`.
impl fmt::Display for Metadata {
   
    
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        
        use traits::Metadata;
        
        f.debug_struct("Metadata")
            .field("read_only", &self.read_only() )
            .field("hidden", &self.hidden() )
            .field("timestamp_creation", &self.created() )
            .field("timestamp_last_access", &self.accessed() )
            .field("timestamp_last_modify", &self.modified() )
            .finish()
    }
}

impl fmt::Display for Timestamp {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

        use traits::Timestamp;
        
        f.debug_struct("Timestamp")
            .field("year", &self.year() )
            .field("month", &self.month() )
            .field("day", &self.day() )
            .field("hour", &self.hour() )
            .field("minute", &self.minute() )
            .field("second", &self.second() )
            .finish()
    }
}