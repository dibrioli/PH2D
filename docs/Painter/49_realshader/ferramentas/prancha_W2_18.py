#!/usr/bin/env python3
"""Prancha dos 18 presets W2 × Rebelle (W2, 27/10): o painel «NOSSO gestos A» de cada prancha já gravada
(coluna do meio, linha 1 de `pranchas/*.png`, 1524×2118) ao lado da foto Rebelle `gestos_A` (1000×640 → 500×320),
com o veredito na etiqueta. Só reaproveita imagens gravadas — nenhuma corrida da bancada.

    python3 -I docs/Painter/49_realshader/ferramentas/prancha_W2_18.py <raiz do repo> <pasta de trabalho>
"""
import pathlib
import subprocess
import sys

RAIZ = pathlib.Path(sys.argv[1])
TMP = pathlib.Path(sys.argv[2])
P = RAIZ / "docs/Painter/Pinceis/pranchas"
R = RAIZ / "docs/Painter/Pinceis/fixtures/rebelle_bateria/OIL_AND_ACRYLIC"
SAIDA = P / "W2_18_presets.jpg"
APROVADOS = {"Dabber", "Knife Smooth 3", "Knife Oily", "Knife Grainy", "Knife Dry"}
# (nome, prancha nossa, foto Rebelle gestos A) — a ordem do INDICE_W2.md.
PRESETS = [
    ("Metallic Flat 3", "W2_Metallic_flat_oleo.png", "metallic_flat_3"),
    ("Metallic Round 3", "W2_Metallic_round_oleo.png", "metallic_round_3"),
    ("Metallic Splats", "W2_Metallic_splats_oleo.png", "metallic_splats"),
    ("Dabber", "dabber.png", "dabber_dabber"),
    ("Dabber 2", "W2_Dabber_2.png", "dabber_dabber_2"),
    ("Dabber 3", "W2_Dabber_3.png", "dabber_dabber_3"),
    ("Dabber 4", "W2_Dabber_4.png", "dabber_dabber_4"),
    ("Dabber Fan", "W2_Dabber_fan.png", "dabber_fan"),
    ("Knife Smooth", "W2_Knife_smooth.png", "knife_knife_smooth"),
    ("Knife Smooth 2", "W2_Knife_smooth_2.png", "knife_knife_smooth_2"),
    ("Knife Smooth 3", "W2_Knife_smooth_3.png", "knife_knife_smooth_3"),
    ("Knife Textured", "W2_Knife_textured.png", "knife_knife_textured"),
    ("Knife Oily", "W2_Knife_oily.png", "knife_knife_oily"),
    ("Knife Stain", "W2_Knife_stain.png", "knife_knife_stain"),
    ("Knife Soft", "W2_Knife_soft.png", "knife_knife_soft"),
    ("Knife Grainy", "W2_Knife_grainy.png", "knife_knife_grainy"),
    ("Knife Dry", "W2_Knife_dry.png", "knife_knife_dry"),
    ("Knife Dry 2", "W2_Knife_dry_2.png", "knife_knife_dry_2"),
]
# O painel NOSSO gestos A: borda cinza em x 511/1012, y 31/352 (medido nas 23 pranchas: mesma grelha).
NOSSO = "500x320+512+32"


def magick(*a):
    subprocess.run(["magick", *map(str, a)], check=True)


TMP.mkdir(parents=True, exist_ok=True)
celulas = []
for i, (nome, prancha, foto) in enumerate(PRESETS):
    ok = nome in APROVADOS
    veredito, cor = ("APROVADO", "#1b7f2a") if ok else ("candidato", "#b8600b")
    nosso = TMP / f"{i:02d}_n.png"
    rebelle = TMP / f"{i:02d}_r.png"
    magick(P / prancha, "-crop", NOSSO, "+repage", nosso)
    magick(R / f"{foto}_gestos_A.png", "-resize", "500x320!", rebelle)
    par = TMP / f"{i:02d}_par.png"
    rot = dict(font="DejaVu-Sans", pointsize=15)

    def sub(img, texto, out):
        magick("(", "-size", "500x22", "-background", "#f2f2f2", "-fill", "#333", "-font", rot["font"],
               "-pointsize", rot["pointsize"], "-gravity", "center", f"label:{texto}", ")", img, "-append", out)

    sub(nosso, "PH2D (W2)", TMP / f"{i:02d}_n2.png")
    sub(rebelle, "Rebelle · gestos A", TMP / f"{i:02d}_r2.png")
    magick(
        "(", "-size", "1004x34", "-background", cor, "-fill", "white", "-font", "DejaVu-Sans-Bold",
        "-pointsize", "22", "-gravity", "center", f"label:{nome} — {veredito}", ")",
        "(", TMP / f"{i:02d}_n2.png", "-size", "4x342", "xc:#999", TMP / f"{i:02d}_r2.png", "+append", ")",
        "-append", "-bordercolor", "white", "-border", "8", par,
    )
    celulas.append(par)

linhas = []
for k in range(0, len(celulas), 3):
    ln = TMP / f"linha_{k // 3}.png"
    magick(*celulas[k:k + 3], "+append", ln)
    linhas.append(ln)
titulo = TMP / "titulo.png"
largura = int(subprocess.run(["magick", "identify", "-format", "%w", linhas[0]], capture_output=True,
                             text=True, check=True).stdout)
magick("-size", f"{largura}x48", "-background", "#222", "-fill", "white", "-font", "DejaVu-Sans-Bold",
       "-pointsize", "26", "-gravity", "center",
       "label:Óleo Real W2 — os 18 presets × Rebelle (verde = aprovado: Dabber, Knife Smooth 3, Oily, Grainy, Dry)",
       titulo)
magick(titulo, *linhas, "-append", "-quality", "88", SAIDA)
print(SAIDA)
