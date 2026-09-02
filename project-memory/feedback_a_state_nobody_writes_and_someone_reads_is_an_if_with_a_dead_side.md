---
name: a-state-nobody-writes-and-someone-reads-is-an-if-with-a-dead-side
description: Um campo de estado sem escritor não é só uma marca em falta — quem o LÊ para decidir tem um ramo que nunca corre
metadata:
  type: feedback
---

Quando um estado não tem escritor, o sintoma **visível** é uma marca que não aparece. O sintoma
**caro** é outro: quem o lê para **decidir** fica com um ramo morto.

**Why:** 2026-08-30. Dez dos treze toggles de módulo (`TOPBAR_VECTOR`, `_MOTION`, …) nunca têm o
`ButtonState` escrito — o laço de reconciliação da shell só percorre os clusters `image_tools` e
`vector_tools` do registry de ferramentas, e os pills de módulo não estão em cluster nenhum
(`hash_node_id("topbar_vector")` ≠ `hash_node_id("vector")`). O `chrome::vector_toggle` **lê** esse
estado para escolher entre *activar* e *cancelar* ⇒ preso em `Normal`, o segundo clique volta a
activar em vez de desligar.

**How to apply:** ao curar uma marca ausente, pergunte primeiro *quem ESCREVE este campo?* e depois
*quem o LÊ para decidir?*. O censo que o apanha compara **antes e depois de um clique real**
(`clicking_a_toggle_row_moves_its_mark`), não a existência do campo — e precisa das três metades:
as novas reprovam, as pendentes estão nomeadas, e **uma pendente que passe a mexer tem de sair da
lista** ([[feedback_a_ratchet_without_a_staleness_census_only_ratchets_up]]).
