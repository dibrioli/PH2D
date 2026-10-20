#!/usr/bin/env python3
"""Fotografa a BATERIA no ArtRage 6 Demo (oráculo sob Wine, tela Xvfb :71, só software — não pede a placa).

Para cada pincel da lista e cada cena: `bateria_artrage.py gera` → File > Play Script… → diálogo (Accelerate
DESLIGADO: o tempo real de 8 ms por evento conta para o aerógrafo) → espera a tela PARAR (3 fotos iguais seguidas)
→ foto da pintura 1000×640 → File > Close Painting > Don't Save. Pré-requisito: `bash ~/Apps/artrage/inicia.sh`.

uso: fotos_artrage.py <lista.tsv> <pasta_saida> [--so NOME]   (saída: <pasta>/<Ferramenta>/<nome>_<cena>_A.png)
lista.tsv: nome<TAB>tool<TAB>preset|-<TAB>base_tool|-<TAB>base_preset|-<TAB>k[<TAB>cenas]
  (nome = Pasta__ficheiro; k = px por unidade de tamanho, ou tabela «s=px,…» — ver bateria_artrage.tamanho)
"""
import argparse
import hashlib
import subprocess
import sys
import time
from pathlib import Path

AQUI = Path(__file__).resolve().parent
PO = Path.home() / "Apps/pincel-oraculos"
TMP = Path.home() / "Apps/artrage/scripts/run"
CANVAS = "1000x640+460+235"             # a pintura nova de 1000×640 a 100 %, com a janela do ArtRage MAXIMIZADA (1920×1080)
CENAS = ["gestos", "tamanhos", "sobre"]


def x(*args):
    subprocess.run(["bash", str(PO / "x.sh"), "artrage", *map(str, args)], check=True, capture_output=True)


def foto(out, geo=CANVAS):
    subprocess.run(["bash", str(PO / "foto.sh"), "artrage", str(out), geo], check=True, capture_output=True)


def pixel(xx, yy):
    from PIL import Image
    t = TMP / "_px.png"
    foto(t, f"1x1+{xx}+{yy}")
    return Image.open(t).convert("RGB").getpixel((0, 0))


def estavel(maximo=400):
    t, ultimo, iguais, t0 = TMP / "_est.png", None, 0, time.time()
    while time.time() - t0 < maximo:
        time.sleep(1.0)
        foto(t)
        h = hashlib.md5(t.read_bytes()).hexdigest()
        iguais = iguais + 1 if h == ultimo else 0
        ultimo = h
        if iguais >= 3:
            return time.time() - t0
    raise SystemExit("a tela não parou")


def toca(script):
    w = "Z:" + str(script).replace("/", "\\")
    x("mousemove", 90, 15, "click", 1, "sleep", 1, "mousemove", 110, 317, "click", 1, "sleep", 2)
    x("type", "--delay", 12, w)
    x("key", "Return")
    time.sleep(2.5)
    r, g, b = pixel(968, 460)            # a caixa «Accelerate Playback»: verde-lima = ligada
    if g > 200 and b < 100:
        x("mousemove", 968, 460, "click", 1, "sleep", 0.5)
    x("mousemove", 1114, 745, "click", 1, "sleep", 0.3, "mousemove", 1580, 150)


def fecha():
    x("mousemove", 90, 15, "click", 1, "sleep", 1, "mousemove", 110, 154, "click", 1, "sleep", 1.5)
    x("mousemove", 979, 622, "click", 1, "sleep", 1, "mousemove", 1580, 150)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("lista")
    ap.add_argument("saida")
    ap.add_argument("--so")
    ap.add_argument("--cenas", default=",".join(CENAS))
    a = ap.parse_args()
    TMP.mkdir(parents=True, exist_ok=True)
    bat = AQUI / "bateria.json"
    for ln in Path(a.lista).read_text().splitlines():
        if not ln.strip() or ln.startswith("#"):
            continue
        campos = ln.split("\t")
        nome, tool, preset, btool, bpreset, k = campos[:6]
        cenas = campos[6] if len(campos) > 6 else a.cenas
        if a.so and nome != a.so:
            continue
        pasta = Path(a.saida) / nome.split("__")[0]
        pasta.mkdir(parents=True, exist_ok=True)
        for cena in cenas.split(","):
            s = TMP / f"{nome}_{cena}.arscript"
            cmd = [sys.executable, "-I", str(AQUI / "bateria_artrage.py"), "gera", str(bat), cena, str(s),
                   "--tool", tool, "--k", k]
            if preset != "-":
                cmd += ["--preset", preset]
            if btool != "-":
                cmd += ["--base-tool", btool] + (["--base-preset", bpreset] if bpreset != "-" else [])
            subprocess.run(cmd, check=True)
            t0 = time.time()
            toca(s)
            estavel()
            out = pasta / f"{nome.split('__')[-1]}_{cena}_A.png"
            foto(out)
            subprocess.run(["magick", str(out), "-strip", "-define", "png:compression-level=9", str(out)], check=True)
            fecha()
            print(f"{nome}\t{cena}\t{time.time() - t0:.0f}s\t{out}", flush=True)


if __name__ == "__main__":
    main()
