#!/usr/bin/env python3
"""Os 11 pincéis do prompt 53 numa rodada só: a IDENTIDADE (cada preset novo, a partir do ficheiro, contra a receita
vencedora do 52) e a MATRIZ (cada preset novo contra a foto de cada um dos 11), pela barra do `avalia_19.py` (uma
métrica conta só se o controlo — o Wet Round à largura da foto — estiver a mais de 3× o ruído do Rebelle; passa quem
fica do lado do Rebelle do ponto médio; a largura ± 2 px).

uso: avalia_53.py receitas <saída.json>                     — escreve o lote da sonda
     avalia_53.py avalia <dir do lote (.ppm)> <log da sonda> — o veredito (o log dá o fnv de cada receita)
"""
import json
import math
import os
import re
import sys

# (preset, foto, alvo da receita vencedora, receita do 52, fnv do 52) — `52_regua/fnv_vencedoras.txt`.
OS_11 = [
    ("Graphite 2H", "pencil2h", 28.8, "lapisb_027", "0x629c8919b62065a4"),
    ("Graphite HB", "pencilhb", 30.3, "lapisb_028", "0xad813e50d0ed9c49"),
    ("Graphite 2B", "pencil2b", 28.8, "lapisb_032", "0x0465e64bb94bd699"),
    ("Colored 4B", "pencil4bcolor", 30.0, "lapisb_041", "0x5fb849b35725b9ab"),
    ("Scratch", "scratch", 32.0, "lapisc_053", "0xe1638a5277dec073"),
    ("Square Hard", "squarehard", 31.0, "sq_007", "0xe0e774e5e6a9a503"),
    ("Wax Scrape", "waxscrape", 36.0, "wx_007", "0x4e2b28739cfbe028"),
    ("Filbert", "filbert", 40.0, "filbert_016", "0xa1b338ecdc0c9d96"),
    ("Dabber", "dabber", 36.0, "dabberc_032", "0x385107747f3b3af3"),
    ("Flat Oily", "flatoily", 10.0, "laminac_018", "0xbc86c77a90196ee9"),
    ("Knife Smooth", "knife", 10.0, "laminac_012", "0x11d44037b349ba62"),
]
# Saíram da fábrica pelo smoke do dono (19/10, doc 53 §8): o Filbert «nada novo», as lâminas «bizarras e inúteis».
# As fotos deles continuam colunas da matriz; as linhas (os presets que se leem de `assets/`) são os que ficaram.
SAIRAM = {"Filbert", "Flat Oily", "Knife Smooth"}
OS_DA_FABRICA = [o for o in OS_11 if o[0] not in SAIRAM]
# O controlo de cada foto: o Wet Round à largura SECA dela (o `k_<grupo>` do 52).
CONTROLO = {"pencil2h": 24, "pencilhb": 24, "pencil2b": 24, "pencil4bcolor": 24, "scratch": 24, "squarehard": 31,
            "waxscrape": 36, "filbert": 40, "dabber": 36, "flatoily": 10, "knife": 10}
VIZINHO = {"Graphite 2H": "Graphite 8B", "Graphite HB": "Graphite 8B", "Graphite 2B": "Graphite 8B",
           "Colored 4B": "Graphite 8B", "Scratch": "Charcoal", "Square Hard": "Soft Pastel", "Wax Scrape": "Soft Pastel",
           "Filbert": "Acrylic Round", "Dabber": "Oil Bristle", "Flat Oily": "Acrylic Round",
           "Knife Smooth": "Acrylic Round"}
RUIDO = {"cob": 0.012, "tex": 0.37, "borda": 0.031, "orla": 1.0, "cv": 0.012, "cruz": 0.03}


def slug(nome):
    return "_".join(nome.lower().split())


