# Build as Library

## Instructions

```console
$ sudo apt install build-essential

$ cargo build --lib
   Compiling ifstat-rs v3.0.0 (/home/q/ifstat-rs)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.59s

$ file target/debug/libifstat_rs.so
target/debug/libifstat_rs.so: ELF 64-bit LSB shared object, x86-64, version 1 (SYSV), dynamically linked, BuildID[sha1]=e67b0c1d9d1a18a28cd0341a961fff3ded1d29f7, with debug_info, not stripped

$ objdump -x target/debug/libifstat_rs.so|grep GetNet
0000000000015750 g     F .text  0000000000000141              GetNetDevStats
$ objdump -x target/debug/libifstat_rs.so|grep FreeCStr
00000000000158a0 g     F .text  000000000000003b              FreeCString
```