"""Pré-filtra os ambientes do Óleo Real (W1c): HDRI CC0 → mapa de irradiância + brilho GGX pré-filtrado (split-sum).

    python3 docs/Painter/49_realshader/oraculo/prefiltra_ambiente.py <pasta dos HDRI> <saída>

Fonte: os HDRI que o Blender instala (`studiolights/world/`, CC0, Poly Haven — `license.txt` ao lado).
Moldura: z para cima; u = atan2(y, −x)/2π + ½, v = elevação/π + ½, linha 0 = zénite. ⚠️ É a do Blender RODADA
de 180° em z (o Cycles usa u = ½ − atan2(y, x)/2π): no produto não importa (o `claro` sai na mesma moldura), mas
quem rodar o HDRI no Cycles lê o `claro` pela convenção do Cycles (`cycles.py`, W2 E2).
A metade de BAIXO do horizonte zera-se (a tela tapa o chão). Escala: a irradiância de um plano virado para
cima = π (tinta lisa sob o ambiente = o albedo). O cabeçalho leva o azimute do lado claro (pesado pelo
cosseno) para o produto o rodar até à direcção da luz.

Formato `.amb` (LE): b"PH2DAMB3", u32 w, u32 h, u32 níveis, f32 claro, f32 rug[níveis],
f32 irr[h/2][w/2][3] (a convolução pelo cosseno: E(n)/π), f32 nível[níveis][h][w][3],
u32 cw, u32 ch, f32 radiância[ch][cw][3] (W2 E2: a metade de CIMA, BRUTA — não convolvida —, a mesma
escala, linha 0 = zénite, ch linhas até ao horizonte: o produto integra-a POR SECTOR de fatia). O `PH2DAMB2`
é o mesmo sem o bloco da radiância.
O SH9 saiu: errava 20 % na irradiância do plano com a metade de baixo zerada (medido 26/10).
"""

import math
import struct
import sys

import numpy as np
import OpenImageIO as oiio

fonte, saida = sys.argv[1], sys.argv[2]
AMBIENTES = {"estudio": "studio", "interior": "interior", "patio": "courtyard"}
W, H = 64, 32
RUG = [0.28, 0.40, 0.52, 0.66]


def direccoes(w, h):
    u = (np.arange(w) + 0.5) / w
    v = 1.0 - (np.arange(h) + 0.5) / h  # linha 0 = zénite
    fi = (u - 0.5) * 2 * math.pi
    te = (v - 0.5) * math.pi
    cz = np.cos(te)[:, None]
    d = np.stack(
        [-np.cos(fi)[None, :] * cz, np.sin(fi)[None, :] * cz, np.broadcast_to(np.sin(te)[:, None], (h, w))],
        -1,
    )
    dw = (2 * math.pi / w) * (math.pi / h) * np.broadcast_to(cz, (h, w))
    return d, dw


def reduz(img, w, h):
    H0, W0 = img.shape[:2]
    return img.reshape(h, H0 // h, w, W0 // w, 3).mean(axis=(1, 3))


for nome, arq in AMBIENTES.items():
    img = oiio.ImageBuf(f"{fonte}/{arq}.exr").get_pixels(oiio.FLOAT)[..., :3].astype(np.float64)
    env = reduz(img, 256, 128)
    d, dw = direccoes(256, 128)
    env = env * (d[..., 2:3] > 0)
    e_plano = float((env.mean(-1) * np.maximum(d[..., 2], 0) * dw).sum())
    env *= math.pi / e_plano
    lum = env @ np.array([0.2126, 0.7152, 0.0722])
    peso = lum * np.maximum(d[..., 2], 0) * dw
    claro = math.atan2((d[..., 1] * peso).sum(), (d[..., 0] * peso).sum())
    # Brilho pré-filtrado (n = v = R): média de L pesada por D_GGX(n·h)·(n·l), em 64×32 sobre 128×64.
    src = reduz(env, 128, 64)
    ds, dws = direccoes(128, 64)
    ds, dws, src = ds.reshape(-1, 3), dws.reshape(-1), src.reshape(-1, 3)
    do, _ = direccoes(W, H)
    do = do.reshape(-1, 3)
    niveis = []
    for r in RUG:
        a2 = r**4
        nl = np.clip(do @ ds.T, 0, None)
        nh = np.sqrt((1 + nl) / 2)
        den = nh * nh * (a2 - 1) + 1
        w = a2 / (math.pi * den * den) * nl * dws[None, :]
        niveis.append(((w @ src) / np.maximum(w.sum(1, keepdims=True), 1e-12)).reshape(H, W, 3))
    # Irradiância relativa E(n)/π, exacta (sem truncamento): plano para cima = 1.
    di, _ = direccoes(W // 2, H // 2)
    di = di.reshape(-1, 3)
    irr = ((np.clip(di @ ds.T, 0, None) * dws[None, :]) @ src / math.pi).reshape(H // 2, W // 2, 3)
    cima = (np.clip(np.array([0, 0, 1.0]) @ ds.T, 0, None) * dws) @ src / math.pi
    assert abs(cima.mean() - 1) < 0.02, cima
    with open(f"{saida}/{nome}.amb", "wb") as f:
        f.write(b"PH2DAMB3")
        f.write(struct.pack("<IIIf", W, H, len(RUG), claro))
        f.write(np.array(RUG, "<f4").tobytes())
        f.write(irr.astype("<f4").tobytes())
        f.write(np.stack(niveis).astype("<f4").tobytes())
        ceu = src.reshape(64, 128, 3)[:32]
        f.write(struct.pack("<II", 128, 32))
        f.write(ceu.astype("<f4").tobytes())
    print(f"{nome}: E_plano bruto {e_plano:.3f} · claro {math.degrees(claro):.0f}° · brilho p50/p99 nível 0 "
          f"{np.percentile(niveis[0].mean(-1), 50):.3f}/{np.percentile(niveis[0].mean(-1), 99):.3f}")
