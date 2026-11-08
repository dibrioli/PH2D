---
name: project_add_component_owner_decisions_2026_11_08
description: Dono 08/11 — Add Component na versão B da Crítica + recibo da A; destaque inicial = 1.º da lista; «brings» vira «+N» (lista na dica); Show all fica
metadata:
  type: project
---

Decisões do dono (08/11) para o painel Add Component, no mesmo padrão do Add Object
(Crítica `GameEngineSamples/critica_ph2d.html#addcomponent`):

1. **Versão B + recibo da A**: a mesma janela grande, 11 assuntos (nenhum grupo de uma entrada só;
   Nav ganha grupo, Label/Button/HUD vão para «On screen», Parallax para «Background»), dica para cada
   uma das 64 entradas, e depois de adicionar uma linha «Also added: X, because …» com **Undo** que
   desfaz tudo de uma vez.
2. **Destaque ao abrir = o 1.º item do 1.º grupo** (a faixa da dica nunca nasce vazia, como Sprite no
   Add Object).
3. **«brings …» vira «+N»** ao lado do nome; a lista inteira do que vem junto vai na dica.
   **Show all fica** como hoje (inaplicável esmaecido, com motivo).

4. **Game Camera SAI do Add Component** (dono 08/11, depois do 1.º smoke): a câmera nasce só por Add Object ▸
   Camera. «Ver se ajustes são necessários»: as peças que só servem numa câmera (Follow, Limits, Shake) não podem
   ser oferecidas a quem não é câmera, e nenhuma cena/tutorial pode depender de anexar a Game Camera pela janela.

5. **Busca acende só com clique ou tecla** (smoke 08/11): ao abrir fica em repouso; clique nela ou 1.ª tecla →
   anel + caret; clique num item apaga.
6. **Fechadas as duas do Add Object** (08/11): destaque do item **cheio (azul)**, como está; **peso das letras
   fica como está** — não mexer em `CrispHeavy` nem trocar para contorno.

**Why:** produto é do dono; ele seguiu as três recomendações da Crítica.
**How to apply:** plano em `docs/GameEngineFuncional/25_add_component_plano.md`. O catálogo por baixo
pode ficar igual — os 11 assuntos são apresentação. Peso das letras e destaque: decididos (item 6). Ver
[[project_add_object_owner_decisions_2026_11_07]].
