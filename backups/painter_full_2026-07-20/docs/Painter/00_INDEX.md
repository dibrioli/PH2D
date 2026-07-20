# Novo Painter — Índice (clean-room port do Blender Texture Paint)

> **Objetivo (Enio, 2026-06-20):** reimplementar a pintura raster do PH2D **clean-room** a partir
> do comportamento do Blender Texture Paint, plugando no **host de Layers + Efeitos já existente**
> ([ADR-0099](../architecture/decisions/0099-remove-painting-brush-engine-preserve-layers-effects.md)).
> Pintar o algoritmo, **não** copiar o código GPL (PH2D é proprietário). Ver §1 do doc de arquitetura.

## Leia nesta ordem

| Doc | O que tem |
|---|---|
| **[01_arquitetura_e_decisoes.md](01_arquitetura_e_decisoes.md)** | Mandato clean-room · mapa do host que já existe (file:line) · as 5 lacunas · **decisão do contrato de ponteiro** · layout de crates · triagem · espaço de cor · kill-criteria · DoD |
| **[02_plano_de_implementacao.md](02_plano_de_implementacao.md)** | **Passo a passo** — Fases 0→6, tasks T0.1…T6, owners, gates, seam tests |
| **[03_algoritmos_referencia_blender.md](03_algoritmos_referencia_blender.md)** | Algoritmos a portar (falloff, dab, spacing, pressão, undo) + mapa arquivo-Blender→alvo-PH2D |
| **[blender_ui_reference/](blender_ui_reference/)** | 17 telas do manual Blender (CC-BY-SA) + manifesto de UI |
| **[BUGS_painter.md](BUGS_painter.md)** | Log de bugs não-triviais (sintoma → causa-raiz → tentativas que falharam → solução). Bug #1: offset de quina (offset-then-trim / split côncavo) |

## Linha de trabalho: dois slots de textura (Shape + Grain) — paridade Procreate

> Pesquisa→design→plano (2026-06-24, a pedido do Enio). **Não implementado ainda** — aguarda aval.
> Checkpoint: tag `painter-pre-shape-grain-2026-06-24` @ `928bd303` + `backups/painter_2026-06-24/`.

| Doc | O que tem |
|---|---|
| **[HANDOFF_shape_grain_dual_texture.md](HANDOFF_shape_grain_dual_texture.md)** | A missão (research + plano de 2 slots) |
| **[04_pesquisa_shape_grain_procreate.md](04_pesquisa_shape_grain_procreate.md)** | Shape/Grain do Procreate (≥2 fontes/claim) + tabela de mapeamento + **decisão Shape substitui o falloff** |
| **[05_design_dois_slots_textura.md](05_design_dois_slots_textura.md)** | Arquitetura dos 2 slots (8 restrições reais), back-compat byte-idêntico + **ADR-0100** (esboço) |
| **[06_plano_dois_slots_textura.md](06_plano_dois_slots_textura.md)** | Waves W0→W5, cada uma com teste e2e; caminho crítico + MVP mínimo |

## Linha de trabalho: Rendering Modes + Wet Mix — paridade Procreate (Glaze / Blending / Wet Edges / Burnt Edges)

> Pesquisa→verificação adversarial→design→handoff (2026-06-26, a pedido do Enio). **Não implementado ainda** — aguarda aval.
> *Enabler:* um **stroke buffer** premultiplicado-linear por traço (composto 1× no pen-up). Default `RenderingMode::Direct` = pipeline atual **byte-idêntico**.
> *Interdependência (pergunta do Enio):* Rendering Mode ⇄ Wet Mix são **acoplados, sem gate duro** (Wet Mix é essencial só nos modos Blending). Detalhe em `07` §0–§1.

