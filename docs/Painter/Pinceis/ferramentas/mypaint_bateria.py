#!/usr/bin/env python3
"""O oráculo MyPaint da BATERIA do EXAME: cada `.myb` instalado pinta bateria.json pelo motor libmypaint (ISC).

O motor corre em `mypaint_bateria.c` (compilado à primeira corrida para ~/.cache/ph2d-oraculos/); este script
traduz a bateria em comandos, compõe a camada (transparente, pré-multiplicada, 1.0 = 1<<15) sobre o papel
branco e grava uma PNG por cena. MyPaint não tem molhado/seco: `simula`/`seca` são no-ops e a foto A = B
(só se grava uma). Lei do traço (o que a bateria não fixa e este oráculo escolhe — 02_ORACULOS.md):
  · tempo: DT s por evento (omissão 0,010) — o passo 3 px = 300 px/s (lento), o passo 25 px = 2500 px/s (rápido)
  · caneta: PAIRA 0,5 s (50 eventos de pressão 0) no 1.º ponto antes de pousar e levanta com 1 evento de pressão 0
    no último — sem pairar, o `reset` deixa o seguimento lento (slow_tracking) a partir de (0,0) e cada traço nasce
    com um GANCHO apontado ao canto superior esquerdo (medido na 1.ª corrida: o «1» das verticais da caneta)
  · tamanho: radius_logarithmic = ln(size_px / 2); cor: color_h/s/v do RGB da bateria; inclinação 0
  · superfície MyPaintTiledSurface2 (o pigmento espectral do MyPaint 2.x), stroke_to_2 (zoom 1, rotação 0)

uso: python3 -I mypaint_bateria.py [--pacotes classic,deevad] [--pincel X.myb ...] [--saida DIR] [--dt 0.01]
     sem --pacotes nem --pincel: todos os pacotes de /usr/share/mypaint-data/2.0/brushes
"""
import argparse
import colorsys
import hashlib
import json
import math
import os
import re
import subprocess
import sys
import tempfile

import numpy as np
from PIL import Image

AQUI = os.path.dirname(os.path.abspath(__file__))
PINCEIS = "/usr/share/mypaint-data/2.0/brushes"
CACHE = os.path.expanduser("~/.cache/ph2d-oraculos")
UM = float(1 << 15)
PAIRA = 50


def binario():
    fonte = os.path.join(AQUI, "mypaint_bateria.c")
    exe = os.path.join(CACHE, "mypaint_bateria")
    if not os.path.exists(exe) or os.path.getmtime(exe) < os.path.getmtime(fonte):
        os.makedirs(CACHE, exist_ok=True)
        flags = subprocess.check_output(["pkg-config", "--cflags", "--libs", "libmypaint"], text=True).split()
        subprocess.check_call(["cc", "-O2", "-std=c11", "-o", exe, fonte, *flags, "-lm"])
    return exe


def nome_seguro(s):
    return re.sub(r"[^A-Za-z0-9._+-]", "_", s)


def comandos_da_cena(cena, preset, base, dt, w, h, raw):
    out = []
    pincel = None
    for op in cena["ops"]:
        k = op["op"]
        if k == "pincel":
            pincel = preset if op["quem"] == "preset" else base
            out.append(f"pincel {pincel}")
        elif k == "cor":
            hh, ss, vv = colorsys.rgb_to_hsv(*(c / 255.0 for c in op["rgb"]))
            out.append(f"hsv {hh:.6f} {ss:.6f} {vv:.6f}")
        elif k == "traco":
            out.append(f"raio_log {math.log(op['size_px'] / 2.0):.6f}")
            pts = op["pts"]
            ev = [(pts[0][0], pts[0][1], 0.0)] * PAIRA + [*map(tuple, pts), (pts[-1][0], pts[-1][1], 0.0)]
            out.append(f"traco {len(ev)} {dt}")
            out.extend(f"{x} {y} {p}" for x, y, p in ev)
        elif k == "foto" and op["nome"] == "A":
            out.append(f"foto {raw}")
        # simula / seca / foto B: o MyPaint não tem tempo de secagem — A = B
    return out


