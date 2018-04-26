# File System

实现了一个只读的 FAT32 文件系统。

### Layout

![mbr-fat-diagram](./mbr-fat-diagram.svg)

如上图，为 FAT32 文件系统的格式。

#### MBR

位于硬盘第一个扇区（sector 0）。包含四个分区信息，每个分区信息包含：

- 文件系统类型；
- 起始扇区；（指向 EBPB）
- boot indicator；
- CHS

#### EBPB

包括 BPB（Bios parameter block）和 FAT 的 Layout，如 FAT 开始的 offset，每个 FAT 所占扇区数，每个扇区的字符数，FAT 的数量等。

#### FAT

FAT 重点描述了每个 cluster 在链表中的下一个 cluster 编号。其规定如下：

- `0x?0000000`: A free, unused cluster.
- `0x?0000001`: Reserved.
- `0x?0000002`-`0x?FFFFFEF`: A data cluster; value points to next cluster in chain.
- `0x?FFFFFF0`-`0x?FFFFFF6`: Reserved.
- `0x?FFFFFF7`: Bad sector in cluster or reserved cluster.
- `0x?FFFFFF8`-`0x?FFFFFFF`: Last cluster in chain. Should be, but may not be, the EOC marker.

如图，图片下边的序号是 FAT（以及对应 Cluster）的序号，图片中的内容是 FAT 所存储的数值：

![cluster-chains](./cluster-chains.svg)

#### Cluster

具体存储数据。



## 具体实现

#### BlockDevice trait

为了文件系统可以通用使用于任何物理、虚拟内存设备于是有了 BlockDevice trait。

`2-fs/fat32/src/trait/block_device.rs`

只要设备实现了 BlockDevice trait，文件系统就可以使用统一的 `read_sector()`、`write_sector()` 等接口来进行对设备的读写操作。

```rust
pub trait BlockDevice: Send {
    fn sector_size(&self) -> u64
    fn read_sector(&mut self, n: u64, buf: &mut [u8]) -> io::Result<usize>;
    fn read_all_sector(&mut self, n: u64, vec: &mut Vec<u8>) -> io::Result<usize>
    fn write_sector(&mut self, n: u64, buf: &[u8]) -> io::Result<usize>;
}
```

#### CachedDevice

`2-fs/fat32/src/vfat/cache.rs`

因为直接读取硬盘的开销很大，所以实现了 CachedDevice 来封装 BlockDevice，把扇区缓存在 HashMap 中。并实现了 `get()` 和 `get_mut()` 接口来获得缓存中的扇区，如果缓存中没有，再从硬盘中读取。

其中 Partition 是一个分区，使用逻辑扇区，其大小是硬盘中物理扇区的大小的倍数。

```rust
pub struct CachedDevice {
    device: Box<BlockDevice>,
    cache: HashMap<u64, CacheEntry>,
    partition: Partition
}
```

#### 读取 MBR

`2-fs/fat32/src/mbr.rs`

- 使用 BlockDevice 的 `read_all_sector()` 接口来读取第 0 个扇区
- 检查是否以 `0x55AA` 结尾
- 检查分区表 (Partition Table) 每个表项的 Boot Indicator
  - `0x0`：表示没有；
  - `0x80`：表示分区是 bootable (or active) 的；
  - 其他：报错

#### 读取 EBPB

`2-fs/fat32/src/vfat/ebpb.rs`

- MBR 中的分区表表项中的 Relative Sector 位指明了该分区的起始扇区，而 EBPB 就是在分区的起始扇区中，所以同样可以使用 `read_all_sector()` 接口来读取此扇区
- 检查是否以 `0x55AA` 结尾

#### 文件系统

`2-fs/fat32/src/vfat/vfat.rs`

##### 初始化

- 读取 MBR
- 对于 MBR 分区表的每个表项，检查 Partition Type 位，如果是 `0x0B` 或 `0x0C` 则表示此分区为 FAT32 文件系统的分区
- 然后读取 EBPB
- 根据 EBPB 设置分区结构体的起始大小和扇区大小（逻辑扇区）
- 然后初始化文件系统的 CachedDevice、扇区大小、每簇扇区数、FAT 扇区数、FAT 起始扇区、数据起始扇区、根目录所在的簇。

##### 结构

```rust
pub struct VFat {
    device: CachedDevice,
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    sectors_per_fat: u32,
    fat_start_sector: u64,
    data_start_sector: u64,
    root_dir_cluster: Cluster,
}
```

##### 接口

```rust
fn open<P: AsRef<Path>>(self, path: P) -> io::Result<Self::Entry>
fn create_file<P: AsRef<Path>>(self, _path: P) -> io::Result<Self::File>
fn create_dir<P>(self, _path: P, _parents: bool) -> io::Result<Self::Dir>
fn rename<P, Q>(self, _from: P, _to: Q) -> io::Result<()>
fn remove<P: AsRef<Path>>(self, _path: P, _children: bool) -> io::Result<()> {
```

#### 文件的 Metadata

`2-fs/fat32/src/vfat/metadata.rs`

Cluster 中每个 Directory Entry 保存了文件/目录的属性、时间、日期等讯息，例如：

1. 属性 Attributes：

- READ_ONLY: `0x01`


- HIDDEN: `0x02`
- DIRECTORY: `0x10`

2. 创建时间、创建日期、上次修改时间、上次修改日期、上次访问日期：

```
15........11 10..........5 4..........0
|   hours   |   minutes   | seconds/2 |

15.........9 8...........5 4..........0
|    Year   |    Month    |    Day    |
```

根据不同的 offset 提取出各项讯息，填入文件的 Metadata 的结构体中：

```rust
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
```

