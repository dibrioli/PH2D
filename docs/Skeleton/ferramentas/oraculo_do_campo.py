#!/usr/bin/env python
"""⭐⭐⭐ O ORÁCULO DO CAMPO DE PESO — a SciPy corrida sobre o NOSSO lattice.

# Porque ele existe

Report do dono (2026-09-20): «a imagem vetorial deforma mal, com várias curvas ao longo do
caminho». Medido, a causa é a DISCRETIZAÇÃO do campo de peso: o *Bounded Biharmonic Weights*
resolve-o com elementos finitos LINEARES, logo o gradiente salta em cada aresta de triângulo, e o
contorno da peça dele atravessa 119 deles.

A pergunta passa a ser de **interpolação de dados dispersos**, e ali há um oráculo que se CORRE
(§0.9 do CLAUDE.md) em vez de uma reimplementação a adivinhar.

# ⚠️ ATRIBUIÇÃO

Usa a **SciPy** (`scipy.interpolate`), **BSD-3-Clause** — porta aberta pela triagem, sem parede.
Os dois lados da comparação saem do MESMO oráculo (`LinearNDInterpolator`, que é o que o produto
fazia, e `CloughTocher2DInterpolator`, o C¹ de referência), e é isso que faz a diferença ser a LEI
e não duas implementações.

⛔ Ele NÃO entra no produto: o que entra é o porte em Rust (`ph2d_vec_skin::pesos_suave`), validado
por PROPRIEDADE (exactidão nos vértices · partição da unidade · a derivada a desaparecer com a
sonda · precisão linear) e não por valor — a SciPy estima os gradientes por minimização global de
curvatura e nós por mínimos quadrados no anel, logo comparar número a número mediria o estimador.

# uso

  python -m venv /tmp/venv && /tmp/venv/bin/pip install scipy
  # despeja o lattice a partir da suíte:
  PH2D_ORACULO_DIR=<pasta> cargo test -p ph2d-skeleton-live --lib diag_b_despeja_para_o_oraculo -- --nocapture
  /tmp/venv/bin/python docs/Skeleton/ferramentas/oraculo_do_campo.py <pasta>
  # e a volta, pelo motor do produto:
  PH2D_ORACULO_DIR=<pasta> cargo test -p ph2d-skeleton-live --lib diag_b_o_oraculo_deformado -- --nocapture

# o que ele mediu em 2026-09-20 (498 vértices × 3 ossos, 1088 pontos de consulta)

  linear         Σw ∈ [1,000000, 1,000000] · pior peso +0,000000 · 68 ondulações
  clough_tocher  Σw ∈ [1,000000, 1,000000] · pior peso −0,000249 · 16 ondulações
  molificado 1×  Σw ∈ [1,000000, 1,000000] · pior peso  0,000000 · 50 ondulações

⭐ A partição da unidade sobrevive ao C¹ (ele é linear nos dados e reproduz constantes) e a
não-negatividade que o BBW existe para garantir sobrevive por MEDIÇÃO: o pior peso é −0,00025 num
universo de 3 264 valores.
"""
import sys, numpy as np
from scipy.interpolate import LinearNDInterpolator, CloughTocher2DInterpolator

d = sys.argv[1]
lat = np.loadtxt(f"{d}/lattice.csv", delimiter=",")
qry = np.loadtxt(f"{d}/consulta.csv", delimiter=",")
pts, w = lat[:, :2], lat[:, 2:]

# A aresta típica da malha — o raio da molificação sai DELA e não de um número escolhido.
from scipy.spatial import Delaunay
tri = Delaunay(pts)
e = []
for s in tri.simplices:
    for a, b in ((0, 1), (1, 2), (2, 0)):
        e.append(np.linalg.norm(pts[s[a]] - pts[s[b]]))
aresta = float(np.median(e))
print(f"  aresta mediana da malha: {aresta:.5f}")

lin = LinearNDInterpolator(pts, w)
ct = CloughTocher2DInterpolator(pts, w)

def molifica(q, raio, k=12):
    """O núcleo é `(1-r²)²` sobre um disco — C¹ na borda (valor E derivada a zero)."""
    acc = np.zeros((q.shape[0], w.shape[1]))
    peso = 0.0
    # anel de k direcções em 2 raios + o centro, com o núcleo a pesar cada anel
    amostras = [(0.0, 1.0)] + [(0.5, (1 - 0.25) ** 2), (1.0, 0.0)]
    for r, kw in amostras:
        if kw == 0.0:
            continue
        if r == 0.0:
            v = np.nan_to_num(lin(q), nan=0.0)
            acc += kw * v
            peso += kw
            continue
        for j in range(k):
            a = 2 * np.pi * j / k
            off = raio * r * np.array([np.cos(a), np.sin(a)])
            v = lin(q + off)
            # ⚠️ Fora do casco o interpolante devolve NaN — ali usa-se o ponto CENTRAL, senão a
            # borda da peça perde massa e a partição da unidade parte-se exactamente lá.
            v = np.where(np.isnan(v), np.nan_to_num(lin(q), nan=0.0), v)
            acc += (kw / k) * v
            peso += kw / k
    return acc / peso

for nome, out in [
    ("linear", np.nan_to_num(lin(qry), nan=0.0)),
    ("clough_tocher", np.nan_to_num(ct(qry), nan=0.0)),
    ("molificado_1.0", molifica(qry, aresta * 1.0)),
    ("molificado_2.0", molifica(qry, aresta * 2.0)),
    ("molificado_4.0", molifica(qry, aresta * 4.0)),
]:
    soma = out.sum(axis=1)
    d2 = np.abs(out[:-2] - 2 * out[1:-1] + out[2:]).max()
    print(f"  {nome:16} Σw ∈ [{soma.min():.6f}, {soma.max():.6f}] · pior peso {out.min():+.6f}"
          f" · pior 2.ª dif {d2:.3e}")
    np.savetxt(f"{d}/pesos_{nome}.csv", out, delimiter=",", fmt="%.12f")
