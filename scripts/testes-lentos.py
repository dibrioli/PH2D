#!/usr/bin/env python3
"""testes-lentos.py — o nível «lento» da suíte (item G, comissão de velocidade 29/10).

Os testes que MEDEM ≥ LIMIAR_S no `junit.xml` do nextest formam o nível lento: saem do
`nextest-impacted.sh` por omissão (`PH2D_NIVEL=rapido`) e correm na suíte completa (integração e
ship, `--workspace`, todos os níveis). A lista é MEDIDA, nunca escrita à mão.

    python3 scripts/testes-lentos.py --regerar [junit.xml …] [--saida F]   # escreve a lista
    python3 scripts/testes-lentos.py --filtro [--lista F] [--menos-prefixo P …]  # a expressão nextest
    python3 scripts/testes-lentos.py --confere junit.xml [--lista F]        # o gate: lento sem nível = exit 1

Com vários junit, vale o MÁXIMO por teste. Um teste que agora mede < LIMIAR_S SAI (a lista
acompanha a medição). Um TIMEOUT conta: o tempo do junit é o que correu antes de ser morto.
"""

import os
import re
import sys
import xml.etree.ElementTree as ET

# ⏱ 60 s: o `slow-timeout.period` do `.config/nextest.toml` (o nextest já os marca SLOW) e o corte
# medido na suíte de 29/10 — 55 testes ≥ 60 s somam 9 335 s de 16 616 s (56 %) sobre 27 883.
LIMIAR_S = 60.0

RAIZ = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
LISTA = os.path.join(RAIZ, ".config", "testes-lentos.tsv")
JUNIT = os.path.join(RAIZ, "target", "nextest", "default", "junit.xml")
LINHA = re.compile(r"^[A-Za-z0-9_.:/-]+\t[^\t]+\t[0-9]+\.[0-9]{3}$")


def ler_junit(caminho):
    """(fonte, {(binary_id, nome): segundos}) — o `classname` do nextest é o binary_id."""
    raiz = ET.parse(caminho).getroot()
    fonte = f"{raiz.get('uuid', '?')} {raiz.get('timestamp', '?')} ({raiz.get('tests', '?')} testes)"
    tempos = {}
    for t in raiz.iter("testcase"):
        k = (t.get("classname"), t.get("name"))
        tempos[k] = max(tempos.get(k, 0.0), float(t.get("time") or 0))
    return fonte, tempos


def lentos_de(junits):
    fontes, tempos = [], {}
    for j in junits:
        f, t = ler_junit(j)
        fontes.append(f)
        for k, s in t.items():
            tempos[k] = max(tempos.get(k, 0.0), s)
    return fontes, {k: s for k, s in tempos.items() if s >= LIMIAR_S}


def ler_lista(caminho):
    out = {}
    with open(caminho, encoding="utf-8") as fh:
        for linha in fh:
            if linha.startswith("#") or not linha.strip():
                continue
            if not LINHA.match(linha.rstrip("\n")):
                sys.exit(f"[testes-lentos] ✗ {caminho}: linha fora do formato gerado: {linha!r}")
            b, n, s = linha.rstrip("\n").split("\t")
            out[(b, n)] = float(s)
    return out


def regerar(junits, saida):
    fontes, lentos = lentos_de(junits)
    antes = ler_lista(saida) if os.path.exists(saida) else {}
    corpo = [f"{b}\t{n}\t{s:.3f}" for (b, n), s in sorted(lentos.items())]
    texto = (
        "# GERADO por scripts/testes-lentos.py --regerar — NÃO edite à mão.\n"
        f"# limiar: {LIMIAR_S:g} s (o slow-timeout do .config/nextest.toml)\n"
        + "".join(f"# fonte: {f}\n" for f in fontes)
        + f"# {len(corpo)} testes, {sum(lentos.values()):.0f} s somados\n"
        "# binary_id<TAB>nome<TAB>segundos\n"
        + "".join(c + "\n" for c in corpo)
    )
    with open(saida, "w", encoding="utf-8") as fh:
        fh.write(texto)
    entram = sorted(set(lentos) - set(antes))
    saem = sorted(set(antes) - set(lentos))
    print(f"[testes-lentos] {len(lentos)} testes ≥ {LIMIAR_S:g} s, {sum(lentos.values()):.0f} s → {saida}"
          f" (+{len(entram)} −{len(saem)})")
    return 0


def esc(s):
    return s.replace("\\", "\\\\").replace(")", "\\)").replace(",", "\\,")


def filtro(lista, menos_prefixo):
    termos = []
    for (b, n) in sorted(ler_lista(lista)):
        rede = next((p for p in menos_prefixo if n.startswith(p)), None)
        if rede:
            print(f"[testes-lentos] ⚠ {b} {n} é REDE OBRIGATÓRIA ({rede}) — volta ao nível rápido",
                  file=sys.stderr)
            continue
        termos.append(f"(binary_id(={esc(b)}) & test(={esc(n)}))")
    print(" | ".join(termos))
    return 0


def confere(junit, lista):
    _, lentos = lentos_de([junit])
    na_lista = ler_lista(lista) if os.path.exists(lista) else {}
    fora = sorted(k for k in lentos if k not in na_lista)
    for b, n in fora:
        print(f"[testes-lentos] ✗ lento SEM nível: {b} {n} ({lentos[(b, n)]:.1f} s)")
    print(f"[testes-lentos] {'✓' if not fora else '✗'} {len(lentos) - len(fora)}/{len(lentos)} "
          f"testes ≥ {LIMIAR_S:g} s estão na lista ({lista})")
    return 1 if fora else 0


def opcao(a, nome, omissao):
    if nome in a:
        i = a.index(nome)
        v = a[i + 1]
        del a[i:i + 2]
        return v
    return omissao


def main(a):
    if not a:
        print(__doc__)
        return 2
    lista = opcao(a, "--lista", LISTA)
    saida = opcao(a, "--saida", lista)
    menos = []
    while "--menos-prefixo" in a:
        menos.append(opcao(a, "--menos-prefixo", None))
    if a[0] == "--regerar":
        return regerar(a[1:] or [JUNIT], saida)
    if a[0] == "--filtro":
        return filtro(lista, menos)
    if a[0] == "--confere" and len(a) == 2:
        return confere(a[1], lista)
    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
