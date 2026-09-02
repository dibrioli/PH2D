---
name: feedback-two-opt-outs-for-the-same-file-mean-it-is-in-the-wrong-directory
description: "Dois opt-outs de gate para o MESMO ficheiro novo é sinal de sítio errado, não de gates chatos — mova-o e os dois somem"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: af27d1c2-3a56-4abe-9acd-e2c91caf58f0
  modified: 2026-09-01T19:23:41.200Z
---

Um tipo novo (`AreaMenu`, 2026-09-01, `line/UIUX`) foi posto em
`crates/ph2d-editor-core/src/widget/` porque carregava `ToolRailEntry`. Isso
disparou **três** portões em cadeia: o teto de LOC do ficheiro que o hospedava,
a cobertura da galeria de widgets (*«todo widget aparece no showcase ou tem
opt-out escrito»*) e o HR-12 (*«todo ficheiro de widget liga semântica de
acessibilidade»*).

A 1.ª reacção foi escrever os opt-outs, um de cada vez, cada um com uma
justificação sincera. **Ao escrever o segundo, a justificação era a mesma frase
nas duas:** *«não é um widget — não pinta nada»*.

**Why:** cada gate de directório afirma uma propriedade do **conjunto**
(*«tudo em `src/widget/` é um widget»*). Um opt-out é a excepção honesta a UMA
propriedade. **Dois opt-outs sobre o mesmo ficheiro não são duas excepções: são
a medição de que o ficheiro não pertence ao conjunto** — e cada opt-out escrito
enfraquece o gate para todos os ficheiros futuros, que é o preço que ninguém vê.

**How to apply:** ao ver o segundo gate de directório a acusar o mesmo ficheiro
novo, **pare de escrever justificações e pergunte de que conjunto ele é.**

- A cura é mudá-lo de directório, e ela costuma apagar também o teto de LOC que
  começou tudo (o ficheiro sai do hospedeiro que estourava).
- O vizinho certo é quem responde à **mesma pergunta do outro lado** — o
  `AreaMenu` (*que menu a área contribui*) foi para junto do `ContextMenuKind`
  (*que menu isto abre*), e não para junto do tipo que ele por acaso carrega.
- ⚠️ **Depender de um tipo não é pertencer ao módulo dele.** Foi essa a
  confusão: `AreaMenu` tem um `Vec<ToolRailEntry>` dentro, e o `interaction/`
  já dependia do `widget/` de qualquer maneira.
- Um opt-out legítimo é solitário e diz o que o ficheiro **é** (*«overlay
  flutuante, sem visual em repouso»*); dois a dizer o que ele **não é** são um
  endereço errado.

Irmãos: [[feedback_loc_cap_split_not_allowlist_and_fmt_reexpands]] ·
[[feedback_a_ratchet_without_a_staleness_census_only_ratchets_up]] ·
[[reference_topic_gate_discipline]] ·
[[feedback_an_arch_gate_anchored_on_a_file_fails_when_the_loc_cap_moves_the_code]]
