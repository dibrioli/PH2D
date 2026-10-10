---
name: feedback_a_per_cell_gate_on_entering_dry_cells_prints_the_grid
description: Uma trava célula a célula na ENTRADA do papel seco (só a componente que desce, «seca» = film ≤ 1e-3) imprime a grelha — o escorrimento sai em rectângulos; ~40 variantes da trava falharam
metadata:
  type: feedback
---
A lei da gota da W1e (Wet Paint) travava, numa célula SECA, só a componente do fluxo que desce, pela
altura da conta de cima. O dono viu «retângulos virtuais» (pontas de fio cortadas a direito, orla de
baixo recta). Medido célula a célula (W1f, 10/10): a altura crítica quase nunca era atingida — a frente
avançava por um degrau binário (a célula destrava com `film > 1e-3`) e a linha nova enchia-se de LADO
(componente livre) antes da seguinte abrir: uma fila inteira de cada vez. Nenhuma das ~40 variantes da
trava (fonte da conta, papel, limiar, pino, fluxo inteiro, transição suave) saiu sem rectângulos ou
escadas; a lei na gravidade não segurava a orla. A lei saiu do produto (05 §4b, W1f).

**Why:** num solver por recolha numa grelha, uma porta binária por célula decide a FORMA da frente, e
a forma herda os eixos da grelha.

**How to apply:** antes de travar a entrada de um fluido em células vazias, meça a FORMA da frente
(pontas e orla, régua `ferramentas/recto.py`) e não só contagens (fios, orla escura) — e procure o
degrau binário escondido (um `> 1e-3`) que faz a lei nunca agir pela altura que diz usar. Ver
[[feature_worse_than_not_existing]].
