---
name: feedback_the_owner_of_a_scrollbar_id_is_the_dispatch
description: Um id de barra de rolagem declarado do lado do PAINEL é invisível ao despacho — o polegar pinta, faz hover e nunca se agarra.
metadata:
  type: feedback
---

Medido em 2026-09-14 (`line/components`, painel *Tags*). O id nasceu em
`ph2d_panel_tags::ids::TAGS_SCROLLBAR`, e dali ele é invisível às duas metades que fazem um polegar
arrastar: o `scrollbar_panel_for_id` (que o mapeia ao painel, para o Move saber que rolagem move) e
o `begin_scrollbar_drag` do `pointer_down`. Resultado: **pintado, com hover, e impossível de
agarrar** — que é indistinguível de *«este painel não tem barra»*.

**Why:** a lei do arrasto de barra vive no `ph2d-editor-core::interaction::dispatch`, e ela consulta
uma **escada de constantes** (`widget::*_SCROLLBAR_ID`) mais um `match` que liga cada uma ao rect do
painel. Um id fora daquela escada não existe para o despacho. *O dono de um id é o mais BAIXO dos
leitores dele* (auditoria A5b), e aqui o leitor é o despacho.

**How to apply:** um painel novo com rolagem declara o id em `widget/scrollbar_ids.rs` (o número
**conta-se** do ficheiro, contra o `main` do dia) e acrescenta o braço no `scrollbar_panel_for_id`.
⚠️ Quem o apanha é o `hit_indexed_ids_are_registered`, e só porque o id estava no módulo `ids` do
painel: um id declarado noutro sítio qualquer passaria mudo.

⛔ **E há um vivo no repo:** o `ph2d-panel-skeleton` pinta a barra dele com o `VECTOR_SCROLLBAR_ID`,
que o mapa liga ao **painel de vetor** — com aquele fechado, `panel_content_h` é `None` e o arrasto
nunca arma. Irmãs: [[reference_topic_panel_registration]] ·
[[feedback_docked_panel_registration_four_sites]]
