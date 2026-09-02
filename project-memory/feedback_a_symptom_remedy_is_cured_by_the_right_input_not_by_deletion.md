---
name: a-symptom-remedy-is-cured-by-the-right-input-not-by-deletion
description: Um remédio-de-sintoma que uma decisão manda apagar costuma curar-se dando-lhe a entrada certa — ele fica inerte onde não é preciso e vivo onde ainda é
metadata:
  type: feedback
---

Quando uma decisão de desenho manda **apagar** um remédio de sintoma (*«com a causa curada ele vira
remédio duplo»*), pergunte primeiro se a causa está curada para **TODOS** os casos que ele cobre.
Muitas vezes está para uns e não para outros — e aí a cura é **dar-lhe a entrada certa**: ele fica
**inerte por construção** onde a causa foi curada, e vivo onde ela permanece.

**Why:** medido em 2026-08-30 (`line/UIUX`). A **D1** mandava remover a fuga do gizmo de navegação
depois de ancorar os painéis. A fuga recebia **o viewport inteiro**, que as colunas docadas tocam;
passar-lhe a **área de desenho** (que começa depois delas) tornou-a inerte para o chrome docado sem
mudar uma linha de lei — e **manteve** a defesa contra as janelas que **declaram flutuar** (Grid
Snap, galeria), que continuam a tocar a aresta. Apagá-la punha o gizmo por baixo dessas.

**How to apply:** antes de apagar, enumere o que o remédio ainda protege. Depois escreva **dois**
gates: *«o caso curado já não o aciona»* e *«o caso que sobra ainda o aciona»* — e no primeiro ponha
o **controlo** com a entrada ANTIGA, senão ele passa com o remédio apagado. ⚠️ E gateie a decisão de
qual entrada o produto lhe dá: ver
[[feedback_a_seam_gate_must_assert_both_sides_or_it_measures_the_wrong_half]].
