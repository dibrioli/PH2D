"""Resume o log da medida_luz2d_w6d_borda por variante: (a), o pior de (a), (b), anéis errados, bits, custo."""
import re
import sys
from collections import defaultdict

a = defaultdict(int)
pior = defaultdict(float)
b = defaultdict(int)
aneis = defaultdict(int)
bits = defaultdict(int)
custo = defaultdict(list)
for linha in open(sys.argv[1], encoding="utf-8"):
    linha = re.sub(r"^.*?\.\.\. ", "", linha)
    m = re.match(r"BORDA · \S+ · zoom [\d.]+ · (?:\d/8 · )?(.+?) · \[.*FAMÍLIAS \(a\) (\d+) \(pior ([-+\d.]+)\) · \(b\) (\d+)", linha)
    if m:
        v = m.group(1)
        a[v] += int(m.group(2))
        pior[v] = min(pior[v], float(m.group(3)))
        b[v] += int(m.group(4))
        continue
    m = re.match(r"ANEIS · \S+ · zoom [\d.]+ · (?:\d/8 · )?(.+?) · PIOR (\d+)e", linha)
    if m:
        aneis[m.group(1)] += int(m.group(2))
        continue
    if linha.startswith("BITS"):
        for v, d in re.findall(r" · ([^·]+?) (\d+)/\d+", linha):
            bits[v.strip()] += int(d)
        continue
    m = re.match(r"CUSTO · cena (\d) · (.+?) · C3 ([\d.]+) ms", linha)
    if m:
        custo[m.group(2)].append(f"cena {m.group(1)} {m.group(3)}")
for v in a:
    print(f"{v:<16} (a) {a[v]} (pior {pior[v]:+.3f}) · (b) {b[v]} · anéis {aneis[v]} · bits {bits.get(v, '—')} · {' / '.join(custo[v])}")
