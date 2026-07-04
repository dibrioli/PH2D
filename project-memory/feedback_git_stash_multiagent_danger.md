---
name: feedback-git-stash-multiagent-danger
description: git stash (even path-scoped) é perigoso em sessão multi-agente com índice sujo compartilhado — pode injetar conflict markers em arquivo de outro agente
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 027bd290-63b5-49bd-9892-04ec7b114bc1
---

`git stash push -- <minhas-pastas>` seguido de `git stash pop` **NÃO é seguro**
quando outro agente tem arquivos STAGED no índice compartilhado. Em 2026-05-28,
durante remediação Painter T2.1, usei stash escopado pra isolar a causa de 4
testes falhando. Enquanto eu estava stashed, o agente asset-ktx2 commitou
(HEAD moveu). No `stash pop`, o git fez merge 3-way e **escreveu conflict
markers (`<<<<<<< Updated upstream` / `>>>>>>> Stashed changes`) dentro de
`crates/ph2d-asset-ktx2/src/lib.rs`** — arquivo FORA do meu pathspec — deixando
o índice `UU`. Felizmente o lado "Stashed" era só uma versão pré-rustfmt de
código já commitado em HEAD, então `git checkout HEAD -- <arquivo-dele>`
restaurou sem perda. Mas podia ter sido data-loss.

**Why:** stash captura/restaura estado de índice; com índice compartilhado sujo
e HEAD movendo entre push e pop, o merge do pop vaza pra arquivos alheios.

**How to apply:** em sessão multi-agente, para isolar "é minha mudança ou de
outro agente?", NÃO use `git stash`. Alternativas seguras: (1) copiar meus
arquivos pra /tmp, `git checkout HEAD -- <só-meus-arquivos>`, testar, restaurar
do /tmp; OU (2) raciocinar estaticamente (quais paths meus toquei vs o que o
teste exercita) — bastou aqui: toquei `ui_snapshot`/`apply_ui_edit`/
`handle_panel_event`, os testes quebrados exercitam `begin_stroke`/`queue`/
sample-count/`detach_journal`/tilt (territorial de WIP alheio em
`dispatch/number_input.rs`+`tick.rs`). Relaciona [[feedback_destructive_reset_collision_2026_05_28]]
e [[feedback_parallel_agent_collision]].
