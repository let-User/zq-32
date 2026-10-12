#!/usr/bin/env python3
"""Конвертирует PSF1/PSF2 8x16 в бинарник 8x16 (4096 байт)."""
import gzip, struct, os

SRC = "/usr/share/kbd/consolefonts/default8x16.psfu.gz"
DST = "fonts/font8x16.bin"

os.makedirs("fonts", exist_ok=True)

with gzip.open(SRC, "rb") as f:
    data = f.read()

magic = struct.unpack("<I", data[:4])[0]

if magic == 0x0436:  # PSF1
    mode, charsize = struct.unpack("<BB", data[2:4])
    num_glyphs = 512 if (mode & 0x01) else 256
    glyph_offset = 4
    print(f"PSF1: charsize={charsize}, glyphs={num_glyphs}")
    assert charsize == 16, f"expected 16 bytes/char, got {charsize}"
    glyphs = data[glyph_offset : glyph_offset + 16 * num_glyphs]

elif magic == 0x864AB572:  # PSF2
    version, headersize, flags, numglyphs, bytesperglyph, height, width = \
        struct.unpack("<IIIIIII", data[4:32])
    print(f"PSF2: version={version}, glyphs={numglyphs}, "
          f"bytes/glyph={bytesperglyph}, {width}x{height}")
    assert width == 8 and height == 16, \
        f"expected 8x16, got {width}x{height}"
    glyphs = data[headersize : headersize + bytesperglyph * numglyphs]

else:
    raise SystemExit(f"unknown magic: {magic:#010x}")

# Нужны только первые 256 глифов
out = glyphs[:256 * 16]
assert len(out) == 4096, f"got {len(out)} bytes instead of 4096"

with open(DST, "wb") as f:
    f.write(out)

print(f"Готово: {DST} ({len(out)} байт)")
