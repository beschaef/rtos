# Variable to get the current time. This use the shell date function
TIME=$(shell date --iso=seconds)

.PHONY: all build clean fmt run debug gdb doc

all:
	$(MAKE) build
	$(MAKE) run

build:
	bootimage build

# cleans up the workspace
clean:
	cargo clean
	#cargo update //overrides versions of crates and may cause incompatibility issues
	rm -f logs/TRACE_*

# used for formatting rust code
fmt:
	cargo fmt

run:
	@qemu-system-x86_64 -drive format=raw,file=bootimage.bin -machine q35,iommu=on -smp 4 -d cpu_reset -d guest_errors -m 1024M -cpu host -enable-kvm -serial file:logs/TRACE_$(TIME) -device isa-debug-exit,iobase=0xf4,iosize=0x04 | true

debug:
	@qemu-system-x86_64 -drive format=raw,file=bootimage.bin -s -S -m 1024M -enable-kvm -serial file:logs/TRACE_$(TIME)

# the rust-os-gdb has to be installed to use
gdb:
	@rust-os-gdb/bin/rust-gdb "target/x86_64-rtos/debug/rtos" -ex "target remote :1234"

doc:
	rm -rf target/doc/
	cargo rustdoc -- --document-private-items
	cargo rustdoc --open
