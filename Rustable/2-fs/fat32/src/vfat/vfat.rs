use std::io;
use std::path::Path;
use std::mem::size_of;
use std::cmp::min;

use util::SliceExt;
use mbr::MasterBootRecord;
use vfat::{Shared, Cluster, File, Dir, Entry, FatEntry, Error, Status};
use vfat::{BiosParameterBlock, CachedDevice, Partition};
use traits::{FileSystem, BlockDevice};

#[derive(Debug)]
pub struct VFat {
    device: CachedDevice,
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    sectors_per_fat: u32,
    fat_start_sector: u64,
    data_start_sector: u64,
    root_dir_cluster: Cluster,
}

impl VFat {
    pub fn from<T>(mut device: T) -> Result<Shared<VFat>, Error>
        where T: BlockDevice + 'static
    {
       let mbr = MasterBootRecord::from(&mut device)?;

        //find the first FAT
        for i in 0..4 {
            match mbr.partition_table[i].partition_type {
                0x0B | 0x0C => { 
                    // let bpb = match BiosParameterBlock::from(&mut device, mbr.partition_table[i].relative_sector as u64) {
                    //     Ok( bpb ) => { bpb },
                    //     Err( e ) => { return Err( e )}
                    // };
                    let ebpb = BiosParameterBlock::from(&mut device, mbr.partition_table[i].relative_sector as u64)?;

                    // if bpb.num_bytes_per_sector == 0 {
                    //     return Err( Error::Io( io::Error::new( io::ErrorKind::Other, "logic sector size invalid" ) ) )
                    // }

                    let partition_start = mbr.partition_table[i].relative_sector as u64;
                    let bytes_per_sector = ebpb.bytes_per_sector();

                    let cache = CachedDevice::new(device, Partition { start: partition_start,
                                                                      sector_size: bytes_per_sector as u64 });

                    let vfat = VFat {
                        device: cache,
                        bytes_per_sector,
                        sectors_per_cluster: ebpb.sectors_per_cluster(),
                        sectors_per_fat: ebpb.sectors_per_fat(),
                        fat_start_sector: partition_start + ebpb.fat_start_sector(),
                        data_start_sector: partition_start + ebpb.data_start_sector(),
                        root_dir_cluster: Cluster::from(ebpb.root_cluster()),
                    };
                    return Ok( Shared::new( vfat ) )

                },
                _ => {}
            };
        };
        Err( Error::Io( io::Error::new( io::ErrorKind::NotFound, "vfat not found") ) )
    }

