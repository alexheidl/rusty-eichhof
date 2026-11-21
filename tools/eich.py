#!/usr/bin/env python3
"""Prototype decoders for The Last Eichhof's BEER.DAT assets.

Usage: python3 -I eich.py BEER.DAT BEER.EXE OUTDIR
Writes PNG previews and WAV files of every asset into OUTDIR.
"""
import os
import struct
import sys
import zlib


# ---------------------------------------------------------------- archive
def parse_dat(d):
    assert d[:28] == b"ALPHA-HELIX COMBINER VER 3.3"
    n = struct.unpack_from("<H", d, 0x20)[0]
    files = {}
    for i in range(n):
        o = 0x22 + i * 24
        name = d[o:o + 14].split(b"\0")[0].decode("ascii").upper()
        size, flag, off = struct.unpack_from("<IHI", d, o + 14)
        files[name] = (d[off:off + size], flag)
    return files


# ---------------------------------------------------------------- palette
def builtin_palette(exe):
    # DS = 0x1a97; DS:0 is at file offset 0x200 + 0xa970.  Default palette
    # (6-bit VGA values) lives at DS:0x0f5c, 768 bytes (copied by FUN_1000_6bc5).
    base = 0x200 + 0xA970 + 0x0F5C
    raw = exe[base:base + 768]
    return [tuple(v6to8(raw[i * 3 + c]) for c in range(3)) for i in range(256)]


def v6to8(v):
    v &= 0x3F
    return (v << 2) | (v >> 4)


# ---------------------------------------------------------------- PCX
def decode_pcx(d):
    xmin, ymin, xmax, ymax = struct.unpack_from("<4H", d, 4)
    bpl = struct.unpack_from("<H", d, 66)[0]
    w, h = xmax - xmin + 1, ymax - ymin + 1
    out = bytearray()
    p = 128
    total = bpl * h
    while len(out) < total:
        b = d[p]
        p += 1
        if b & 0xC0 == 0xC0:
            out += bytes([d[p]]) * (b & 0x3F)
            p += 1
        else:
            out.append(b)
    pix = bytearray()
    for y in range(h):
        pix += out[y * bpl:y * bpl + w]
    pal_off = len(d) - 769
    assert d[pal_off] == 0x0C, "no 256-colour palette marker"
    raw = d[pal_off + 1:]
    # the game shifts each byte right by 2 (8-bit -> 6-bit DAC)
    pal = [tuple(v6to8(raw[i * 3 + c] >> 2) for c in range(3)) for i in range(256)]
    return w, h, bytes(pix), pal, p, pal_off


# ---------------------------------------------------------------- sprites
def decode_sprite(d, pos=0):
    w, h, n = struct.unpack_from("<HHH", d, pos)
    frames = []
    q = pos + 6
    for _ in range(n):
        frames.append(d[q:q + w * h])
        q += w * h
    return w, h, frames, q


def decode_sli(d):
    n = struct.unpack_from("<H", d, 0)[0]
    sets = []
    for i in range(n):
        off, seg, param = struct.unpack_from("<HHH", d, 2 + i * 6)
        pos = 2 + seg * 16 + off  # far pointer relative to file+2
        w, h, frames, end = decode_sprite(d, pos)
        sets.append((param, w, h, frames))
    return sets


def ptr_table(d, entry_size=4):
    n = struct.unpack_from("<H", d, 0)[0]
    out = []
    for i in range(n):
        off, seg = struct.unpack_from("<HH", d, 2 + i * entry_size)
        out.append(2 + seg * 16 + off)
    return out


# ---------------------------------------------------------------- sound
SCALE = [0, 1, 2, 3, 4, 5, 6, 7, 0, -1, -2, -3, -4, -5, -6, -7,
         1, 3, 5, 7, 9, 11, 13, 15, -1, -3, -5, -7, -9, -11, -13, -15,
         2, 6, 10, 14, 18, 22, 26, 30, -2, -6, -10, -14, -18, -22, -26, -30,
         4, 12, 20, 28, 36, 44, 52, 60, -4, -12, -20, -28, -36, -44, -52, -60]
