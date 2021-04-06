# Spy.ko

New files:
- `rust/kernel/timer.rs` — Abstraction over kernel timers (WIP).
- `drivers/input/ps2_counter.rs` — Kernel module

Modified:
- `rust/kernel/bindings_helper.h` — Add bindings to interrupts and timers
- `rust/kernel/lib.rs` — Add timer.rs
- `drivers/input/Kconfig.diff`
- `drivers/input/Makefile.diff`
