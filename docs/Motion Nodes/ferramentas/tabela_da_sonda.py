#!/usr/bin/env python3
"""tabela_da_sonda.py — a tabela de uma rodada de `mede_sonda_das_estrelas.sh` (doc 121 §9.15).

Uma linha por ficheiro: a SOMA dos passes da 2.ª janela do perfilador (o regime — a 1.ª inclui os
quadros de antes da capacidade medida), cada passe, o `gpu-busy(span)`, a parede da placa e a do
Vello, o relógio de CPU por fase e as arestas. ⚠️ A régua é a SOMA: o relógio POR passe mente na
fronteira (doc 121 §9.13).

uso: python3 tabela_da_sonda.py <dir-da-rodada> [--md]
"""
import re
import sys
from collections import defaultdict
from pathlib import Path

PASSES = ("render.contorno.conta", "render.contorno.escreve", "render.contorno.celulas", "render.formas")


def le(f: Path):
    t = f.read_text(errors="replace")
    janelas = re.findall(r"gpu-busy\(span\)=([\d.]+)ms.*\n\[gpu\]\s+(.*)", t)
    if len(janelas) < 2:
        return None
    span, detalhe = janelas[1]
    por = dict(re.findall(r"(render\.[\w.]+)=([\d.]+)ms", detalhe))
    m = re.search(r"placa ([\d.]+) ms · vello ([\d.]+) ms", t)
    cpu = re.search(r"decide ([\d.]+) ms · desenha ([\d.]+) ms · espera ([\d.]+) ms", t)
    ar = re.search(r"reservadas (\d+) · escritas (\d+) \(do contorno (\d+)\)", t)
    carga = re.search(r"ANTES: ([\d.]+)", t)
    med = re.search(r"mediana ([\d.]+) ms · primeiros ([\d. ]+) ms(?: · o da cena nova ([\d.]+) ms)?", t)
    linha = {p: float(por.get(p, 0.0)) for p in PASSES}
    linha["soma"] = sum(linha[p] for p in PASSES)
    linha["span"] = float(span)
    linha["placa"] = float(m.group(1)) if m else float("nan")
    linha["vello"] = float(m.group(2)) if m else float("nan")
    linha["cpu"] = tuple(float(x) for x in cpu.groups()) if cpu else None
    linha["arestas"] = tuple(int(x) for x in ar.groups()) if ar else None
    linha["carga"] = float(carga.group(1)) if carga else float("nan")
    linha["mediana"] = float(med.group(1)) if med else float("nan")
    linha["primeiro"] = float(med.group(2).split()[0]) if med else float("nan")
    linha["cena_nova"] = float(med.group(3)) if med and med.group(3) else float("nan")
    return linha


def main():
    d = Path(sys.argv[1])
    md = "--md" in sys.argv
    celulas = defaultdict(list)
    for f in sorted(d.glob("*.txt")):
        if f.name in ("tabela.txt",):
            continue
        m = re.match(r"(?:(.+)_)?(igpu|rtx)_(\w+?)(?:_tr(\d))?_(\d+)\.txt$", f.name)
        if not m:
            continue
        rotulo, placa, arranjo, tr, _ = m.groups()
        l = le(f)
        if l is None:
            print(f"# VAZIA: {f.name}", file=sys.stderr)
            continue
        celulas[(placa, arranjo, tr or "-", rotulo or "-")].append(l)
    cab = ["placa", "arranjo", "tr", "binario", "soma", "conta+escreve", "celulas", "formas", "span",
           "parede mediana", "vello", "mediana-span", "1.º cronometrado", "cena nova", "cpu decide/desenha/espera", "arestas res/escr/contorno", "carga"]
    sep = " | " if md else "\t"
    if md:
        print("| " + " | ".join(cab) + " |")
        print("|" + "---|" * len(cab))
    else:
        print(sep.join(cab))
    for (placa, arranjo, tr, rotulo), ls in sorted(celulas.items()):
        def col(fn, fmt="{:.2f}"):
            return " · ".join(fmt.format(fn(l)) for l in ls)
        cpu = " · ".join("/".join(f"{x:.3f}" for x in l["cpu"]) if l["cpu"] else "-" for l in ls)
        ar = ls[0]["arestas"]
        ar = "/".join(str(x) for x in ar) if ar else "-"
        row = [placa, arranjo, tr, rotulo,
               col(lambda l: l["soma"]),
               col(lambda l: l["render.contorno.conta"] + l["render.contorno.escreve"]),
               col(lambda l: l["render.contorno.celulas"]),
               col(lambda l: l["render.formas"]),
               col(lambda l: l["span"]),
               col(lambda l: l["mediana"]),
               col(lambda l: l["vello"]),
               col(lambda l: l["mediana"] - l["span"]),
               col(lambda l: l["primeiro"]),
               col(lambda l: l["cena_nova"]),
               cpu, ar,
               col(lambda l: l["carga"], "{:.1f}")]
        print(("| " + " | ".join(row) + " |") if md else sep.join(row))


if __name__ == "__main__":
    main()
