---
name: a-cut-moves-the-imports-too-relint-after-it
description: Um corte de LOC move código E os imports dele — a lint corrida ANTES do corte não vale; foi o build do dono que apanhou os órfãos
metadata:
  type: feedback
---

⛔⛔ **Um corte por responsabilidade move o CÓDIGO e deixa os `use` para trás — e a lint corrida
ANTES do corte não afirma nada sobre o ficheiro depois dele.**

Medido 2026-09-21 (`line/3DModeling`): o portão de fecho correu na ordem `clippy → varredura
impactada`, a varredura acusou três tectos de LOC, e a cura (corte para ficheiros irmãos) veio
**depois** do clippy. Resultado: `RigRaw` e `ShadeRaw` viajaram para o irmão e as duas linhas de
`use` ficaram órfãs no pai — e **quem as viu foi o build que o DONO correu**, no meio do smoke.

⚠️ **E há a forma inversa, mais subtil:** um `#[path]` filho escrito com `use super::CameraRaw`
resolve-se pelo **import do pai**, não pela casa do tipo. O pai fica com um import que só o filho
usa, e quem arrumar os imports do pai parte o filho — em silêncio para o `cargo check` do laço
interno, barulhento só num `-D warnings`.

**Why:** o tecto de LOC é medido pela varredura IMPACTADA, que corre depois do clippy no roteiro de
fecho; a cura dele é sempre um corte, e um corte é a única operação do portão que **cria trabalho
novo para uma etapa já passada**.

**How to apply:** depois de curar um tecto de LOC por corte, **re-corra o clippy `--all-targets -D
warnings` nas crates cortadas** — ele é barato e é o único que vê um `use` órfão. E num `#[path]`
filho, importe da **casa** do tipo (`crate::modulo::Tipo`), nunca por `super::` a atravessar o `use`
do pai.

Ver [[reference_topic_code_gotchas]] · [[reference_topic_gate_discipline]].
