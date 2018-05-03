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
    pub fn read_cluster(
        &mut self,
        cluster: Cluster,
        offset: usize,
        buf: &mut [u8]
    ) -> io::Result<usize> {
        let cluster_start_sector = self.data_start_sector as usize + 
                                   cluster.data_index() * self.sectors_per_cluster as usize;
        let mut sector_index = offset / self.bytes_per_sector as usize;
        let sector_size = self.device.sector_size() as usize;
        let cluster_bytes = sector_size as usize * self.sectors_per_cluster as usize;

        // bytes of data to read
        let bytes_to_read = if buf.len() < cluster_bytes - offset {
            buf.len()
        } else {
            cluster_bytes - offset
        };

        let mut bytes_offset = offset % self.bytes_per_sector as usize;
        let mut bytes_read = 0;
        loop {
            if bytes_read >= bytes_to_read {
                break;
            }
            let sector_data : &[u8] = self.device.get( (cluster_start_sector + sector_index) as u64 )?;

            // calculate the bytes read in the current sector
            let bytes_copy = if bytes_to_read - bytes_read < sector_size - bytes_offset {
                bytes_to_read - bytes_read
            } else {
                sector_size - bytes_offset
            };
            buf[bytes_read..bytes_read+bytes_copy].copy_from_slice(&sector_data[bytes_offset..bytes_offset+bytes_copy]);

            bytes_offset = 0;
            sector_index += 1;
            bytes_read += bytes_copy;
        }

        Ok(bytes_read)


    }

    fn append_cluster_data(
        &mut self,
        cluster: Cluster,
        buf: &mut Vec<u8>
    ) -> io::Result<usize> {
        let cluster_size = self.bytes_per_sector as usize * self.sectors_per_cluster as usize;
        buf.reserve(cluster_size);

        let len_before = buf.len();

        unsafe {
            buf.set_len(len_before + cluster_size);
        }

        let bytes_read = self.read_cluster(
            cluster, 0, &mut buf[len_before..])?;

        // Set the vector back to its actual size.
        unsafe {
            buf.set_len(len_before + bytes_read);
        }

        Ok(bytes_read)
    }
    //
    //  * A method to read all of the clusters chained from a starting cluster
    //    into a vector.
    //
    pub fn read_chain(
        &mut self,
        start: Cluster,
        buf: &mut Vec<u8>
    ) -> io::Result<usize> {
        let mut cur_cluster = start;
        let mut bytes_read = 0;

        loop {
            let fat_entry = self.fat_entry(cur_cluster)?.status();

            match fat_entry {
                Status::Data(next_cluster) => {
                    bytes_read += self.append_cluster_data(cur_cluster, buf)?;
                    cur_cluster = next_cluster;
                }
                Status::Eoc(_) => {
                    bytes_read += self.append_cluster_data(cur_cluster, buf)?;
                    break;
                }
                _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid cluster entry")),
            }
        }
        Ok(bytes_read)
    }
    //
    //  * A method to return a reference to a `FatEntry` for a cluster where the
    //    reference points directly into a cached sector.
    //
    pub fn fat_entry(&mut self, cluster: Cluster) -> io::Result<&FatEntry> {
        use std::mem;
        const size : usize = mem::size_of::<FatEntry>();
        let fat_sector = cluster.fat_index() * size / self.bytes_per_sector as usize;
        let fat_index = cluster.fat_index() * size % self.bytes_per_sector as usize;
        // if fat_sector >= self.sectors_per_fat {
        //     return Err(io::Error::new(io::ErrorKind::NotFound,
        //                               "Invalid cluster index"));
        // }
        let data = self.device.get(self.fat_start_sector + fat_sector as u64)?;
        Ok(unsafe { &data[fat_index..fat_index + size].cast()[0] })
        // use std::mem;
        // use std::slice;
        
        // const s : usize = mem::size_of::<FatEntry>();
        // let sector_whole = cluster.fat_index() * s / self.bytes_per_sector as usize;
        // let bytes_remainder = cluster.fat_index() * s % self.bytes_per_sector as usize;
        // let sector_offset = self.fat_start_sector + sector_whole as u64;
        // let cached_sector_slice : &[u8] = self.device.get( sector_offset )?;
        // let fat_entry = unsafe { slice::from_raw_parts( & cached_sector_slice[bytes_remainder] as * const u8 as * const FatEntry, 1 ) };
        
        // Ok( &fat_entry[0] )
    }

    pub fn find_sector(&mut self, start: Cluster, offset: usize)
        -> io::Result<(Cluster, usize)>
    {
        let cluster_size = self.bytes_per_sector as usize
                            * self.sectors_per_cluster as usize;

        let cluster_index = offset / cluster_size;
        let mut cluster = start;

        for i in 0..cluster_index {
            let fat_entry = self.fat_entry(cluster)?.status();

            match fat_entry {
                Status::Data(next) => {
                    cluster = next;
                },
                Status::Eoc(_) => {
                    if i + 1 != cluster_index {
                        return Err(io::Error::new(
                                                io::ErrorKind::UnexpectedEof,
                                                "Data does not match size"));
                    }

                    cluster = Cluster::from(0xFFFFFFFF);
                },
                _ => return Err(io::Error::new(io::ErrorKind::InvalidData,
                                               "Invalid cluster entry")),
            }
        }

        Ok((cluster, cluster_index * cluster_size))
    }
}

impl<'a> FileSystem for &'a Shared<VFat> {
    type File = File;
    type Dir = Dir;
    type Entry = Entry;

    fn open<P: AsRef<Path>>(self, path: P) -> io::Result<Self::Entry> {
        use vfat::{Entry, Metadata};
        use std::path::Component;

        let root_cluster = self.borrow().root_dir_cluster;
        let mut dir = Entry::new_dir("".to_string(),
                                     Metadata::default(),
                                     Dir::new(root_cluster, self.clone()));

        for component in path.as_ref().components() {
            match component {
                Component::ParentDir => {
                    use traits::Entry;
                    dir = dir.into_dir().ok_or(
                        io::Error::new(io::ErrorKind::NotFound,
                                       "Expected dir"))?.find("..")?;
                },
                Component::Normal(name) => {
                    use traits::Entry;
                    dir = dir.into_dir().ok_or(
                        io::Error::new(io::ErrorKind::NotFound,
                                       "Expected dir"))?.find(name)?;
                }
                _ => (),
            }
            
        }
        Ok(dir)

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
