#!/usr/bin/env python3
# ⭐ CORRE O ORÁCULO DOS PARÂMETROS NOVOS DAS FORMAS (F10) — o Inkscape 1.4.4 (GPL/LGPL, triado por
# `pacman -Qi inkscape`: caixa preta, só a SAÍDA é gravada). Sem rato: cada forma NOSSA entra num SVG
# com os atributos `sodipodi:*`/`inkscape:*` do parâmetro, o Inkscape cozinha-a pela linha de comando
# (`object-to-path` + `export-plain-svg`) e o `d` que ele escreve é a fixture.
#
# Parâmetros medidos (os que o Affinity/Illustrator não dão a correr e o Inkscape sim):
#   · estrela — o `arg2` fora do meio (a TORÇÃO; `twist_deg_ydown` > 0 = horário no ecrã);
#   · estrela / polígono — `inkscape:rounded` (vértices LISOS: alças perpendiculares ao raio) e
#     `inkscape:randomized` (o vértice desloca-se ao acaso);
#   · espiral — `sodipodi:expansion` (a divergência: r = R·t^exp) e `sodipodi:t0` (onde a espiral começa).
#
# uso:  python3 -I corre_inkscape_params.py          (grava fixtures/<família>.json)
# requer: inkscape.
import json, math, os, re, subprocess, sys, tempfile, datetime

AQUI = os.path.dirname(os.path.abspath(__file__))
FIX = os.path.join(AQUI, "fixtures")
NS_S = "http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd"
NS_I = "http://www.inkscape.org/namespaces/inkscape"
CAB = ('<?xml version="1.0" encoding="UTF-8"?>\n<svg xmlns="http://www.w3.org/2000/svg" '
       'xmlns:sodipodi="%s" xmlns:inkscape="%s" width="800" height="600" viewBox="0 0 800 600">\n'
       % (NS_S, NS_I))
EST = 'style="fill:#88aaff;stroke:#000000;stroke-width:1"'


def star(i, sides, flat, rounded, randomized, twist=0.0, r2=45.0):
    a1 = -math.pi / 2
    a2 = a1 + math.pi / sides + math.radians(twist)
    return ('<path id="S%d" sodipodi:type="star" sodipodi:sides="%d" sodipodi:cx="400" sodipodi:cy="300" '
            'sodipodi:r1="100" sodipodi:r2="%g" sodipodi:arg1="%.12f" sodipodi:arg2="%.12f" '
            'inkscape:flatsided="%s" inkscape:rounded="%g" inkscape:randomized="%g" d="M 0,0" %s/>'
            % (i, sides, r2, a1, a2, "true" if flat else "false", rounded, randomized, EST))


def spiral(i, revo, expansion, t0):
    return ('<path id="S%d" sodipodi:type="spiral" sodipodi:cx="400" sodipodi:cy="300" '
            'sodipodi:expansion="%g" sodipodi:revolution="%g" sodipodi:radius="150" sodipodi:argument="0" '
            'sodipodi:t0="%g" d="M 0,0" style="fill:none;stroke:#000000;stroke-width:1"/>'
            % (i, expansion, revo, t0))


def familias():
    est, pol, esp = [], [], []
    for rounded in (0.0, 0.1, 0.25, 0.5, 1.0, -0.25):
        est.append(({"sides": 5, "r1": 100, "r2": 45, "rounded": rounded, "randomized": 0}, ("star", 5, False, rounded, 0)))
        pol.append(({"sides": 6, "r1": 100, "rounded": rounded, "randomized": 0}, ("star", 6, True, rounded, 0)))
    for randomized in (0.05, 0.2):
        est.append(({"sides": 5, "r1": 100, "r2": 45, "rounded": 0, "randomized": randomized}, ("star", 5, False, 0, randomized)))
        pol.append(({"sides": 6, "r1": 100, "rounded": 0, "randomized": randomized}, ("star", 6, True, 0, randomized)))
    for expansion in (0.5, 1.0, 2.0, 4.0):
        for t0 in (0.0, 0.25, 0.5):
            esp.append(({"revolution": 3, "radius": 150, "expansion": expansion, "t0": t0}, ("spiral", 3, expansion, t0)))
    tor = []
    for twist, rounded in ((15.0, 0.0), (-20.0, 0.0), (15.0, 0.25)):
        tor.append(({"sides": 5, "r1": 100, "r2": 45, "rounded": rounded, "randomized": 0,
                     "twist_deg_ydown": twist}, ("star", 5, False, rounded, 0, twist)))
    return {"estrela_rounded_randomized": est, "poligono_rounded_randomized": pol,
            "espiral_expansion_t0": esp, "estrela_torcida": tor}


def coze(formas):
    """Um SVG com todas as formas da família → o `d` de cada uma, cozido pelo Inkscape."""
    corpo = []
    for i, (_, f) in enumerate(formas):
        corpo.append(star(i, *f[1:]) if f[0] == "star" else spiral(i, *f[1:]))
    with tempfile.TemporaryDirectory(dir="/dev/shm") as d:
        src, out = os.path.join(d, "in.svg"), os.path.join(d, "out.svg")
        open(src, "w").write(CAB + "\n".join(corpo) + "\n</svg>\n")
        acts = "select-all:all;object-to-path;export-plain-svg;export-filename:%s;export-do" % out
        r = subprocess.run(["inkscape", "--actions=" + acts, src], capture_output=True, text=True,
                           env={**os.environ, "HOME": d}, timeout=120)
        txt = open(out).read() if os.path.exists(out) else ""
        if not txt:
            sys.exit("inkscape não escreveu a saída:\n" + r.stderr[-2000:])
    ds = {}
    for m in re.finditer(r'<path\b[^>]*>', txt):
        tag = m.group(0)
        mid = re.search(r'\bid="S(\d+)"', tag)
        md = re.search(r'\bd="([^"]*)"', tag)
        if mid and md:
            ds[int(mid.group(1))] = md.group(1)
    return ds


def main():
    os.makedirs(FIX, exist_ok=True)
    ver = subprocess.run(["inkscape", "--version"], capture_output=True, text=True).stdout.strip()
    for nome, formas in familias().items():
        ds = coze(formas)
        exps = [{"params": p, "d": ds.get(i)} for i, (p, _) in enumerate(formas)]
        falta = [e["params"] for e in exps if not e["d"]]
        if falta:
            sys.exit("%s: sem `d` para %s" % (nome, falta))
        doc = {"_cabecalho": {
            "oraculo": ver, "licenca": "GPL/LGPL (pacman -Qi inkscape) — caixa preta, só a saída",
            "como": "object-to-path + export-plain-svg sobre formas NOSSAS (corre_inkscape_params.py)",
            "centro": [400, 300], "y": "para BAIXO (SVG)", "data": datetime.date.today().isoformat()},
            "experiencias": exps}
        caminho = os.path.join(FIX, nome + ".json")
        json.dump(doc, open(caminho, "w"), indent=1, ensure_ascii=False)
        print("%-32s %2d formas → %s" % (nome, len(exps), os.path.relpath(caminho, AQUI)))


if __name__ == "__main__":
    main()
