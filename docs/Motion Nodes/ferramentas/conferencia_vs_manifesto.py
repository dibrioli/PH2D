#!/usr/bin/env python3
"""A conferência contra o MANIFESTO — o instrumento que apanha a célula que envelheceu.

⚠️ **Porque ele existe.** Cada célula do doc 89 tem uma coluna «params hoje», e ela é uma
FOTOGRAFIA do nó no dia em que a célula foi escrita. Uma wave que acrescenta um param fecha a
sua célula e deixa as vizinhas a descrever um nó que já não existe — e o placar passa a contar
como ABERTO o que já shipou. Medido em 2026-08-19 na folha 01: a célula dizia
*"`motion.emitter`, 10 params"*, o manifesto tinha **20**, e **cinco das seis** linhas P1
daquela folha pediam coisas que já lá estavam.

⚠️ **Ele NÃO decide nada.** Uma contagem diferente é um sinal de *"vá reler esta célula"*, não
uma prova de que o item fechou: um param novo pode ser de outra célula. O que ele faz é impedir
que a diferença passe despercebida — o placar conta o que está escrito, e este conta se o
escrito ainda descreve o produto.

Correr (a fonte é o registry, não um doc):

    python3 "docs/Motion Nodes/ferramentas/conferencia_vs_manifesto.py"

Ele invoca a sonda `measure_node_params` por si. Com `--dump <ficheiro>` lê uma saída já
guardada (útil quando não se quer pagar o build).

Sai **vermelho** se alguma célula ABERTA declarar menos params do que o nó tem hoje.
"""

import os
import re
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
FOLHAS = os.path.join(os.path.dirname(HERE), '89_conferencia')
REPO = os.path.dirname(os.path.dirname(os.path.dirname(HERE)))

SPLIT = re.compile(r'\s*\|\s*')
SEP = re.compile(r'\|[\s:\-|]+\|')
# `motion.emitter` · **`field.box`** · `motion.mirror` · `motion.kaleidoscope`
NODE = re.compile(r'`([a-z_]+\.[a-z_0-9]+)`')
# "10 (`rate·life·…`)" · "**1** (`mode`)" · "5 (`key`…)"
COUNT = re.compile(r'\*{0,2}(\d+)\*{0,2}\s*\(')


def dump_from_cargo():
    """O param list de cada nó, do registry."""
    out = subprocess.run(
        ['cargo', 'test', '-q', '-p', 'ph2d-node-registry-init',
         '--test', 'measure_node_params', '--', '--ignored', '--nocapture'],
        cwd=REPO, capture_output=True, text=True)
    if out.returncode != 0:
        print(out.stdout + out.stderr, file=sys.stderr)
        sys.exit('a sonda measure_node_params nao correu')
    return out.stdout


def parse_dump(text):
    real = {}
    for line in text.splitlines():
        parts = line.split('\t')
        if len(parts) == 3 and '.' in parts[0] and parts[1].isdigit():
            real[parts[0]] = (int(parts[1]), parts[2].split())
    if not real:
        sys.exit('a sonda nao imprimiu nenhum no')
    return real


def cells(line):
    return [c.strip() for c in SPLIT.split(line.strip())[1:-1]]


def classify_open(v):
    """`True` se o veredito da célula ainda está ABERTO (P0/P1/P2, não fechado/refutado)."""
    if v.startswith('✅') or v.startswith('~~') or v.startswith('⛔'):
        return False
    return bool(re.match(r'\*?\*?P[012]', v))


def sweep(path, real):
    """Devolve as linhas suspeitas: (folha, nó, declarado, real, params)."""
    base = os.path.basename(path)
    lines = open(path, encoding='utf-8').read().split('\n')
    out, last = [], {}
    for i, ln in enumerate(lines):
        if not ln.lstrip().startswith('|'):
            continue
        c = cells(ln)
        if len(c) < 7 or SEP.fullmatch(ln.strip()):
            continue
        node = NODE.search(c[0])
        if not node or node.group(1) not in real:
            continue
        node = node.group(1)
        # «idem» herda a contagem da linha anterior do MESMO nó.
        m = COUNT.search(c[1])
        if m:
            last[node] = int(m.group(1))
        declared = last.get(node)
        if declared is None or not classify_open(c[5]):
            continue
        n, params = real[node]
        if declared != n:
            out.append((base, i + 1, node, declared, n, params))
    return out


def main():
    dump = None
    if '--dump' in sys.argv:
        dump = open(sys.argv[sys.argv.index('--dump') + 1], encoding='utf-8').read()
    real = parse_dump(dump if dump is not None else dump_from_cargo())

    suspects = []
    for f in sorted(os.listdir(FOLHAS)):
        if f.endswith('.md') and f != 'README.md':
            suspects += sweep(os.path.join(FOLHAS, f), real)

    if not suspects:
        print(f'✓ nenhuma celula aberta discorda do manifesto ({len(real)} nos lidos)')
        return 0

    print(f'{"folha":<30} {"linha":>5}  {"no":<26} {"diz":>4} {"tem":>4}')
    print('-' * 78)
    seen = set()
    for base, ln, node, declared, n, params in suspects:
        print(f'{base:<30} {ln:>5}  {node:<26} {declared:>4} {n:>4}')
        if node not in seen:
            seen.add(node)
            print(f'{"":<30} {"":>5}  → {" ".join(params)}')
    print()
    print(f'⚠️  {len(suspects)} celula(s) ABERTA(s) em {len(seen)} no(s) descrevem um nó que mudou.')
    print('    Releia cada uma: o que ela pede pode já estar no manifesto acima.')
    print('    (Uma contagem diferente NAO prova que a celula fechou — prova que ela envelheceu.)')
    return 1


if __name__ == '__main__':
    sys.exit(main())
