#!/bin/bash
set -e
cd "$(dirname "$0")"

# 1. Сборка
cargo +nightly build --release --target x86_64-unknown-uefi
EFI=target/x86_64-unknown-uefi/release/zq.efi

# 2. FAT-образ поменьше — 8 МБ, FAT12
dd if=/dev/zero of=efi.img bs=1M count=8 status=none
mkfs.vfat -F 12 efi.img > /dev/null
mmd -i efi.img ::/EFI
mmd -i efi.img ::/EFI/BOOT
mcopy -i efi.img "$EFI" ::/EFI/BOOT/BOOTX64.EFI

# 3. Готовим staging-директорию для ISO
rm -rf iso_root
mkdir -p iso_root/EFI/BOOT
cp "$EFI" iso_root/EFI/BOOT/BOOTX64.EFI
cp efi.img iso_root/efi.img

# 4. Собираем ISO из staging — БЕЗ мусора проекта
xorriso -as mkisofs \
    -o goob-kill-uefi.iso \
    -V "GOOB_KILL" \
    -e efi.img \
    -no-emul-boot \
    -isohybrid-gpt-basdat \
    iso_root

echo "Готово:"
ls -la goob-kill-uefi.iso
