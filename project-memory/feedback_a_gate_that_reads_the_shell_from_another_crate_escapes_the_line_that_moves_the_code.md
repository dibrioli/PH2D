---
name: feedback-a-gate-that-reads-the-shell-from-another-crate-escapes-the-line-that-moves-the-code
description: "Gates de FAMÍLIA que leem `shells/desktop/src/…` pelo caminho não são re-apontados pela linha que muda o código da shell de sítio — três reprovaram na ponta da line/render-loop; varra `crates/` por esse caminho antes de fechar (13/09)"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: e1350c97-44b0-4738-9885-e8cbde973bb1
  modified: 2026-09-13T11:53:16.616Z
---

A `line/render-loop` partiu o `run_render_frame` em 125 fases e re-apontou os gates da SHELL com prova
de mutação — e três gates ficaram vermelhos na ponta dela sem ninguém ver: moram em crates de FAMÍLIA
(`ph2d-app-field3d` export e mode, `ph2d-app-skeleton` overlays do osso) e liam `render_loop/mod.rs`
por `read_to_string`/`include_str!` a partir de fora. Quem os apanhou foi a suíte da árvore combinada
na integração de 2026-09-13.

**Why:** a sonda da linha («quem lê o `mod.rs`?») varreu os gates da crate que ela editava; um gate
cujo SUJEITO é a shell pode morar na família que ele protege (HOWTO §2.6), e a linha nunca corre a
suíte dessa família.

**How to apply:** quem mudar código de sítio dentro da shell corre
`git grep -n 'shells/desktop/src' -- crates tools` e trata cada leitor como gate seu. A lente de um
gate de PRESENÇA sobre o quadro é o `mod.rs` + `fase_*.rs` com piso de população, nunca um ficheiro.
Irmã de [[feedback_the_orphan_and_the_double_declaration_are_one_audit]].
