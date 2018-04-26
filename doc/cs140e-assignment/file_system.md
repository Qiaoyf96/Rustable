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