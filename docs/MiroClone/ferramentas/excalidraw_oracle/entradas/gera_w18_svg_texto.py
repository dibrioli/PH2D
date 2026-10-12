#!/usr/bin/env python3
"""W18 — SVG com <text> para o oráculo (o Chromium) e para o gate do lado Rust.

Cada ficheiro isola UMA pergunta. Cores-chave (vermelho/verde/azul) separam elementos na mesma imagem.
Escreve em entradas/w18_texto/. Determinístico (só texto).
"""
import pathlib

OUT = pathlib.Path(__file__).resolve().parent / "w18_texto"
OUT.mkdir(exist_ok=True)
R, G, B = "#dc0000", "#00a000", "#0000dc"


def svg(w, h, corpo, extra=""):
    return (f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}"{extra}>\n'
            f'{corpo}\n</svg>\n')


CASOS = {
    # família presente no sistema (fc-list) e a genérica
    "t01_familia_presente": svg(400, 80, f'<text id="a" x="20" y="55" font-family="DejaVu Sans" font-size="40" fill="{R}">Hamburg</text>'),
    "t02_familia_ausente": svg(400, 80, f'<text id="a" x="20" y="55" font-family="Nao Existe Font" font-size="40" fill="{R}">Hamburg</text>'),
    "t03_sem_familia": svg(400, 80, f'<text id="a" x="20" y="55" font-size="40" fill="{R}">Hamburg</text>'),
    "t04_genericas": svg(420, 170,
        f'<text id="s" x="10" y="45" font-family="sans-serif" font-size="36" fill="{R}">Sans 123</text>'
        f'<text id="f" x="10" y="95" font-family="serif" font-size="36" fill="{G}">Serif 123</text>'
        f'<text id="m" x="10" y="145" font-family="monospace" font-size="36" fill="{B}">Mono 123</text>'),
    "t05_lista_familias": svg(400, 80, f'<text id="a" x="20" y="55" font-family="\'Nao Existe\', \'DejaVu Serif\', serif" font-size="40" fill="{R}">Hamburg</text>'),
    "t06_anchor": svg(400, 170,
        '<line x1="200" y1="0" x2="200" y2="170" stroke="#000" stroke-width="1"/>'
        f'<text id="s" x="200" y="45" font-family="DejaVu Sans" font-size="32" text-anchor="start" fill="{R}">start</text>'
        f'<text id="m" x="200" y="95" font-family="DejaVu Sans" font-size="32" text-anchor="middle" fill="{G}">middle</text>'
        f'<text id="e" x="200" y="145" font-family="DejaVu Sans" font-size="32" text-anchor="end" fill="{B}">end</text>'),
    "t07_tspan": svg(420, 160,
        f'<text id="a" x="10" y="50" font-family="DejaVu Sans" font-size="30" fill="{R}">Ab'
        f'<tspan id="b" fill="{G}" dx="15" dy="-10">cd</tspan>'
        f'<tspan id="c" x="200" y="120" fill="{B}" font-size="40">ef</tspan>gh</text>'),
    "t08_transform": svg(400, 300,
        f'<g transform="translate(200 150)"><text id="a" x="0" y="0" font-family="DejaVu Sans" font-size="36" fill="{R}" transform="rotate(30)">Rotated</text></g>'
        f'<text id="b" x="20" y="40" font-family="DejaVu Sans" font-size="20" fill="{B}" transform="scale(2 1)">Wide</text>'),
    "t09_peso_italico": svg(420, 170,
        f'<text id="n" x="10" y="45" font-family="DejaVu Sans" font-size="36" fill="{R}">Normal</text>'
        f'<text id="b" x="10" y="95" font-family="DejaVu Sans" font-size="36" font-weight="bold" fill="{G}">Bold</text>'
        f'<text id="i" x="10" y="145" font-family="DejaVu Sans" font-size="36" font-style="italic" fill="{B}">Italic</text>'),
    "t10_unidades_estilo": svg(420, 170,
        f'<text id="pt" x="10" y="45" font-family="DejaVu Sans" font-size="24pt" fill="{R}">24pt</text>'
        f'<text id="px" x="10" y="95" style="font-family: DejaVu Sans; font-size: 32px; fill: {G}">style 32px</text>'
        f'<g font-size="20" font-family="DejaVu Sans" fill="{B}"><text id="g" x="10" y="145">inherited 20</text></g>'),
    "t11_espacos": svg(420, 120,
        f'<text id="a" x="10" y="45" font-family="DejaVu Sans" font-size="28" fill="{R}">a    b\n  c</text>'
        f'<text id="b" x="10" y="95" font-family="DejaVu Sans" font-size="28" fill="{B}" xml:space="preserve">a    b</text>'),
    "t12_contorno": svg(400, 90, f'<text id="a" x="20" y="65" font-family="DejaVu Sans" font-size="56" font-weight="bold" fill="none" stroke="{R}" stroke-width="2">Hollow</text>'),
    "t13_viewbox": svg(400, 200, f'<text id="a" x="5" y="30" font-family="DejaVu Sans" font-size="20" fill="{R}">viewBox</text>').replace('viewBox="0 0 400 200"', 'viewBox="0 0 100 50"'),
    "t14_acentos": svg(420, 80, f'<text id="a" x="10" y="55" font-family="DejaVu Sans" font-size="36" fill="{R}">ação Über ñ</text>'),
    "t15_letter_spacing": svg(420, 80, f'<text id="a" x="10" y="55" font-family="DejaVu Sans" font-size="32" letter-spacing="8" fill="{R}">spaced</text>'),
    "t16_lista_x": svg(420, 80, f'<text id="a" x="10 60 110 160" y="55" font-family="DejaVu Sans" font-size="36" fill="{R}">abcdef</text>'),
}

for nome, txt in CASOS.items():
    (OUT / f"{nome}.svg").write_text(txt, encoding="utf-8")
print(f"{len(CASOS)} SVG em {OUT}")
