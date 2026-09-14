---
name: cargo-check-never-evaluates-an-inline-const-assert
description: um `const { assert!(…) }` dentro de uma função fecha VERDE no `cargo check --all-targets` e só o build devolve o E0080 — o laço interno é cego a ele
metadata:
  type: feedback
---

Medido em 2026-09-14 (`line/sculpt3d`). O `clippy` acusa
`assertions_on_constants` quando os dois lados de um `assert!` são constantes, e
a cura certa é `const { assert!(…) }` — ela é **mais forte**, porque passa de
reprovar numa corrida a reprovar na **compilação**.

⚠️ **Só que o `cargo check` não a avalia.** Com o limiar mutado para um valor
impossível:

| comando | veredito |
|---|---|
| `cargo check -p <crate> --all-targets` | ✅ **verde** |
| `cargo test -p <crate> --lib` | ❌ `E0080: evaluation of … failed here` |

*Um `const` de dentro de uma função só é avaliado quando essa função é
CONSTRUÍDA*, e o `check` pára na metadata.

**Why:** o laço interno deste repo é `cargo check -p` por lei (§2), e quem
escrever a asserção vai vê-la verde nas dezenas de `check` que corre entre
edições — a reprova só aparece no portão de fecho, a horas de distância. É o
mesmo ponto cego de família do `--bins` que não alcança `tests/` e do `check
--all-targets` que não compila os testes de uma **dependência**: cada um tem um
`cargo` que o vê, e nenhum é o do laço interno.

**How to apply:** ao converter um `assert!` de teste num `const { assert!(…) }`,
**prove-o com `cargo test`**, nunca com `check` — e escreva a cegueira ao lado,
senão o próximo a mexer no número acredita no verde. A mensagem também perde a
formatação (um `const` não formata argumentos): ou o número sai do texto, ou a
asserção fica a ser de corrida. Irmã de
[[a-bins-run-never-reaches-the-gates-that-live-in-tests]] e de
[[the-inner-loop-check-script-is-blind-to-its-own-integration-tests]].
