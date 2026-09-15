---
name: feedback-a-sweep-that-freezes-an-order-must-read-the-same-key-the-tree-reads
description: Uma varredura que "congela a ordem que a árvore já mostra" tem de LER a árvore pela porta dela — uma cópia da chave inverteu a pilha de z de toda a app por três semanas, em silêncio.
metadata:
  type: feedback
---

Medido 2026-09-15 (`line/components`, report do dono: *«para o Hero ser visível deve ficar abaixo na
Hierarchy»*). A `ph2d_ecs::assign_missing_root_order` promete por escrito *«congelar a ordem que a
árvore mostra HOJE — então a tela não muda»*, e congelava por **`to_bits()`**, que no bevy é a ordem
de criação **INVERTIDA**. A árvore, desde que a chave se unificou na porta `root_key` (2026-08-27),
lê **`index()`**.

⇒ quatro raízes criadas em sequência saíam com `RootOrder [3, 2, 1, 0]`: o **chão desenhava por cima
de tudo**, em toda cena cujas raízes nascem sem número.

**Why:** a lei estava escrita em TRÊS sítios e a unificação mudou dois. *Uma lei escrita em três
sítios só viaja para os dois de que alguém se lembrou* — e o terceiro fica com um doc-comment que
descreve a casa de outra época, indistinguível de um que descreve a de agora. Pior: o repo **já
media** a inversão, num gate da função irmã (`to_bits_is_not_creation_order_which_is_why_the_sweep_uses_index`).

⛔⛔ E o ficheiro **não tinha um único teste próprio** — foundational, a correr em todo quadro, a
decidir a pilha de z do canvas inteiro. Foi isso que deixou a inversão viver três semanas.

**How to apply:** quando uma varredura diz que **preserva** uma ordem, ela tem de a ler pela PORTA
do consumidor, nunca por uma cópia da chave — e o gate é *«correr a varredura não muda a lista»*,
com um segundo a fixar **qual** ordem é (senão «estável mas arbitrária» passa). Ao unificar uma
chave numa porta, conte os leitores com `grep` antes de fechar. Relacionado:
[[feedback_the_door_with_the_right_law_had_no_caller_and_the_consumer_used_a_third]] ·
[[reference_topic_gate_discipline]] · [[feedback_a_gesture_written_in_two_halves_accepts_a_new_variant_in_only_one]]
