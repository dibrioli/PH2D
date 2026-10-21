#!/usr/bin/env python3
"""A régua do portão por foto da família `aerografo` (J3) — as MESMAS contas de `Identidade::de` em
`crates/ph2d-tool-painter/tests/it/familias/aerografo.rs`, na cena `gestos` (A) da bateria.

uso: aerografo_foto.py <png|ppm> [...]   → uma linha por foto com a identidade

A cobertura = máx|c − 255| ÷ (255 − 23) (o azul cheio = 1). A faixa do recto lento: colunas 150..470, linhas 0..180
(o recto está a y 90; o spray do Krita espalha até ±45). O perfil = a média por linha; a LARGURA = as linhas com
perfil ≥ 0,25 do pico; a COBERTURA = a média do perfil dentro da largura; o VAZIO = a fracção de píxeis do miolo
(a metade central da largura) com cobertura < 0,1; os PONTOS = as manchas (8-vizinhança, cobertura ≥ 0,35) por
100 px de comprimento; a GOTA = a área mediana dessas manchas (px); o MATIZ = o desvio-padrão do matiz (graus) dos
píxeis com cobertura ≥ 0,5; a RAMPA = a cobertura média da rampa 0→1 a x 150..200 ÷ a de x 480..530 (linhas 340..460).
"""
import struct
import sys
import zlib

CHEIO = 255.0 - 23.0


def le(p):
    d = open(p, 'rb').read()
    if d[:2] == b'P6':
        parts = d.split(b'\n', 3)
        w, h = map(int, parts[1].split())
        return w, h, 3, parts[3]
    i, w, h, ct, idat = 8, 0, 0, 0, b''
    while i < len(d):
        n = struct.unpack('>I', d[i:i + 4])[0]
        t = d[i + 4:i + 8]
        c = d[i + 8:i + 8 + n]
        if t == b'IHDR':
            w, h, bd, ct = struct.unpack('>IIBB', c[:10])
            assert bd == 8, p
        elif t == b'IDAT':
            idat += c
        elif t == b'PLTE':
            pal = c
        i += 12 + n
    raw = zlib.decompress(idat)
    bpp = {2: 3, 6: 4, 0: 1, 4: 2, 3: 1}[ct]
    out = bytearray(w * h * bpp)
    prev = bytearray(w * bpp)
    k = 0
    for y in range(h):
        f = raw[k]
        k += 1
        line = bytearray(raw[k:k + w * bpp])
        k += w * bpp
        for x in range(w * bpp):
            a = line[x - bpp] if x >= bpp else 0
            b = prev[x]
            cc = prev[x - bpp] if x >= bpp else 0
            if f == 1:
                line[x] = (line[x] + a) & 255
            elif f == 2:
                line[x] = (line[x] + b) & 255
            elif f == 3:
                line[x] = (line[x] + (a + b) // 2) & 255
            elif f == 4:
                pa, pb, pc = abs(b - cc), abs(a - cc), abs(a + b - 2 * cc)
                line[x] = (line[x] + (a if pa <= pb and pa <= pc else b if pb <= pc else cc)) & 255
        out[y * w * bpp:(y + 1) * w * bpp] = line
        prev = line
    if ct == 3:
        rgb = bytearray()
        for v in out:
            rgb += pal[3 * v:3 * v + 3]
        return w, h, 3, bytes(rgb)
    if ct in (0, 4):
        rgb = bytearray()
        for j in range(0, len(out), bpp):
            rgb += bytes([out[j]] * 3)
        return w, h, 3, bytes(rgb)
    return w, h, bpp, bytes(out)


def identidade(path):
    w, h, bpp, px = le(path)

    def rgb(x, y):
        j = (y * w + x) * bpp
        return px[j], px[j + 1], px[j + 2]

    def c(x, y):
        return max(abs(255 - v) for v in rgb(x, y)) / CHEIO

    x0, x1, y0, y1 = 150, 470, 0, 180
    prof = [sum(c(x, y) for x in range(x0, x1)) / (x1 - x0) for y in range(y0, y1)]
    pico = max(prof)
    dentro = [y0 + i for i, v in enumerate(prof) if v >= 0.25 * pico]
    largura = len(dentro)
    cob = sum(prof[y - y0] for y in dentro) / max(1, largura)
    ya, yb = dentro[0], dentro[-1]
    q = (yb - ya) / 4.0
    miolo = [y for y in range(y0, y1) if ya + q <= y <= yb - q]
    vaz = [c(x, y) < 0.1 for y in miolo for x in range(x0, x1)]
    vazio = sum(vaz) / max(1, len(vaz))
    # manchas
    seen = set()
    areas = []
    for y in range(y0, y1):
        for x in range(x0, x1):
            if (x, y) in seen or c(x, y) < 0.35:
                continue
            pilha, a = [(x, y)], 0
            seen.add((x, y))
            while pilha:
                u, v = pilha.pop()
                a += 1
                for du in (-1, 0, 1):
                    for dv in (-1, 0, 1):
                        uu, vv = u + du, v + dv
                        if x0 <= uu < x1 and y0 <= vv < y1 and (uu, vv) not in seen and c(uu, vv) >= 0.35:
                            seen.add((uu, vv))
                            pilha.append((uu, vv))
            areas.append(a)
    areas.sort()
    pontos = len(areas) / ((x1 - x0) / 100.0)
    gota = float(areas[len(areas) // 2]) if areas else 0.0
    # matiz
    import colorsys
    hs = []
    for y in range(y0, y1, 2):
        for x in range(x0, x1, 2):
            if c(x, y) >= 0.5:
                r, g, b = (v / 255.0 for v in rgb(x, y))
                hs.append(colorsys.rgb_to_hsv(r, g, b)[0] * 360.0)
    if hs:
        m = sum(hs) / len(hs)
        matiz = (sum((v - m) ** 2 for v in hs) / len(hs)) ** 0.5
    else:
        matiz = 0.0

    def media(xa, xb):
        return sum(c(x, y) for x in range(xa, xb) for y in range(340, 460)) / ((xb - xa) * 120)

    rampa = media(150, 200) / max(1e-6, media(480, 530))
    return dict(largura=largura, cobertura=round(cob, 3), vazio=round(vazio, 3), pontos=round(pontos, 2),
                gota=gota, matiz=round(matiz, 2), rampa=round(rampa, 3))


if __name__ == '__main__':
    for p in sys.argv[1:]:
        print(p.split('/')[-1], identidade(p))
