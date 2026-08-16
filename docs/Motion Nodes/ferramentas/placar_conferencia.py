#!/usr/bin/env python3
"""O placar da conferência 89 — DERIVADO das folhas, nunca escrito à mão.

    python3 "docs/Motion Nodes/ferramentas/placar_conferencia.py"

⚠️ **Esta ferramenta existe porque a contagem envelheceu SEIS vezes.** Uma linha
de `**Contagem:**` escrita à mão descreve a folha do dia em que alguém a
escreveu, e a folha continua a andar: a 13 chegou a dizer `3 P0 · 7 P1` com a
prosa três parágrafos abaixo a dizer *"não tem item aberto"*, e sete das
dezassete folhas não tinham contagem nenhuma. A cura não é reconferir com mais
cuidado — é **tornar o número reproduzível num comando**.

**Como ela lê.** Toda folha tem a mesma tabela canónica de sete colunas, e a 6ª
é o veredito (`P`). A varredura acha a tabela pelo CABEÇALHO (não pela contagem
de colunas — a folha 03 tem outra tabela de sete) e, em cada linha, varre os
campos a partir do 5º tomando o PRIMEIRO que classifica como veredito.

⚠️ **O passo do "primeiro que classifica" não é preguiça, é o que sobrevive aos
dados reais:** uma célula pode conter `|` dentro de um code span, e aí a linha
ganha campos a mais e o índice fixo aponta para o meio de outra frase. Foi
verificado linha a linha contra as catorze que quebravam o índice fixo.
"""

import glob
import os
import re
import sys

HDR = ("nó|params hoje|falta (referência CITADA)|exprimível? (a cadeia tentada)|"
       "natureza/omissão|P|default que reduz")
SEP = re.compile(r'\|(-{3,}\|){7}')
SPLIT = re.compile(r'(?<!\\)\|')

# As linhas cujo veredito não está na coluna `P` — lidas uma a uma. Todas são
# natureza / fechado / fora de escopo; NENHUMA é célula viva.
HAND = {
    ('01_distribuicao_emissao.md', 36): 'natureza',
    ('08_stream_utilidade.md', 56): 'natureza',   # "ACHADO, não gap"
    ('15_value.md', 89): 'natureza',              # decisão de produto
    ('15_value.md', 141): 'fora',                 # nó novo, fora desta conferência
    ('17_zero_param_debug.md', 78): 'fech',
}


def cells(line):
    return [c.strip() for c in SPLIT.split(line.strip())[1:-1]]


def classify(v):
    """A classe de um veredito. `''` quando o campo não é um veredito."""
    if v.startswith('⛔'):
        return 'refut'
    if v.startswith('✅') or v.startswith('~~'):
        return 'fech'
    m = re.match(r'\*?\*?(P[012])', v)
    if not m:
        return ''
    pr = m.group(1)
    if '⏸' in v:                              # família deferida (RIG)
        return pr + '-defer'
    if re.search(r'→\s*\*\*P2\*\*', v):        # rebaixado por medição
        return 'P2'
    if pr == 'P0' and 'P0/P1' in v:            # marcador ambíguo
        return 'P0/P1'
    return pr


def sweep(path):
    base = os.path.basename(path)
    lines = open(path, encoding='utf-8').read().split('\n')
    acc, unknown = {}, []
    for i, ln in enumerate(lines):
        if not SEP.fullmatch(ln.strip()):
            continue
        if '|'.join(cells(lines[i - 1])) != HDR:
            continue
        j = i + 1
        while j < len(lines) and lines[j].lstrip().startswith('|'):
            key = HAND.get((base, j + 1))
            if key is None:
                key = ''
                for field in cells(lines[j])[4:]:
                    key = classify(field)
                    if key:
                        break
                if not key:
                    unknown.append(j + 1)
                    key = 'OUTRO'
            acc[key] = acc.get(key, 0) + 1
            j += 1
    return acc, unknown


def main():
    root = os.path.join(os.path.dirname(__file__), '..', '89_conferencia')
    files = sorted(glob.glob(os.path.join(root, '*.md')))
    if not files:
        print('nenhuma folha encontrada em', root)
        return 1
    tot, bad = {}, 0
    head = f"{'folha':<28} {'P0':>3} {'P0/P1':>6} {'P1':>3} {'P2':>3} | {'⏸':>3} {'✅':>3} {'⛔':>3} {'nat':>4}"
    print(head)
    print('-' * len(head))
    for f in files:
        acc, unknown = sweep(f)
        for k, v in acc.items():
            tot[k] = tot.get(k, 0) + v
        bad += len(unknown)
        g = acc.get
        defer = g('P0-defer', 0) + g('P1-defer', 0) + g('P2-defer', 0)
        line = (f"{os.path.basename(f):<28} {g('P0',0):>3} {g('P0/P1',0):>6} "
                f"{g('P1',0):>3} {g('P2',0):>3} | {defer:>3} {g('fech',0):>3} "
                f"{g('refut',0):>3} {g('natureza',0):>4}")
        if unknown:
            line += f"   !! linhas sem veredito: {unknown}"
        print(line)
    g = tot.get
    defer = g('P0-defer', 0) + g('P1-defer', 0) + g('P2-defer', 0)
    print('-' * len(head))
    print(f"{'TOTAL':<28} {g('P0',0):>3} {g('P0/P1',0):>6} {g('P1',0):>3} "
          f"{g('P2',0):>3} | {defer:>3} {g('fech',0):>3} {g('refut',0):>3} {g('natureza',0):>4}")
    print(f"\n{sum(tot.values())} linhas de conferência em {len(files)} folhas.")
    # ⚠️ Sair vermelho é o ponto: uma linha que a varredura não sabe ler é uma
    # linha que a próxima contagem vai errar em silêncio.
    return 1 if bad else 0


if __name__ == '__main__':
    sys.exit(main())
