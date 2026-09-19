#!/bin/bash
set -e

rm -rf isodir
mkdir -p isodir/boot/grub

cp "$1" isodir/boot/kernel
cp grub.cfg isodir/boot/grub/grub.cfg

grub-mkrescue -o ../target/kernel.iso isodir >/dev/null

exec qemu-system-i386 -cdrom ../target/kernel.iso