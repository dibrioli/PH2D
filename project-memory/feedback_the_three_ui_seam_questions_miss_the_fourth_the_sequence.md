---
name: feedback_the_three_ui_seam_questions_miss_the_fourth_the_sequence
description: "o widget existe · é pintado · o clique chega ao barramento" podem estar os TRÊS verdes enquanto a feature é inalcançável — a quarta pergunta é a SEQUÊNCIA, e ela precisa de gate próprio
metadata:
  type: feedback
---

A booleana viva nos estados de UI shipou com os três seams provados: a fileira **existe**, é
**pintada e registrada** (com gate de gesto real), e o clique **chega ao barramento**. E a feature
era **inalcançável**: tocar um operando seleciona o GRUPO inteiro (lei deliberada), a seção STATES
exigia forma ÚNICA, e ela **não era sequer pintada** — nem cabeçalho, nem o interruptor de preview,
nem uma palavra a dizer o que faltava.

**Why:** as três perguntas de sempre são sobre UM widget. A quarta é sobre o **caminho**: *a partir
do que o artista tem em mãos, existe rota até aqui?* Nenhum gate de widget a faz, e ela falha
exactamente onde duas leis corretas se cruzam — aqui, *"tocar um filho seleciona o grupo"* e *"a
seção é da forma única"*, cada uma certa sozinha.

⚠️ E o sintoma é o pior possível: **ausência**. Um botão dimmed convida a perguntar porquê; uma
seção que não é pintada não deixa rasto nenhum.

**How to apply:** ao fechar uma feature de UI, escreva um gate de **ROTA** com a seleção que um
gesto de facto produz (`object_selection_for`, nunca uma lista montada à mão) e exija que a porta
que PINTA devolva o assunto certo — não apenas `Some`, que a **face vazia** também é. Família:
`shells/desktop/src/vec_bool_reach_tests.rs`, `field3d_reach_tests.rs`.
E se a resposta for *"não há rota"*, a cura é a **face vazia com a dica**, nunca o desaparecimento
([[feedback_a_rule_only_exists_if_it_is_on_the_path_of_who_executes_it]]).
