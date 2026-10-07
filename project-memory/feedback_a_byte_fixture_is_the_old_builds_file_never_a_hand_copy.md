---
name: feedback-a-byte-fixture-is-the-old-builds-file-never-a-hand-copy
description: Uma fixture de BYTES de um formato antigo é o ficheiro que o build antigo GRAVOU (include_bytes!), nunca uma lista de números copiada à mão de um eprintln — duas cópias minhas de 251 bytes saíram com 253 e 257
metadata:
  type: feedback
---

Na W3 do Quadro (06/10) o formato subiu de 2 para 3 e o teste do leitor do formato 2 precisava dos
bytes de um ficheiro antigo. Imprimi-os com `eprintln!("{:?}")` e colei a lista no teste — e
reparti-a em linhas à mão. Duas vezes: 253 e 257 números, contra 251 reais. Na 1.ª o teste até
LIA (com uma seta a z `""` em vez de `"a1"`); só falhou numa asserção de ordem.

**Why:** um modelo não copia 251 inteiros sem erro, e o erro não é visível — o postcard lê bytes
errados como outro documento válido. A régua (a fixture) mentia sobre o leitor.

**How to apply:** compile as fontes do commit antigo À PARTE (`git ls-tree` + `git show` para uma
pasta do scratchpad, `[workspace]` próprio, `CARGO_TARGET_DIR` próprio — nunca o da worktree, ver
[[feedback_sharing_a_target_dir_between_worktrees_corrupts_the_build]]), deixe esse build ESCREVER o
ficheiro, e o teste lê-o por `include_bytes!` (`crates/ph2d-board-model/fixtures/format_v2.bin`).
Família: [[reference_topic_fixture_discipline]].
