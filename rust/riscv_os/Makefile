QEMU = qemu-system-riscv64

TARGET_DEBUG = target/riscv64imac-unknown-none-elf/debug/riscv_os

QEMUOPTS = -machine virt -bios none -kernel $(TARGET_DEBUG) -m 3G -smp 3 -nographic

QEMUGDB = -gdb tcp::12345

qemu-gdb: $(TARGET_DEBUG)
	$(QEMU) $(QEMUOPTS) -S $(QEMUGDB)

$(TARGET_DEBUG):
	cargo build

