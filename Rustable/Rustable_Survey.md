# Rustable Survey

Concentrate on the hardware(arm arch) interaction part of the os.


### bootloader

CS140e has already implemented the very beginning of the bootloader(`bootcode.bin`, `config.txt`, and `start.elf`).
These specially-named files are recognized by the Raspberry Pi’s GPU on boot-up and used to configure and boostrap the system. `bootcode.bin` is the GPU’s first-stage bootloader. Its primary job is to load `start.elf`, the GPU’s second-stage bootloader. `start.elf` initializes the ARM CPU, configuring it as indicated in `config.txt`, loads `kernel8.img` into memory, and instructs the CPU to start executing the newly loaded code from `kernel8.img`.

That is to say, the Raspberry Pi 3 loads files named `kernel8.img` at address `0x80000`. But in order to develop and debug, according to cs140e, we implemented a uart-bootloader (it's more like a kernel init func rather than the bootloader which defined in the ucore) that can receive kernel_img binaries from uart.

To maintain compatibility with default settings, we’ve chosen `0x4000000` as the start address for our uart-bootloader. we can ask the Pi to load our binary at a different address via a `kernel_address` parameter in the firmware’s `config.txt`.

Kernel’s linker script can be found in `os/kernel/ext/layout.ld`. The address `0x400000` instructs the linker to begin allocating addresses at `0x400000`.

In conclusion, our memory allocation looks like this:

```

     uart-bootloader
-------------------------- 0x400000
          kernel
-------------------------- 0x80000

-------------------------- 0x0
```

### Interrupt and exception

#### Exception Levels

At any point in time, an ARMv8 CPU is executing at a given exception level (guide: 3). Each exception level corresponds to a privilege level: the higher the exception level, the greater the privileges programs running at that level have. There are 4 exception levels:

- EL0 (user) - Typically used to run untrusted user applications.
- EL1 (kernel) - Typically used to run privileged operating system kernels.
- EL2 (hypervisor) - Typically used to run virtual machine hypervisors.
- EL3 (monitor) - Typically used to run low-level firmware.

The Raspberry Pi’s CPU boots into EL3. At that point, the firmware provided by the Raspberry Pi foundation runs, switches to EL2, and runs our kernel8.img file. Thus, our kernel starts executing in EL2. Later, we'll switch from EL2 to EL1 so that our kernel is running in the appropriate exception level.

To switch from a higher level to a lower level (a privilege decrease), the running program must return from the exception level using the `eret` instruction. Switching from a lower level to a higher level only occurs as a result of an exception.

#### Exception Vectors

When an exception occurs, the CPU jumps to the exception vector for that exception. There are 4 types of exceptions each with 4 possible exception sources for a total of 16 exception vectors. The four types of exceptions are:

- Synchronous - an exception resulting from an instruction like svc or brk
- IRQ - an asynchronous interrupt request from an external source
- FIQ - an asynchronous fast interrupt request from an external source
- SError - a “system error” interrupt

The four sources are:

- Same exception level when source SP = SP_EL0
- Same exception level when source SP = SP_ELx
- Lower exception level running on AArch64
- Lower exception level running on AArch32

#### Interrupt Handling

```
HANDLER(source, kind) ->
context_save ->
handle_exception(info: Info, esr: u32, tf: &mut TrapFrame) ->
context_restore ->
eret
```

### Physical Memory Management
#### ATAGS

specifies how much memory is available on the system

The Raspberry Pi places an array of ATAG structures at address `0x100`

```rust
struct AtagHeader {
    dwords: u32,		// the size of the complete ATAG
    					// Determine the addr of next ATAG.
    tag: u32,			// the type of the ATAG
}
```

Tags are laid out sequentially in memory with zero padding between each tag. The first tag is specified to be a `CORE` tag while the final tag is indicated by the `NONE` tag.

```
｜ core | tag 1 | tag 2 | ... | tag n | none |
```

The type of tag determines how the data after the header should be interpreted.

```
struct Mem {
    size: u32,
    start: u32
}
```

Implementation details:

```rust
pub struct Atag {		// header
    dwords: u32,		// size
    tag: u32,			// determines the interpretation of data
    kind: Kind			// the data after the header
}

pub union Kind {
    core: Core,			// First tag used to start list
    mem: Mem,			// Describes a physical area of memory
    cmd: Cmd			// Command line to pass to kernel
}
```

#### Alloc and Dealloc

##### Alignment

In C, the alignment of a memory address returned from a libC allocator is guaranteed to be 8-byte aligned on 32-bit systems and 16-byte aligned on 64-bit systems. **The caller has no control over the alignment of the returned memory address.**

```c
void *malloc(size_t size);
void free(void *pointer);
```

Rust has two unsafe functions.

**alloc**

- the allocator need to return properly aligned memory address;
- The _caller_ must ensure that
  - `layout.size() > 0` and that`layout.align()` is a power of two.

**dealloc**

- the caller need to remember the requested size and alignment of an allocation.
- The _caller_ must ensure the following:
  -  `ptr` must denote a block of memory currently allocated via this allocator
  - `layout` must properly represent the original layout used in the allocation call that returned `ptr`

```rust
unsafe fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr>;

unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout);
```

##### Thread Safety

Memory allocators like libC’s `malloc()` are *global*: they can be called by any thread at any point in time. As such, the allocator needs to be *thread safe*. Rust takes thread safety very seriously.

Thus, we need to **wrap our allocator in Mutex** ensuring that it is thread-safe by virtue of exclusion.

#### Memory Map

This `memory_map` function is called by the `Allocator::initialize()` method which in-turn is called in `kmain()`. The `initialize()` method constructs an instance of the internal `imp::Allocator` structure for use in later allocations and deallocations.

The `memory_map` function is responsible for **returning the start and end address of all of the *free* memory on the system**. Note that the amount of *free* memory is unlikely to be equal to the *total* amount of memory on the system, the latter of which is identified by ATAGS. This is because memory is already being used by data like the kernel’s binary. `memory_map` should take care not to mark used memory as free.

#### Bin Allocator

A bin segments memory allocations into bins.

- **Allocation**
  - Allocations are rounded up to the nearest bin: if there is an item in the bin’s linked list, it is **popped** and returned.
  - If there is no free memory in that bin, new memory is allocated from the global pool and returned.
- **Deallocation**
  -  Deallocation **pushes** an item to the linked list in the corresponding bin.

##### Fragmentation

- **internal fragmentation**

  The amount of memory wasted by an allocator to due to rounding up allocations. For a bin allocator, this is the difference between a request’s allocation size and the size class of the bin it is handled from.

- **external fragmentation**

  The amount of memory wasted by an allocator due to being unable to use free memory for new allocations. For a bin allocator, this is equivalent to the amount of free space in every bin that can’t be used to handle an allocation for a larger request even though the sum of all of the free space meets or exceeds the requested size.

### Virtual Memory Management

### File System

#### read-only FAT32 file system

Common file systems include EXT4 on Linux, HFS+ and APFS on macOS, and NTFS on Windows. FAT32 is another file system that is implemented by most operating systems, including Linux, macOS, and Windows, and was used in older versions of Windows and later versions of DOS. Its main advantage is its ubiquity: no other file system sees such cross-platform support.

#### Disk Layout

physical layout of an MBR-partitioned disk with a FAT32 file system

![mbr-fat-diagram](/Users/Raina/Downloads/mbr-fat-diagram.svg)

##### Master Boot Record (MBR)

| Offset | Size     | Description                         |
| ------ | -------- | ----------------------------------- |
| 446    | 64 bytes | MBR Partition Table, with 4 entries |

The MBR contains four partition entries, each indicating the partition type (the file system on the partition), the offset in sectors of the partition from the start of the disk, and a boot/active indicator that dictates whether the partition is being used by a bootable system. FAT32 partitions have a [partition type](https://en.wikipedia.org/wiki/Partition_type) of `0xB` or `0xC`.

> 硬盤分區表：64bytes
>
> - 描述分區狀態和位置
> - 每個分區描述信息占 16 bytes

##### Paritition Entry

| Offset | Size   | Description                               |
| ------ | ------ | ----------------------------------------- |
| 4      | 1 byte | Partition Type (`0xB` or `oxC` for FAT32) |

##### BPB (Bio s Parameter Block) and EBPB

The first sector of a FAT32 partition contains BPB and EBPB. These structures define the layout of the FAT file system.

“number of reserved sectors”. This is an offset from the start of the FAT32 partition

| Offset | Size    | Description                |
| ------ | ------- | -------------------------- |
| 14     | 2 bytes | Number of reserved sectors |

**data region**

Immediately after the last FAT is the data region which holds the data for *clusters*. FATs, the data region, and clusters are explained next.

##### Clusters

All data stored in a FAT file system in separated into *clusters*. The size of a cluster is determined by the “number of sectors per cluster” field of the EBPB. Clusters are numbered starting at 2.

##### File Allocation Table

*FAT* stands for “file allocation table”. As the name implies, a FAT is a table (an array) of FAT entries. In FAT32, each entry is 32-bits wide. For the sake of redundancy checking, there can be more than one FAT in a FAT32 file system. The number of FATs is determined by the EBPB.

Besides entries 0 and 1, each entry in the FAT determines the *status* of a cluster. Entry 2 determines the status of cluster 2, entry 3 the status of cluster 3, and so on. Every cluster has an associated FAT entry in the FAT.

FAT entries 0 and 1 are special:

- **Entry 0**: `0xFFFFFFFN`, an ID.
- **Entry 1**: The *end of chain* marker.

Aside from these two entries, all other entries correspond to a cluster whose data is in the data region. While FAT entries are physically 32-bits wide, only 28-bits are actually used; the upper 4 bits are ignored. The value is one of:

- `0x?0000000`: A free, unused cluster.
- `0x?0000001`: Reserved.
- `0x?0000002`-`0x?FFFFFEF`: A data cluster; value points to next cluster in chain.
- `0x?FFFFFF0`-`0x?FFFFFF6`: Reserved.
- `0x?FFFFFF7`: Bad sector in cluster or reserved cluster.
- `0x?FFFFFF8`-`0x?FFFFFFF`: Last cluster in chain. Should be, but may not be, the EOC marker.

#### Directories and Entries

A chain of clusters makes up the data for a file or directory.

Directories are special files that map file names and associated metadata to the starting cluster for a file’s date.

The root directory is the only file or directory that is not linked to via a directory entry. The starting cluster for the root directory is instead recorded in the EBPB.
