---
name: a-windowed-drive-races-the-real-cursor
description: Smoke auto-dirigido com janela real — o cursor FÍSICO e o WM injetam eventos que corrompem o gesto sintético; re-afirme a posição todo frame e nunca conclua bug de app por um run
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 85e38f84-1b86-49d2-aee2-91da101e1fd7
  modified: 2026-07-21T00:43:44.609Z
---

Num smoke auto-dirigido que abre JANELA REAL (nível 18 do vector, 2026-07-20), o gesto
sintético compartilha a fila de eventos com o desktop vivo: o KWin reposiciona a janela
recém-aberta sob o cursor FÍSICO parado e isso emite `CursorMoved` REAIS — que um slider
ATIVO obedece. Um hold sintético foi teleportado de d=+1,3 para d=−4 sem nenhum move do
roteiro, e runs idênticos divergiram (flip num, commit corrompido noutro), imitando bug
de app não-determinístico.

**Why:** o `update_drag_value` não distingue a origem do Move; qualquer evento real
(reposicionamento de janela, screenshot tool, foco) entra na conta do gesto. A
investigação perseguiu o "flip do slider" como bug do produto por horas — era o AMBIENTE.

**How to apply:** (1) num roteiro dirigido, RE-AFIRME a posição sintética TODO frame de
hold/drag (sobrescreve qualquer drift real no frame seguinte); (2) anomalia que só
aparece em ALGUNS runs do mesmo script = suspeite do ambiente antes do app; (3) a
cobertura DETERMINÍSTICA mora em teste headless do motor/função — o smoke janelado é
para OLHOS (screenshots), nunca o gate. Relacionado: [[feedback_harness_reproduces_mechanism_not_context]],
[[feedback_a_green_gate_may_be_green_by_accident]].