ADJ = [0, 0, 0, 0, 0, 16, 16, 16] * 2 + [240, 0, 0, 0, 0, 16, 16, 16] * 4 + \
      [240, 0, 0, 0, 0, 0, 0, 0] * 2


def adpcm4(data):
    ref = data[0]
    scale = 0
    out = bytearray()
    for b in data[1:]:
        for nib in (b >> 4, b & 15):
            s = max(0, min(63, nib + scale))
            ref = max(0, min(255, ref + SCALE[s]))
            scale = (scale + ADJ[s]) & 0xFF
            out.append(ref)
    return bytes(out)


def decode_snd(d, pos=0):
    prio, khz, flags, length = struct.unpack_from("<HHHI", d, pos)
    data = d[pos + 10:pos + 10 + length]
    rate = 1000000 // (1000 // khz)
    pcm = adpcm4(data) if flags & 1 else data
    return prio, khz, flags, length, rate, pcm


def wav(rate, pcm):
    return (b"RIFF" + struct.pack("<I", 36 + len(pcm)) + b"WAVEfmt " +
            struct.pack("<IHHIIHH", 16, 1, 1, rate, rate, 1, 8) +
            b"data" + struct.pack("<I", len(pcm)) + pcm +
            (b"\0" if len(pcm) & 1 else b""))


# ---------------------------------------------------------------- BLICK.PAK
def decode_pak(d):
    """640x350 16-colour image: 4 bit planes (1,2,4,8) of 28000 bytes each,
    PackBits-like RLE: c<0x80 -> repeat next byte c+1 times, else copy
    c-0x7f literal bytes. The RLE stream continues across planes."""
    planes = []
    p = 0
    for _ in range(4):
        buf = bytearray()
        while len(buf) < 28000:
            c = d[p]
            p += 1
            if c < 0x80:
                buf += bytes([d[p]]) * (c + 1)
                p += 1
            else:
                buf += d[p:p + c - 0x7F]
                p += c - 0x7F
        planes.append(buf[:28000])
    pix = bytearray(640 * 350)
    for pl in range(4):
        buf = planes[pl]
        for i in range(28000):
            b = buf[i]
            for bit in range(8):
                if b & (0x80 >> bit):
                    pix[i * 8 + bit] |= 1 << pl
    return 640, 350, bytes(pix), p