def compoe(raw, w, h, papel):
    a16 = np.fromfile(raw, dtype=np.uint16).reshape(h, w, 4).astype(np.float64) / UM
    assert a16.max() <= 1.0 + 1e-6, f"escala inesperada: max {a16.max()}"
    alfa = a16[..., 3:4]
    rgb = a16[..., :3] + np.array(papel, dtype=np.float64) / 255.0 * (1.0 - alfa)
    img = np.clip(np.round(rgb * 255.0), 0, 255).astype(np.uint8)
    return img, float(alfa.sum()), int((alfa[..., 0] > 1.0 / 255).sum())


def pinta(exe, bateria, preset, saida_base, dt):
    w, h = bateria["tela"]
    papel = bateria["papel"]
    linhas = [f"tela {w} {h}"]
    raws = {}
    tmp = tempfile.mkdtemp(prefix="mpb_")
    for cena in bateria["cenas"]:
        raw = os.path.join(tmp, cena["nome"] + ".raw")
        raws[cena["nome"]] = raw
        linhas.append("limpa")
        linhas += comandos_da_cena(cena, preset, preset, dt, w, h, raw)
    r = subprocess.run([exe], input="\n".join(linhas) + "\n", text=True, capture_output=True)
    if r.returncode != 0:
        raise RuntimeError(f"{preset}: rc={r.returncode} {r.stderr.strip()}")
    medidas = {}
    for nome, raw in raws.items():
        img, tinta, px = compoe(raw, w, h, papel)
        png = f"{saida_base}_{nome}.png"
        Image.fromarray(img, "RGB").save(png, optimize=True)
        medidas[nome] = {"tinta_sum_alfa": round(tinta, 1), "px_com_tinta": px,
                         "sha256": hashlib.sha256(open(png, "rb").read()).hexdigest()[:16]}
        os.remove(raw)
    os.rmdir(tmp)
    return medidas


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--pacotes", default="")
    ap.add_argument("--pincel", nargs="*", default=[])
    ap.add_argument("--saida", default=os.path.join(AQUI, "..", "fixtures", "mypaint_bateria"))
    ap.add_argument("--dt", type=float, default=0.010)
    ap.add_argument("--bateria", default=os.path.join(AQUI, "bateria.json"))
    a = ap.parse_args()
    bat_bytes = open(a.bateria, "rb").read()
    bateria = json.loads(bat_bytes)
    exe = binario()
    alvos = [(os.path.basename(os.path.dirname(p)), p) for p in a.pincel]
    pacotes = a.pacotes.split(",") if a.pacotes else ([] if a.pincel else sorted(
        d for d in os.listdir(PINCEIS) if os.path.isdir(os.path.join(PINCEIS, d))))
    for pac in pacotes:
        for f in sorted(os.listdir(os.path.join(PINCEIS, pac))):
            if f.endswith(".myb"):
                alvos.append((pac, os.path.join(PINCEIS, pac, f)))
    manifesto = {"bateria_sha256": hashlib.sha256(bat_bytes).hexdigest(), "dt_s": a.dt, "pinceis": {}}
    falhas = []
    for pac, p in alvos:
        d = os.path.join(a.saida, pac)
        os.makedirs(d, exist_ok=True)
        nome = os.path.basename(p)[:-4]
        try:
            m = pinta(exe, bateria, p, os.path.join(d, nome_seguro(nome)), a.dt)
            manifesto["pinceis"][f"{pac}/{nome}"] = {"ficheiro": f"{pac}/{nome_seguro(nome)}", **m}
            print(f"ok {pac}/{nome}", flush=True)
        except Exception as e:  # um pincel que o motor recusa não pára o lote — fica no manifesto
            falhas.append(f"{pac}/{nome}: {e}")
            print(f"FALHA {pac}/{nome}: {e}", file=sys.stderr, flush=True)
    manifesto["falhas"] = falhas
    mpath = os.path.join(a.saida, "manifesto.json")
    if os.path.exists(mpath) and (a.pacotes or a.pincel):
        velho = json.load(open(mpath))
        velho["pinceis"].update(manifesto["pinceis"])
        velho["falhas"] = sorted(set(velho.get("falhas", [])) | set(falhas))
        manifesto = {**velho, "bateria_sha256": manifesto["bateria_sha256"], "dt_s": a.dt}
    json.dump(manifesto, open(mpath, "w"), indent=1, ensure_ascii=False, sort_keys=True)
    print(f"{len(alvos) - len(falhas)} pintados, {len(falhas)} falhas")


if __name__ == "__main__":
    main()
