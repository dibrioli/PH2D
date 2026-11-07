#!/usr/bin/env python3
"""P10c M8: o Godot remonta o mapa de aceitação a partir do nosso PNG + JSON, píxel a píxel.

Grava crates/ph2d-pixel-tiles/tests/fixtures/oracle_godot_remonta.json; o gate é `o_godot_remonta_o_mapa`.
Uso: PH2D_GPU=1 bash scripts/ph2d-run.sh python3 -I docs/PixelArt/ferramentas/oraculo_godot_remonta.py <pasta>
"""
import hashlib
import json
import os
import pathlib
import subprocess
import sys

from PIL import Image

AQUI = pathlib.Path(__file__).resolve().parent
RAIZ = AQUI.parents[2]
SAIDA = RAIZ / "crates/ph2d-pixel-tiles/tests/fixtures/oracle_godot_remonta.json"
FORMAS = ["square", "iso", "hex", "hex_flat"]
PRAZO = 90


def main():
    pasta = pathlib.Path(sys.argv[1]).resolve()
    pasta.mkdir(parents=True, exist_ok=True)
    env = dict(os.environ, PH2D_P10C_SAIDA=str(pasta))
    subprocess.run(["cargo", "test", "-p", "ph2d-pixel-io", "--profile", "smoke", "--test", "aceitacao_tiles",
                    "escreve_a_entrada_do_oraculo"], cwd=RAIZ, env=env, check=True, capture_output=True)
    casos = []
    for nome in FORMAS:
        d = pasta / nome
        j = json.loads((d / "mapa.json").read_text())
        r = subprocess.run(
            ["xvfb-run", "-a", "godot", "--rendering-driver", "opengl3", "--script",
             str(AQUI / "oraculo_godot_remonta.gd"), "--", str(d / "mapa.json"), str(d)],
            capture_output=True, text=True, timeout=PRAZO)
        corrida = [l for l in r.stdout.splitlines() if l.startswith("REMONTOU")]
        assert corrida, r.stdout[-500:] + r.stderr[-500:]
        w, h = j["canvas"]
        diferem, nossos = [], hashlib.sha256()
        amostras = []
        for q in range(len(j["frames"])):
            nosso = (d / f"nosso_{q}.rgba").read_bytes()
            nossos.update(nosso)
            foto = Image.open(d / f"godot_{q}.png").convert("RGBA").tobytes()
            assert len(foto) == len(nosso) == w * h * 4, (len(foto), len(nosso), w, h)
            n = 0
            for i in range(0, len(nosso), 4):
                a, b = nosso[i:i + 4], foto[i:i + 4]
                # Sem alfa, o RGB não conta: o Godot não o guarda igual.
                if a[3] == 0 and b[3] == 0:
                    continue
                if a != b:
                    n += 1
                    if len(amostras) < 4:
                        amostras.append([q, i // 4 % w, i // 4 // w, list(a), list(b)])
            diferem.append(n)
        casos.append({
            "forma": nome,
            "quadros": len(j["frames"]),
            "diferem": diferem,
            "amostras": amostras,
            "json_sha": (d / "json_sha.txt").read_text(),
            "tiles_sha": (d / "tiles_sha.txt").read_text(),
            "nossos_sha": nossos.hexdigest(),
            "corrida": corrida[0],
        })
        print(nome, "quadros", len(j["frames"]), "diferem", diferem, amostras[:2])
    SAIDA.write_text(json.dumps({
        "_cabecalho": {
            "oraculo": "Godot 4.7.2 (MIT), xvfb com opengl3 (llvmpipe), um TileMapLayer por camada, render real",
            "comando": "PH2D_GPU=1 bash scripts/ph2d-run.sh python3 -I docs/PixelArt/ferramentas/oraculo_godot_remonta.py <pasta>",
            "data": "2026-11-06",
            "regra": "diferem conta os pixeis de cada quadro; sem alfa nos dois, o RGB nao conta",
        },
        "casos": casos,
    }, indent=1))


main()
