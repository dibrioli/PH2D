---
name: feedback-a-target-derived-from-the-box-makes-a-finer-input-pure-waste
description: Se a primeira fase deriva o alvo da CAIXA e não da densidade, toda entrada mais fina é custo sem informação — e pode ser custo com PIOR resultado.
metadata:
  type: feedback
---

`ph2d_remesh_iso::target_edge(mesh, alpha) = alpha · diagonal_da_caixa` — ele **não olha para a
densidade da malha**. A cadeia de quads remalha para esse alvo venha a entrada de que
profundidade vier, então tudo o que uma grade mais fina traz a mais é **deitado fora pela fase
zero, depois de pago**.

Medido 2026-08-25 (esfera, exportação do módulo 3D):

| grade | quads que entram | F1 ms | cadeia ms | o que sai |
|---|---|---|---|---|
| **6** | 17 550 | **632** | ⭐ **4 613** | 6,4° · 2 539 quads |
| 7 | 69 966 | 4 513 | 8 193 | 6,3° · 2 471 quads |
| 8 | 280 062 | — | 47 454 | ⛔ 55,5° (o veto recusa) |
| 9 | 1 120 158 | ⛔ **482 451** | ⛔ **495 244** | 6,4° · 2 436 quads |

**8 min 15 s, 107× o preço, para a MESMA resposta a uma casa decimal** — 97 % disso é a fase zero
a mastigar um milhão de faces até 2 436 quads.

**Why:** e não é só preço. A fidelidade — medida no CAMPO, que é exacto — **piora** com a grade
fina (`|f|` máx de 0,043 % para 0,087 % e depois 11,3 % da diagonal), e nas peças curvas a
profundidade 8 destrói a peça. *Uma grade mais fina não é mais informação para a cadeia: é ruído
que ela tem de mastigar e depois segue mal.*

**How to apply:** antes de alimentar um passe com «o melhor que tenho», leia como ele deriva a
própria escala. Se ela sai da **forma** (caixa, diagonal, raio) e não da **densidade**, a entrada
certa é a mais grossa que ainda resolve essa escala — e o resto é desperdício, possivelmente com
regressão. A régua que decide isto tem de ter as três colunas: relógio, forma da face e
**fidelidade**; sem a terceira, «mais barato e igualmente bonito» pode ser dito sobre uma peça
que encolheu ([[feedback-a-cure-measured-on-a-fixture-that-lacks-the-phenomenon-reads-as-useless]]).
