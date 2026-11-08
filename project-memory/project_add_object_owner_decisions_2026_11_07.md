---
name: project-add-object-owner-decisions-2026-11-07
description: Dono 07/11 — Add Object versão B da Crítica, Sprite pré-selecionada com dica embaixo, 1 clique destaca, 2 cliques criam e fecham; nome «Paint a new image…» fica; Physics Body e Ambient Light saem
metadata:
  type: project
---

Dono, 07/11, sobre o painel Add Object (Crítica `#addobject`, `GameEngineSamples/critica_ph2d.html`):

1. **Versão B** (a mesma janela, grupos Pictures · Drawing · Game · Empty, Recent no topo, dica em
   todas, busca que não acha ensina a montar, ex. «coin»), **com uma mudança dele**: ao abrir, a
   **Sprite vem destacada** e a dica dela aparece **embaixo, na própria janela**. A dica **deixa de ser
   no hover**: **um clique destaca** a entrada e mostra a dica; **dois cliques rápidos** criam o objeto
   e **fecham** a janela.
2. O nome **«Paint a new image…»** fica (a Crítica viu o antigo «Image…»; já tinha sido trocado).
   Ctrl+N continua.
3. **Physics Body e Ambient Light saem do Add.** Physics Body mora no Add Component; Ambient = Light +
   `Type: Ambient` no cartão (o seletor `Point · Ambient` já existia, `luz_edits::TIPOS`).

**Why:** produto é do dono; a Crítica mediu que o problema de hoje é mais de explicar que de tamanho.
**How to apply:** plano em `docs/GameEngineFuncional/24_add_object_plano.md`; o painel de arrastar
da versão C fica para depois. Ver [[project-game-engine-refounded-item-by-item-2026-11-05]].
