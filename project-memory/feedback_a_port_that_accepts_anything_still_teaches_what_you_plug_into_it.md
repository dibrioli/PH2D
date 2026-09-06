---
name: a-port-that-accepts-anything-still-teaches-what-you-plug-into-it
description: Num tipo largo, "compila e desenha" não é a barra de uma cena de demonstração — a porta ensina o idioma, e o idioma errado viaja para o trabalho do artista.
metadata:
  type: feedback
---

Uma porta tipada de forma LARGA (aqui: `Domain::Instances` — qualquer corrente de elementos)
aceita muita coisa que não é o que ela quer dizer. O `motion.duplicator` tem duas: `shape` (*o
que desenhar*) e `points` (*onde*). As duas aceitam o mesmo tipo, então **só a semântica as
distingue** — e um `motion.grid` de uma célula na porta `shape` compila, cozinha e desenha.

⛔ **E está errado numa cena de demonstração, por uma razão que o teste não vê:** o artista lê o
grafo e aprende com ele. *«Para fazer uma forma, use uma grelha de 1×1»* é o que a minha cena
ensinava, e ele leva isso para o trabalho dele. O report foi do dono, olhando o grafo:
*«você colocou grid entrando em Shape de Duplicator! Essa aplicação é correta?»*

⇒ **Antes de ligar qualquer coisa a uma porta de tipo largo numa cena que alguém vai LER,
procure o nó cuja razão de existir é aquela porta.** Aqui era o `source.shape`, e o doc dele
nomeia a composição à letra: *«cross it with a `motion.grid` through a `motion.duplicator` and
the shape is stamped, crisp, at every point»*. A resposta estava escrita no nó que eu não
procurei.

⭐⭐ **E o idioma certo saiu MAIS BARATO, o que é o sinal de que era mesmo o certo:** o ladrilho
precisava de `grid + transform + scale + tint` (quatro nós) para ser «uma forma»; o
`source.shape` é **um** nó com `kind` e `size` como params. A cena passou de **140 nós para
102**, ficou vectorial (nítida em qualquer zoom) e ganhou silhuetas de verdade — o `Pick`
deixou de escolher entre três quadrados de cores diferentes e passa a escolher entre um
círculo, uma estrela e um coração, que é a pergunta que ele responde.

⚠️ **O gate que fecha isto segue a porta até à ORIGEM do braço** (sobe pela entrada 0 até um nó
sem entradas) e afirma o tipo dela — não basta olhar o vizinho imediato, porque entre a fonte e
o carimbo há transformes e tints.

⚠️ **Preço a saber:** um `source.shape` lê um EXTERNAL que a shell publica, então **num cook nu
ele emite zero** — os gates da cena têm de cozinhar através de um `MotionState` depois de
`motion_shape_gen::publish`, senão medem uma cena vazia e dizem que ela não monta.

Relacionado: [[feedback_a_correct_property_can_cost_the_legibility_that_was_the_whole_point]] ·
[[feedback_the_design_being_asked_for_may_already_be_law_in_another_half_of_the_app]] ·
[[feedback_a_door_the_neighbour_does_not_call_is_not_a_door_yet]]
