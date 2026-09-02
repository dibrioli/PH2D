---
name: deleting-the-only-painter-leaves-every-registration-gate-green
description: Tirar de cena o único sítio que PINTA um controlo torna-o inalcançável sem mover um gate — os gates medem registo e despacho, que continuam certos
metadata:
  type: feedback
---

Retirar a superfície que **pinta** um controlo deixa-o inalcançável **sem mover um único gate**:
os gates de costura deste repo medem *registo* (`InteractiveState` no store) e *despacho* (o braço
que consome o `Click`) — as duas metades certas de uma pergunta que **pressupõe que alguém o
desenha**.

⇒ *Um controlo correcto que ninguém pinta lê-se, de fora, exactamente como um controlo partido.*

**Why:** 2026-08-30, `line/UIUX`. Esconder a barra de pills tornou **o Painter e as dez ferramentas
de imagem** inalcançáveis (sem menu, sem atalho, sem projecção na paleta) — e com o Painter foi
**toda a face de pintura** da fila nova, porque ela exige `active_tool_id == Some("painter")`. Os
dois gates que existiam (`every_image_tool_pill_dispatches`,
`..._is_registered_and_therefore_focusable`) ficaram verdes o tempo todo. Também perdeu a porta a
lista de cenas e o *rebobinar* do transporte.

**How to apply:** ao remover ou esconder uma superfície de chrome, **enumere os ids que ela era a
única a pintar** e prove que cada um tem outra porta — ou escreva-o numa lista de excepções com o
motivo medido. Molde: `every_topbar_verb_has_a_door_that_is_not_the_legacy_key` (lê os consts do
ficheiro de ids, com o slug tirado da própria linha) e
`every_image_tool_is_reachable_without_the_legacy_bar` (pinta um quadro e pergunta ao `HitIndex`).
Irmão: [[feedback_a_dead_knob_has_two_species_no_probe_catches]].
