## 物理内存

#### 探测物理内存

ARM tags (ATAGs) 指明了有多少内存是可用的。ATAG 数组位于 `0x100` 

ATAG 有不同的类型，`CORE` 代表数组的第一个 ATAG，`MEM` 表示描述一段物理内存，`CMDLINE`

在 `os/pi/src/atags` 中

- `raw.rs`：实现了 ATAG 的结构

  ```rust
  pub struct Atag {
      pub dwords: u32,
      pub tag: u32,
      pub kind: Kind // Core, Mem, Cmd
  }

  pub union Kind {
      core: Core,
      mem: Mem,
      cmd: Cmd
  }
  ```

  根据 Atag Header 中 `dwords` 的大小，实现 `next()` 函数计算出下一块 ATAG：

  ```rust
  pub fn next(&self) -> Option<&Atag> {
          let curr_addr = (self as *const Atag as *const u32);
          let next_addr = unsafe{ &*(curr_addr.add(self.dwords as usize) as *const Atag) };
          if next_addr.tag == Atag::NONE {
              return None;
          }
          Some(next_addr)
      }
  ```

- `atag.rs`：

  - 使用 union 是 unsafe 的所以需要将 `raw::Atag` 封装一层，并实现`from()` 函数把 `raw::Atag` 转换为 `atag::Atag``
  - ``atag::Atag` 是一个 enum，是不同类型的 ATAG 块的结构体，`from()` 函数会根据 `raw::Atag` 的类型，从 union 中以对应的结构体读取 ATAG 的内容，并把相应的结构体返回。

- `mod.rs`：实现 ATAG 数组的 Iterator，提供接口被其他函数调用。

#### 对齐

实现了向上对齐 `align_up()` 和向下对齐 `align_down()` 函数：

```rust
pub fn align_down(addr: usize, align: usize) -> usize {
    if align == 0 || align & (align - 1) > 0 { panic!("ERROR: Align is not power of 2"); }
    addr / align * align
}

pub fn align_up(addr: usize, align: usize) -> usize {
    if align == 0 || align & (align - 1) > 0 { panic!("ERROR: Align is not power of 2"); }
    (addr + align - 1) / align * align
}
```

#### 可用空间

实现了 `memory_map()` 函数查找可用的内存空间，并返回起始地址和结束地址。主要工作是要找出没有被 kernel 占用的空间，而具体的查找可用内存、内存分配和释放留给实现了物理内存分配算法的 Allocator 来做。

```rust
fn memory_map() -> Option<(usize, usize)> {
    let binary_end = unsafe { (&_end as *const u8) as u32 };
    for atag in Atags::get() {
        match atag.mem() {
            Some(mem) => {
                let start_addr = max(mem.start, binary_end) as usize;
                let end_addr = (start_addr + mem.size as usize) as usize;
                return Some((start_addr, end_addr));
            },
            None => {}
        }
    }
    None
}
```



#### 物理内存分配算法

分别实现了 First-Fit 算法和 Buddy System 算法。

- `bump.rs`：First-Fit 算法
- `bin.rs`：Buddy System 算法