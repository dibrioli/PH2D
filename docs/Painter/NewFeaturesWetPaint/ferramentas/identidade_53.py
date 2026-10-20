#!/usr/bin/env python3
"""⭐ A prova de IDENTIDADE dos 11 do prompt 53: o preset gravado em `assets/brushes/WetPainter/` É a receita vencedora do
doc 52 — o ficheiro do vizinho (`base`) com os overrides dela (`knobs` → `engine`, `brush` → `brush`), campo a campo.
Só podem diferir o nome, o grupo, o tamanho (`volumes.size_px`, o do preset do Rebelle) e a lei da largura
(`width_table`, medida do preset novo). Um fnv da cena não serve: a lei própria muda o raio que o motor vê.

uso: identidade_53.py <raiz do repo>   — sai 1 se algum diferir
"""
import glob
import json
import os
import struct
import sys
import zlib

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from avalia_53 import OS_11, VIZINHO  # noqa: E402

raiz = sys.argv[1]
# Os ajustes MEDIDOS no 53 sobre a receita do 52 (o porquê no comentário da receita em `receitas_de_fabrica.rs`).
AJUSTES_DO_53 = {"Knife Smooth": {"pigmentPerDab": 250.0}, "Graphite 2H": {"pigmentPerDab": 50.0}}
LIVRES = {("name",), ("group",), ("volumes", "size_px"), ("width_table",)}


def parametros(nome):
    [p] = glob.glob(f"{raiz}/assets/brushes/WetPainter/*/*/{nome}.ph2d-brush.png")
    b = open(p, "rb").read()
    i = 8
    while i < len(b):
        (n,) = struct.unpack(">I", b[i:i + 4])
        tipo, dados = b[i + 4:i + 8], b[i + 8:i + 8 + n]
        i += 12 + n
        if tipo in (b"tEXt", b"zTXt", b"iTXt"):
            chave, _, resto = dados.partition(b"\0")
            if tipo == b"zTXt":
                texto = zlib.decompress(resto[1:])
            elif tipo == b"iTXt":
                comprimido, resto = resto[0], resto[2:]
                resto = resto.partition(b"\0")[2].partition(b"\0")[2]
                texto = zlib.decompress(resto) if comprimido else resto
            else:
                texto = resto
            try:
                j = json.loads(texto)
            except ValueError:
                continue
            if isinstance(j, dict) and "engine" in j.get("parametros", j):
                return j.get("parametros", j)
    raise SystemExit(f"{nome}: sem parâmetros")


def folhas(x, caminho=()):
    if isinstance(x, dict):
        for k, v in x.items():
            yield from folhas(v, caminho + (k,))
    else:
        yield caminho, x


receitas = {}
for f in ("receitas_19.json", "receitas_19b.json", "receitas_19bc.json"):
    for r in json.load(open(f"{raiz}/docs/Painter/NewFeaturesWetPaint/52_regua/{f}")):
        receitas.setdefault(r["nome"], r)

falhas = 0
for preset, _, _, nome_receita, _ in OS_11:
    r = receitas[nome_receita]
    assert r["base"] == VIZINHO[preset], (preset, r["base"])
    esperado = parametros(r["base"])
    for k, v in r["knobs"].items():
        esperado["engine"][k] = float(v)
    for k, v in r["brush"].items():
        esperado["brush"][k] = v
    for k, v in AJUSTES_DO_53.get(preset, {}).items():
        esperado["engine"][k] = v
    if not glob.glob(f"{raiz}/assets/brushes/WetPainter/*/*/{preset}.ph2d-brush.png"):
        falhas += 1
        print(f"{preset:<13} ✗ sem ficheiro")
        continue
    gravado = parametros(preset)
    a, b = dict(folhas(esperado)), dict(folhas(gravado))
    difere = sorted(c for c in set(a) | set(b)
                    if c not in LIVRES and c[:1] not in {("width_table",)}
                    and not (isinstance(a.get(c), (int, float)) and isinstance(b.get(c), (int, float))
                             and abs(float(a[c]) - float(b[c])) < 1e-9)
                    and a.get(c) != b.get(c))
    falhas += bool(difere)
    print(f"{preset:<13} = {nome_receita:<12} sobre {r['base']:<13} tamanho {gravado['volumes']['size_px']:>6}"
          + (" ✓" if not difere else " ✗ " + "; ".join(f"{'.'.join(c)}: {a.get(c)!r} → {b.get(c)!r}" for c in difere)))
sys.exit(1 if falhas else 0)
