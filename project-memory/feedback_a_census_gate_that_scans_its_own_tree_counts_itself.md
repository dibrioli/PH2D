---
name: feedback-a-census-gate-that-scans-its-own-tree-counts-itself
description: Gate de censo que varre a workspace onde ele próprio mora conta-se a si mesmo — o da âmbar acusou os dados da própria metade justa e leu 7 chamadas onde havia 4, com o piso verde para UM chamador real (13/09)
metadata:
  type: feedback
---

O gate `the_editor_amber_has_one_door` (integração de 2026-09-13) varre todo `.rs` de `crates/`,
`shells/` e `tools/` à procura do literal da âmbar e conta as chamadas à porta. Na 1.ª corrida
reprovou acusando o PRÓPRIO ficheiro: a metade justa escreve o literal como dado de teste. Excluído
só do censo do literal, ficou verde — e continuava errado: as mensagens de `assert!` nomeiam
`editor_highlight::amber(` três vezes, a contagem lia 7 com 4 chamadores reais, e o piso de 4
ficava satisfeito com UM. A prova foi a mutação `PISO_CHAMADAS = 5`, que só reprova depois de o
gate se excluir das DUAS contagens.

**Why:** um gate que procura um padrão escreve esse padrão (nos dados da metade justa, nas
mensagens que dizem a cura). A auto-acusação é alta e barata; a auto-contagem é MUDA e engorda
o piso.

**How to apply:** um censo que varre a árvore onde mora salta o próprio ficheiro no topo do laço,
antes de TODAS as contagens, e prova o piso com a mutação `piso + 1`: se ela sobrevive, a contagem
inclui ruído. Irmã de [[feedback_a_mutation_proof_needs_a_control_on_its_own_filter]].
