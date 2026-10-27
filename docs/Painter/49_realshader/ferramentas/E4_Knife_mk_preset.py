#!/usr/bin/env python3
"""Escreve um .ph2d-brush.png a partir de um preset BASE + overrides (JSON), no formato de ph2d-brush-presets/png.rs.

uso: mk_preset.py <base.ph2d-brush.png> <overrides.json> <saida.ph2d-brush.png> [<miniatura.png>]
overrides: {"name":…, "group":…, "volumes":{…}, "engine":{…}, "brush":{…}} — engine/brush fundem-se (chaves ordenadas,
como o BTreeMap); o resto substitui. Sem miniatura, a da base.
"""
import json, struct, sys, zlib
from PIL import Image

def chunk(t, d):
    return struct.pack(">I", len(d)) + t + d + struct.pack(">I", zlib.crc32(t + d) & 0xFFFFFFFF)

def main():
    base, ov, out = sys.argv[1:4]
    im = Image.open(base)
    j = json.loads(im.info["ph2d-brush"])
    o = json.load(open(ov))
    for k, v in o.items():
        if k in ("engine", "brush", "volumes"):
            m = dict(j.get(k, {}))
            for kk, vv in v.items():
                if vv is None:
                    m.pop(kk, None)
                else:
                    m[kk] = vv
            j[k] = dict(sorted(m.items())) if k != "volumes" else m
        else:
            j[k] = v
    for k in ("engine", "brush"):
        j[k] = dict(sorted(j[k].items()))
    texto = json.dumps(j, indent=2, ensure_ascii=False)
    mini = Image.open(sys.argv[4]).convert("RGBA").resize((256, 256)) if len(sys.argv) > 4 else im.convert("RGBA")
    w, h = mini.size
    px = mini.tobytes()
    cru = b"".join(b"\0" + px[y * w * 4:(y + 1) * w * 4] for y in range(h))
    s = b"\x89PNG\r\n\x1a\n"
    s += chunk(b"IHDR", struct.pack(">II", w, h) + bytes([8, 6, 0, 0, 0]))
    s += chunk(b"tEXt", b"ph2d-origin\0factory")
    s += chunk(b"iTXt", b"ph2d-brush\0\x01\0\0\0" + zlib.compress(texto.encode(), 9))
    s += chunk(b"IDAT", zlib.compress(cru, 9))
    s += chunk(b"IEND", b"")
    open(out, "wb").write(s)
    # confere que volta
    j2 = json.loads(Image.open(out).info["ph2d-brush"])
    assert j2 == j
    print(out, j["name"])

main()
