"""Resumo da varredura F0 e o VEREDICTO do portão-mãe (E9). Uso: resumo_f0.py <tsv> [--pior].

Sem `--pior`: o veredicto — por célula e por lei, os limiares finais da F0 §4 (F0b) sobre as colunas `*_plano`;
a tabela «Velha × Pigmento: células que passam / total, por papel»; a lista das células que reprovam na Pigmento
com a métrica que reprovou. Os limiares são os de `portao_mae_f0.rs` (o teste mínimo): mude os dois juntos.
Com `--pior`: a tabela antiga «pior célula por pincel» (F0 §3.2).
"""
import csv, sys, math
from collections import defaultdict

MANCHAS_AZUL_MAX = 1.0
KM_P99_MAX = 8.0
KM_MAX_MAX = 20.0
FERRUGEM_MAX = 0.05
FUROS_PR1_COBRE = 1.0
DEGRAUS_X_MAX = 2

rows = list(csv.DictReader(open(sys.argv[1]), delimiter='\t'))
f = lambda r, k: float(r.get(k, 'nan') or 'nan')
cel = lambda r: f"P{f(r, 'pigment'):g}/W{f(r, 'water'):g}/pr{f(r, 'pressao'):g}/N{r['n']}"
papel = lambda r: r.get('papel') or 'liso'
lei = lambda r: r.get('lei') or 'velha'

if '--pior' in sys.argv:
    por = defaultdict(list)
    for r in rows:
        por[r['pincel']].append(r)
    mets = ['km_p99', 'manchas_pct', 'manchas_azul_pct', 'ferrugem_pct', 'furos_pct', 'rug']
    print('| pincel | ' + ' | '.join(mets) + ' |')
    print('|---|' + '---|' * len(mets))
    for p, rs in por.items():
        out = []
        for m in mets:
            v = [r for r in rs if not math.isnan(f(r, m))]
            if not v:
                out.append('—'); continue
            w = max(v, key=lambda r: f(r, m))
            out.append(f"{f(w, m):.1f} `{cel(w)}`")
        print(f"| {p.split('/',1)[-1]} | " + ' | '.join(out) + ' |')
    sys.exit(0)

chave = lambda r, pr: (r['pincel'], r['pigment'], r['water'], pr, r['n'], r.get('pausa'), r.get('papel_depth'),
                       r.get('ordem'), papel(r), lei(r))
idx = {chave(r, r['pressao']): r for r in rows}


def mistura(r):
    """A régua da mistura é a da LEI (E9 §5.3): `km_*` na velha, `mix_*` (segmento Oklab) na pigmento.
    TSV antigo sem `mix_*` cai na km."""
    return 'mix' if lei(r) == 'pigmento' and 'mix_p99_plano' in r else 'km'


def reprova(r):
    """As métricas que reprovam (vazio = passa). NaN não reprova (a régua não mediu: nada com alfa ≥ 0,9)."""
    out = []
    mx = mistura(r)
    for col, lim in [('manchas_azul_plano', MANCHAS_AZUL_MAX), (f'{mx}_p99_plano', KM_P99_MAX),
                     (f'{mx}_max_plano', KM_MAX_MAX), ('ferrugem_plano', FERRUGEM_MAX)]:
        if f(r, col) > lim:
            out.append(f"{col} {f(r, col):.2f}")
    par = idx.get(chave(r, '1'))
    if par is not None and f(par, 'furos_pct') < FUROS_PR1_COBRE and f(r, 'furos_pct') > 0:
        out.append(f"furos {f(r, 'furos_pct'):.2f}")
    if f(r, 'degraus_y_plano') > int(r['n']) - 1:
        out.append(f"degraus_y_plano {int(f(r, 'degraus_y_plano'))}")
    if f(r, 'degraus_x_plano') > DEGRAUS_X_MAX:
        out.append(f"degraus_x_plano {int(f(r, 'degraus_x_plano'))}")
    return out


placar = defaultdict(lambda: [0, 0, 0])  # (papel, lei) -> [passa, total, sem medida (não conta como passa)]
reprovadas = []
print('| pincel | célula | papel | lei | veredicto |')
print('|---|---|---|---|---|')
for r in rows:
    m = reprova(r)
    k = (papel(r), lei(r))
    placar[k][1] += 1
    sem = math.isnan(f(r, f'{mistura(r)}_p99_plano'))
    placar[k][0] += not m and not sem
    placar[k][2] += sem
    v = '; '.join(m) if m else ('sem medida (alfa < 0,9)' if sem else 'passa')
    print(f"| {r['pincel']} | {cel(r)} | {papel(r)} | {lei(r)} | {v} |")
    if m and lei(r) == 'pigmento':
        reprovadas.append((papel(r), r['pincel'], cel(r), m))

print('\n### Velha × Pigmento: células que passam / total, por papel\n')
print('| papel | velha | pigmento |')
print('|---|---|---|')
for pa in sorted({p for p, _ in placar}, key=lambda p: (p != 'padrao', p)):
    c = lambda l: (lambda v: f"{v[0]} / {v[1]}" + (f" ({v[2]} sem medida)" if v[2] else '') if v[1] else '—')(placar[(pa, l)])
    print(f"| {pa} | {c('velha')} | {c('pigmento')} |")

print('\n### Células que reprovam na Pigmento\n')
for pa, p, c, m in sorted(reprovadas, key=lambda x: (x[0] != 'padrao', x[0], x[1], x[2])):
    print(f"- `{pa}` {p} `{c}`: {'; '.join(m)}")
if not reprovadas:
    print('(nenhuma)')
