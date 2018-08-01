# Variable to get the current time. This uses the shell date function
TIME=$(shell date --iso=seconds)

.PHONY: all build clean fmt run debug gdb doc

# rule build and run the system
all: build run

# rule to build the os
build:
	bootimage build

# cleans up the workspace
clean:
	cargo clean
	#cargo update //overrides versions of crates and may cause incompatibility issues
	rm -rf logs/

# used for formatting rust code
fmt:
	cargo fmt

# runs os in qemu
run:
	@mkdir -p logs
	@qemu-system-x86_64 -drive format=raw,file=bootimage.bin -m 1024M -cpu host -enable-kvm -serial file:logs/TRACE_$(TIME) -device isa-debug-exit,iobase=0xf4,iosize=0x04 | true

# used for debugging, starting os stopped
debug:
	@qemu-system-x86_64 -drive format=raw,file=bootimage.bin -s -S -m 1024M -enable-kvm -serial file:logs/TRACE_$(TIME)

# the rust-os-gdb has to be installed to use
gdb:
	@rust-os-gdb/bin/rust-gdb "target/x86_64-rtos/debug/rtos" -ex "target remote :1234"

# documenting the os. Sometimes the `cargo rustdoc --open` is documenting a second time. Not everything is  documented then.
doc:
	cargo rustdoc --open -- --no-defaults --passes collapse-docs --passes unindent-comments --passes strip-priv-imports
