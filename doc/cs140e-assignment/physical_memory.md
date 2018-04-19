## 物理內存

#### 探測物理內存

ARM tags (ATAGs) 指明了有多少內存是可用的。ATAG 數組位於 `0x100` 

ATAG 有不同的類型，`CORE` 代表數組的第一個 ATAG，`MEM` 表示描述一段物理內存，`CMDLINE`

在 `os/pi/src/atags` 中

- `raw.rs`：實現了 ATAG 的結構

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

  根據 Atag Header 中 `dwords` 的大小，實現 `next()` 函數計算出下一塊 ATAG：

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

  - 使用 union 是 unsafe 的所以需要將 `raw::Atag` 封裝一層，並實現`from()` 函數把 `raw::Atag` 轉換為 `atag::Atag``
  - ``atag::Atag` 是一個 enum，是不同類型的 ATAG 塊的結構體，`from()` 函數會根據 `raw::Atag` 的類型，從 union 中以對應的結構體讀取 ATAG 的內容，並把相應的結構體返回。

- `mod.rs`：實現 ATAG 數組的 Iterator，提供接口被其他函數調用。

#### 對齊

實現了向上對齊 `align_up()` 和向下對齊 `align_down()` 函數：

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

#### 可用空間

實現了 `memory_map()` 函數查找可用的內存空間，並返回起始地址和結束地址。主要工作是要找出沒有被 kernel 占用的空間，而具體的查找可用內存、內存分配和釋放留給實現了物理內存分配算法的 Allocator 來做。

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



#### 物理內存分配算法

分別實現了 First-Fit 算法和 Buddy System 算法。

- `bump.rs`：First-Fit 算法
- `bin.rs`：Buddy System 算法