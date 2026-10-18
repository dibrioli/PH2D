#!/usr/bin/env python3
"""As PONTAS da rampa de pressão (W6, doc 47_medicao): a largura a 5 % e a 95 % do comprimento ÷ a largura a 50 %,
pela régua de sempre (`mede_traco.py`, recorte da rampa da bancada: linha 2 esquerda).

uso: pontas_rampa.py <png|ppm>...   → uma linha por ficheiro: nome, 5/50/95, pontas (%), largura do recto
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from mede_traco import mede  # noqa: E402

RAMPA = (20, 240, 560, 120)
RECTO = (20, 60, 580, 120)

for f in sys.argv[1:]:
    r = mede(f, RAMPA)
    if "erro" in r:
        print(f"{os.path.basename(f):<40} {r['erro']}")
        continue
    a, m, b = r["largura_5_50_95"]
    rec = mede(f, RECTO)
    print(f"{os.path.basename(f):<40} {a:>4.0f}/{m:>3.0f}/{b:>3.0f}  pontas {a / m:.2f} {b / m:.2f}  "
          f"média {(a + b) / (2 * m):.2f}  recto {rec.get('largura_mediana_px', 0):.0f}")