    // TODO: The following methods may be useful here:
    //
    //  * A method to read from an offset of a cluster into a buffer.
    //
    fn read_cluster(
        &mut self,
        cluster: Cluster,
        offset: usize,
        buf: &mut [u8]
    ) -> io::Result<usize> {
        // let cluster_start_sector = self.data_start_sector
        //                            + cluster.data_index() as u64 * self.sectors_per_cluster as u64;
        // let mut bytes_read: usize = 0;
        // loop {
        //     let sector_index = (offset + bytes_read) as u64 / self.bytes_per_sector as u64;

        //     if sector_index >= self.sectors_per_cluster as u64 {
        //         break;
        //     } else {
        //         let byte_offset = (offset + bytes_read) as usize
        //                            - sector_index as usize * self.bytes_per_sector as usize;
        //         let data = self.device.get(cluster_start_sector + sector_index)?;
        //         assert!(byte_offset < data.len());

        //         let bytes = buf.write(&data[byte_offset..])?;
        //         bytes_read += bytes;

        //         if buf.is_empty() {
        //             break;
        //         }
        //     }
        // }

        // Ok(bytes_read)
        let sector_size = self.device.sector_size() as usize;
        let len_bytes_cluster = sector_size * self.sectors_per_cluster as usize;
        
        let mut sector = self.data_start_sector as usize +
            cluster.data_index() * self.sectors_per_cluster as usize + //data clusters starts at 2
            offset as usize / self.bytes_per_sector as usize;

        //amount of data to read
        let len_to_read = if buf.len() < len_bytes_cluster - offset {
            buf.len()
        } else {
            len_bytes_cluster - offset
        };
        //starting offset of the read
        let mut bytes_remain = offset % self.bytes_per_sector as usize;

        let mut read = 0;
        loop {

            if read >= len_to_read {
                break;
            }
            
            let sector_data : &[u8] = self.device.get( sector as u64 )?;
            
            let device_read = sector_data.len();

            //amount of data to be read from the current sector
            let len_copy = if len_to_read - read < sector_size - bytes_remain {
                len_to_read - read
            } else {
                sector_size - bytes_remain
            };
            
            buf[ read.. read + len_copy ].copy_from_slice( &sector_data[ bytes_remain.. bytes_remain + len_copy ] );

            bytes_remain = 0; //zero the offset after first read
            sector += 1;
            read += len_copy;
        }

        Ok( read )
    }
    //
    //  * A method to read all of the clusters chained from a starting cluster
    //    into a vector.
    //
    fn read_chain(
        &mut self,
        start: Cluster,
        buf: &mut Vec<u8>
    ) -> io::Result<usize> {
        let bytes_per_cluster = self.bytes_per_sector as usize * self.sectors_per_cluster as usize;
        let mut read = 0;
        let mut current = start;
        
        buf.clear();

        let mut cycle_detect = None;
        
        //check status of current fat entry
        match self.fat_entry( current )?.status() {
            Status::Data(x) => {
                cycle_detect = Some( x );
            },
            Status::Eoc(x) => {},
            _ => { return Err( io::Error::new( io::ErrorKind::InvalidData,
                                            "Invalid cluster chain" ) )
            },
        }
        
        loop {

            // println!("read chain loop");
            
            if let Some(x) = cycle_detect {
                if current.fat_index() == x.fat_index() {
                    return Err( io::Error::new( io::ErrorKind::InvalidData,
                                                "FAT cluster chain has a cycle" ) )
                }
            }

            buf.resize( read + bytes_per_cluster, 0 );

            let offset = 0;
            let bytes_read = self.read_cluster( current, offset, & mut buf[read..] )?;
            read += bytes_read;

            //advance to next cluster
            match self.fat_entry( current )?.status() {
                Status::Data( x ) => {
                    current = x;
                },
                Status::Eoc( x ) => {
                    break; //done
                },
                _ => { return Err( io::Error::new( io::ErrorKind::InvalidData,
                                                "Invalid cluster chain" ) )
                },
            }

            //advance the cycle detector twice as fast
            for _ in 0..2 {
                if let Some( x ) = cycle_detect {
                    match self.fat_entry( x )?.status() {
                        Status::Data( y ) => {
                            cycle_detect = Some( y );
                        },
                        Status::Eoc(_) => {
                            cycle_detect = None;
                        },
                        _ => { return Err( io::Error::new( io::ErrorKind::InvalidData,
                                                        "Invalid cluster chain" ) )
                        },
                    }
                }
            }
        }

        Ok( read )
    }
    //
    //  * A method to return a reference to a `FatEntry` for a cluster where the
    //    reference points directly into a cached sector.
    //
    pub fn fat_entry(&mut self, cluster: Cluster) -> io::Result<&FatEntry> {

        use std::mem;
        use std::slice;
        
        const s : usize = mem::size_of::<FatEntry>();
        let sector_whole = cluster.fat_index() * s / self.bytes_per_sector as usize;
        let bytes_remainder = cluster.fat_index() * s % self.bytes_per_sector as usize;
        let sector_offset = self.fat_start_sector + sector_whole as u64;
        let cached_sector_slice : &[u8] = self.device.get( sector_offset )?;
        let fat_entry = unsafe { slice::from_raw_parts( & cached_sector_slice[bytes_remainder] as * const u8 as * const FatEntry, 1 ) };
        
        Ok( &fat_entry[0] )
    }
}

impl<'a> FileSystem for &'a Shared<VFat> {
    type File = ::traits::Dummy;
    type Dir = ::traits::Dummy;
    type Entry = ::traits::Dummy;

    fn open<P: AsRef<Path>>(self, path: P) -> io::Result<Self::Entry> {
        unimplemented!("FileSystem::open()")
    }

    fn create_file<P: AsRef<Path>>(self, _path: P) -> io::Result<Self::File> {
        unimplemented!("read only file system")
    }

    fn create_dir<P>(self, _path: P, _parents: bool) -> io::Result<Self::Dir>
        where P: AsRef<Path>
    {
        unimplemented!("read only file system")
    }

    fn rename<P, Q>(self, _from: P, _to: Q) -> io::Result<()>
        where P: AsRef<Path>, Q: AsRef<Path>
    {
        unimplemented!("read only file system")
    }

    fn remove<P: AsRef<Path>>(self, _path: P, _children: bool) -> io::Result<()> {
        unimplemented!("read only file system")
    }
}
