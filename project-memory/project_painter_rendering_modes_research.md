---
name: project-painter-rendering-modes-research
description: "Painter Rendering Modes + Wet Mix (Procreate Glaze/Blending/Wet+Burnt Edges) — research+design done 2026-06-26, awaiting impl"
metadata: 
  node_type: memory
  type: project
  originSessionId: d9b55362-1ed6-4da3-8f39-9695c3b0d663
---

Enio pediu (2026-06-26) levar o Painter a "outro nível" com os Rendering Modes do Procreate: **Uniform/Intense Glaze, Uniform/Intense Blending, Wet Edges, Burnt Edges** + o painel **Wet Mix**. Pesquisa multi-agente + verificação adversarial → **design + handoff** escritos; **NÃO implementado ainda** (aguarda aval).

- **Docs entregues:** [`docs/Painter/07_rendering_modes_wet_mix.md`](design) + [`docs/Painter/HANDOFF_rendering_modes_wet_mix.md`](handoff). Indexados em `docs/Painter/00_INDEX.md`.
- **Interdependência (a pergunta central do Enio):** Rendering Mode ⇄ Wet Mix são **acoplados, sem gate liga/desliga duro**. Verbatim Handbook: *Intense Blending* "gives a full flow effect to the paint's Wet Mix". Wet Mix (smudge/mixer state) é **essencial só nos 2 modos Blending**; dispensável p/ Glaze e p/ Wet/Burnt Edges.
- **Enabler arquitetural:** um **stroke buffer** RGBA premultiplicado-**linear** por traço, composto **1× no pen-up**. Glaze = acumulação MAX (uniforme); Intense = additive; Blending = lê destino + lerp. Wet/Burnt Edges = `blur(α)` separável no finalize (`rim = max(0,α−blur(α))` + ColorBurn). **Sem re-sim de fluido** (ADR-0096).
- **Seed já no código:** o flag `accumulate` + `stroke_mask` (cap de cobertura por-traço, `dab.rs:532–541` / `paint.rs:201/287`) já é ~"Uniform Glaze" direto na camada — falta generalizar p/ buffer separado + composite-único.
- **Não-destrutivo:** default `RenderingMode::Direct` = pipeline atual **byte-idêntico** (golden hash test pina o caminho legado); tudo novo atrás do modo/flags; checkpoint = tag `painter-pre-rendering-modes` + backup + branch.
- **Cor:** sem Mixbox/K–M alcançável (só backup) → Blending usa **lerp RGB linear** no MVP; Mixbox residual é upgrade opcional (Fase 5).

Honestidade epistêmica registrada no doc §0: separa **verbatim-Handbook** (fato) de **inferência de engenharia** (recipe — Procreate é closed-source). Ver [[no-industrial-claims-without-verification]], [[feedback-perfection-no-deferrals]]. Engine não é contract-gateado ([[project-painter-brush-came-back-cleanroom]]).
