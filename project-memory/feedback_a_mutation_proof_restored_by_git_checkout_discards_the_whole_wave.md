---
name: feedback-a-mutation-proof-restored-by-git-checkout-discards-the-whole-wave
description: "Restaurar uma mutação com `git checkout -- <ficheiro>` apaga TODO o trabalho não comitado dele, não só a mutação — restaure de um backup do ficheiro"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: af27d1c2-3a56-4abe-9acd-e2c91caf58f0
  modified: 2026-08-31T18:13:44.431Z
---

Uma prova de mutação é *backup → mutar → testar → **restaurar***. ⛔ **O passo de restaurar nunca é
`git checkout -- <ficheiro>`:** ele repõe o ficheiro como está em **HEAD**, e num ficheiro com
trabalho por comitar isso apaga a wave inteira, não a mutação.

**Medido na `line/UIUX`, 2026-08-31:** provei red-first o gate
`no_layout_opens_a_panel_that_a_tool_bridge_owns` mutando o `task_layout.rs` (um layout volta a
nomear um painel de ferramenta). O gate reprovou como devia — e o `git checkout --` a seguir
devolveu o ficheiro ao estado de HEAD, apagando o `CanvasOwner`, a tabela nova e três blocos de
doc-comment. Tive de os reescrever de memória.

> *O `git checkout` não sabe qual das duas edições do ficheiro é a mutação.*

⭐ **A cura é a mesma que o script de mutação já usa nos outros passos:**

```fish
cp <ficheiro> /tmp/.../f.bak     # ANTES de mutar
…mutar, testar…
cp /tmp/.../f.bak <ficheiro>; touch <ficheiro>
```

⚠️ E o `touch` continua obrigatório — ver
[[feedback_a_mutation_restore_that_preserves_mtime_leaves_cargo_stale]].

**Why:** o sinal de perigo é o ficheiro estar **sujo antes** da mutação, que é o caso normal de uma
prova feita no meio de uma wave. Comitar antes de mutar também resolve, mas obriga a um commit por
prova.

**How to apply:** antes de escrever `git checkout --`, `git restore` ou `git stash` num ficheiro que
estou a editar, pergunte *«o que este ficheiro tem por comitar?»*. Se a resposta não for «nada»,
restaure do backup.

Relacionadas: [[reference_topic_mutation_proofs]] · [[reference_topic_git_hazards]] ·
[[feedback_a_mutation_restore_that_preserves_mtime_leaves_cargo_stale]]
