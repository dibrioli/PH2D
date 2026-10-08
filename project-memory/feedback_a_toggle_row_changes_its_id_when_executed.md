---
name: feedback_a_toggle_row_changes_its_id_when_executed
description: Uma linha-interruptor (Snap ▸ …) troca de id ao ser executada — quem guarda o id escolhido para o achar depois (piscar, realce) acha nada no quadro seguinte
metadata:
  type: feedback
---

Em `ph2d-panel-vector` as linhas dos pulldowns são interruptores cujo id É o estado
(`VECTOR_SNAP_PATH_ON` ↔ `_OFF`): executar a linha faz o módulo republicar a mesma linha com o OUTRO
id. A paleta (D10 F4, 08/10) guardava o id escolhido para fazer piscar «onde mora» — e no quadro
seguinte ele não estava em lado nenhum. A foto da cena 10 mostrou o *Snap* sem anel; o gate estava
verde porque não republicava as barras depois da escolha (o app republica a cada quadro).

**Why:** um teste que publica UMA vez mede um app que não existe; o id de um interruptor é efémero.
**How to apply:** o contexto de um comando (o pulldown que o contém, o botão-pai) lê-se NA ESCOLHA
(`WidgetStore::flash_pill`), e o gate republica (e aplica a ponte do módulo) antes de olhar. Relacionado:
[[feedback_a_key_and_a_text_of_the_same_type_is_a_defect_waiting]].
