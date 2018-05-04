# File System

实现了一个只读的 FAT32 文件系统。

## Layout

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

### BlockDevice trait

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

### CachedDevice

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

### 读取 MBR

`2-fs/fat32/src/mbr.rs`

- 使用 BlockDevice 的 `read_all_sector()` 接口来读取第 0 个扇区
- 检查是否以 `0x55AA` 结尾
- 检查分区表 (Partition Table) 每个表项的 Boot Indicator
  - `0x0`：表示没有；
  - `0x80`：表示分区是 bootable (or active) 的；
  - 其他：报错

### 读取 EBPB

`2-fs/fat32/src/vfat/ebpb.rs`

- MBR 中的分区表表项中的 Relative Sector 位指明了该分区的起始扇区，而 EBPB 就是在分区的起始扇区中，所以同样可以使用 `read_all_sector()` 接口来读取此扇区
- 检查是否以 `0x55AA` 结尾

### 实现文件系统

`2-fs/fat32/src/vfat/vfat.rs`

#### 初始化

- 读取 MBR
- 对于 MBR 分区表的每个表项，检查 Partition Type 位，如果是 `0x0B` 或 `0x0C` 则表示此分区为 FAT32 文件系统的分区
- 然后读取 EBPB
- 根据 EBPB 设置分区结构体的起始大小和扇区大小（逻辑扇区）
- 然后初始化文件系统的 CachedDevice、扇区大小、每簇扇区数、FAT 扇区数、FAT 起始扇区、数据起始扇区、根目录所在的簇。

#### 結構

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

#### 接口

```rust
fn open<P: AsRef<Path>>(self, path: P) -> io::Result<Self::Entry>
fn create_file<P: AsRef<Path>>(self, _path: P) -> io::Result<Self::File>
fn create_dir<P>(self, _path: P, _parents: bool) -> io::Result<Self::Dir>
fn rename<P, Q>(self, _from: P, _to: Q) -> io::Result<()>
fn remove<P: AsRef<Path>>(self, _path: P, _children: bool) -> io::Result<()> {
```



### 實現文件的 Metadata

`2-fs/fat32/src/vfat/metadata.rs`

Cluster 中每個目錄項保存了文件/目錄的元數據（Metadata）結構體：

```rust
pub struct Metadata {
    attributes: Attributes,
    created: Timestamp,
    accessed: Timestamp,
    modified: Timestamp,
}
```

根據不同的 offset 從硬盤中目錄項中提取出各項訊息，填入文件的 Metadata 的結構體中，其中使用了屬性、時間戳的結構體：

1. **屬性 Attributes**：該結構體用來保存目錄項中的屬性字節，目錄項中的屬性是 8 bit 所以結構體也只有一個 u8 類型的成員，其中該成員為以下不同值會表示目錄項有不同的屬性。

- READ_ONLY: `0x01`


- HIDDEN: `0x02`
- SYSTEM: `0x04`
- VOLUME_ID: `0x08`
- DIRECTORY: `0x10`
- ARCHIVE: `0x20`

2. **時間戳 Timestamp**：用以保存創建時間、創建日期、上次修改時間、上次修改日期、上次訪問日期。

```rust
pub struct Timestamp {
    pub time: Time,
    pub date: Date
}
```

使用了結構體 `Time` 和 `Date` 負責按指定數據位抽取信息：

```
15........11 10..........5 4..........0
|   hours   |   minutes   | seconds/2 |

15.........9 8...........5 4..........0
|    Year   |    Month    |    Day    |
```



### 實現 Directory

Dir 結構體是抽象保存目錄的數據結構，提供接口來對目錄進行查找。

```rust
pub struct Dir {
    start_cluster: Cluster,		// 目錄對應的起始 cluster
    vfat: Shared<VFat> 			// 目錄所在的文件系統
}
```

實現了 `entries` 函數，讀取目錄對應的 cluster 鏈的數據，並返回遍歷目錄里的目錄項的 DirIterator（後面有說明）

實現了 `find` 函數，根據給定名字，使用 `entries` 函數來遍歷目錄里的目錄項找出名字相同的 Entry（後面有說明）。其中查找是大小寫不敏感的。

#### 目錄項

和結構體 Dir 不同，目錄項是根據硬盤上實際保存的數據位分布來保存信息的數據結構。

因為 Dir 不同類型，分別是：

- Unknown Directory Entry：未知目錄項，用於判斷目錄是否有效目錄
- Regular Directory Entry：正常目錄項
- Long File Name (LFN) Entry：長文件名目錄項

使用 `union` 來保存目錄項，因為可以通過 unsafe 來以不同的結構來解析內容。

```rust
pub union VFatDirEntry {
    unknown: VFatUnknownDirEntry,
    regular: VFatRegularDirEntry,
    long_filename: VFatLfnDirEntry,
}
```

##### 正常目錄項

VFatRegularDirEntry：正常目錄項的數據位分布如下

| Offset (bytes) | Length (bytes) | Meaning                                    |
| -------------- | -------------- | ------------------------------------------ |
| 0              | 8              | 文件名（可以以 `0x00` 或 `0x20` 提早結束） |
| 8              | 3              | 文件擴展名                                 |
| 11             | 1              | 文件屬性（使用結構體 Attributes）          |
| 12             | 2              | 沒有使用                                   |
| 14             | 2              | 創建時間（使用結構體 Timestamp）           |
| 16             | 2              | 創建日期（使用結構體 Timestamp）           |
| 18             | 2              | 上次訪問日期（使用結構體 Timestamp）       |
| 20             | 2              | 數據所在的起始 Cluster 編號的高 16 位      |
| 22             | 2              | 上次修改時間（使用結構體 Timestamp）       |
| 24             | 2              | 上次修改日期（使用結構體 Timestamp）       |
| 26             | 2              | 數據所在的起始 Cluster 編號的高 16 位      |
| 28             | 4              | 文件大小（bytes）                          |

