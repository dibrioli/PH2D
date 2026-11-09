#!/usr/bin/env python3
"""P14 (3) — a normal de 16 bits que sobe à placa em 8 (`normal_map::sincronizar` por `image_rgba8`): a perda no
quadro FINAL, em níveis de 8 bits. Lei = `receber.wgsl` `luz_em_n`: E = max(F + nx*Mx + ny*My, 0)/2pi, saída =
sRGB(clamp(cor*E)) (tonemap BYPASS_LUT, alvo sRGB). Normal suave = o hemisfério em N x N texels; referência =
16 bits (Unorm), contra 8 bits (Rgba8Unorm, o de hoje) e meio-float do valor CODIFICADO (Rgba16Float).
Corre: `python3 -I docs/PixelArt/ferramentas/mede_normal_16_bits.py` (~1 min)."""
import math
import struct


def srgb(x):
    x = min(max(x, 0.0), 1.0)
    return 12.92 * x if x <= 0.0031308 else 1.055 * x ** (1 / 2.4) - 0.055


def q(bits):
    m = (1 << bits) - 1
    return lambda v: round((v * 0.5 + 0.5) * m) / m * 2 - 1


def qh(v):
    e = struct.unpack("<e", struct.pack("<e", v * 0.5 + 0.5))[0]
    return e * 2 - 1


def nxy(nx, ny, nz):
    l = max(math.sqrt(nx * nx + ny * ny + nz * nz), 1e-9)
    return nx / l, ny / l


N = 512
q16, q8 = q(16), q(8)
pior = {"8": (0, None), "half": (0, None)}
conta = {"8": {}, "half": {}}
for fs in [0.05, 0.1, 0.2, 0.35, 0.5, 0.75, 1.0, 1.5, 2.0]:  # F/2pi (exposição)
    for k in [0.8, 0.95]:  # |M|/F
        for j in range(N):
            for i in range(N):
                x, y = (i + 0.5) / N * 2 - 1, (j + 0.5) / N * 2 - 1
                r = math.hypot(x, y)
                if r > 1:
                    continue
                nz = math.sqrt(max(0.0, 1 - r * r))
                o = []
                for qq in (q16, q8, qh):
                    a, _ = nxy(qq(x), qq(y), qq(nz))
                    o.append(round(255 * srgb(max(fs + a * fs * k, 0.0))))
                for nome, d in (("8", abs(o[0] - o[1])), ("half", abs(o[0] - o[2]))):
                    conta[nome][d] = conta[nome].get(d, 0) + 1
                    if d > pior[nome][0]:
                        pior[nome] = (d, (fs, k, round(x, 3), round(y, 3), o))
for nome in ("8", "half"):
    tot = sum(conta[nome].values())
    print(nome, "pior:", pior[nome], {d: f"{c} ({100 * c / tot:.4f}%)" for d, c in sorted(conta[nome].items())})