| Doc | O que tem |
|---|---|
| **[07_rendering_modes_wet_mix.md](07_rendering_modes_wet_mix.md)** | Design completo: §0 status de verificação · §2 as 6 features (math + mapeamento) · §3 stroke buffer · §4 Wet Mix · §5 Wet/Burnt Edges · §7 BrushSpec · §8 UI · §10 plano faseado · §11 testes |
| **[HANDOFF_rendering_modes_wet_mix.md](HANDOFF_rendering_modes_wet_mix.md)** | Roteiro operacional: checkpoint + rollout não-destrutivo (golden Direct) · ordem faseada · anchors `file:line` verificados · gotchas · aceite e2e |

## Linha de trabalho: Aquarela — avaliação vs padrão-ouro + plano (SEM física real)

> Estudo (2026-07-06, a pedido do Enio). **Nenhum código alterado** — diagnóstico + plano com alvos.

| Doc | O que tem |
|---|---|
| **[11_aquarela_avaliacao_padrao_ouro.md](11_aquarela_avaliacao_padrao_ouro.md)** | Veredito · modelo atual (file:line) · padrão-ouro Tier-1/Tier-2 (verificado) · **diagnóstico do bege** (papel virtual assado no bake) · **diagnóstico da rediluição** (referência errada + stateless) · plano F1 (backdrop real + campo Paper color) → F2 (charge/dilution/recentness) → F3 (escala S + bleed por permanência) → F4 (K–M opcional), cada fase com asserções-vermelhas · §5.1/§5.2 landing notes F1+F3+F2 |
| **[12_aquarela_auditoria_pos_f123_padrao_ouro.md](12_aquarela_auditoria_pos_f123_padrao_ouro.md)** | **Auditoria sistemática pós-F1/F2/F3 (2026-07-07)**: 6 lentes multi-agente c/ fontes EXTERNAS fetchadas + verificação adversarial — 28 achados priorizados (film BL=KM S=0 correto, fica; mistura naive-RGB no mixer/RYB = P1; espectro do papel medido por FFT = causa do "mottled"; backrun inalcançável; Charge sem depleção) · perf MEDIDA (claims doc 11 confirmados) · fences ratificadas · plano W-A..W-D |
| **[13_fila_integracao_watercolor_secoes.md](13_fila_integracao_watercolor_secoes.md)** | **Fila de integração painel ⟷ watercolor (2026-07-07)**: mapa do que flui/não flui pelo desvio em `stamp_dabs` · ✅ fix Seleção+proteção (3 camadas keyed em `splat_keep`) · **#1: Shape "Automatic"** (checkbox modo-aquarela; desmarcado abre Falloff novo "Watercolor" + Shape image + rotação) · Tiling, shape-editors, Blend/Composite (decisão), Jitter Rotate, alpha-lock |

## Fontes de referência (untracked no git — decisão de licença pendente)

- **Código:** [`reference/blender-texture-paint/`](../../reference/blender-texture-paint/) — recorte GPL do Texture Painter (Blender 5.2), só estudo.
- **UI:** [`blender_ui_reference/`](blender_ui_reference/) — imagens CC-BY-SA do manual.

## Resumo do plano (1 tela)

```
host PRONTO (ADR-0099): layer stack + compositor GPU 22-modos + adjustments + undo estrutural + Apply
                         + ph2d-painter-effects (blend GPL-free)  ← o brush SÓ escreve pixels no layer ativo

lacuna #1 (foundational): ponteiro de canvas NÃO chega ao tool  → FASE 0 (Coord + ADR amendment 0040; Tool 11→12)
lacuna #2: dab/stamp                                            → FASE 1 (crate nova ph2d-painter-brush)
lacuna #3: stroke (spacing/pressão/smooth)                      → FASE 2 (mesma crate)
lacuna #4: undo por-stroke (tiles)                              → FASE 3 (cola no ph2d-tool-painter)
lacuna #5: UI de brush                                          → FASE 4 (panel novo ph2d-panel-painter-brush)
extras (eraser/smudge/fill/clone/presets)                       → FASE 5 (fan-out)
dab GPU                                                          → FASE 6 (só se kill-criterion CPU disparar)
```

