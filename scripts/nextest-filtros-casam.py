#!/usr/bin/env python3
"""Cada termo de cada `filter = '…'` do `.config/nextest.toml` casa >= 1 teste REAL.

Uso: cargo nextest list --workspace --cargo-profile ci-test --message-format json \\
         | python3 scripts/nextest-filtros-casam.py - [.config/nextest.toml]

Porquê (29/10, line/PainterWatercolor): partir `presets_de_fabrica.rs` em filhos mudou o nome do teste para
`presets_de_fabrica::portoes::a_largura_e_a_prometida`; o termo `test(=presets_de_fabrica::a_largura_e_a_prometida)`
do prazo de 10 min deixou de casar, o teste caiu no prazo global de 180 s e morreu na suíte (precisa ~600 s). Um
termo que casa ZERO é um prazo (ou uma faixa) que não se aplica a ninguém — e lê-se como «está protegido».

A semântica é a do nextest (filterset DSL): `test(=x)` nome exacto · `test(/re/)` regex (search) · `test(#g)` glob ·
`test(x)` contém · `package(x)` glob do nome do pacote (o default do `package`). Exit 0 todos casam · 1 algum casa
zero · 2 a entrada não se lê.
"""

import fnmatch
import json
import re
import sys


def termos(toml: str):
    """(linha, predicado, argumento) de cada `test(…)`/`package(…)` dos `filter = '…'`."""
    for n, linha in enumerate(toml.splitlines(), 1):
        m = re.match(r"\s*filter\s*=\s*'(.*)'\s*$", linha)
        if not m:
            continue
        for t in re.finditer(r"\b(test|package)\(([^)]*)\)", m.group(1)):
            yield n, t.group(1), t.group(2).strip()


def casa(pred: str, arg: str, nome: str) -> bool:
    if arg.startswith("="):
        return nome == arg[1:]
    if arg.startswith("/") and arg.endswith("/") and len(arg) >= 2:
        return re.search(arg[1:-1], nome) is not None
    if arg.startswith("#"):
        return fnmatch.fnmatchcase(nome, arg[1:])
    if arg.startswith("~"):
        return arg[1:] in nome
    return fnmatch.fnmatchcase(nome, arg) if pred == "package" else arg in nome


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__, file=sys.stderr)
        return 2
    try:
        lista = json.load(sys.stdin if sys.argv[1] == "-" else open(sys.argv[1]))
        toml = open(sys.argv[2] if len(sys.argv) > 2 else ".config/nextest.toml").read()
    except (OSError, ValueError) as e:
        print(f"[filtros] ✗ não li a entrada: {e}", file=sys.stderr)
        return 2
    testes, pacotes = [], set()
    for suite in lista.get("rust-suites", {}).values():
        pacotes.add(suite["package-name"])
        testes.extend(suite.get("testcases", {}).keys())
    # Piso de população: uma lista vazia (a compilação falhou, o formato mudou) passaria por vácuo.
    if len(testes) < 1000:
        print(f"[filtros] ✗ a lista deu só {len(testes)} testes — não é a da workspace", file=sys.stderr)
        return 2
    vistos = list(termos(toml))
    if not vistos:
        print("[filtros] ✗ nenhum `filter =` no nextest.toml — o formato mudou", file=sys.stderr)
        return 2
    mortos = [
        (n, p, a)
        for n, p, a in vistos
        if not any(casa(p, a, x) for x in (pacotes if p == "package" else testes))
    ]
    for n, p, a in mortos:
        print(f"[filtros] ✗ .config/nextest.toml:{n}: {p}({a}) casa ZERO testes", file=sys.stderr)
    print(f"[filtros] {len(vistos) - len(mortos)}/{len(vistos)} termos casam ({len(testes)} testes, {len(pacotes)} pacotes)")
    return 1 if mortos else 0


if __name__ == "__main__":
    sys.exit(main())
