---
name: feedback_a_flake_red_hides_the_rest_of_the_suite
description: "O nextest cancela no primeiro ✗ — um vermelho de flake deixa mil testes por correr, e a suíte parece medida quando foi só amostrada; `--no-fail-fast` é o que a torna uma medição"
metadata:
  type: feedback
---

Gate batched do fecho de uma linha, 2026-08-23. A primeira corrida parou assim:

> `Summary [21s] 10233/11240 tests run: 10232 passed, 1 failed`
> `warning: 1007/11240 tests were not run due to test failure`

O ✗ era uma **flake** de relógio, numa crate que a linha não tocava. Se eu tivesse
parado ali — «um só ✗, e é flake conhecida, verde» — teria fechado a linha sobre
**1.007 testes que nunca correram**. A re-corrida com `--no-fail-fast` deu **17.865**
testes e **outra** falha, em crate diferente, que a primeira nunca chegou a ver.

**Why:** a lista de flakes conhecidas cria a armadilha. Quando o único ✗ é um nome
que já está registado, o instinto é riscá-lo e seguir — e é exactamente aí que o
fail-fast cobra: o teste seguinte ao ✗ não é *"mais um"*, são **todos os que
faltavam**. ⚠️ Uma suíte com flakes conhecidas e fail-fast ligado **não é uma
medição, é uma amostra que pára no primeiro ruído**.

**How to apply:**
1. O gate batched de fecho corre **sempre** com `--no-fail-fast`:
   `CARGO_INCREMENTAL=0 cargo nextest run --workspace --cargo-profile ci-test --no-fail-fast`
2. Antes de atribuir um ✗ a uma flake, **leia o `X/Y tests run`**. Se `X < Y`, a
   corrida não terminou e o veredito ainda não existe.
3. Só depois re-corra o suspeito sozinho (o protocolo do `CLAUDE.md` §5.0) — e
   registe ali toda flake nova, com o **número de corridas sozinho** ao lado.

*O `CLAUDE.md` §5.0 já mandava usar `--no-fail-fast` para os gates de GPU, e a razão
é a mesma; o que faltava era a frase valer para a suíte inteira.*
