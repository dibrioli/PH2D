#!/usr/bin/env python3
"""Régua de um traço HORIZONTAL numa foto (PNG): a mesma régua para o Rebelle e para o PH2D.

uso: mede_traco.py <png> --crop x,y,w,h [--bg r,g,b] [--label nome] [--json]

Mede, sobre a cobertura c = distância de cor ao papel (0..1):
  largura    — mediana da largura (px) por coluna, e a largura a 5 % / 50 % / 95 % do comprimento (taper)
  orla       — largura da transição 10 %→90 % na borda de cima e de baixo (px): quão dura é a borda
  uniformid. — coeficiente de variação da cobertura média ao longo do traço (o «colar de contas»)
  período    — o período dominante (px) ao longo do traço e a fracção da potência que ele leva (carimbo visível)
  textura    — desvio-padrão do resíduo (c − média local 5×5) dentro do traço, em % da cobertura média
  borda esc. — cobertura na orla de 3 px / cobertura no miolo (> 1 = borda mais escura: aquarela)
  cobertura  — cobertura média no miolo, e a cor média (sRGB) do miolo
Só numpy + PIL. A cobertura é relativa ao papel estimado (mediana das 4 linhas do bordo do recorte)
ou ao `--bg` dado.
"""
import argparse
import json
import sys

import numpy as np
from PIL import Image


def box_mean(a, r):
    """Média local numa janela (2r+1)² por somas acumuladas (sem scipy)."""
    n, m = a.shape
    p = np.pad(a, r, mode="edge")
    s = np.zeros((p.shape[0] + 1, p.shape[1] + 1), dtype=np.float64)
    s[1:, 1:] = p.cumsum(0).cumsum(1)
    k = 2 * r + 1
    out = s[k:k + n, k:k + m] - s[:n, k:k + m] - s[k:k + n, :m] + s[:n, :m]
    return out / (k * k)


def mede(png, crop, bg=None):
    img = np.asarray(Image.open(png).convert("RGB"), dtype=np.float32)
    x, y, w, h = crop
    a = img[y:y + h, x:x + w]
    if bg is None:
        borda = np.concatenate([a[:2].reshape(-1, 3), a[-2:].reshape(-1, 3)])
        bg = np.median(borda, axis=0)
    bg = np.asarray(bg, dtype=np.float32)
    c = np.abs(a - bg).max(axis=2) / 255.0
    mask = c > 0.08
    cols = np.where(mask.any(axis=0))[0]
    if len(cols) < 8:
        return {"erro": "traço não encontrado", "bg": bg.round().tolist()}
    x0, x1 = cols[0], cols[-1]
    larguras, orlas, perfil, centros = [], [], [], []
    for cx in range(x0, x1 + 1):
        col = c[:, cx]
        m = col.max()
        if m < 0.08:
            larguras.append(0.0); perfil.append(0.0); centros.append(np.nan)
            continue
        ys = np.where(col >= 0.5 * m)[0]
        larguras.append(float(ys[-1] - ys[0] + 1))
        centros.append(float((col * np.arange(len(col))).sum() / col.sum()))
        perfil.append(float(col[col > 0.08].mean()))
        # orla: 10 %→90 % do máximo da coluna, de fora para dentro, em cima e em baixo
        hi, lo = 0.9 * m, 0.1 * m
        ys_lo = np.where(col >= lo)[0]; ys_hi = np.where(col >= hi)[0]
        orlas.append(float(ys_hi[0] - ys_lo[0]))
        orlas.append(float(ys_lo[-1] - ys_hi[-1]))
    larguras = np.array(larguras); perfil = np.array(perfil)
    n = len(larguras)
    i5, i50, i95 = int(0.05 * n), int(0.5 * n), int(0.95 * n)
    # uniformidade e período ao longo do traço (só no miolo, fora dos 10 % de cada ponta)
    miolo = perfil[int(0.1 * n):int(0.9 * n)]
    cv = float(miolo.std() / max(miolo.mean(), 1e-6))
    det = miolo - box_mean(miolo[None, :], 15)[0]
    spec = np.abs(np.fft.rfft(det * np.hanning(len(det)))) ** 2
    freqs = np.fft.rfftfreq(len(det))
    spec[freqs < 1.0 / 200] = 0  # períodos acima de 200 px não são carimbo
    k = int(spec.argmax())
    periodo = float(1.0 / freqs[k]) if freqs[k] > 0 else float("inf")
    fracao = float(spec[k] / max(spec.sum(), 1e-9))
    # textura no interior (máscara erodida 3 px) e borda escura
    inner = mask.copy()
    for _ in range(3):
        inner = inner & np.roll(inner, 1, 0) & np.roll(inner, -1, 0) & np.roll(inner, 1, 1) & np.roll(inner, -1, 1)
    ring = mask & ~inner
    res = c - box_mean(c, 2)
    textura = float(res[inner].std() / max(c[inner].mean(), 1e-6)) if inner.sum() > 50 else float("nan")
    borda = float(c[ring].mean() / max(c[inner].mean(), 1e-6)) if inner.sum() > 50 and ring.sum() > 50 else float("nan")
    cor = a[inner].mean(axis=0) if inner.sum() > 50 else a[mask].mean(axis=0)
    return {
        "bg": bg.round().tolist(),
        "comprimento_px": int(x1 - x0 + 1),
        "largura_mediana_px": float(np.median(larguras[larguras > 0])),
        "largura_5_50_95": [float(larguras[i5]), float(larguras[i50]), float(larguras[i95])],
        "orla_10_90_px": float(np.median(orlas)),
        "uniformidade_cv": cv,
        "periodo_px": periodo,
        "periodo_fracao": fracao,
        "textura_pct": 100 * textura,
        "borda_escura": borda,
        "cobertura_miolo": float(c[inner].mean()) if inner.sum() > 50 else float(c[mask].mean()),
        "cor_miolo": cor.round().tolist(),
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("png"); ap.add_argument("--crop", required=True); ap.add_argument("--bg")
    ap.add_argument("--label", default=""); ap.add_argument("--json", action="store_true")
    a = ap.parse_args()
    crop = [int(v) for v in a.crop.split(",")]
    bg = [float(v) for v in a.bg.split(",")] if a.bg else None
    r = mede(a.png, crop, bg)
    if a.json:
        print(json.dumps({"label": a.label, **r}))
        return
    if "erro" in r:
        print(f"{a.label:22s} {r['erro']} (papel {r['bg']})"); return
    print(f"{a.label:22s} larg {r['largura_mediana_px']:5.1f}px  taper {r['largura_5_50_95'][0]:.0f}/{r['largura_5_50_95'][1]:.0f}/{r['largura_5_50_95'][2]:.0f}  "
          f"orla {r['orla_10_90_px']:4.1f}px  CV {r['uniformidade_cv']:.3f}  período {r['periodo_px']:5.1f}px ({100*r['periodo_fracao']:.0f}%)  "
          f"textura {r['textura_pct']:4.1f}%  borda {r['borda_escura']:.2f}  cob {r['cobertura_miolo']:.2f}  cor {r['cor_miolo']}")


if __name__ == "__main__":
    main()
