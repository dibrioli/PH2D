---
name: project_painter_brush_came_back_cleanroom
description: "A pintura VOLTOU — ph2d-painter-brush é reimplemento clean-room do Blender (ativo); docs/memórias que dizem \"deletada\" estão stale"
metadata: 
  node_type: memory
  type: project
  originSessionId: 8280ce26-ee54-41a2-9ecd-718f47b3fe33
---

A PINTURA está VIVA e em dev ativo, apesar de CLAUDE.md §5 e [[project_painting_removed_layers_effects_kept]] dizerem que foi deletada (ADR-0099, 2026-06-20).

Ground-truth (2026-06-24): `crates/ph2d-painter-brush` existe — um **reimplemento clean-room do Blender Texture Paint** (engine NOVO, não o que ADR-0099 deletou; o próprio `lib.rs` afirma isso). Dirigido por `ph2d-tool-painter` (host) via `CanvasPaintTool::on_canvas_pointer`. Painel `ph2d-panel-painter-layers` hospeda as seções Brush/Stroke/Texture + a layer stack. Plano vivo: `docs/Painter/`. Ref Blender vendored: `reference/blender-texture-paint/` (GPL → clean-room, só comportamento).

Git confirma: commits recentes `0d316ce6`→`dbfe9848` são todos `feat(painter)` (Texture Layer, Color Ramp, per-dab jitter). Em 2026-06-24 landou per-dab **Randomize Color + Jitter Scale + Jitter Rotate** (módulo `jitter.rs` no brush, splitmix64 + HSV transcendental-free; `docs/HANDOFF_brush_jitter_color.md`).

**Why:** a nota "deletada" custou ~10min de investigação no início da sessão (o handoff mandava editar `ph2d-painter-brush`, que a memória dizia removida). O repo é a fonte de verdade, não a memória.

**How to apply:** ao tocar Painter/brush, confie no repo (`ls crates/ph2d-painter-*`, git log) antes da memória/CLAUDE.md. As memórias de brush marcadas "HISTÓRICO" descrevem o engine ANTIGO (Procreate-parity, ADR-0097) — o atual é Blender-parity, modelo diferente. `ph2d-painter-brush` NÃO é contract-gateado (refactor livre, respeitando LOC cap 600 + sweep de transcendentais HR-5).
