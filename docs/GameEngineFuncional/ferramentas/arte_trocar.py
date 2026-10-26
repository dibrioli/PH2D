#!/usr/bin/env python3
"""Plano 12: as duas imagens do smoke de «trocar a imagem» — a folha do herói AZUL e os montes ao PÔR DO SOL.

São a arte de fábrica (os SVG de `crates/ph2d-render/src/atlas/fabrica/`) noutra cor, gravada como PNG que o
artista importa (o projecto tem 100 px/m):

- `heroi_azul.png` 800 × 400 = 8 × 4 células de 100 px (1 m, o tamanho do herói de fábrica);
- `montes_por_do_sol.png` 800 × 600 = um ladrilho de 8 × 6 m (o dos montes de fábrica).

    python3 docs/GameEngineFuncional/ferramentas/arte_trocar.py
"""
import colorsys
import io
import pathlib
import subprocess

from PIL import Image

RAIZ = pathlib.Path(__file__).resolve().parents[3]
FABRICA = RAIZ / "crates/ph2d-render/src/atlas/fabrica"
SAIDA = RAIZ / "docs/GameEngineFuncional/arte"


def rasteriza(svg: str, w: int, h: int) -> Image.Image:
    png = subprocess.run(
        ["rsvg-convert", "-w", str(w), "-h", str(h), str(FABRICA / svg)],
        check=True,
        capture_output=True,
    ).stdout
    return Image.open(io.BytesIO(png)).convert("RGBA")


def recolore(img: Image.Image, matiz: float, sat: float) -> Image.Image:
    """Roda a matiz (voltas) e escala a saturação; o alfa e a claridade ficam."""
    px = img.load()
    for y in range(img.height):
        for x in range(img.width):
            r, g, b, a = px[x, y]
            if a == 0:
                continue
            h, l, s = colorsys.rgb_to_hls(r / 255, g / 255, b / 255)
            r2, g2, b2 = colorsys.hls_to_rgb((h + matiz) % 1.0, l, min(1.0, s * sat))
            px[x, y] = (round(r2 * 255), round(g2 * 255), round(b2 * 255), a)
    return img


def main() -> None:
    SAIDA.mkdir(parents=True, exist_ok=True)
    # Verde do herói → azul.
    recolore(rasteriza("boneco.svg", 800, 400), 0.22, 1.0).save(SAIDA / "heroi_azul.png")
    # Montes azul-claros → laranja quente do fim de tarde.
    recolore(rasteriza("montes.svg", 800, 600), 0.58, 2.2).save(SAIDA / "montes_por_do_sol.png")


if __name__ == "__main__":
    main()
