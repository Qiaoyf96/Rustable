# OS2018spring-projects-g13 Rustable
http://os.cs.tsinghua.edu.cn/oscourse/OS2018spring/projects/g13

*Rust 实现的 Raspberry Pi 3（AArch64）平台 OS*

计55 乔逸凡 2015013188

计55 谭咏霖 2015011491



本实验基于 Stanford CS140e 课程实验框架，复现了课程作业内容，并在此基础上参考 µcore 为其添加了物理内存按页分配、虚拟内存、用户进程管理、进程调度等功能，并完成了在 Raspberry Pi 3 上的真机测试。

### 环境配置

由于开发环境为 rust，首先需要安装 rust：

```shell
curl https://sh.rustup.rs -sSf | sh
```

rust 版本迭代快，因此较新的版本可能会使得本实验无法通过编译。需要将版本控制在 `nightly-2018-01-09`：

```shell
rustup default nightly-2018-01-09
rustup component add rust-src
```

接着，本实验使用 xargo 包管理器，因此需要安装 xargo：

```shell
cargo install xargo
```

实验包含汇编代码，因此需要 gcc 交叉编译环境（aarch64-none-elf)。

macOS 下可使用如下指令安装：

```shell
brew tap SergioBenitez/osxct
brew install aarch64-none-elf
```

Linux 下可使用如下指令安装：

```shell
wget https://web.stanford.edu/class/cs140e/files/aarch64-none-elf-linux-x64.tar.gz
tar -xzvf aarch64-none-elf-linux-x64.tar.gz
sudo mv aarch64-none-elf /usr/local/bin
```

并设置环境变量：

```shell
PATH="/usr/local/bin/aarch64-none-elf/bin:$PATH"
```

注：kernel 的 `ext/init.S` 在 Linux 的 aarch64-none-elf-gcc 下会报编译错误，因此 kernel 部分可能无法编译。

如此就完成了实验所需的环境配置。

#### 编译 & 运行

 `/Rustable/os/bootloader` （“伪”bootloader）、`/Rustable/os/kernel`（os kernel）、 `/Rustable/user/user` （用户程序）下均可分别编译。以`/Rustable/os/bootloader` 为例：

```shell
cd Rustable/os/bootloader
make
```

执行上述指令便可完成编译。

将上述过程生成的文件 `/Rustable/os/bootloader/build/bootloader.bin` 改名为 `kernel8.img` 并拷入插在 Raspberry Pi 3 上的 sd 卡根目录下，便可完成 bootloader 设置。（需有 CS140e 提供的 `config.txt` 和 `bootcode.bin`）

接着，将 Raspberry Pi 3 插入电脑，在 `/Rustable/os/kernel` 文件夹下便可通过如下命令编译 kernel，并将 kernel 传入 Raspberry Pi 3：

```shell
cd ../kernel
make screen
```

### 实验复现部分

##### [环境配置](https://github.com/oscourse-tsinghua/OS2018spring-projects-g13/blob/master/doc/Rustable/环境配置.md)

##### [bootloader 与 启动](https://github.com/oscourse-tsinghua/OS2018spring-projects-g13/blob/master/doc/cs140e-assignment/bootloader.md)

##### [同步互斥](https://github.com/oscourse-tsinghua/OS2018spring-projects-g13/blob/master/doc/Rustable/同步互斥.md)

##### [文件系统](https://github.com/oscourse-tsinghua/OS2018spring-projects-g13/blob/master/doc/cs140e-assignment/file_system.md)

### 移植 µcore 部分

##### [物理内存管理](https://github.com/oscourse-tsinghua/OS2018spring-projects-g13/blob/master/doc/Rustable/物理内存管理.md)

##### [虚拟内存管理](https://github.com/oscourse-tsinghua/OS2018spring-projects-g13/blob/master/doc/Rustable/虚拟内存管理.md)

##### [进程控制块](https://github.com/oscourse-tsinghua/OS2018spring-projects-g13/blob/master/doc/Rustable/进程控制块.md)

##### [用户进程管理](https://github.com/oscourse-tsinghua/OS2018spring-projects-g13/blob/master/doc/Rustable/用户进程管理.md)

##### [进程调度](https://github.com/oscourse-tsinghua/OS2018spring-projects-g13/blob/master/doc/Rustable/进程调度.md)

### [Rust 评价](https://github.com/oscourse-tsinghua/OS2018spring-projects-g13/blob/master/doc/Rustable/Rust评价.md)

