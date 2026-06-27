KERNEL := target/x86_64-unknown-none/debug/archbolk-kernel
ISO := archbolk.iso

.PHONY: all build iso run clean

all: iso

build:
	cargo +nightly build

iso: build
	mkdir -p iso/boot/grub
	cp $(KERNEL) iso/boot/archbolk-kernel
	cp boot/grub/grub.cfg iso/boot/grub/grub.cfg
	grub2-mkrescue -o $(ISO) iso 2>/dev/null
	rm -rf iso

run: iso
	qemu-system-x86_64 -cdrom $(ISO) -serial stdio

run-nographic: iso
	qemu-system-x86_64 -cdrom $(ISO) -nographic

clean:
	cargo +nightly clean
	rm -rf iso $(ISO)
