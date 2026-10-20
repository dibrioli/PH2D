#!/usr/bin/env python3
"""O índice das fotos da bateria do Rebelle: uma linha por (preset, cena), a partir da lista e das PNG em disco
(o `log.csv` dá o motivo das falhas). Colunas: preset · TOOL · nome · cena · ficheiro_A · ficheiro_B · estado ·
motivo · A_igual_B (as duas fotos byte-idênticas: a SIMULATION não mudou nada antes do DRY) · tinta_A_pct (% de
pixels da foto A a mais de 30 níveis (soma RGB) do papel branco: perto de 0 = o preset não marcou o papel).

uso: python3 -I indice_bateria.py <lista> <pasta das fotos> > <pasta>/indice.csv
"""
import csv
import sys
from pathlib import Path

import numpy as np
from PIL import Image


def main():
    lista, pasta = Path(sys.argv[1]), Path(sys.argv[2])
    motivo = {}
    log = pasta / "log.csv"
    if log.exists():
        for r in csv.DictReader(log.open()):
            chave = (r["TOOL"], r["nome"], r["cena"])
            motivo[chave] = "" if r["estado"] == "ok" else r["motivo"]
    w = csv.writer(sys.stdout, lineterminator="\n")
    w.writerow(["preset", "TOOL", "nome", "cena", "ficheiro_A", "ficheiro_B", "estado", "motivo", "A_igual_B", "tinta_A_pct"])
    for linha in lista.read_text().splitlines():
        if not linha or linha.startswith("#"):
            continue
        nm, tool, preset, _base, cenas = (linha.split("|") + ["", ""])[:5]
        for cena in (cenas or "gestos tamanhos sobre").split():
            a, b = f"{tool}/{nm}_{cena}_A.png", f"{tool}/{nm}_{cena}_B.png"
            ok = (pasta / a).exists() and (pasta / b).exists()
            igual = ok and (pasta / a).read_bytes() == (pasta / b).read_bytes()
            m = "" if ok else (motivo.get((tool, nm, cena)) or "não tentada")
            tinta = ""
            if ok:
                px = np.asarray(Image.open(pasta / a).convert("RGB")).astype(int)
                tinta = f"{100.0 * ((765 - px.sum(2)) > 30).mean():.2f}"
            w.writerow([preset, tool, nm, cena, a if ok else "", b if ok else "", "ok" if ok else "falha", m,
                        ("sim" if igual else "não") if ok else "", tinta])


if __name__ == "__main__":
    main()
