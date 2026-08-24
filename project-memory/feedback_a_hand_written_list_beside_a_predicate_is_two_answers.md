---
name: feedback_a_hand_written_list_beside_a_predicate_is_two_answers
description: Um diálogo/menu que ENUMERA ao lado de um predicado que DECIDE são duas respostas à mesma pergunta — e a que o utilizador vê é a que envelhece
metadata: 
  node_type: memory
  type: feedback
  originSessionId: d971358c-b4ab-4ed0-ab84-65cd6d892c68
  modified: 2026-08-23T22:58:53.958Z
---

Quando uma superfície precisa de **enumerar** o que o app aceita (o filtro de um `FileDialog`, os
itens de um menu, os chips de um picker) e outra o **decide** por um predicado, a lista acaba
escrita à mão — porque um predicado não se enumera. As duas divergem em silêncio, e a que o
utilizador vê é sempre a que ficou para trás.

**Why:** medido no PH2D em 2026-08-23. O drag & drop roteava por
`ph2d_asset::is_supported_image_extension` (**11** extensões) e o botão «Import…» oferecia uma
lista escrita à mão com **4**. O `.gif`, o `.psd`, o `.ora`, o `.tiff` e o `.apng` entravam por
arrasto e eram **invisíveis** no seletor **há meses**. O Enio só reportou (*«.ase não aparece no
dialog de import»*) quando uma extensão NOVA caiu no mesmo buraco — o defeito antigo nunca gerou
report, porque ninguém procura no diálogo o que já sabe arrastar.

**How to apply:** faça da **LISTA** a fonte (`pub const X_EXTENSIONS: &[&str]`) e **derive** o
predicado dela — nunca o contrário. Depois ponha um gate que afirme **as duas metades**: *tudo o
que a superfície oferece o roteador aceita* **e** *tudo o que o roteador aceita a superfície
oferece*; cada uma sozinha fica verde no caso degenerado (uma lista vazia passa a primeira, uma
lista com tudo passa a segunda). E se as duas superfícies fazem a mesma coisa depois de decidir,
faça-as chamar **a mesma função** — a diferença entre elas passa a ser só de onde vêm os dados,
que é a única diferença que elas de facto têm.

Irmã de [[feedback_paint_and_dispatch_must_read_the_same_source]] (lá são duas fontes para pintar
e despachar; aqui são duas fontes para enumerar e decidir) e de
[[reference_topic_ui_seam_discipline]].
