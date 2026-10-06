#!/bin/bash
set -e

kernel="$(realpath "$1")"
cd "$(dirname "$0")"

iso_dir=../target/isodir
rm -rf "$iso_dir"
mkdir -p "$iso_dir/boot/grub"

cp "$kernel" "$iso_dir/boot/kernel"
cp ../kernel/grub.cfg "$iso_dir/boot/grub/grub.cfg"

grub-mkrescue -o ../target/kernel.iso "$iso_dir" >/dev/null

exec qemu-system-i386 \
    -cdrom ../target/kernel.iso \
    -serial stdio