#!/usr/bin/env python3
"""Oráculo das rampas do Pixel Art (P13 R1): corre o coloraide sobre bases e configurações NOSSAS e
grava a fixture `crates/ph2d-pixel-rampa/tests/fixtures/coloraide_rampas.json`.

Uso:  ~/.local/opt/oraculos-py/bin/python3 -I docs/PixelArt/ferramentas/oraculo_rampas_coloraide.py

A lei planeada (a mesma de `ph2d_pixel_rampa::planeada`), por degrau i de n, t = i/(n-1), u = 2t-1:
    L = l0 + (l1 - l0)·t
    C = C_base · max(0, 1 - curva·u²)                       (croma máximo no meio)
    h = h_base + sentido·desvio·u                           (a luz vai para o amarelo sRGB)
    sentido = +1 se a volta mais curta de h_base até à matiz OKLCH do amarelo sRGB é positiva
O corte de gama é do COLORAIDE: fit('srgb', method='minde-chroma', pspace='oklch', jnd=0.0) — só o
croma baixa; L e h ficam. O que se compara é o OKLab DEPOIS do corte e ANTES dos 8 bits.
"""

import datetime
import json
import os
import sys

import coloraide
from coloraide import Color

BASES = {
    "skin_d9a066": [0xD9, 0xA0, 0x66],
    "water_5b6ee1": [0x5B, 0x6E, 0xE1],
    "leaf_6abe30": [0x6A, 0xBE, 0x30],
}
CONFIGS = {
    "a5": {"passos": 5, "desvio_matiz_graus": 20.0, "curva_croma": 0.6, "faixa_l": [0.25, 0.92]},
    "b8": {"passos": 8, "desvio_matiz_graus": 35.0, "curva_croma": 0.9, "faixa_l": [0.12, 0.97]},
}


def matiz_quente():
    return Color("srgb", [1, 1, 0]).convert("oklch")["hue"]


def caso(rgb, cfg):
    base = Color("srgb", [v / 255 for v in rgb]).convert("oklch")
    lb, cb, hb = base["lightness"], base["chroma"], base["hue"]
    d = (matiz_quente() - hb + 540.0) % 360.0 - 180.0
    sentido = 1.0 if d >= 0 else -1.0
    n = cfg["passos"]
    l0, l1 = cfg["faixa_l"]
    degraus = []
    for i in range(n):
        t = 0.5 if n == 1 else i / (n - 1)
        u = 2 * t - 1
        p = Color("oklch", [l0 + (l1 - l0) * t, cb * max(0.0, 1 - cfg["curva_croma"] * u * u),
                            (hb + sentido * cfg["desvio_matiz_graus"] * u) % 360.0])
        planeado = p.coords()
        f = p.clone().fit("srgb", method="minde-chroma", pspace="oklch", jnd=0.0)
        lab = f.convert("oklab").coords()
        srgb = f.convert("srgb").clip()
        degraus.append({
            "oklch_planeado": planeado,
            "oklch_na_gama": f.coords(),
            "oklab_na_gama": lab,
            "srgb8": [round(v * 255) for v in srgb.coords()],
        })
    return {"base_srgb8": rgb, "base_oklch": [lb, cb, hb], "degraus": degraus}


def main():
    out = {
        "_cabecalho": {
            "ferramenta": "coloraide",
            "versao": coloraide.__version__,
            "licenca": "MIT (pip show coloraide; venv ~/.local/opt/oraculos-py)",
            "comando": "~/.local/opt/oraculos-py/bin/python3 -I "
                       "docs/PixelArt/ferramentas/oraculo_rampas_coloraide.py",
            "data": datetime.date.today().isoformat(),
            "corte_de_gama": "fit('srgb', method='minde-chroma', pspace='oklch', jnd=0.0)",
            "matiz_quente_graus": matiz_quente(),
        },
        "casos": [
            dict(nome=f"{b}_{c}", config=CONFIGS[c], **caso(BASES[b], CONFIGS[c]))
            for b in BASES for c in CONFIGS
        ],
    }
    raiz = os.path.dirname(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))
    dest = os.path.join(raiz, "crates/ph2d-pixel-rampa/tests/fixtures/coloraide_rampas.json")
    with open(dest, "w") as f:
        json.dump(out, f, indent=1)
        f.write("\n")
    print(dest, len(out["casos"]), "casos", file=sys.stderr)


if __name__ == "__main__":
    main()
