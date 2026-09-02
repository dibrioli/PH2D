---
name: feedback_a_shared_mask_is_wider_than_the_question_you_asked
description: "Reusar um canal existente é barato e certo — até o canal responder a MAIS perguntas do que a sua; grepe TODOS os leitores antes de escrever nele"
metadata:
  type: feedback
---

**L-System, 2026-08-30.** Enio: *"uma opção para livrar as folhas, os frutos do tint que pinta
tudo na árvore"*. Fui procurar como o `motion.tint` decide, achei que ele faz
`lerp(existente, alvo, falloff)`, e escrevi `falloff = 0` nas linhas de folha. Escrevi no
commit, com orgulho: *«não é um canal novo: é o que a casa inteira já fala»*.

⛔ **E era esse o problema.** O `falloff` é a máscara de **todos** os modificadores — o
`motion.move` faz `P' = P + (dx, dy) · falloff`. Na cena de smoke, que move cada coluna com um
`motion.move`, as folhas ficavam **paradas enquanto a árvore andava**. O report seguinte foi
*"Keep own color não funciona, as folhas não aparecem"*.

**Why:** eu grepei o canal no consumidor que me interessava e parei aí. Um canal partilhado é
uma resposta a uma pergunta GERAL (*«quanto os modificadores alcançam esta linha?»*), e a minha
era específica (*«a COR alcança esta linha?»*). Escrever num canal geral para responder a uma
pergunta específica responde, de graça, a todas as outras — com o valor errado.

⚠️ **E o gate não me salvou porque media a COLUNA e não a CONSEQUÊNCIA:** ele afirmava «a
máscara nasce com 0 nas folhas», o que era verdade, e nada sobre o que um nó a jusante faz com
ela. O gate certo coze um `motion.move` **a sério** e mede que as folhas ainda andam com a
planta.

**How to apply:** antes de ESCREVER num canal partilhado, `grep` **todos** os leitores dele em
produção — não só aquele que motivou a mudança — e pergunte de cada um *«o valor que vou pôr
está certo para ESTA pergunta também?»*. Se a resposta é não para algum, o canal é largo demais:
o certo é uma coluna própria, multiplicativa com a geral e **ausente ⇒ neutra** (aqui,
`attr::TINT_MASK_COLUMN`). E gateie a CONSEQUÊNCIA a jusante, correndo o nó real — não a coluna
que acabou de escrever. Relacionado:
[[feedback_a_parameter_that_changes_nothing_is_discarded_downstream]] ·
[[feedback_a_dead_knob_has_two_species_no_probe_catches]] ·
[[feedback_a_promise_that_justifies_a_decision_must_have_a_reader]]
