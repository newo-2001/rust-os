#!/bin/bash
set -euo pipefail

script_dir=$(cd -- "$(dirname -- "$0")" && pwd)
kernel_elf="$script_dir/../target/i686-unknown-none/debug/kernel"

printf '%s\n' "$@" | tr -s '[:space:]' '\n' | while read -r address; do
	[[ -n "$address" ]] || continue
	printf '0x%x\n' "$address"
done | addr2line -e "$kernel_elf" -a -f -C -i -p