Novas crates: **`ph2d-painter-brush`** (engine pura) + **`ph2d-panel-painter-brush`** (UI). Modifica
**`ph2d-tool-painter`** (cola) e, **só na Fase 0**, o contrato `Tool` (Coord-only + ADR).

## Estado

- ✅ Referência de código + UI baixadas e isoladas (clean-room source map).
- ✅ Host auditado (file:line) — gaps identificados.
- ✅ Plano exaustivo escrito (este diretório).
- ✅ **Fase 1 — engine puro `ph2d-painter-brush`** (`BrushSpec` + 9 falloffs Blender + 24 blend modes Blender + `stamp_dab`). 20 testes de mesa, clippy limpo. Commit `fd06554e`.
- ✅ **Fase 2 — motor de stroke "Space" + dynamics de pressão** (spacing por arc-length, jitter determinístico). +9 testes (29 total), clippy limpo. Commit `31ad397b`. *(Smooth-stroke/airbrush deferidos — T2.3/T2.4.)*
- ✅ **Fase 0 — contrato de ponteiro** (Coord-only + ADR-0040 Amendment 3): sub-trait `CanvasPaintTool` + `as_canvas_paint_mut`, cap `Tool` 11→12 + `CanvasPaintTool ≤ 1`, gate verde + 2 unit tests. Commit `5704ca35`.
- ✅ **Fase 0b/3 (núcleo) — pinta fim-a-fim:**
  - Consumidor: `PainterTool impl CanvasPaintTool` (`tool/paint.rs`) — Down/Move/Up → `Stroke` → `stamp_dab` no `canvas_rgba` + dirty-rect + preview. **4 testes comportamentais white-box** (traço pinta linha; hover/stray-move não pintam). Commit `77d7014f` (+ default brush `9-?`).
  - Shell: `input_dispatch/painter_canvas_input.rs` — Down/Move/Up roteados, tela→imagem via footprint do sprite. Compila + clippy limpo. Commit `6fc18454`. **⏳ aguardando smoke do Enio** (DoD do wiring de shell).
- ✅ **Fase 3 (correção)** — undo de stroke por-traço (snapshot/commit) + **alpha-lock honrado** no dab; clip/máscara já são composite-time. Commit `340d1d5f`.
- ✅ **Fase 4 — UI de brush (size/cor/blend)** + atalhos de teclado:
  - **Teclado `[` / `]`** nudge o raio do brush ativo (passo multiplicativo, piso ±1px). Shell `painter_nudge_brush_size` (downcast gate) + handler em `keyboard.rs`.
  - **Brush section** no topo do painel de Layers: slider de **Size** (+ readout px), **swatch de cor ao vivo + sliders R/G/B**, e **dropdown de blend (24 modos `BrushBlend`)**. `paint_brush.rs` + popover deferido; eventos roteados antes do dispatch per-layer (`try_apply_brush_event`).
  - Engine: `BrushBlend::{ALL,from_u8,to_u8,name}`. Tool: `BrushSettings` snapshot + `set_brush_size_norm/size_px`, `nudge_brush_size`, `set_brush_color_channel`, `set_brush_blend`. Bridge publica `brush_settings()` por frame.
  - +96 testes (engine round-trip/nomes; seam tool panel-event dirige size/cor/blend + o traço resultante; nudge clampa). clippy limpo; gates LOC + tool-contract verdes. Commit `0a8bf3d4`.
- ⏳ **Pendências** — undo de stroke por TILES (hoje é snapshot full-canvas/traço); color picker rico (wheel/HSV — hoje RGB sliders); **Fase 5** brushes extras (eraser/smudge/fill/clone/presets). ⚠️ Layout da UI **precisa do smoke do Enio** (não enxergo o visual).

> **Estado:** o pincel pinta fim-a-fim, desfaz por-traço, honra alpha-lock, e agora tem UI de
> Size/Cor/Blend (painel) + `[` `]` (teclado). Próximo: **smoke do Enio** (layout/feel da Brush
> section + sliders), depois Fase 5 (brushes extras) ou color picker rico.
