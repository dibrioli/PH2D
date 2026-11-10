#!/usr/bin/env python3
"""P13 R3 — oráculo da conversão RGB → Indexed (docs/PixelArt/P13_plano.md §3, §7).

Gera a fixture NOSSA `foto_p13` (64×64: céu, pele, folhagem; LCG determinista) e a
`paleta_rampas` (3 rampas + 3 soltas), corre o ImageMagick 7 sobre elas e grava
`crates/ph2d-pixel-rampa/tests/fixtures/imagemagick_indexed.json` com cabeçalho.
O pngquant (GPL-3.0+/ISC/BSD-2, só se CORRE) entra se estiver instalado.

Uso: python3 -I docs/PixelArt/ferramentas/oraculo_indexed_imagemagick.py
"""
import datetime, json, os, shutil, subprocess, sys, tempfile

AQUI = os.path.dirname(os.path.abspath(__file__))
RAIZ = os.path.abspath(os.path.join(AQUI, "..", "..", ".."))
SAIDA = os.path.join(RAIZ, "crates/ph2d-pixel-rampa/tests/fixtures/imagemagick_indexed.json")
W = H = 64

# paleta_rampas: pele 5 (0..4), água 4 (5..8), folha 6 (9..14), soltas 3 (15..17).
PALETA = [
    (74, 38, 30), (124, 70, 52), (176, 112, 84), (222, 160, 128), (250, 212, 186),
    (20, 40, 92), (40, 86, 160), (84, 146, 214), (160, 208, 240),
    (16, 40, 20), (30, 72, 30), (50, 108, 40), (84, 146, 52), (130, 182, 70), (186, 218, 120),
    (0, 0, 0), (255, 255, 255), (200, 30, 40),
]
RAMPAS = [("pele", [0, 1, 2, 3, 4]), ("agua", [5, 6, 7, 8]), ("folha", [9, 10, 11, 12, 13, 14])]


def foto():
    st = [12345]

    def rnd():
        st[0] = (st[0] * 1103515245 + 12345) & 0x7FFFFFFF
        return st[0] / 0x7FFFFFFF

    px = []
    for y in range(H):
        for x in range(W):
            if y < 24:  # céu: azul escuro no topo → ciano claro no horizonte
                t = y / 23
                c = (int(30 + 150 * t), int(70 + 150 * t), int(150 + 95 * t))
            elif y < 44:  # pele: esfera iluminada de cima-esquerda, em fundo de céu claro
                dx, dy = (x - 32) / 14, (y - 34) / 10
                if dx * dx + dy * dy <= 1:
                    s = max(0.0, min(1.0, 0.75 - 0.35 * dx - 0.45 * dy))
                    c = (int(70 + 180 * s), int(36 + 175 * s), int(28 + 158 * s))
                else:
                    c = (184, 222, 246)
            else:  # folhagem: verde em degradê com ruído
                t = (y - 44) / 19
                n = rnd() - 0.5
                g = 200 - 140 * t + 60 * n
                c = (int(max(0, min(255, 0.55 * g - 10))), int(max(0, min(255, g))),
                     int(max(0, min(255, 0.3 * g))))
            px.append(c)
    return px


def grava_ppm(caminho, w, h, px):
    with open(caminho, "wb") as f:
        f.write(b"P6\n%d %d\n255\n" % (w, h))
        f.write(bytes(v for c in px for v in c))


def le_ppm(caminho):
    with open(caminho, "rb") as f:
        dados = f.read()
    partes = dados.split(maxsplit=4)
    assert partes[0] == b"P6" and partes[3] == b"255", partes[:4]
    w, h, raw = int(partes[1]), int(partes[2]), partes[4]
    assert len(raw) == w * h * 3
    return [tuple(raw[i:i + 3]) for i in range(0, len(raw), 3)]


def para_indices(px):
    idx = {c: i for i, c in enumerate(PALETA)}
    fora = [c for c in px if c not in idx]
    assert not fora, f"cor fora da paleta: {fora[:3]}"
    return [idx[c] for c in px]


def versao(cmd):
    return subprocess.run(cmd, capture_output=True, text=True).stdout.splitlines()[0]


def main():
    px = foto()
    tmp = tempfile.mkdtemp()
    entrada, pal = os.path.join(tmp, "foto_p13.ppm"), os.path.join(tmp, "pal.ppm")
    grava_ppm(entrada, W, H, px)
    grava_ppm(pal, len(PALETA), 1, PALETA)
    comandos = {
        "nenhum": ["magick", entrada, "+dither", "-remap", pal],
        "o8x8": ["magick", entrada, "-ordered-dither", "o8x8", "-remap", pal],
        "floyd_steinberg": ["magick", entrada, "-dither", "FloydSteinberg", "-remap", pal],
    }
    saidas = {}
    for nome, cmd in comandos.items():
        out = os.path.join(tmp, nome + ".ppm")
        subprocess.run(cmd + [out], check=True)
        saidas[nome] = {"comando": " ".join(c.replace(tmp + "/", "") for c in cmd) + " out.ppm",
                        "indices": para_indices(le_ppm(out))}
    fx = {
        "ferramenta": "ImageMagick",
        "versao": versao(["magick", "-version"]),
        "licenca": "ImageMagick License (permissiva; /usr/share/doc/ImageMagick-7/LICENSE)",
        "data": datetime.date.today().isoformat(),
        "gerador": "docs/PixelArt/ferramentas/oraculo_indexed_imagemagick.py",
        "largura": W, "altura": H,
        "foto_p13_rgb": "".join("%02x%02x%02x" % c for c in px),
        "paleta": [list(c) for c in PALETA],
        "rampas": [{"nome": n, "indices": i} for n, i in RAMPAS],
        "saidas": saidas,
    }
    if shutil.which("pngquant"):
        fx["pngquant"] = {"versao": versao(["pngquant", "--version"]),
                          "licenca": "GPL-3.0-or-later/ISC/BSD-2 (só se corre)"}
    else:
        fx["pngquant"] = None
    with open(SAIDA, "w") as f:
        json.dump(fx, f, separators=(",", ":"))
        f.write("\n")
    print("gravado", SAIDA, {k: len(v["indices"]) for k, v in saidas.items()})


if __name__ == "__main__":
    sys.exit(main())
