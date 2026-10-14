#!/usr/bin/env python3
"""A régua do prompt 35 — o Soft Pastel do PH2D contra o Rebelle «Pastel Dry / Dry Thin», POR ASPECTO.

uso: pastel_regua.py <pasta_rebelle> <pasta_ph2d> [--prancha saida.png] [--json saida.json]
  pasta_rebelle: as fotos de `pastel_tela.sh` (janela 1920×1080, tela 1000×640 a 100 % em (480, 220));
  pasta_ph2d:    os PPM de `sonda_pastel_no_tempo` (1000×640), `pastel_<gesto>_<instante>.ppm`.

A tinta lê-se pelo canal VERMELHO: o azul da bancada tem R = 23, logo `a = (255 − R) / (255 − 23)` é a
cobertura do píxel (0 papel, 1 tinta plena). As réguas, cada uma com o seu CONTROLO (a régua tem de separar
o Rebelle de um BORRÃO dele mesmo — o Rebelle desfocado a σ = 1,5 px):
  (a) textura fina — contraste local a 2–8 px: o desvio de `a − blur(a, σ=4)` sobre o CORPO, ÷ a média; e a
      fração de BURACOS do corpo (a < 0,5 · mediana do corpo). Corpo = os píxeis com `blur(a, σ=6) ≥ 0,5 · máx`.
  (b) borda — o perfil transversal do recto (média das colunas x ∈ [100, 240]): a largura a 50 %, a subida
      10→90 % (px) e a POEIRA fora do traço (fração dos píxeis a 2–8 px fora do contorno de 10 % com a > 0,15).
      E a MANCHA GROSSA (8–32 px): o desvio de `blur(a, 4) − blur(a, 16)` no corpo ÷ a média.
  (b') a ONDULAÇÃO da borda do recto: o desvio-padrão, coluna a coluna, das orlas a 50 % (px).
  (c) cobertura — a mediana de `a` no corpo (a opacidade do miolo) e a fração do corpo com a > 0,5.
  (d) tempo — píxeis que mudam (|Δ| > 6 num canal) entre o pen-up e 0,5 · 1 · 3 · 6 s; e o «a meio».
"""
import argparse
import json
from pathlib import Path

import numpy as np
from PIL import Image


def gaussian_filter(a, sigma):
    """Desfoque gaussiano separável (reflexo nas bordas) — sem scipy nesta máquina."""
    r = int(3 * sigma + 0.5)
    k = np.exp(-0.5 * (np.arange(-r, r + 1) / sigma) ** 2)
    k /= k.sum()
    a = np.asarray(a, float)
    for eixo in (0, 1):
        pad = [(0, 0), (0, 0)]
        pad[eixo] = (r, r)
        b = np.pad(a, pad, mode="reflect")
        a = sum(k[i] * np.take(b, range(i, i + a.shape[eixo]), axis=eixo) for i in range(2 * r + 1))
    return a


def binary_dilation(m, iterations=1):
    """Dilatação em cruz (vizinhança 4), `iterations` vezes."""
    m = m.copy()
    for _ in range(iterations):
        n = m.copy()
        n[1:] |= m[:-1]
        n[:-1] |= m[1:]
        n[:, 1:] |= m[:, :-1]
        n[:, :-1] |= m[:, 1:]
        m = n
    return m

X0, Y0, W, H = 480, 220, 1000, 640
JANELAS = {"recto": (40, 30, 300, 110), "zigue": (50, 130, 480, 345)}  # x0, y0, x1, y1 na tela
INSTANTES = ["meio", "up", "0.5", "1", "3", "6"]


def tela_rebelle(p):
    """A foto da janela (1920×1080) ou já o recorte da tela (a fixture, 1000×640)."""
    a = np.asarray(Image.open(p).convert("RGB"))
    return a if a.shape[:2] == (H, W) else a[Y0:Y0 + H, X0:X0 + W]


def tela_ph2d(p):
    return np.asarray(Image.open(p).convert("RGB"))


def cobertura(rgb):
    return np.clip((255.0 - rgb[..., 0].astype(float)) / (255.0 - 23.0), 0.0, 1.0)


def janela(img, g):
    x0, y0, x1, y1 = JANELAS[g]
    return img[y0:y1, x0:x1]


def textura(a):
    corpo = gaussian_filter(a, 6) >= 0.5 * gaussian_filter(a, 6).max()
    fino = a - gaussian_filter(a, 4)
    med = np.median(a[corpo])
    return {
        "contraste_local": float(fino[corpo].std() / max(a[corpo].mean(), 1e-6)),
        "buracos": float((a[corpo] < 0.5 * med).mean()),
        "miolo_mediana": float(med),
        "miolo_cheio": float((a[corpo] > 0.5).mean()),
        "mancha_grossa": float((gaussian_filter(a, 4) - gaussian_filter(a, 16))[corpo].std()
                               / max(a[corpo].mean(), 1e-6)),
    }


