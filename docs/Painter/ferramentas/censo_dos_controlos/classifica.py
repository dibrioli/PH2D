#!/usr/bin/env python3
"""Classifica cada linha da sonda `sonda_a_tabela_dos_controlos` (já passada pelo `nomes_dos_ids.py`).

Colunas: CENSO meio nome tipo n soma max ao_tool ajuste outras estado=<pré-condições> gesto.
  PÂNICO         o ensaio entrou em pânico
  sem-ferramenta nada chegou à ferramenta (cromo do painel — ou um MUDO, a rever à mão)
  já-era         chegou, o ajuste não mudou e a imagem também não (a opção já escolhida, o Reset na fábrica)
  CANDIDATO      o ajuste MUDOU e a imagem não — morto, ou inerte por pré-condição
  vivo           a imagem mudou
Uso: classifica.py [-v] < nomeado.tsv   (-v imprime também os vivos)
"""
import sys
from collections import Counter

def classe(c):
    if c[3] == 'PÂNICO':
        return 'PÂNICO'
    if c[2] == 'CONTROLO':
        return 'controlo'
    n, ao_tool, ajuste = int(c[4]), int(c[7]), c[8]
    if ao_tool == 0 and n == 0:
        return 'sem-ferramenta'
    if n > 0:
        return 'vivo'
    return 'CANDIDATO' if ajuste == 'mudou' else 'já-era'

contas = {}
for linha in sys.stdin:
    if not linha.startswith('CENSO\t'):
        continue
    c = linha.rstrip('\n').split('\t')
    k = classe(c)
    contas.setdefault(c[1], Counter())[k] += 1
    if k == 'controlo':
        if c[4] != '0':
            print(f'{c[1]:<10} RUÍDO n={c[4]} {c[10]}')
        continue
    if k == 'vivo' and '-v' not in sys.argv:
        continue
    estado = next((x[7:] for x in c if x.startswith('estado=')), '')
    print(f'{c[1]:<10} {k:<15} {c[2]:<46} {c[3]:<8} n={c[4]:<6} [{estado}]')
for m, k in contas.items():
    print('#', m, dict(k))
