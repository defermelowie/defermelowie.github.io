+++
title = "Berkeley Host-Target Interface"
date = "2024-7-8"
[taxonomies]
tags = ["riscv", "sail"]
+++

Emulators extracted from the official [RISC-V Sail model] use the so called Host-Target Interface (HTIF) to communicate with the host machine.
This interface is originally developed at UC Berkeley for their processor/emulator prototypes, but it ended up in several RISC-V emulators such as [Spike] and QEMU (through its spike "machine").

HTIF exists out of two memory mapped registers: `tohost` and `fromhost`.
Labels for these registers should be present in the ELF-file that is provided to the emulator.
The HTIF `tohost` register contains three bitfields formatted as follows:

```
 63    56 55     48 47                                              0 
+--------+---------+-------------------------------------------------+
| device | command |                     payload                     |
+--------+---------+-------------------------------------------------+
```

The current Sail implementation supports two different devices.
- `0x00`: A terminal device
- `0x01`: A syscall proxy

## Terminal (`0x00`)

The terminal device supports two different commands:
- `0x00`: Read an ASCII encoded character 
- `0x01`: Write an ASCII encoded character

## Syscall(`0x01`)

If the least significant payload bit is `0b1`, the emulator process exits.
The remaining payload bits indicate the exit code and the command field is unused.

# Further reading

- [riscv-sodor/issues/13](https://github.com/ucb-bar/riscv-sodor/issues/13)
- [riscv-isa-sim/issues/364](https://github.com/riscv-software-src/riscv-isa-sim/issues/364#issuecomment-607657754)
- [OpenSBI implementation](https://github.com/riscv-software-src/opensbi/blob/master/lib/utils/sys/htif.c)
- [Sail implementation](https://github.com/riscv/sail-riscv/blob/master/model/sys/platform.sail)

[RISC-V Sail model]: https://github.com/riscv/sail-riscv
[Spike]: https://github.com/riscv-software-src/riscv-isa-sim
