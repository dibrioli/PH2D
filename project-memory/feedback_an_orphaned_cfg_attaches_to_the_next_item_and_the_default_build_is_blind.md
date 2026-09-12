---
name: an-orphaned-cfg-attaches-to-the-next-item-and-the-default-build-is-blind
description: "Apagar um item deixa o `#[cfg]` dele a gatear o VIZINHO — e com a feature ligada por omissão, nenhuma build que a CI corre o vê"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: cbcca673-89ac-4ca6-be35-ef2308fe97cb
  modified: 2026-09-12T03:12:49.104Z
---

Apagar uma declaração e esquecer o atributo em cima dela não dá erro: o `#[cfg]` cola-se ao item
**seguinte**. Se a feature está no `default`, o caminho de omissão — e o que a CI corre — fica
verde, e o defeito só existe numa configuração que ninguém compila.

Medido 2026-09-11 (W2/L3-B): apagar `pub(crate) mod sculpt3d_panel_bridge;` deixou o
`#[cfg(feature = "sculpt3d")]` a gatear o `mod timeline_bridge;` seguinte. **14 erros**, todos
invisíveis até uma build `--no-default-features`. A mesma build revelou ainda **2** chamadores
ungated de predicados gateados, pré-existentes.

⚠️ **É a mesma forma que a `line/app-physics` já tinha nomeado** — *«apagar um `mod` re-liga o
`#[cfg(test)]` dele ao vizinho, em silêncio»* — com `feature` no lugar de `test`.

**Why:** o custo de não a apanhar é um utilizador com a feature desligada a receber um binário
partido (ou, pior, um campo que desaparece e um save que grava vazio por cima da obra).

**How to apply:** ao apagar um item, apague **a prosa E o atributo** com ele. E corra uma vez
`cargo check --no-default-features --features <default menos a sua>` ao fechar a linha: é a única
lente que vê esta classe. ⛔ E quando 2 chamadores ficam ungated, a cura raramente é gateá-los —
é dar ao predicado um **gémeo neutro** `#[cfg(not(...))]`, senão toda chamada futura herda a
obrigação de se lembrar.
