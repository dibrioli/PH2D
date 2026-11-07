#!/usr/bin/env python3
"""P10c: a rodada do Godot antes da lei de iso e hex (docs/PixelArt/P10_oraculos.md, M3b, M2iso, M8a).

Corre sem janela o clique pixel a pixel e o terreno em iso; com janela (xvfb) a ordem de desenho e os
flips renderizados. Grava crates/ph2d-pixel-tiles/tests/fixtures/oracle_godot_p10c.json.
Uso: PH2D_GPU=1 bash scripts/ph2d-run.sh python3 -I docs/PixelArt/ferramentas/oraculo_godot_p10c.py <pasta>
"""
import json
import pathlib
import subprocess
import sys

from PIL import Image

AQUI = pathlib.Path(__file__).resolve().parent
RAIZ = AQUI.parents[2]
SAIDA = RAIZ / "crates/ph2d-pixel-tiles/tests/fixtures/oracle_godot_p10c.json"
PRAZO = 90
BITS = {"x": 0x80000000, "y": 0x40000000, "d": 0x20000000, "id_mask": 0x1FFFFFFF}


def origem(forma, c, r, w, h):
    if forma == "iso":
        return ((c - r) * w // 2, (c + r) * h // 2)
    if forma == "hex":
        return (c * w + (r % 2) * (w // 2), r * 3 * h // 4)
    if forma == "hex_flat":
        return (c * 3 * w // 4, r * h + (c % 2) * (h // 2))
    return (c * w, r * h)


def entrada(pasta, nome, forma, w, h, n, colunas, canvas, origem_px, tiles, pinta):
    """Uma entrada no formato do nosso JSON: `pinta(id, x, y)` dá a cor do tile `id`."""
    d = pasta / nome
    d.mkdir(parents=True, exist_ok=True)
    linhas = (n + colunas - 1) // colunas
    img = Image.new("RGBA", (colunas * w, linhas * h), (0, 0, 0, 0))
    px = img.load()
    for i in range(1, n):
        for y in range(h):
            for x in range(w):
                px[i % colunas * w + x, i // colunas * h + y] = pinta(i, x, y)
    img.save(d / "tiles.png")
    j = {
        "format": "ph2d-tilemap",
        "version": 1,
        "image": "tiles.png",
        "columns": colunas,
        "tile_count": n,
        "tile_w": w,
        "tile_h": h,
        "shape": forma,
        "flip_bits": BITS,
        "canvas": canvas,
        "frames": [{"duration_ms": 100}],
        "terrains": [],
        "layers": [
            {
                "name": "t",
                "opacity": 255,
                "visible": True,
                "cels": [dict(tiles, frame=0, origin=origem_px)],
            }
        ],
    }
    (d / "mapa.json").write_text(json.dumps(j))
    return d


def godot_janela(d):
    r = subprocess.run(
        ["xvfb-run", "-a", "godot", "--rendering-driver", "opengl3", "--script",
         str(AQUI / "oraculo_godot_remonta.gd"), "--", str(d / "mapa.json"), str(d)],
        capture_output=True, text=True, timeout=PRAZO)
    return [l for l in r.stdout.splitlines() if l.startswith("REMONTOU")], r.stderr[-400:]


def cor_de(i):
    return ((i * 37) % 256, (i >> 8) * 60 % 256, 200, 255)


def ordem(pasta, forma, w, h):
    """Tiles opacos, um por célula: a foto diz quem fica por cima."""
    nx, ny = 20, 20
    n = nx * ny + 1
    tiles = {"col": 0, "row": 0, "w": nx, "h": ny, "tiles": list(range(1, n))}
    pts = [origem(forma, c, r, w, h) for r in range(ny) for c in range(nx)]
    ox, oy = -min(p[0] for p in pts), -min(p[1] for p in pts)
    cw, ch = max(p[0] for p in pts) + ox + w, max(p[1] for p in pts) + oy + h
    d = entrada(pasta, f"ordem_{forma}", forma, w, h, n, 20, [cw, ch], [ox, oy], tiles,
                lambda i, x, y: cor_de(i))
    saiu, erro = godot_janela(d)
    foto = Image.open(d / "godot_0.png").convert("RGBA")
    assert foto.size == (cw, ch), foto.size
    candidatos = {
        "linha_coluna": sorted(((r, c) for r in range(ny) for c in range(nx))),
        "y_x": sorted(((r, c) for r in range(ny) for c in range(nx)),
                      key=lambda rc: (origem(forma, rc[1], rc[0], w, h)[1], origem(forma, rc[1], rc[0], w, h)[0])),
        "coluna_linha": sorted(((r, c) for r in range(ny) for c in range(nx)), key=lambda rc: (rc[1], rc[0])),
    }
    fp = foto.load()
    res = {}
    for nome, lista in candidatos.items():
        topo = {}
        for r, c in lista:
            x0, y0 = origem(forma, c, r, w, h)
            i = r * nx + c + 1
            for y in range(h):
                for x in range(w):
                    topo[(x0 + ox + x, y0 + oy + y)] = i
        res[nome] = sum(1 for (x, y), i in topo.items() if fp[x, y] != cor_de(i))
    return {"forma": forma, "tile": [w, h], "pixeis": cw * ch, "diferem_por_ordem": res,
            "corrida": saiu, "erro": erro if not saiu else ""}


def flips(pasta):
    """As 8 combinações: a nossa lei é D primeiro, depois X e Y (M4, Trial)."""
    w = 8
    assim = lambda x, y: (x * 30, y * 30, 90, 255)
    def lei(t, sx, sy):
        x, y = (sy, sx) if t & BITS["d"] else (sx, sy)
        if t & BITS["x"]:
            x = w - 1 - x
        if t & BITS["y"]:
            y = w - 1 - y
        return x, y
    combos = [1 | (BITS["x"] if m & 1 else 0) | (BITS["y"] if m & 2 else 0) | (BITS["d"] if m & 4 else 0)
              for m in range(8)]
    tiles = {"col": 0, "row": 0, "w": 8, "h": 1, "tiles": combos}
    d = entrada(pasta, "flips", "square", w, w, 2, 2, [64, 8], [0, 0], tiles, lambda i, x, y: assim(x, y))
    saiu, erro = godot_janela(d)
    fp = Image.open(d / "godot_0.png").convert("RGBA").load()
    difere = 0
    for k, t in enumerate(combos):
        for sy in range(w):
            for sx in range(w):
                x, y = lei(t, sx, sy)
                if fp[k * w + x, y] != assim(sx, sy):
                    difere += 1
    return {"combinacoes": 8, "pixeis_diferem": difere, "corrida": saiu, "erro": erro if not saiu else ""}


def main():
    pasta = pathlib.Path(sys.argv[1]).resolve()
    pasta.mkdir(parents=True, exist_ok=True)
    r = subprocess.run(["godot", "--headless", "--script", str(AQUI / "oraculo_godot_p10c.gd")],
                       capture_output=True, text=True, timeout=PRAZO)
    m3b, m2 = [], None
    versao = None
    for l in r.stdout.splitlines():
        if l.startswith("M3B "):
            m3b.append(json.loads(l[4:]))
        elif l.startswith("M2ISO "):
            m2 = json.loads(l[6:])
        elif l.startswith("M0 "):
            versao = json.loads(l[3:])["versao"]
    assert m3b and m2, r.stderr[-800:]
    ordens = [ordem(pasta, "iso", 32, 16), ordem(pasta, "hex", 16, 16), ordem(pasta, "hex_flat", 16, 16)]
    fl = flips(pasta)
    fix = {
        "_cabecalho": {
            "oraculo": f"Godot {versao} (MIT), --headless e xvfb com opengl3",
            "comando": "PH2D_GPU=1 bash scripts/ph2d-run.sh python3 -I docs/PixelArt/ferramentas/oraculo_godot_p10c.py <pasta>",
            "data": "2026-11-06",
            "m3b": "local_to_map do centro de cada pixel; linhas em corridas [quantos,c,r]; centros [c,r,x,y]",
        },
        "M3B": m3b,
        "M2ISO": m2,
        "ORDEM": ordens,
        "FLIPS": fl,
    }
    SAIDA.write_text(json.dumps(fix, separators=(",", ":")))
    print("versao", versao, "bytes", SAIDA.stat().st_size)
    print("M2ISO", {k: v for k, v in m2.items()})
    for o in ordens:
        print("ORDEM", o)
    print("FLIPS", fl)


main()
