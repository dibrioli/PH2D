"""Pior célula por pincel e métrica da varredura F0 (tabela markdown). Uso: resumo_f0.py <tsv>."""
import csv, sys, math
from collections import defaultdict
rows = list(csv.DictReader(open(sys.argv[1]), delimiter='\t'))
por = defaultdict(list)
for r in rows:
    por[r['pincel']].append(r)
mets = ['km_p99', 'manchas_pct', 'manchas_azul_pct', 'ferrugem_pct', 'furos_pct', 'rug']
cel = lambda r: f"P{float(r['pigment']):g}/W{float(r['water']):g}/pr{float(r['pressao']):g}/N{r['n']}"
print('| pincel | ' + ' | '.join(mets) + ' |')
print('|---|' + '---|' * len(mets))
for p, rs in por.items():
    out = []
    for m in mets:
        v = [r for r in rs if not math.isnan(float(r[m]))]
        if not v:
            out.append('—'); continue
        w = max(v, key=lambda r: float(r[m]))
        out.append(f"{float(w[m]):.1f} `{cel(w)}`")
    print(f"| {p.split('/',1)[-1]} | " + ' | '.join(out) + ' |')