def receitas(saida):
    lote = []
    for p, foto, alvo, _, _ in OS_DA_FABRICA:
        lote.append({"nome": f"id_{slug(p)}", "base": p, "foto": foto, "alvo": alvo})
        for _, outra, alvo_outra, _, _ in OS_11:
            if outra != foto:
                lote.append({"nome": f"x_{slug(p)}__{outra}", "base": p, "foto": outra, "alvo": alvo_outra})
    # O vizinho de fábrica de cada um (`NASCE_DE`), sem mudança, à largura da foto: a prancha do smoke.
    for p, foto, alvo, _, _ in OS_DA_FABRICA:
        lote.append({"nome": f"v_{slug(p)}", "base": VIZINHO[p], "foto": foto, "alvo": alvo})
    for larg in sorted(set(CONTROLO.values())):
        lote.append({"nome": f"k_{larg}", "base": "Wet Round", "alvo": larg})
    json.dump(lote, open(saida, "w"), indent=1)
    print(f"{len(lote)} receitas → {saida}")


def avalia(d, log):
    from PIL import Image

    sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
    from midias_regua import metricas

    F = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
    linhas = re.findall(r"\[midia\] (\S+)\s.*?raio (\S+) fnv (0x[0-9a-f]+)", open(log).read())
    fnv = {n: f for n, _, f in linhas}
    raio = {n: r for n, r, _ in linhas}
    cache = {}

    def m(png):
        if png not in cache:
            if not os.path.exists(png):  # o preset ainda sem ficheiro
                return None
            if png.endswith(".ppm"):
                q = png[:-4] + ".png"
                Image.open(png).save(q)
                png = q
            try:
                cache[png] = metricas(png)
            except IndexError:  # um traço sem ponta achada: sem medida
                cache[png] = None
        return cache[png]

    # A identidade (o preset gravado = a receita do 52) é por DADOS: `identidade_53.py`.
    print("## Raio de cada preset à largura da própria foto")
    for p, _, _, _, _ in OS_DA_FABRICA:
        n = f"id_{slug(p)}"
        print(f"   {p:<13} raio {raio.get(n, '?'):>6} fnv {fnv.get(n, '?')}")

    print("\n## Matriz: a linha é o preset, a coluna a foto; `✓` = passa TUDO, senão quantas das que contam")
    fotos = [f for _, f, _, _, _ in OS_11]
    print("   " + " " * 13 + " ".join(f"{f[:7]:>7}" for f in fotos))
    passa = {}
    for p, propria, _, _, _ in OS_DA_FABRICA:
        celulas = []
        for _, foto, _, _, _ in OS_11:
            R = m(f"{F}/fixtures/rebelle/{foto}.png")
            C = m(f"{d}/k_{CONTROLO[foto]}.ppm")
            conta = [k for k in RUIDO if not math.isnan(R[k]) and not math.isnan(C[k])
                     and abs(C[k] - R[k]) > 3 * RUIDO[k]]
            nome = f"id_{slug(p)}" if foto == propria else f"x_{slug(p)}__{foto}"
            N = m(f"{d}/{nome}.ppm")
            if N is None:
                celulas.append("   s/m ")
                continue
            ok = [k for k in conta if abs(N[k] - R[k]) < abs(0.5 * (R[k] + C[k]) - R[k])]
            lw = abs(N["larg"] - R["larg"]) <= 2
            tudo = len(ok) == len(conta) and lw
            passa[(p, foto)] = tudo
            if foto == propria:
                falta = [k for k in conta if k not in ok] + ([] if lw else ["larg"])
                print(f"   {p}: {' '.join(f'{k} {N[k]:.3f}/{R[k]:.3f}' for k in conta)} larg {N['larg']:.0f}/{R['larg']:.0f}"
                      + (f" — FALHA {','.join(falta)}" if falta else " — passa"))
            celulas.append(f"{'✓' if tudo else f'{len(ok) + lw}/{len(conta) + 1}':>7}")
        print("   " + f"{p:<13}" + " ".join(celulas))
    print("\n## Veredito")
    for p, propria, _, _, _ in OS_DA_FABRICA:
        outras = [f for (q, f), v in passa.items() if q == p and v and f != propria]
        print(f"   {p:<13} a sua foto {'✓' if passa.get((p, propria)) else '✗'} · passa também em: {', '.join(outras) or '—'}")


if __name__ == "__main__":
    if sys.argv[1] == "receitas":
        receitas(sys.argv[2])
    else:
        avalia(sys.argv[2], sys.argv[3])
