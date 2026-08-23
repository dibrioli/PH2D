---
name: feedback-checkout-ours-discards-the-hunks-git-already-merged
description: "Num conflito de rebase, `git checkout --ours -- f` substitui o arquivo INTEIRO pelo estágio 2 — e joga fora os hunks NÃO-conflitantes de theirs que o git já tinha fundido no working file. Resolva só o hunk marcado, ou reaplique todos os hunks de theirs à mão."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: d2f2dbec-7784-4b38-bcf8-424045e2fd3c
  modified: 2026-08-23T00:53:07.243Z
---

**O que aconteceu (integração de 2026-08-22, `line/3DModeling` W4):** `topbar/mod.rs` tinha UM hunk
em conflito (`mod image_action_row;` vs `mod tooltips;`) e DOIS hunks de theirs que o git já
tinha fundido sozinho no working file (o registro do `TOPBAR_MODEL3D` e a troca da tabela inline
por `tooltips::seed_tooltips`). Fiz `git checkout --ours -- f` para «partir do lado main» e
reapliquei só o hunk conflitante. Os dois fundidos evaporaram **sem erro**, a rebase seguiu, o
`check --workspace` passou — e o gate `topbar_painted_pills_are_all_registered` apanhou o pill
morto 10 minutos depois. O mesmo método nos arquivos de `PROJECT_SCHEMA`/registro funcionou só
porque lá eu reapliquei **todos** os hunks de theirs à mão (eram o conflito inteiro).

**Why:** `--ours`/`--theirs` escrevem o ESTÁGIO inteiro (`:2`/`:3`) por cima do working file; o
working file era o único lugar onde o auto-merge dos hunks limpos vivia. «Resolver pelos estágios»
(a regra da memória) significa LER `:1/:2/:3` para decidir — não substituir o arquivo por um deles.

**How to apply:**
- Antes de resolver, meça: `git diff :1:f :3:f` (tudo o que theirs mudou) vs os hunks marcados no
  working file. Se theirs tem hunks FORA dos marcadores, `checkout --ours` vai perdê-los.
- Resolva editando **só a região entre marcadores** (com o conteúdo de `:2`/`:3` já lido), ou, se
  partir de um estágio, reaplique **cada** hunk de theirs e confira com
  `diff <(git show <tip-original>:f) f` — o resto tem de ser só o que o main trouxe.
- Um `Cargo.toml`/lista resolvido por união direta no hunk (como fiz no `shells/desktop/Cargo.toml`)
  preserva o auto-merge; foi a resolução certa.
- Relacionado: [[feedback-resolve-conflicts-from-index-stages-not-markers]],
  [[feedback-clean-text-merge-can-be-semantically-broken]].
