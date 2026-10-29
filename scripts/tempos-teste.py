#!/usr/bin/env python3
"""Agrega o log de `tempos-teste-lib.sh`: quanto das corridas de teste é COMPILAR e quanto é CORRER.

    python3 scripts/tempos-teste.py                 # o log deste clone
    python3 scripts/tempos-teste.py --desde 2026-10-29 [ficheiro.tsv]

compilar = check + `Finished` do cargo; correr = o resto do relógio. O loadavg vem ao lado porque
acima de ~5 o relógio não vale nada (FLAKES_DE_CARGA.md): as corridas sob carga contam-se à parte.
"""
import csv
import subprocess
import sys
from collections import defaultdict


def ficheiro_padrao():
    d = subprocess.run(["git", "rev-parse", "--git-common-dir"], capture_output=True, text=True).stdout.strip()
    return f"{d or '.'}/ph2d-tempos-teste.tsv"


def main(argv):
    desde, caminho = "", None
    it = iter(argv)
    for a in it:
        if a == "--desde":
            desde = next(it)
        else:
            caminho = a
    caminho = caminho or ficheiro_padrao()
    try:
        linhas = list(csv.DictReader(open(caminho), delimiter="\t"))
    except FileNotFoundError:
        print(f"sem log em {caminho} — nenhuma corrida registada ainda")
        return 0
    linhas = [l for l in linhas if l["data"] >= desde]
    if not linhas:
        print("nenhuma corrida no intervalo")
        return 0
    soma = defaultdict(lambda: [0, 0.0, 0.0])
    for l in linhas:
        comp = float(l["check_s"]) + float(l["compilar_s"])
        corr = float(l["correr_s"])
        carga = "load>5" if max(float(l["load_ini"] or 0), float(l["load_fim"] or 0)) > 5 else "calmo"
        for k in (l["script"], f"{l['script']} ({carga})", "TOTAL"):
            soma[k][0] += 1
            soma[k][1] += comp
            soma[k][2] += corr
    print(f"{'':34} {'corridas':>8} {'compilar':>10} {'correr':>10} {'compilar %':>10}")
    for k in sorted(soma, key=lambda k: (k == "TOTAL", k)):
        n, c, r = soma[k]
        pct = 100 * c / (c + r) if c + r else 0
        print(f"{k:34} {n:8} {c:9.0f}s {r:9.0f}s {pct:9.0f}%")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
