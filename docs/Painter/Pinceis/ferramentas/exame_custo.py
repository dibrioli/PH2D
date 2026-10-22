#!/usr/bin/env python3
"""A tabela do CUSTO (00_MISSAO §3.6) a partir das linhas `[custo]` do `exame_bancada::exame_custo` (uma por tamanho).

uso: exame_custo.py <custo.jsonl>
"""
import json
import sys

ORCAMENTO = 1000 / 60
linhas = [json.loads(l) for l in open(sys.argv[1]) if l.strip()]
print(f"{'pincel':<16} {'px':>4} {'px/qd':>6} | {'ms/quadro a pintar: mediana · p95 · pior':>40} | {'> 16,7':>6} | "
      f"{'parado: mediana · p95':>22} | {'núcleos':>7} | {'RSS pico':>9} | load")
for r in linhas:
    q, p = r["quadros_a_pintar_ms"], r["quadros_parados_ms"]
    v = r.get("px_por_evento")
    vq = f"{v * r['eventos_por_quadro']:>6.0f}" if v else f"{'bat.':>6}"
    print(f"{r['pincel']:<16} {r['tamanho_px']:>4.0f} {vq} | {q['mediana']:>14.2f} · {q['p95']:>6.2f} · {q['pior']:>7.2f}"
          f"{'':>8} | {q['acima_de_16_7']:>3}/{q['n']:<3}| {p['mediana']:>9.2f} · {p['p95']:>8.2f} | "
          f"{r['nucleos']:>7.2f} | {r['vm_hwm_kib'] / 1024:>6.0f} MiB | {r['load'].split()[0]}")
pior = max((r["quadros_a_pintar_ms"]["p95"] for r in linhas), default=0)
print(f"\np95 pior = {pior:.2f} ms ({'dentro' if pior <= ORCAMENTO else 'FORA'} do orçamento de {ORCAMENTO:.1f} ms a "
      f"60 Hz). ⚠️ perfil smoke; o `pior` inclui o 1.º quadro (a sessão nasce); a água corre no fio dela (núcleos). "
      f"px/qd = px por quadro (bat. = os pontos da bateria, ~9 px: um traço LENTO; o app anda 30–135).")
