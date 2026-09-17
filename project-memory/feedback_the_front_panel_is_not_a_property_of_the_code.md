---
name: feedback-the-front-panel-is-not-a-property-of-the-code
description: O painel que está À FRENTE vem de ~/.ph2d/layout.txt, fora do repo e reescrito por outras árvores a correr — e um bump de z no quadro do arranque fica por baixo do que o reconcile_z acrescenta a seguir
metadata:
  type: feedback
---

⛔⛔ **Um passo de smoke que diz *«veja no painel da direita»* está a afirmar qual painel está à
frente — e isso NÃO é uma propriedade do código.** Medido em 2026-09-16 (`line/components`, cena do
emissor): três fotografias da MESMA cena, na mesma árvore, deram o painel do **esqueleto**, o do
**vector** e o **Inspector** — porque a arrumação vive em `~/.ph2d/layout.txt`, **fora do
repositório**, e estava a ser reescrita por outra worktree a correr em paralelo.

⚠️ **E a cura óbvia não chega:** `hero.store.bump_panel_z(id)` feito no quadro em que a cena monta
fica **por baixo** dos painéis que o `reconcile_z` acrescenta à ordem z no **início de cada quadro**
(ele apende os que ainda lá não estão). A subida tem de acontecer **num quadro seguinte** — e tem de
**PARAR** (repeti-la por quadro roubaria ao dono a aba que ele escolhesse).

**Why:** o smoke é onde o dono APRENDE a ferramenta (CLAUDE.md §0.8), e um passo que nomeia uma
superfície que ele não tem à vista ensina que a feature não existe. É a mesma lei de
[[feedback_a_smoke_step_that_names_a_panel_row_must_prove_the_row_is_in_the_list]], um nível acima:
ali a LINHA tinha de estar na lista, aqui o PAINEL tem de estar à frente.

**How to apply:** ao escrever uma cena de smoke que manda olhar para um painel, (1) abra-o
(`panel_visibility`), (2) traga-o à frente **por alguns quadros** depois do arranque, (3) abra a cena
com o objecto **já escolhido** (senão o painel diz *«Select an entity…»*), e (4) **fotografe** — a
foto é a única régua que vê isto ([[feedback_a_smoke_for_the_owner_explains_what_each_thing_on_screen_is]]).
