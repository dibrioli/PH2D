#!/usr/bin/env python3
"""A conferência contra o MANIFESTO — o instrumento que apanha a célula que envelheceu.

⚠️ **Porque ele existe.** Cada célula do doc 89 tem uma coluna «params hoje», e ela é uma
FOTOGRAFIA do nó no dia em que a célula foi escrita. Uma wave que acrescenta um param fecha a
sua célula e deixa as vizinhas a descrever um nó que já não existe — e o placar passa a contar
como ABERTO o que já shipou. Medido em 2026-08-19 na folha 01: a célula dizia
*"`motion.emitter`, 10 params"*, o manifesto tinha **20**, e **cinco das seis** linhas P1
daquela folha pediam coisas que já lá estavam.

⚠️ **Ele NÃO decide nada.** Um aviso é um sinal de *"vá reler esta célula"*, não uma prova de
que o item fechou. O que ele faz é impedir que a diferença passe despercebida — o placar conta
o que está escrito, e este conta se o escrito ainda descreve o produto.

**Ele imprime DOIS sinais, e são de forças diferentes:**

1. **O sinal FORTE** — o param que a coluna «default que reduz» nomeia **já está no
   manifesto**. Ele aponta o item, não a vizinhança dele.
2. **O sinal fraco** — a contagem da coluna «params hoje» discorda da real. Diz só que o nó
   mudou desde que a célula foi escrita; o que mudou pode ser de outra célula.

⚠️ **A calibração do sinal FORTE, medida em 2026-08-19 sobre as 7 células que ele acusou:
DUAS eram verdadeiras** (o `probability` do `motion.emitter` e o `lacunarity` do
`motion.noise`, as duas já shipadas) **e cinco eram falsos positivos, todos da mesma forma:
o nome existe, com outro significado ou com menos valores.**

| falso positivo | o param existe… | …e a célula pede |
|---|---|---|
| `motion.emitter` `emit_mode` | `Time` / `Burst` | um **terceiro** valor, `Distance` |
| `motion.stagger` `ease_curve` | oito curvas enumeradas | uma **nona**, `Custom`, com curva livre |
| `motion.path` `align` | `Tangent` | um modo **`Normal`** ao lado dele |
| `motion.mixer` `mode` | Avg / Add / Blend | o **vocabulário** de oito do `field.combine` |
| `source.shape` `start` | o início do **SWEEP** (graus) | o `start` do **TRIM** (fracção do contorno) |

*Um homónimo e um enum com um valor a menos leem igual num nome.* É por isso que a saída
imprime também o início da coluna «falta» de cada linha: o que decide é ler.

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
# "10 (`rate·life·…`)" · "**1** (`mode`)" · "3 — `rise` 0.5 …"
#
# ⚠️ **ANCORADA no início da célula, e isso é uma correção de 2026-08-19.** Sem a âncora ela
# casava o `5` de `` `rise` 0.5 (−10‥5) `` no MEIO de uma célula e acusava o `pulse.compare`
# de declarar 5 params quando ele declara 3 — um instrumento que grita sobre nada é um
# instrumento que se aprende a ignorar.
COUNT = re.compile(r'^\*{0,2}(\d+)\*{0,2}\s*[(\u2014-]')

# **O SINAL FORTE.** A coluna «default que reduz» nomeia o param que a CURA acrescentaria
# (`align=0`, `size_random = 0`, `dir_mode = Angle`, `probability = 1`). Se esse nome já está
# no manifesto, a célula quase de certeza já shipou — e ao contrário da contagem, isto aponta
# o item, não a vizinhança dele.
#
# ⚠️ Medido na folha 01 (a única cujo estado eu conhecia ao escrever isto): ele acerta nas
# CINCO que tinham shipado e **não** acusa a sexta (`inherit = 0`, que de facto não existe).
DEFAULT_PARAM = re.compile(r'`?\b([a-z][a-z_0-9]{2,})`?\s*=')


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
    """Devolve `(ja_existe, contagem_discorda, sem_contagem)` desta folha."""
    base = os.path.basename(path)
    lines = open(path, encoding='utf-8').read().split('\n')
    shipped, out, last, unread = [], [], {}, 0
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
        if not classify_open(c[5]):
            continue
        # O SINAL FORTE, e ele não precisa da coluna «params hoje» nenhuma.
        named = {m for m in DEFAULT_PARAM.findall(c[6]) if m in real[node][1]}
        if named:
            shipped.append((base, i + 1, node, sorted(named), c[2][:70]))
        if declared is None:
            # ⚠️ Uma célula cuja coluna «params hoje» não começa por um número não é
            # comparável — e o silêncio sobre ela tem de ser DITO, senão um `✓` limpo pode
            # querer dizer *"nada discorda"* ou *"não consegui ler nada"*.
            unread += 1
            continue
        n, params = real[node]
        if declared != n:
            out.append((base, i + 1, node, declared, n, params))
    return shipped, out, unread


def main():
    dump = None
    if '--dump' in sys.argv:
        dump = open(sys.argv[sys.argv.index('--dump') + 1], encoding='utf-8').read()
    real = parse_dump(dump if dump is not None else dump_from_cargo())

    shipped, suspects, unread = [], [], 0
    for f in sorted(os.listdir(FOLHAS)):
        if f.endswith('.md') and f != 'README.md':
            sh, s, u = sweep(os.path.join(FOLHAS, f), real)
            shipped += sh
            suspects += s
            unread += u

    # ⛔⛔ **A COBERTURA — o buraco que só um censo vê.** A auditoria de 2026-08-27 achou um nó
    # de PRODUÇÃO (`motion.randomize`: registado, com manifesto, usado em 4 cenas de smoke) que
    # **não tem célula em nenhuma das 18 folhas**. O placar dizia *"zero P0/P1/P2 em 455 linhas"*
    # — e aquelas 455 linhas nunca lhe perguntaram nada. *Zero defeitos NO QUE SE OLHOU não é
    # zero defeitos, e a diferença entre os dois só aparece quando alguém conta as duas listas.*
    coberto = set()
    for f in sorted(os.listdir(FOLHAS)):
        if f.endswith('.md'):
            txt = open(os.path.join(FOLHAS, f), encoding='utf-8').read()
            coberto.update(n for n in real if f'`{n}`' in txt)
    sem_celula = sorted(n for n in real if n not in coberto)
    if sem_celula:
        print('=== NOS DE PRODUCAO SEM CELULA EM FOLHA NENHUMA (a conferencia nao os cobre) ===')
        for n in sem_celula:
            print(f'  {n}')
        print()

    if shipped:
        print('=== JA EXISTE NO MANIFESTO o param que a cura desta celula acrescentaria ===')
        print(f'{"folha":<30} {"linha":>5}  {"no":<24} param(s)')
        print('-' * 92)
        for base, ln, node, names, falta in shipped:
            print(f'{base:<30} {ln:>5}  {node:<24} {", ".join(names)}')
            print(f'{"":<38}   … {falta}')
        print()

    if not suspects:
        print(f'✓ nenhuma contagem de «params hoje» discorda do manifesto ({len(real)} nos lidos)')
        print(f'  ({unread} celula(s) aberta(s) sem contagem legivel na coluna «params hoje» — '
              'nao comparadas)')
        return 1 if shipped else 0

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
    print(f'    (+ {unread} sem contagem legivel na coluna «params hoje» — nao comparadas)')
    print('    Releia cada uma: o que ela pede pode já estar no manifesto acima.')
    print('    (Uma contagem diferente NAO prova que a celula fechou — prova que ela envelheceu.)')
    return 1 if (suspects or shipped) else 0


if __name__ == '__main__':
    sys.exit(main())