def borda(a):
    perfil = a[:, 100 - JANELAS["recto"][0]:240 - JANELAS["recto"][0]].mean(1)
    m = perfil.max()
    acima = lambda f: np.where(perfil >= f * m)[0]
    l50 = acima(0.5)
    sobe = []
    for lado in (slice(None, int(perfil.argmax()) + 1), slice(int(perfil.argmax()), None)):
        p = perfil[lado]
        i10 = np.argmin(np.abs(p - 0.1 * m))
        i90 = np.argmin(np.abs(p - 0.9 * m))
        sobe.append(abs(int(i90) - int(i10)))
    dentro = gaussian_filter(a, 2) >= 0.1 * m
    anel = binary_dilation(dentro, iterations=8) & ~binary_dilation(dentro, iterations=2)
    # A ONDULAÇÃO da borda: o desvio, coluna a coluna, das duas orlas a 50 % (o Rebelle é um bastão recto).
    lisa = gaussian_filter(a, 1.0)
    cols = range(100 - JANELAS["recto"][0], 240 - JANELAS["recto"][0])
    orlas = []
    for x in cols:
        i = np.where(lisa[:, x] >= 0.5 * m)[0]
        if len(i):
            orlas.append((i.min(), i.max()))
    orlas = np.array(orlas, float)
    return {
        "ondulacao": float(orlas.std(0).mean()) if len(orlas) else 0.0,
        "largura_50": int(l50.max() - l50.min() + 1),
        "subida_10_90": float(np.mean(sobe)),
        "poeira": float((a[anel] > 0.15).mean()) if anel.any() else 0.0,
    }


def deriva(fotos, g):
    up = fotos[f"{g}_up"].astype(int)
    out = {}
    for t in ["0.5", "1", "3", "6"]:
        d = np.abs(fotos[f"{g}_{t}"].astype(int) - up).max(2) > 6
        out[t] = int(d.sum())
    return out


def mede(fotos):
    r = {}
    for g in JANELAS:
        a = cobertura(janela(fotos[f"{g}_6"], g))
        r[g] = {"textura": textura(a), "deriva": deriva({k: janela(v, g) for k, v in fotos.items()}, g)}
    r["recto"]["borda"] = borda(cobertura(janela(fotos["recto_6"], "recto")))
    return r


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("rebelle")
    ap.add_argument("ph2d")
    ap.add_argument("--prancha")
    ap.add_argument("--json")
    a = ap.parse_args()
    reb = {f"{g}_{t}": tela_rebelle(Path(a.rebelle) / f"{g}_{t}.png") for g in JANELAS for t in INSTANTES}
    ph = {f"{g}_{t}": tela_ph2d(Path(a.ph2d) / f"pastel_{g}_{t}.ppm") for g in JANELAS for t in INSTANTES}
    # O CONTROLO: o Rebelle borrado (σ = 1,5 px) — uma régua de textura que não o separa do Rebelle mede só tinta.
    borrao = {k: np.stack([gaussian_filter(v[..., c].astype(float), 1.5) for c in range(3)], -1).astype(np.uint8)
              for k, v in reb.items()}
    res = {"rebelle": mede(reb), "ph2d": mede(ph), "controlo_rebelle_borrado": mede(borrao)}
    print(f"{'régua':34s} {'Rebelle':>9s} {'borrão':>9s} {'PH2D':>9s}")
    for g in JANELAS:
        for k in res["rebelle"][g]["textura"]:
            v = [res[x][g]["textura"][k] for x in ("rebelle", "controlo_rebelle_borrado", "ph2d")]
            print(f"{g + ' ' + k:34s} {v[0]:9.3f} {v[1]:9.3f} {v[2]:9.3f}")
        for t in ["0.5", "1", "3", "6"]:
            v = [res[x][g]["deriva"][t] for x in ("rebelle", "controlo_rebelle_borrado", "ph2d")]
            print(f"{g + ' deriva ' + t + ' s (px)':34s} {v[0]:9d} {v[1]:9d} {v[2]:9d}")
    for k in res["rebelle"]["recto"]["borda"]:
        v = [res[x]["recto"]["borda"][k] for x in ("rebelle", "controlo_rebelle_borrado", "ph2d")]
        print(f"{'recto ' + k:34s} {v[0]:9.3f} {v[1]:9.3f} {v[2]:9.3f}")
    if a.json:
        Path(a.json).write_text(json.dumps(res, indent=1))
    if a.prancha:
        # A prancha: Rebelle em cima, PH2D em baixo, a MESMA escala (2×), recto e zigue a 6 s, e o «a meio».
        def tira(f):
            r = janela(f["recto_6"], "recto")
            z = janela(f["zigue_6"], "zigue")
            m = janela(f["zigue_meio"], "zigue")
            alto = max(r.shape[0], z.shape[0])
            pad = lambda x: np.pad(x, ((0, alto - x.shape[0]), (0, 8), (0, 0)), constant_values=255)
            return np.concatenate([pad(r), pad(z), pad(m)], 1)
        img = np.concatenate([tira(reb), np.full((8, tira(reb).shape[1], 3), 128, np.uint8), tira(ph)], 0)
        Image.fromarray(img).resize((img.shape[1] * 2, img.shape[0] * 2), Image.NEAREST).save(a.prancha)


if __name__ == "__main__":
    main()
