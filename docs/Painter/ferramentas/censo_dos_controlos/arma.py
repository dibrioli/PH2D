#!/usr/bin/env python3
"""Lista as linhas ARMA da sonda (a procura automática de pré-condições), com os ids nomeados.

Uso: arma.py <log> [--so-mortos]
"""
import re, sys
exec(open(__file__.replace('arma.py', 'nomes_dos_ids.py')).read().split('hexa = re.compile')[0])
hexa = re.compile(r'0x[0-9a-f]{16}')
for l in open(sys.argv[1]):
    if not l.startswith('ARMA'):
        continue
    l = hexa.sub(lambda m: nomes.get(int(m.group(0), 16), m.group(0)), l)
    c = l.rstrip('\n').split('\t')
    if '--so-mortos' in sys.argv and c[3] != 'MORTO':
        continue
    print(f"{c[1]:<10} {c[3]:<11} {c[2]:<44} ← {c[4]:<42} {c[5]:<8} [{c[6][7:]}]"[:240])
