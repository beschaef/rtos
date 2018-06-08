# Variable to get the current time. This use the shell date function
TIME=$(shell date --iso=seconds)

all:
	$(MAKE) build
	$(MAKE) run

build:
	bootimage build

# cleans up the workspace
clean:
	cargo clean
	cargo update
	rm TRACE_*

# used for formatting rust code
fmt:
	cargo fmt

run:
	@qemu-system-x86_64 -drive format=raw,file=bootimage.bin -m 1024M -serial file:TRACE_$(TIME)

debug:
	@qemu-system-x86_64 -drive format=raw,file=bootimage.bin -s -S

# the rust-os-gdb has to be installed to use
gdb:
	@rust-os-gdb/bin/rust-gdb "target/x86_64-rtos/debug/rtos" -ex "target remote :1234"
