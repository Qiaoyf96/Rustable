use std::fmt;
use std::ptr;

use traits::BlockDevice;
use vfat::Error;

#[repr(C, packed)]
pub struct BiosParameterBlock {
    first_three_bytes: [u8; 3],
    oem_identifier: [u8; 8],
    pub num_bytes_per_sector: u16,
    pub num_sectors_per_cluster: u8,
    pub num_reserved_sectors: u16,
    pub num_file_allocation_tables: u8,
    max_num_directory_entries: [ u8; 2 ],
    total_logical_sections: [ u8; 2 ],
    media_descriptor_type: u8,
    pub num_sectors_per_fat: u16,
    num_sectors_per_track: [ u8; 2 ],
    num_heads_sides: [ u8; 2 ],
    num_hidden_sectors: [ u8; 4 ],
    total_logical_sectors: [ u8; 4 ],
    //extended BPB below
    pub sectors_per_fat: u32,
    flags: [ u8; 2 ],
    fat_version_number: [ u8; 2 ],
    pub cluster_num_root_dir: u32,
    sector_num_FSInfo: [ u8; 2 ],
    sector_num_backup_boot_sector: [ u8; 2 ],
    _reserved: [ u8; 12 ],
    drive_num: u8,
    flags_win_nt: u8,
    signature: u8,
    volumeid_serial_num: [ u8; 4 ],
    volume_label_string: [ u8; 11 ],
    system_identifier_string: [ u8; 8 ],
    boot_code: [ u8; 420 ],
    bootable_partition_signature: [ u8; 2 ],
}

impl BiosParameterBlock {
    /// Reads the FAT32 extended BIOS parameter block from sector `sector` of
    /// device `device`.
    ///
    /// # Errors
    ///
    /// If the EBPB signature is invalid, returns an error of `BadSignature`.
    pub fn from<T: BlockDevice>(
        mut device: T,
        sector: u64
    ) -> Result<BiosParameterBlock, Error> {
        let mut buf = vec![];
        let read = match device.read_all_sector(sector, &mut buf) {
            Ok(read) => { read },
            Err(err) => { return Err(Error::Io(err))}
        };

        let bpb = unsafe { ptr::read( (&buf[0]) as *const u8 as *const BiosParameterBlock ) };
        if bpb.bootable_partition_signature != [0x55, 0xAA] {
            return Err(Error::BadSignature);
        }
        Ok(bpb)
    }
}

impl fmt::Debug for BiosParameterBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unimplemented!("BiosParameterBlock::debug()")
    }
}
