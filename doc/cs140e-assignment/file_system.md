## File System

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



### 具体实现

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

#### 实现文件系统

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



#### 实现文件的 Metadata

`2-fs/fat32/src/vfat/metadata.rs`

Cluster 中每个目录项保存了文件/目录的元数据（Metadata）结构体：

```rust
pub struct Metadata {
    attributes: Attributes,
    created: Timestamp,
    accessed: Timestamp,
    modified: Timestamp,
}
```

根据不同的 offset 从硬盘中目录项中提取出各项讯息，填入文件的 Metadata 的结构体中，其中使用了属性、时间戳的结构体：

1. **属性 Attributes**：该结构体用来保存目录项中的属性字节，目录项中的属性是 8 bit 所以结构体也只有一个 u8 类型的成员，其中该成员为以下不同值会表示目录项有不同的属性。

- READ_ONLY: `0x01`


- HIDDEN: `0x02`
- SYSTEM: `0x04`
- VOLUME_ID: `0x08`
- DIRECTORY: `0x10`
- ARCHIVE: `0x20`

2. **时间戳 Timestamp**：用以保存创建时间、创建日期、上次修改时间、上次修改日期、上次访问日期。

```rust
pub struct Timestamp {
    pub time: Time,
    pub date: Date
}
```

使用了结构体 `Time` 和 `Date` 负责按指定数据位抽取信息：

```
15........11 10..........5 4..........0
|   hours   |   minutes   | seconds/2 |

15.........9 8...........5 4..........0
|    Year   |    Month    |    Day    |
```



#### 实现 Directory

Dir 结构体是抽象保存目录的数据结构，提供接口来对目录进行查找。

```rust
pub struct Dir {
    start_cluster: Cluster,		// 目录对应的起始 cluster
    vfat: Shared<VFat> 			// 目录所在的文件系统
}
```

实现了 `entries` 函数，读取目录对应的 cluster 链的数据，并返回遍历目录里的目录项的 DirIterator（后面有说明）

实现了 `find` 函数，根据给定名字，使用 `entries` 函数来遍历目录里的目录项找出名字相同的 Entry（后面有说明）。其中查找是大小写不敏感的。

##### 目录项

和结构体 Dir 不同，目录项是根据硬盘上实际保存的数据位分布来保存信息的数据结构。

因为 Dir 不同类型，分别是：

- Unknown Directory Entry：未知目录项，用于判断目录是否有效目录
- Regular Directory Entry：正常目录项
- Long File Name (LFN) Entry：长文件名目录项

使用 `union` 来保存目录项，因为可以通过 unsafe 来以不同的结构来解析内容。

```rust
pub union VFatDirEntry {
    unknown: VFatUnknownDirEntry,
    regular: VFatRegularDirEntry,
    long_filename: VFatLfnDirEntry,
}
```

**正常目录项**

VFatRegularDirEntry：正常目录项的数据位分布如下

| Offset (bytes) | Length (bytes) | Meaning                                    |
| -------------- | -------------- | ------------------------------------------ |
| 0              | 8              | 文件名（可以以 `0x00` 或 `0x20` 提早结束） |
| 8              | 3              | 文件扩展名                                 |
| 11             | 1              | 文件属性（使用结构体 Attributes）          |
| 12             | 2              | 没有使用                                   |
| 14             | 2              | 创建时间（使用结构体 Timestamp）           |
| 16             | 2              | 创建日期（使用结构体 Timestamp）           |
| 18             | 2              | 上次访问日期（使用结构体 Timestamp）       |
| 20             | 2              | 数据所在的起始 Cluster 编号的高 16 位      |
| 22             | 2              | 上次修改时间（使用结构体 Timestamp）       |
| 24             | 2              | 上次修改日期（使用结构体 Timestamp）       |
| 26             | 2              | 数据所在的起始 Cluster 编号的高 16 位      |
| 28             | 4              | 文件大小（bytes）                          |

因此我们根据以上表格来构造结构体：

```rust
pub struct VFatRegularDirEntry {
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
```

**长文件名目录项**

VFatLfnDirEntry：长文件名目录项的数据位分布如下

| Offset (bytes) | Length (bytes) | Meaning                                     |
| -------------- | -------------- | ------------------------------------------- |
| 0              | 1              | 序号                                        |
| 1              | 10             | 文件名1（可以以 `0x00` 或 `0xFF` 提早结束） |
| 11             | 1              | 文件属性（使用结构体 Attributes）           |
| 12             | 1              | 没有使用                                    |
| 13             |                | 校验和                                      |
| 14             | 12             | 文件名2（可以以 `0x00` 或 `0xFF` 提早结束） |
| 26             | 2              | 没有使用                                    |
| 28             | 4              | 文件名3（可以以 `0x00` 或 `0xFF` 提早结束） |

