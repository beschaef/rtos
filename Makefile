build:
	bootimage build
clean:
	cargo clean
	cargo update
all:
	$(MAKE) clean
	$(MAKE) build
fmt:
	cargo fmt
run:
	@qemu-system-x86_64 -drive format=raw,file=bootimage.bin -cpu host -enable-kvm -s

debug:
	@qemu-system-x86_64 -drive format=raw,file=bootimage.bin -cpu host -enable-kvm -s -S

gdb:
	@rust-os-gdb/bin/rust-gdb "bootimage.bin" -ex "target remote :1234"