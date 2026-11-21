# Contact sheet of every sprite in a level's .SLI (first frame each), for visual checks.
import struct, sys, zlib
D = open(sys.argv[1], 'rb').read()
def get(nm):
    n = struct.unpack_from('<H', D, 0x20)[0]
    for i in range(n):
        o = 0x22 + i * 24
        if D[o:o+14].split(b'\0')[0].decode().upper() == nm.upper():
            size, flag, off = struct.unpack_from('<IHI', D, o + 14)
            return D[off:off+size]
pcx = get('MENU.PCX'); pal = [x for x in pcx[-768:]]
sli = get(sys.argv[2]); n = struct.unpack_from('<H', sli, 0)[0]
tiles = []
for i in range(n):
    off, seg, param = struct.unpack_from('<HHH', sli, 2 + i * 6)
    p = 2 + seg * 16 + off
    w, h, f = struct.unpack_from('<HHH', sli, p)
    tiles.append((w, h, sli[p + 6:p + 6 + w * h]))
W = sum(t[0] + 4 for t in tiles); H = max(t[1] for t in tiles)
img = bytearray(W * H * 3); x0 = 0
for w, h, px in tiles:
    for y in range(h):
        for x in range(w):
            c = px[y * w + x]
            img[(y * W + x0 + x) * 3:(y * W + x0 + x) * 3 + 3] = bytes(pal[c*3:c*3+3]) if c else b'\x30\x30\x30'
    x0 += w + 4
raw = b''.join(b'\0' + bytes(img[y * W * 3:(y + 1) * W * 3]) for y in range(H))
def chunk(t, d): return struct.pack('>I', len(d)) + t + d + struct.pack('>I', zlib.crc32(t + d) & 0xffffffff)
open(sys.argv[3], 'wb').write(b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', W, H, 8, 2, 0, 0, 0)) + chunk(b'IDAT', zlib.compress(raw)) + chunk(b'IEND', b''))
print(n, [(t[0], t[1]) for t in tiles])
