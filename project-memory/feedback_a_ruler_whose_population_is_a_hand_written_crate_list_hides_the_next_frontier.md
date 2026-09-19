---
name: feedback_a_ruler_whose_population_is_a_hand_written_crate_list_hides_the_next_frontier
description: Uma régua cuja POPULAÇÃO é uma lista de crates escrita à mão fica verde sobre tudo o que não está na lista — e a lista nunca cresce sozinha
metadata:
  type: feedback
---

As 30 réguas do HR-15 deste repo varrem **os painéis, as `ph2d-app-*`, a `ph2d-editor-core` e a
shell**. Essa enumeração está escrita à mão em cada instrumento (o `--resumo` do
`ph2d-label-census`, os gates por crate), e **nenhum motor está nela**. ⇒ um rótulo que uma crate
`ph2d-tool-*` / `ph2d-ecs` publica e um painel pinta ficava **invisível aos dois lados**: do lado do
motor não há pintor a seguir, e do lado do painel o literal não existe.

Medido em 2026-09-19: **203** nomes publicados em 16 crates, **115** sem ponte. O defeito já tinha
aparecido **três** vezes (o catálogo de componentes, os motores de pincel, o motor da escultura) e as
três foram achadas por uma **fotografia do dono**, não por um instrumento.

**Why:** o cabeçalho do `ph2d-i18n/src/sculpt_engine.rs` já escrevia a cegueira por extenso —
*«um censo cuja crate não é DONA do texto que ela pinta fica verde sobre texto cru»*. *Uma cegueira
escrita em prosa não é medida, e uma que só o dono encontra custa um report por ocorrência.*

**How to apply:** a população de uma régua DERIVA-SE de um facto do repo (uma dependência no
`Cargo.toml`, um trait implementado), nunca de uma lista de nomes. Quando a lista for inevitável,
ponha um **piso de população** e escreva ao lado dela o que ela NÃO vê. E ao construir a régua nova,
conte o que ela colhe contra a irmã — a minha 1.ª redacção exigia `-> &'static str` e lia **zero**
nos dez nomes de ferramenta, porque `fn label(&self) -> &str` é a assinatura que o *trait* impõe:
*uma função de trait não escolhe o tipo de retorno dela*.

Irmã de [[feedback_a_hand_written_list_beside_a_predicate_is_two_answers]] e de
[[feedback_an_unmeasured_rule_in_a_briefing_is_confirmed_by_the_agents_who_follow_it]].
