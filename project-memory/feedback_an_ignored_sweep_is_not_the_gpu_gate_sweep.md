---
name: feedback-an-ignored-sweep-is-not-the-gpu-gate-sweep
description: "`cargo test -- --ignored` varre TRÊS coisas diferentes; só uma é o gate de GPU que o handoff manda rodar"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 55ac7554-e543-455e-be55-15b5f3eb4809
  modified: 2026-08-08T04:46:49.068Z
---

`#[ignore]` é usado neste repo para **três** propósitos incompatíveis, e uma varredura
`-- --ignored` do workspace roda os três juntos:

1. **gates de GPU** (precisam de adapter — é o que "skip não é verde" quer dizer);
2. **sondas de MEDIÇÃO** (ignoradas por serem lentas: uma delas leva 12 min, outra 5+);
3. **placeholders `unimplemented!()`** de wave futura, que **panicam por desenho** e trazem
   a razão escrita no próprio `#[ignore] = "…"`.

**Why:** na integração de 2026-08-02 a varredura `--ignored` do workspace devolveu **6
vermelhos** e nenhum era regressão — 4 eram placeholders de 2026-05-28 e 2 eram falhas
pré-existentes provadas por controle. Perseguir isso custou ~40 min, e a varredura serial
que eu iniciei para "ser rigoroso" ficou presa numa sonda de instrumento.

⚠️ **E a lista de crates sai do DIFF, nunca do handoff.** Em 2026-08-07 fechei uma wave
relatando "GPU 44/44" — 44 gates do módulo 3D, todos verdes, e a wave tinha editado um
**shader da `ph2d-render`**. Os seis gates que cobrem aquele shader são `#[ignore]` e ficaram
de fora; eles estavam **vermelhos**, e o defeito era um uniform 16 bytes maior no WGSL que no
Rust — ou seja **todo dispatch da luz de impasto na GPU panicava**, não só o do módulo novo.
O Enio o encontrou em dois minutos de smoke. *Uma varredura por crate ainda é uma
ENUMERAÇÃO; `git diff --name-only | cut -d/ -f2 | sort -u` é a lista que não apodrece.*

**How to apply:** rode **por CRATE, os nomes que o handoff cita** (`ph2d-mesh-render::gpu_render`,
`ph2d-flip-render`, `ph2d-gpu-cook`), não `--workspace -- --ignored`. E quando um `--ignored`
reprovar, leia a **razão do `#[ignore]`** antes de investigar: se ela diz *"Un-ignore quando a
wave X chegar"*, não há defeito. Para os que sobram, o oráculo é o **CONTROLE** — uma worktree
no commit pré-jornada ([[feedback_a_negative_search_needs_a_positive_control]]); um valor de
falha **byte-idêntico** nos dois lados encerra a questão.

⚠️ E as sondas de perf de GPU do MESMO binário rodam em **paralelo entre si** por default,
disputando o device: um gate de RAZÃO ali mede a contenção, não o código
([[feedback_probes_that_measure_parallelism_must_run_alone]]) — `--test-threads=1`.
