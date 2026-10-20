#!/usr/bin/env python3
"""O tamanho de FÁBRICA de cada preset do Rebelle (prompt 53): o `volume_params.size` do chunk `ParametersEx` do
`.rebelle-brush.png` instalado — a fonte do `size_px` das receitas («o tamanho é o do preset dele»). Confere os 8 de
antes (103 · 78 · 64 · 6,5 · 37 · 29 · 77 · 29) e lê os novos.

uso: tamanho_dos_presets.py ["Pencil/Graphite Pencil/2H" …]   (sem argumentos: os 19 do 53)
"""
import glob
import json
import os
import struct
import sys
import zlib

REBELLE = os.path.expanduser("~/Apps/rebelle/wineprefix/drive_c/Program Files/Rebelle 8")
OS_19 = [
    # os 8 de fábrica de antes (a conferência da fonte)
    "Oil/Bristle/Bristle Flat", "Oil/Round/Round Hard", "ExpressOil/Round/Hairy", "Pencil/Graphite Pencil/8B",
    "Pencil/Charcoal/Charcoal", "Pastel/Pastel Dry/Dry Thin", "Pastel/Pastel Oil/Oil Pastel 2",
    "Watercolor/Watercolor/Liner",
    # os 11 do 53
    "Pencil/Graphite Pencil/2H", "Pencil/Graphite Pencil/HB", "Pencil/Graphite Pencil/2B",
    "Pencil/Colored Pencil/4B Color", "Pencil/Scratches/Scratch", "Pastel/Pastel Dry/Square Hard",
    "Pastel/Scrape/Wax Scrape", "Oil/Flat/Filbert", "Oil/Dabber/Dabber", "Oil/Flat/Flat Oily", "Oil/Knife/Knife Smooth",
]


def parametros_ex(caminho):
    b = open(caminho, "rb").read()
    i = 8
    while i < len(b):
        (n,) = struct.unpack(">I", b[i:i + 4])
        dados = b[i + 8:i + 8 + n]
        i += 12 + n
        if b[i - 12 - n + 4:i - n - 4] == b"zTXt" and dados.startswith(b"ParametersEx\0"):
            return json.loads(zlib.decompress(dados[len("ParametersEx") + 2:]))
    return None


alvos = sys.argv[1:] or OS_19
todos = {p.split("/", 1)[1].rsplit(".rebelle-brush.png", 1)[0]: p
         for p in glob.glob(f"{REBELLE}/Brushes*/**/*.rebelle-brush.png", recursive=True)
         for p in [os.path.relpath(p, REBELLE)]}
for a in alvos:
    ex = parametros_ex(os.path.join(REBELLE, todos[a])) if a in todos else None
    print(f"{a:40s} {ex['volume_params']['size']:9.3f}" if ex else f"{a:40s} SEM FICHEIRO")
