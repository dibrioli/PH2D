---
name: feedback_a_touch_does_not_measure_an_edit_and_timings_inflate_under_contention
description: "Ao medir compilação — `touch` NÃO mede uma edição (o incremental vê o hash igual e faz nada, e o mtime suja TODOS os targets de uma vez); e a soma de unidades do `cargo --timings` é parede por unidade, que dobra sob contenção — nunca é CPU"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: e1350c97-44b0-4738-9885-e8cbde973bb1
  modified: 2026-09-11T00:36:47.840Z
---

Na auditoria de velocidade de 2026-09-10 três sondas responderam ao contrário até
serem refeitas:

1. **`touch` não é uma edição.** `touch crates/ph2d-color/src/lib.rs` + `cargo check -p
   ph2d-host-desktop` re-verificou 103 unidades em **2,8 s**; uma `fn` acrescentada de
   verdade custou 3,3 s no `check` mas **34,7 s** no `ci-test` (o incremental do `rustc`
   vê o hash da crate igual e devolve o cache; o cargo só viu o mtime).
2. **O mtime é global, o target não.** Um `touch` feito para medir em `target/debug`
   sujou o mesmo ficheiro em `target/audit-check` e em `target/ci-test`: a sonda seguinte
   leu **302 unidades** de «cascata de unificação de features» que não existia (refeita
   sem `touch` residual: 1 · 1 · 0 unidades — o `cargo-hakari` ficou recusado por medição).
3. **`--timings` mede parede por unidade.** O mesmo `cargo check --workspace` somou
   1 968 s de unidades a `-j 32` e 920 s a `-j 6`; a «CPU» inflou 2× por contenção, e o
   «mínimo teórico» derivado dela (328 s) errou por 2× contra o observado (156 s). A
   unidade da shell leu 36 s com 28 vizinhas e 18,6 s sozinha.

**Why:** cada uma destas leituras teria entrado no relatório como facto — a segunda
mandava adoptar uma ferramenta (hakari) para curar um defeito inexistente.

**How to apply:** (1) medir uma edição = acrescentar uma `fn` e reverter com `git checkout
-- <ficheiro>`, nunca `touch`; (2) entre sondas em targets diferentes, confirmar `git
status` limpo e correr a sonda duas vezes (a 2.ª tem de dar 0); (3) do `--timings` tirar
o **caminho crítico** e a **concorrência média**, não a soma; comparar unidades só quando
correm no mesmo regime de carga; (4) imprimir `/proc/loadavg` ao lado de cada número.
Relatório: `docs/DevOps/AUDITORIA_VELOCIDADE_DE_DESENVOLVIMENTO_2026-09-10.md` §9.
[[reference_topic_measurement_discipline]] · [[feedback_a_probe_that_waits_on_pgrep_catches_the_other_worktrees_compiler]]
