#!/usr/bin/env python3
"""A lista do `fotos_bateria.sh` para os 281 presets de fábrica do Rebelle (`rebelle_presets_281.json`).

Linha: `nome|TOOL|Grupo/Nome|base|cenas`. O `Grupo/Nome` é o do FICHEIRO (o SET_BRUSH resolve
`Brushes/<pasta do TOOL>/<Grupo/Nome>.rebelle-brush.png`; o campo `Name` difere em 9 presets).
Ordem: lápis e carvão · pastéis · óleo/acrílica · express oil · aguarela/guache · tinta · marcadores ·
aerógrafo · borradores/blenders · o resto. As ferramentas SEM tinta (Blend, Smudge, Eraser, Clone, Water, Dry)
e os Marker/Effect não deixam marca no papel branco: fazem só a cena «sobre», com uma base de tinta (óleo
Round Hard e aguarela Round; os Effect sobre o Marker Bullet).

uso: python3 -I lista_rebelle.py > lista_281.txt
"""
import json
import re
from pathlib import Path

FX = Path(__file__).resolve().parent.parent.parent / "NewFeaturesWetPaint" / "fixtures" / "rebelle_presets_281.json"
TOOL = {"Pencil": "PENCIL", "Pastel": "PASTEL", "Oil": "OIL_AND_ACRYLIC", "ExpressOil": "EXPRESS_OIL",
        "Watercolor": "WATERCOLOR", "Ink": "INK_PEN", "Marker": "MARKER", "Airbrush": "AIRBRUSH", "Blend": "BLEND",
        "Smudge": "SMUDGE", "Eraser": "ERASER", "Wet": "WATER", "Dry": "DRY", "Clone": "CLONE"}
ORDEM = list(TOOL)
OLEO, AGUARELA, BULLET = "OIL_AND_ACRYLIC:Round/Round Hard", "WATERCOLOR:Watercolor/Round", "MARKER:Marker/Bullet"
# (sufixo do nome, base) por pasta das ferramentas sem tinta
SEM_TINTA = {"Blend": [("", OLEO), ("_aq", AGUARELA)], "Smudge": [("", OLEO), ("_aq", AGUARELA)],
             "Eraser": [("", OLEO), ("_aq", AGUARELA)], "Clone": [("", OLEO), ("_aq", AGUARELA)],
             "Wet": [("", AGUARELA)], "Dry": [("", AGUARELA)]}


def nome(grupo, n):
    return re.sub(r"_+", "_", re.sub(r"[^a-z0-9]+", "_", f"{grupo}_{n}".lower())).strip("_")


def main():
    linhas, vistos = [], set()
    for p in json.loads(FX.read_text())["pinceis"]:
        pasta, grupo, fich = p["file"].split("/")[-3:]
        n = fich.split(".rebelle-brush.png")[0]
        linhas.append((ORDEM.index(pasta), pasta, grupo, n))
    for _, pasta, grupo, n in sorted(linhas):
        t, base = TOOL[pasta], f"{grupo}/{n}"
        nm = nome(grupo, n)
        assert (t, nm) not in vistos, (t, nm)
        vistos.add((t, nm))
        if pasta in SEM_TINTA:
            for suf, b in SEM_TINTA[pasta]:
                print(f"{nm}{suf}|{t}|{base}|{b}|sobre")
        elif pasta == "Marker" and grupo == "Effect":
            print(f"{nm}|{t}|{base}|{BULLET}|sobre")
        else:
            print(f"{nm}|{t}|{base}||gestos tamanhos sobre")


if __name__ == "__main__":
    main()