长文件名目录项中的文件名以 Unicode 表示，文件名可以通过把每个长文件名目录项的三个文件名都连接起来获得。一串长文件名目录项后面还会跟一个短文件名目录项，这个目录项记录了除文件名以外的这个文件的信息。

根据以上表格来构造结构体：

```rust
pub struct VFatLfnDirEntry {
    sequence_number: u8,
    name_1: [u16; 5],
    attributes: u8,
    unused_1: Unused<u8>,
    checksum: u8,
    name_2: [u16; 6],
    unused_2: Unused<u16>,
    name_3: [u16; 2],
}
```

**未知目录项**

VFatUnknownDirEntry

未知目录项只明确保存了目录项的第一个字节和保存其属性的字节，如以判断此目录性的类型。

目录项的第一个字节：

- `0x00`：表示目录的结束
- `0xE5`：表示没有使用/已删除的目录项
- 其他情况表示正常目录项或长文件名目录项的序号

属性字节：

- 如果是 `0x0F` 则表示是长文件名目录项，其他情况表示是正常目录项。

```rust
pub struct VFatUnknownDirEntry {
    entry_info: u8,
    unknown: Unused<[u8; 10]>,
    attributes: u8,
    unknown_2: Unused<[u8; 20]>,
}
```

##### 迭代器 DirIterator

为 Dir 实现了一个 Iterator，用来遍历目录里的各个项。

```rust
pub struct DirIterator {
    data: Vec<VFatDirEntry>,	// 
    offset: usize,				// 当前遍历到的位置
    vfat: Shared<VFat>,
}
```

`data` 是保存该当前目录的 cluster 链所读出来的数据。

实现了 Iterator trait 的 `next` 函数：遍历时，想要取得当前目录里的下一个目录项时，只需要从 `data` 的 `offset` 处开始找，以未知目录项来解析数据：

- 如果表示目录结束，则停止；
- 如果表示没有使用或已删除的目录项，则不做任何处理；
- 如果是正常目录项，则返回目录项，更新 `offset`；
- 如果是长文件名目录项，则压入数组，继续查看下一个目录项，并更新 `offset`。直到遇到正常目录项，就可以把这个数组返回；

同时也实现了 `create_entry` 函数，用于在遍历时把获得的正常目录项或长文件名目录项数组初始化为一个目录或文件的Entry（Entry 将会在之后展开说明）。



#### 实现 File

File 结构体是抽象保存文件的数据结构，提供接口来读取文件。

```rust
pub struct File {
    start_cluster: Cluster,			// 文件数据起始 Cluster
    vfat: Shared<VFat>,				// 文件所在的文件系统
    size: u32,						// 文件大小
    pointer: u64,					// 读取指针（当前位置）
    cluster_current: Cluster,		// 当前读取的 Cluster
    cluster_current_start: usize,	// 当前读取的 Cluster 的起始地址   
}
```

为 File 实现 `io::Read`、`io::Write` 和  `io::Seek` 使 File 有读、写和在把指针设在指定位置的功能。



#### 实现 Entry 

Entry 是一个表示文件或目录的结构体，是文件系统操作所使用的数据结构，其定义如下：

```rust
pub struct Entry {
    item: EntryData,
    name: String,
    metadata: Metadata,
}
```

其中 EntryData 是一个 enum 类型，表示该 Entry 是文件还是目录，同时储存了数据。

Entry 实现了如下的函数：

- `new_file`：给定文件名、Metadata 和 File 结构体，创建文件的 Entry
- `new_dir`：给定目录名、Metadata 和 Dir 结构体，创建目录的 Entry
- `name`：返回文件名或目录名
- `metadata`：返回 Metadata 的引用
- `as_file` ：如果是一个文件的 Entry 则返回其 File 结构体的引用，否则返回 None
- `as_dir` ：如果是一个目录的 Entry 则返回其 Dir 结构体的引用，否则返回 None
- `into_file` ：如果是一个文件的 Entry 则返回其 File 结构体，否则返回 None
- `into_dir` ：如果是一个目录的 Entry 则返回其 Dir 结构体，否则返回 None



#### 文件系统的功能

因为目前只是一个 Read-only 的文件系统，所以只实现了 `open` 函数，用于打开指定路径。该函数使用了标准库里的 Path 结构，它提供了 `component` 函数可以返回一个路径拆分成目录或文件的名字的数组。先初始化根目录的 Entry ，遍历这个数据，使用 Dir 的 `find` 函数来在当前目录里根据名字来获取相应的 Entry，并更新当前目录，一层一层地进入目录，直到数组结束，即可得到给定的目录或文件的 Entry 并返回。