---
name: feedback-a-registry-cannot-tell-a-missing-feature-from-a-typo-ask-the-tree
description: Gate que confirma «este id existe» perguntando a um registry acusa ids CORRECTOS quando a build de teste tem menos features que o app — o oráculo é a árvore
metadata: 
  node_type: memory
  type: feedback
  originSessionId: af27d1c2-3a56-4abe-9acd-e2c91caf58f0
  modified: 2026-08-31T16:32:59.490Z
---

Um gate que confere *«esta tabela nomeia coisas que existem?»* costuma perguntar ao **registry**. Ele
está certo até ao dia em que a build do teste e a do **produto** têm conjuntos de features
diferentes — e aí ele acusa entradas **correctas**.

**Medido na `line/UIUX`, 2026-08-30 (entrega 26):** a tabela dos layouts nomeia `painter_layers`,
`flip` e `flip_frames`. Eles estão nas features do **shell** e **não** nas de omissão da crate onde o
teste corre ⇒ o gate acusou três ids certos de *«não é um painel registado»*.

> *Uma ausência por feature e um erro de escrita leem-se iguais num registry; só a árvore os separa.*

⭐ **A cura são DOIS gates, cada um com o oráculo da sua pergunta:**

| pergunta | oráculo |
|---|---|
| *este id está escrito certo?* | a **pasta da crate** (`crates/ph2d-panel-<id>`) — independente de features |
| *o que esta build tem de facto abre?* | o **registry**, sobre a intersecção com as features presentes, com piso `>= N` |

**Why:** o registry responde *«o que está montado agora»* e uma tabela estática afirma *«o que existe
no repo»*. São perguntas diferentes, e usar a primeira para a segunda torna o gate dependente do
`Cargo.toml` de quem o corre.

**How to apply:** antes de escrever `assert!(registry.contains(id))` sobre uma tabela estática,
**compare as features da crate de teste com as do binário**. Se diferirem, o oráculo é a árvore — e o
gate do registry passa a medir só o que a build tem, com um piso para não medir o vazio.

Relacionadas: [[feedback_a_new_feature_can_empty_an_existing_gates_population]] ·
[[project_ci_runs_26_of_313_workspace_members]] ·
[[feedback_a_tree_scanning_gate_is_never_reached_by_a_name_filter]]
