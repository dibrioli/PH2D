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
#
# ⚠️ **A chave é um TRECHO DA LINHA, não o número dela — e a troca foi paga.**
# Enquanto a chave era `(arquivo, nº)`, **inserir uma linha acima desalinhava a
# tabela em silêncio**: em 2026-08-18 uma linha nova na folha 15 empurrou a
# exceção `141` para cima da vizinha, que passou a ser contada como *fora* em vez
# de ✅ — o placar imprimiu **um ✅ a menos** e o único sintoma foi um `!!` numa
# linha que ninguém tinha tocado. Um número de linha é uma referência que o
# próprio ato de editar invalida; um trecho do texto viaja com a linha.
#
# ⚠️ E a troca só é segura porque ela se VERIFICA: cada chave tem de casar
# **exactamente uma** linha da sua folha, e o `sweep` sai vermelho quando não
# casa (chave morta = exceção que deixou de se aplicar; chave ambígua = duas
# linhas a disputá-la). Sem esse par, um trecho renomeado seria a mesma falha
# silenciosa noutra roupa.
HAND = {
    ('01_distribuicao_emissao.md', '`motion.lattice` | idem | (nada mais com citação)'): 'natureza',
    ('08_stream_utilidade.md', 'É A SEGUNDA PORTA — ver §2.'): 'natureza',
    ('15_value.md', 'observação estrutural, não gap'): 'natureza',
    ('15_value.md', 'blur ESPACIAL'): 'fora',     # nó novo, fora desta conferência
    ('17_zero_param_debug.md', '**`motion.output`** | 1 | **BLEND MODE.**'): 'fech',
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
    acc, unknown, problems = {}, [], []
    mine = [k for (f, k) in HAND if f == base]
    hits = dict.fromkeys(mine, 0)
    for i, ln in enumerate(lines):
        if not SEP.fullmatch(ln.strip()):
            continue
        if '|'.join(cells(lines[i - 1])) != HDR:
            continue
        j = i + 1
        while j < len(lines) and lines[j].lstrip().startswith('|'):
            hand = [k for k in mine if k in lines[j]]
            for k in hand:
                hits[k] += 1
            if len(hand) > 1:
                problems.append(f'{base}:{j + 1} casa {len(hand)} excecoes: {hand}')
            key = HAND[(base, hand[0])] if hand else ''
            if not key:
                for field in cells(lines[j])[4:]:
                    key = classify(field)
                    if key:
                        break
                if not key:
                    unknown.append(j + 1)
                    key = 'OUTRO'
            acc[key] = acc.get(key, 0) + 1
            j += 1
    # ⚠️ A metade que torna a chave-trecho segura: uma exceção que não casa NADA é
    # uma linha que mudou de texto (ou saiu da folha) e uma classificação que
    # deixou de se aplicar — silenciosa, como o número de linha era.
    for k, n in hits.items():
        if n != 1:
            problems.append(f'{base}: a excecao {k!r} casa {n} linhas, tem de casar 1')
    return acc, unknown, problems


def main():
    root = os.path.join(os.path.dirname(__file__), '..', '89_conferencia')
    files = sorted(glob.glob(os.path.join(root, '*.md')))
    if not files:
        print('nenhuma folha encontrada em', root)
        return 1
    tot, bad, broken = {}, 0, []
    head = f"{'folha':<28} {'P0':>3} {'P0/P1':>6} {'P1':>3} {'P2':>3} | {'⏸':>3} {'✅':>3} {'⛔':>3} {'nat':>4}"
    print(head)
    print('-' * len(head))
    for f in files:
        acc, unknown, problems = sweep(f)
        broken += problems
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
    for p in broken:
        print(f"!! {p}")
    # ⚠️ Sair vermelho é o ponto: uma linha que a varredura não sabe ler é uma
    # linha que a próxima contagem vai errar em silêncio — e uma exceção que não
    # casa exactamente uma linha é a MESMA falha, um passo antes.
    return 1 if bad or broken else 0


if __name__ == '__main__':
    sys.exit(main())
