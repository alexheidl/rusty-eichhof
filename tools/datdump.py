# Dumps the script tables (shots, foes, explosions, level timeline) from BEER.DAT.
import struct, sys
D = open(sys.argv[1], 'rb').read()
def get(nm):
    n = struct.unpack_from('<H', D, 0x20)[0]
    for i in range(n):
        o = 0x22 + i * 24
        if D[o:o+14].split(b'\0')[0].decode().upper() == nm.upper():
            size, flag, off = struct.unpack_from('<IHI', D, o + 14)
            return D[off:off+size]
def ptrs(b, stride=4):
    n = struct.unpack_from('<H', b, 0)[0]
    out = []
    for i in range(n):
        off, seg = struct.unpack_from('<HH', b, 2 + i * stride)
        out.append(2 + seg * 16 + off)  # relative to the byte after the count
    return out
def words(b, p, n):
    return list(struct.unpack_from('<%dh' % n, b, p))
what = sys.argv[2]; name = sys.argv[3]
b = get(name)
if what == 'ptrs':
    ps = ptrs(b)
    for i, p in enumerate(ps):
        end = ps[i+1] if i + 1 < len(ps) else len(b)
        w = words(b, p, (end - p) // 2)
        print(i, hex(p), ' '.join(('%04x' % (x & 0xffff)) if x < -0x7000 else str(x) for x in w))
elif what == 'tbl':
    n = struct.unpack_from('<H', b, 0)[0]
    for i in range(n):
        print(words(b, 2 + i * 8, 4))
