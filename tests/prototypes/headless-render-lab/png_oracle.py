"""Independent CPU-only decoder for the fixed 65x49 RGBA8 proof images."""
import struct
import zlib


def decode(path):
    data = path.read_bytes()
    assert data[:8] == b"\x89PNG\r\n\x1a\n"
    at, compressed, header = 8, bytearray(), None
    while at < len(data):
        size = struct.unpack_from(">I", data, at)[0]
        kind, payload = data[at + 4:at + 8], data[at + 8:at + 8 + size]
        crc = struct.unpack_from(">I", data, at + 8 + size)[0]
        assert zlib.crc32(kind + payload) == crc, "PNG CRC"
        if kind == b"IHDR":
            header = struct.unpack(">IIBBBBB", payload)
        elif kind == b"IDAT":
            compressed.extend(payload)
        at += size + 12
    assert header == (65, 49, 8, 6, 0, 0, 0), header
    raw = zlib.decompress(compressed)
    assert len(raw) == 49 * (1 + 65 * 4)
    rows, previous, offset = [], bytearray(260), 0
    for _ in range(49):
        filter_kind = raw[offset]
        scan = bytearray(raw[offset + 1:offset + 261])
        offset += 261
        for x in range(260):
            a = scan[x - 4] if x >= 4 else 0
            b = previous[x]
            c = previous[x - 4] if x >= 4 else 0
            if filter_kind == 0:
                predictor = 0
            elif filter_kind == 1:
                predictor = a
            elif filter_kind == 2:
                predictor = b
            elif filter_kind == 3:
                predictor = (a + b) // 2
            elif filter_kind == 4:
                p = a + b - c
                distances = (abs(p - a), abs(p - b), abs(p - c))
                predictor = (a, b, c)[distances.index(min(distances))]
            else:
                raise AssertionError("unsupported PNG filter")
            scan[x] = (scan[x] + predictor) % 256
        rows.append(bytes(scan))
        previous = scan
    return b"".join(rows)
