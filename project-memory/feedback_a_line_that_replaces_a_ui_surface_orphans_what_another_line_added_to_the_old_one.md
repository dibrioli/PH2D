---
name: feedback_a_line_that_replaces_a_ui_surface_orphans_what_another_line_added_to_the_old_one
description: "Linha A troca a superfície (o botão que abre o menu); linha B acrescenta um verbo ao menu antigo. O merge é limpo, o verbo compila, e o artista deixa de o ver — sem warning, sem gate"
metadata:
  type: feedback
---

Report do Enio, 2026-09-04: *«depois de integração funções criadas por outros módulos não aparecem
na UI. Exemplo: Exportar Vector SVG.»*

**O mecanismo, com datas:** em 30/08 a `line/UIUX` trocou os 29 *pills* do topo por uma **barra de
menus** — o pill `TOPBAR_SAVE` (o único botão que abria o `SaveMenu`) deixou de ser pintado, e a
barra prometeu levar as rows dele para o menu *File*. Em 02/09 a `line/Vector` acrescentou o
*Export SVG…* ao **`SaveMenu`**, que era o menu que ela via na árvore dela. As duas linhas **nunca
tocaram na mesma linha de texto**: o merge foi limpo, o verbo continua declarado, registado,
despachado e **testado** — e não existe gesto nenhum que lá chegue.

⛔ **Nenhum sintoma da família do `dead_code` aparece aqui**: a row é construída, o handler é
chamado por um teste, o id é registado. O que morreu foi a **aresta** entre a superfície e o verbo,
e o compilador não tem opinião sobre arestas.

**Why:** é o irmão SILENCIOSO do
[[feedback_two_lines_can_refactor_the_same_code_differently_and_both_survive_the_merge]] — lá o
resíduo é uma função morta que o `clippy` denuncia; aqui o resíduo é **alcance**, e o único
instrumento é o dono a abrir o menu e não ver o item. E é pior que um botão MUDO: um item mudo o
artista vê e conclui que agiu; este ele não vê e conclui que **a funcionalidade não existe**.

**How to apply:**
- ⭐⭐ **Quem TROCA uma superfície de UI deixa atrás um gate que mede o REALOJAMENTO por rows, não
  por prosa.** Cura de 04/09:
  `ph2d-editor-core/tests/the_bar_relocated_every_row_of_the_menus_it_replaced.rs` — parte dos
  títulos da barra, **carrega em cada linha** e segue as cascatas reais (⛔ nunca uma tabela
  `row → submenu` escrita à mão: já existem duas no produto e uma terceira divergiria em silêncio),
  e exige que toda row de [`LEGACY_PILL_MENUS`] caia nesse fecho.
- ⚠️ **A pergunta certa não é «este id tem handler?» — é «que GESTO chega a ele?»**. O gate que
  existia (`every_topbar_verb_has_a_door_that_is_not_the_legacy_key`) mede a população dos **ids de
  pill declarados**; o buraco estava nas **rows** que cada pill abria, que é outra população.
- ⛔ **Uma paleta de comandos NÃO é uma porta**: ela é busca por nome, e só serve quem já sabe que
  a funcionalidade existe. O `Export SVG…` estava lá o tempo todo e o dono não o encontrou.
- ⚠️ **Ao integrar, a pergunta a fazer a cada linha não é só «que ficheiros tocaste»** (é o que o
  `collision-surface.sh` responde) — é **«que SUPERFÍCIE de UI substituíste, e quem escreveu nela
  desde o teu fork?»**. Uma substituição de superfície não colide com nada e não aparece em tabela
  de colisão nenhuma.

Relacionado: [[feedback_an_opt_out_can_name_a_consumer_that_does_not_exist]] ·
[[feedback_two_lines_can_refactor_the_same_code_differently_and_both_survive_the_merge]] ·
[[reference_topic_ui_seam_discipline]] ·
[[feedback_a_door_the_neighbour_does_not_call_is_not_a_door_yet]]
