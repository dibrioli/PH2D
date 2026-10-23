"""Gera as duas imagens da porta da cena 8 do editor de scripts (PH2D_CODE_SMOKE=8).

door_closed.png: a porta de madeira fechada, com a moldura e a maçaneta.
door_open.png: a mesma moldura com o vão aberto (luz lá dentro) e a folha encostada à esquerda.
Uso: python3 gera_portas.py (escreve ao lado deste ficheiro).
"""
from pathlib import Path
from PIL import Image, ImageDraw

W, H = 64, 128
MOLDURA = (70, 46, 30, 255)
MADEIRA = (176, 120, 80, 255)
TABUA = (140, 92, 58, 255)
METAL = (240, 200, 90, 255)
LUZ = (255, 236, 170, 255)
LUZ_FUNDO = (250, 214, 120, 255)

def moldura(d):
    d.rectangle([0, 0, W - 1, H - 1], fill=MOLDURA)

def fechada():
    im = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    moldura(d)
    d.rectangle([6, 6, W - 7, H - 1], fill=MADEIRA)
    for x in (20, 34, 48):
        d.line([x, 6, x, H - 1], fill=TABUA, width=2)
    d.ellipse([46, 64, 54, 72], fill=METAL)
    return im

def aberta():
    im = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    moldura(d)
    d.rectangle([6, 6, W - 7, H - 1], fill=LUZ)
    d.rectangle([6, H // 2, W - 7, H - 1], fill=LUZ_FUNDO)
    d.polygon([(6, 6), (18, 14), (18, H - 1), (6, H - 1)], fill=MADEIRA)
    return im

aqui = Path(__file__).resolve().parent
fechada().save(aqui / "door_closed.png")
aberta().save(aqui / "door_open.png")
