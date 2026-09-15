---
name: reference_topic_code_pattern_gotchas
description: Padrões de código — os gotchas silenciosos dobrados do índice, verbatim, uma linha por memória
metadata:
  type: reference
---

# Padrões de código (gotchas silenciosos) — lições dobradas do índice (2026-09-10)

> Cada linha é uma memória com o gancho ORIGINAL do índice; abra o ficheiro para o mecanismo.
> Dobradas aqui porque o `MEMORY.md` passou o teto suportado (32 KB > 24 KB): ficam a dois saltos.

- [UI = gallery + inspector](feedback_ui_source_of_truth_gallery_inspector.md) · [UI em inglês](feedback_app_ui_english_only.md)
- [Nada de → em string literal (asserts também)](feedback_no_tofu_arrows_in_string_literals.md)
- [Pré-multiplicada inverte o Multiply](feedback_a_premultiplied_source_breaks_the_blend_whose_identity_is_one.md) · [clone + id de ponteiro = CoW; use versão](feedback_a_held_clone_plus_pointer_identity_change_detection_forces_copy_on_write.md)
- [Atributo separado do item por doc muda de dono](feedback_an_attribute_separated_from_its_item_by_a_doc_comment_changes_owner.md)
- ⭐ [A nota ao lado de uma CONTA descreve a população que ela tinha quando foi escrita — item novo herda-a sem a reconferir](feedback_the_note_beside_a_count_describes_the_population_it_had_when_it_was_written.md)
- ⭐ [Chave cujo dono nunca MORRE não tem sepultador: nem viva nem enterrada = invisível](feedback_a_key_whose_owner_never_dies_has_no_gravedigger.md) · [emparelhar por NOME solto parte a árvore — a chave é o CAMINHO](feedback_matching_by_a_loose_name_breaks_the_tree_the_key_is_a_path.md)
- ⛔⛔ **RE-DERIVAR por subtracção uma grandeza que já foi MEDIDA não a devolve, em `f32`.** Medido
  2026-09-14 (`line/UIUX`, report do dono *«3 pontos mesmo com folga»*, foto do cartão JUMP): um
  rótulo alinhado à direita começa em `recuo = x + (col_w − largura)` e o pintor recebia o orçamento
  `x + col_w − recuo`, *pretendendo* dizer `largura`. O cancelamento cai um ULP abaixo em **310 de
  3 600** células (9 rótulos × 400 posições de `x`, coluna `140 px`, o mais largo `100,3`), e o
  pintor **volta a cortar** um texto que já cabia — pondo reticências num nome com 40–77 px de
  folga. ⚠️ **A assinatura na foto é que o defeito NÃO ordena pelo comprimento**: `Takeoff Gravity`
  (`94,97`) saía inteiro e `Air Jumps` (`62,92`) cortado, na MESMA coluna — *quando o defeito não
  ordena pela grandeza que a lei usa, a causa não é a lei*, e isso mata a hipótese óbvia antes de se
  abrir um ficheiro. ⇒ **quem mede uma grandeza DEVOLVE-A** (a porta passou a `(texto, x, largura)`;
  ela já a calculava e deitava fora). ⛔ E a lei já estava escrita **um nível abaixo**, no próprio
  pintor que foi enganado: *«`INFINITY`, not `max_width`: it fits, and passing the budget back would
  let a sub-pixel measurement disagreement re-introduce the wrap»* — *o comentário descrevia
  exactamente o que o chamador fazia.*
