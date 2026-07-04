---
name: feedback-ship-committed-vs-worktree-wip
description: "ship.sh on a WIP-dirty tree can pass while CI (committed) fails; verify the committed state + fix drift via a detached worktree, never touching foreign WIP"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 600bb7a6-aa23-4b71-bc5d-edb4bb88d292
---

Quando há **WIP alheio não-commitado** no working tree (impl ativo), `ship.sh`
local roda sobre `committed + WIP` e pode passar VERDE enquanto o **CI (que testa
só o committed) fica VERMELHO** — porque o WIP do outro agente mascara drift
(fmt/clippy/lint) nas versões **committadas** dos mesmos arquivos.

**Why:** CI faz checkout limpo (sem WIP); `cargo fmt --all -- --check` /
`clippy` local veem as versões WIP (que o impl já formatou), não as committadas.
Mordeu 2026-06-04: CI quebrou em `fmt --check` de 4 arquivos Vector cujas
versões committadas tinham drift, mas o working tree tinha o WIP fmt-clean.

**How to apply:**
1. Antes do push, valide o estado **committado**, não o working tree: crie um
   `git worktree add --detach /tmp/wt HEAD` (committed, zero WIP) e rode
   `cargo fmt --all -- --check` + `clippy` + `nextest` lá (use
   `CARGO_TARGET_DIR=<slot warm>` p/ não rebuildar do zero).
2. Para CORRIGIR drift committado sem tocar no WIP alheio: rustfmt/edite **no
   worktree**, commite lá, `git -C /tmp/wt push origin HEAD:main`. O WIP no
   working tree principal fica intacto.
3. Reconcilie o tree principal: `git update-ref refs/heads/main <sha>` +
   `git reset --mixed HEAD` (NÃO toca working tree; sincroniza o index, senão um
   commit reverteria o fix — vi `MM` phantom-staged). NUNCA `git stash`
   (injeta conflito, vide [[feedback-git-stash-multiagent-danger]]).
4. `nextest --no-fail-fast` enumera TODOS os gates de uma vez; `ship.sh` é
   fail-fast e esconde falhas depois da 1ª ([[feedback-ship-prep-no-fail-fast]]).

Relacionado: [[feedback-cargo-fmt-p-reformats-foreign-wip]],
[[feedback-parallel-agent-collision]], [[feedback-ci-direct-lint-gates-and-fmt-skew]].
