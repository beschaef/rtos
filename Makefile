TIME=$(shell date --iso=seconds)

all:
	$(MAKE) build
	$(MAKE) run

build:
	bootimage build

clean:
	cargo clean
	cargo update
	rm TRACE_*

fmt:
	cargo fmt

run:
	@qemu-system-x86_64 -drive format=raw,file=bootimage.bin -m 1024M -serial file:TRACE_$(TIME)

debug:
	@qemu-system-x86_64 -drive format=raw,file=bootimage.bin -s -S

gdb:
	@rust-os-gdb/bin/rust-gdb "target/x86_64-rtos/debug/rtos" -ex "target remote :1234"
