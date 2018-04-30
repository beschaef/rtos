all:
	bootimage build
fmt:
	cargo fmt
run:
	@qemu-system-x86_64 -drive format=raw,file=bootimage.bin -cpu host -enable-kvm -s

debug:
	@qemu-system-x86_64 -drive format=raw,file=bootimage.bin -cpu host -enable-kvm -s -S