因此我們根據以上表格來構造結構體：

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

##### 長文件名目錄項

VFatLfnDirEntry：長文件名目錄項的數據位分布如下

| Offset (bytes) | Length (bytes) | Meaning                                     |
| -------------- | -------------- | ------------------------------------------- |
| 0              | 1              | 序號                                        |
| 1              | 10             | 文件名1（可以以 `0x00` 或 `0xFF` 提早結束） |
| 11             | 1              | 文件屬性（使用結構體 Attributes）           |
| 12             | 1              | 沒有使用                                    |
| 13             |                | 校驗和                                      |
| 14             | 12             | 文件名2（可以以 `0x00` 或 `0xFF` 提早結束） |
| 26             | 2              | 沒有使用                                    |
| 28             | 4              | 文件名3（可以以 `0x00` 或 `0xFF` 提早結束） |

長文件名目錄項中的文件名以 Unicode 表示，文件名可以通過把每個長文件名目錄項的三個文件名都連接起來獲得。一串长文件名目录项后面还会跟一个短文件名目录项，这个目录项记录了除文件名以外的这个文件的信息。

根據以上表格來構造結構體：

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

##### 未知目錄項

VFatUnknownDirEntry

未知目錄項只明確保存了目錄項的第一個字節和保存其屬性的字節，如以判斷此目錄性的類型。

目錄項的第一個字節：

- `0x00`：表示目錄的結束
- `0xE5`：表示沒有使用/已刪除的目錄項
- 其他情況表示正常目錄項或長文件名目錄項的序號

屬性字節：

- 如果是 `0x0F` 則表示是長文件名目錄項，其他情況表示是正常目錄項。

```rust
pub struct VFatUnknownDirEntry {
    entry_info: u8,
    unknown: Unused<[u8; 10]>,
    attributes: u8,
    unknown_2: Unused<[u8; 20]>,
}
```

#### 迭代器 DirIterator

為 Dir 實現了一個 Iterator，用來遍歷目錄里的各個項。

```rust
pub struct DirIterator {
    data: Vec<VFatDirEntry>,	// 
    offset: usize,				// 當前遍歷到的位置
    vfat: Shared<VFat>,
}
```

`data` 是保存該當前目錄的 cluster 鏈所讀出來的數據。

實現了 Iterator trait 的 `next` 函數：遍歷時，想要取得當前目錄里的下一個目錄項時，只需要從 `data` 的 `offset` 處開始找，以未知目錄項來解析數據：

- 如果表示目錄結束，則停止；
- 如果表示沒有使用或已刪除的目錄項，則不做任何處理；
- 如果是正常目錄項，則返回目錄項，更新 `offset`；
- 如果是長文件名目錄項，則壓入數組，繼續查看下一個目錄項，並更新 `offset`。直到遇到正常目錄項，就可以把這個數組返回；

同時也實現了 `create_entry` 函數，用於在遍歷時把獲得的正常目錄項或長文件名目錄項數組初始化為一個目錄或文件的Entry（Entry 將會在之後展開說明）。



### 實現 File

File 結構體是抽象保存文件的數據結構，提供接口來讀取文件。

```rust
pub struct File {
    start_cluster: Cluster,			// 文件數據起始 Cluster
    vfat: Shared<VFat>,				// 文件所在的文件系統
    size: u32,						// 文件大小
    pointer: u64,					// 讀取指針（當前位置）
    cluster_current: Cluster,		// 當前讀取的 Cluster
    cluster_current_start: usize,	// 當前讀取的 Cluster 的起始地址   
}
```

為 File 實現 `io::Read`、`io::Write` 和  `io::Seek` 使 File 有讀、寫和在把指針設在指定位置的功能。



### 實現 Entry 

Entry 是一個表示文件或目錄的結構體，是文件系統操作所使用的數據結構，其定義如下：

```rust
pub struct Entry {
    item: EntryData,
    name: String,
    metadata: Metadata,
}
```

其中 EntryData 是一個 enum 類型，表示該 Entry 是文件還是目錄，同時儲存了數據。

Entry 實現了如下的函數：

- `new_file`：給定文件名、Metadata 和 File 結構體，創建文件的 Entry
- `new_dir`：給定目錄名、Metadata 和 Dir 結構體，創建目錄的 Entry
- `name`：返回文件名或目錄名
- `metadata`：返回 Metadata 的引用
- `as_file` ：如果是一個文件的 Entry 則返回其 File 結構體的引用，否則返回 None
- `as_dir` ：如果是一個目錄的 Entry 則返回其 Dir 結構體的引用，否則返回 None
- `into_file` ：如果是一個文件的 Entry 則返回其 File 結構體，否則返回 None
- `into_dir` ：如果是一個目錄的 Entry 則返回其 Dir 結構體，否則返回 None



### 文件系統的功能

因為目前只是一個 Read-only 的文件系統，所以只實現了 `open` 函數，用於打開指定路徑。該函數使用了標準庫里的 Path 結構，它提供了 `component` 函數可以返回一個路徑拆分成目錄或文件的名字的數組。先初始化根目錄的 Entry ，遍歷這個數據，使用 Dir 的 `find` 函數來在當前目錄里根據名字來獲取相應的 Entry，並更新當前目錄，一層一層地進入目錄，直到數組結束，即可得到給定的目錄或文件的 Entry 並返回。