def decode_paper(d):
    """PAPER.FNT: 59 glyphs (chars 0x20..0x5a), each 5 byte-columns x 40 rows,
    1 bit per pixel, MSB = leftmost. Glyph = 40x40 px."""
    glyphs = []
    for g in range(len(d) // 200):
        pix = bytearray(40 * 40)
        for col in range(5):
            for row in range(40):
                b = d[g * 200 + col * 40 + row]
                for bit in range(8):
                    if b & (0x80 >> bit):
                        pix[row * 40 + col * 8 + bit] = 15
        glyphs.append(bytes(pix))
    return glyphs


# ---------------------------------------------------------------- PNG
def png(path, w, h, pix, pal, transparent=None, scale=1):
    rows = bytearray()
    for y in range(h * scale):
        rows.append(0)
        line = pix[(y // scale) * w:(y // scale + 1) * w]
        for x in range(w * scale):
            i = line[x // scale]
            r, g, b = pal[i]
            a = 0 if transparent is not None and i == transparent else 255
            rows += bytes((r, g, b, a))

    def chunk(t, data):
        c = struct.pack(">I", len(data)) + t + data
        return c + struct.pack(">I", zlib.crc32(t + data) & 0xFFFFFFFF)
    data = (b"\x89PNG\r\n\x1a\n" +
            chunk(b"IHDR", struct.pack(">IIBBBBB", w * scale, h * scale, 8, 6, 0, 0, 0)) +
            chunk(b"IDAT", zlib.compress(bytes(rows), 9)) + chunk(b"IEND", b""))
    with open(path, "wb") as f:
        f.write(data)


def sheet(frames, w, h, cols=16, gap=1, bg=0):
    n = len(frames)
    cols = min(cols, n)
    rows = (n + cols - 1) // cols
    W, H = cols * (w + gap), rows * (h + gap)
    pix = bytearray([bg]) * (W * H)
    for k, f in enumerate(frames):
        ox, oy = (k % cols) * (w + gap), (k // cols) * (h + gap)
        for y in range(h):
            pix[(oy + y) * W + ox:(oy + y) * W + ox + w] = f[y * w:(y + 1) * w]
    return W, H, bytes(pix)


def main():
    dat = open(sys.argv[1], "rb").read()
    exe = open(sys.argv[2], "rb").read()
    out = sys.argv[3]
    os.makedirs(out, exist_ok=True)
    files = parse_dat(dat)
    gpal = builtin_palette(exe)
    # magenta-ish background index for sheets is not available, use 0 + alpha
    for name, (d, flag) in sorted(files.items()):
        ext = name.rsplit(".", 1)[1]
        stem = os.path.join(out, name.replace(".", "_"))
        if ext == "PCX":
            w, h, pix, pal, end, pal_off = decode_pcx(d)
            print(name, w, h, "rle_end", end, "pal_off", pal_off,
                  "same_as_builtin", pal == gpal)
            png(stem + ".png", w, h, pix, pal)
        elif ext == "SPR":
            w, h, frames, end = decode_sprite(d)
            W, H, pix = sheet(frames, w, h, cols=16 if w < 40 else 1)
            png(stem + ".png", W, H, pix, gpal, transparent=0, scale=2)
            print(name, w, h, len(frames), "trailing", len(d) - end)
        elif ext == "SLI":
            for k, (param, w, h, frames) in enumerate(decode_sli(d)):
                W, H, pix = sheet(frames, w, h, cols=12)
                png("%s_%02d_p%x.png" % (stem, k, param), W, H, pix, gpal,
                    transparent=0, scale=2)
        elif ext == "SND":
            if name.startswith("LEVEL"):
                for k, p in enumerate(ptr_table(d)):
                    pr, khz, fl, ln, rate, pcm = decode_snd(d, p)
                    print(name, k, "prio", pr, "khz", khz, "flags", fl, "len", ln, "rate", rate)
                    open("%s_%02d.wav" % (stem, k), "wb").write(wav(rate, pcm))
            else:
                pr, khz, fl, ln, rate, pcm = decode_snd(d)
                print(name, "prio", pr, "khz", khz, "flags", fl, "len", ln,
                      "filelen-10", len(d) - 10, "rate", rate)
                open(stem + ".wav", "wb").write(wav(rate, pcm))
        elif name == "BLICK.PAK":
            w, h, pix, end = decode_pak(d)
            print(name, "consumed", end, "of", len(d))
            png(stem + ".png", w, h, pix, gpal)
            ega = [(0, 0, 0), (0, 0, 170), (0, 170, 0), (0, 170, 170), (170, 0, 0),
                   (170, 0, 170), (170, 85, 0), (170, 170, 170), (85, 85, 85),
                   (85, 85, 255), (85, 255, 85), (85, 255, 255), (255, 85, 85),
                   (255, 85, 255), (255, 255, 85), (255, 255, 255)] + [(0, 0, 0)] * 240
            png(stem + "_ega.png", w, h, pix, ega)
        elif name == "PAPER.FNT":
            g = decode_paper(d)
            W, H, pix = sheet(g, 40, 40, cols=10)
            png(stem + ".png", W, H, pix, gpal)
    # palette swatch
    pix = bytes(range(256))
    png(os.path.join(out, "builtin_palette.png"), 16, 16, pix, gpal, scale=16)


if __name__ == "__main__":
    main()
