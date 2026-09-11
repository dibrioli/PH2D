#!/usr/bin/env python3
"""nextest-list-diff.py — prova que uma reorganização de testes não perde nenhum.

Compara duas saídas de `cargo nextest list --workspace --cargo-profile ci-test`
(antes e depois) pelo NOME do teste, tolerando o que uma reorganização muda de
propósito:

  · W1 (2026-09-10): `pkg::ficheiro fn`  →  `pkg::it ficheiro::fn`
    (um binário por crate: o ficheiro virou módulo, o nome ganhou o prefixo);
  · W2 (partir a shell): `ph2d-host-desktop::bin/ph2d-host-desktop familia::mod::fn`
    →  `ph2d-app-familia familia::mod::fn`  (o teste mudou de PACOTE, não de nome).

Chave de comparação: os últimos N segmentos do caminho `a::b::fn` SEM o pacote/binário —
por omissão N = 1 (o NOME da função, com contagem: dois testes homónimos contam 2 e os dois
têm de existir nos dois lados). `--depth 2` aperta para `módulo::fn` quando as duas listas têm
a mesma forma (⚠️ na forma da W1 o lado antigo só tem `fn`, e `--depth 2` leria a renomeação
como perda + novo — foi o que o auto-teste apanhou em 11/09). O relatório separa três populações:
  · MOVED    — mesma chave, pacote/binário diferente (esperado numa extracção);
  · ONLY-A   — estava antes e não está depois: um teste PERDEU-SE (vermelho);
  · ONLY-B   — novo depois (aceitável se for teste NOVO; o handoff lista-os).
Sai 0 só se ONLY-A estiver vazio. Um mesmo nome em dois sítios conta 2× (a
contagem também tem de bater).

Uso:  cargo nextest list --workspace --cargo-profile ci-test > /tmp/antes.txt
      … reorganização …
      cargo nextest list --workspace --cargo-profile ci-test > /tmp/depois.txt
      python3 scripts/nextest-list-diff.py /tmp/antes.txt /tmp/depois.txt [--depth 2]

⚠️ Corra as duas listas na MESMA árvore de dependências (mesmo `Cargo.lock`) e sem
filtro: uma lista filtrada devolve ONLY-A por construção.
"""
import re
import sys
import collections


def parse(path, depth):
    """→ (Counter{chave: n}, dict{chave: set(pacote::binário)})"""
    keys = collections.Counter()
    where = collections.defaultdict(set)
    for line in open(path, encoding="utf-8"):
        line = line.rstrip("\n")
        m = re.match(r"^(\S+)\s+(\S+)$", line)
        if not m:
            continue
        bin_id, test = m.group(1), m.group(2)
        segs = test.split("::")
        key = "::".join(segs[-depth:]) if len(segs) >= depth else test
        keys[key] += 1
        where[key].add(bin_id)
    return keys, where


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    depth = 1
    for i, a in enumerate(sys.argv):
        if a == "--depth":
            depth = int(sys.argv[i + 1])
    if len(args) != 2:
        print(__doc__)
        return 2
    a, wa = parse(args[0], depth)
    b, wb = parse(args[1], depth)
    only_a = {k: n for k, n in a.items() if b.get(k, 0) < n}
    only_b = {k: n for k, n in b.items() if a.get(k, 0) < n}
    moved = [k for k in a if k in b and wa[k] != wb[k]]
    print(f"antes: {sum(a.values())} testes ({len(a)} chaves) | depois: {sum(b.values())} ({len(b)})")
    print(f"MOVED (mesma chave, outro pacote/binário): {len(moved)}")
    for k in sorted(moved)[:20]:
        print(f"   {k}: {sorted(wa[k])} -> {sorted(wb[k])}")
    if len(moved) > 20:
        print(f"   … +{len(moved) - 20}")
    print(f"ONLY-A (perdidos): {len(only_a)}")
    for k in sorted(only_a)[:40]:
        print(f"   ✗ {k}  (em {sorted(wa[k])})")
    print(f"ONLY-B (novos): {len(only_b)}")
    for k in sorted(only_b)[:40]:
        print(f"   + {k}  (em {sorted(wb[k])})")
    return 1 if only_a else 0


if __name__ == "__main__":
    sys.exit(main())
