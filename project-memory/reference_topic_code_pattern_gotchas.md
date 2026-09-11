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
