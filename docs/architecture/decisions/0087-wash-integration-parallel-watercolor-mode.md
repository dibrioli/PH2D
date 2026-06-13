# ADR-0087 — Integração do `ph2d-painter-wash` como modo de aquarela PARALELO (lado-a-lado com o v2)

**Status:** PROPOSTO (aguarda ratificação do Enio antes de codar) · **Data:** 2026-06-13
**Depende de:** [ADR-0086](0086-watercolor-minimal-core-wash.md) (o núcleo mínimo). **Não
supersede nada** — o caminho fluid v2 (ADR-0078..0085) fica **100% intacto**.

> **Decisão:** plugar o `WashSolver` no app como um **segundo modo de aquarela**, selecionado
> por um bool novo `wash_enabled`, dirigido por um **bridge irmão** (`painter_wash_bridge.rs`)
> que **copia a forma** do `drive_fluid_gpu` mas roda contra a sua própria sessão e crate.
> Mutuamente exclusivo com o fluid v2. **Zero edição no caminho v2.** Assim o Enio pinta com
> os dois e compara lado-a-lado.

---

## 1. Como o v2 se integra hoje (a forma que vamos espelhar)

(Mapa verificado, file:line no apêndice.) O fluid v2 **não** é um `RenderingMode` — é um
**bool** `brush.rendering.fluid_enabled`. O shell chama **um ponto de entrada por frame**,
`painter_fluid_bridge::drive_fluid_gpu(...) -> Option<PreviewOverride>`, que é dono de um
`thread_local! SESSION: RefCell<Option<FluidSession>>` (solver + compositor + slot de preview).
O resultado é um `PreviewOverride { entity_bits, texture_id, premultiplied }` mesclado numa
cadeia `.or()` em `mod.rs`, fazendo o sprite renderer amostrar uma textura own-by-fluid em vez
do sprite fonte. O step + composição acontecem em **single-submit** no hot-path
(`encode_single_submit_frame`). O heartbeat keep-wet é "de graça": o bridge roda **todo frame**
(mesmo sem dabs), então a sim segue bloom/secando até o dry-check derrubar o campo.

## 2. A forma paralela (o que vamos construir)

Um clone isolado dessa espinha, contra a crate nova:

```
seleção:   brush.rendering.wash_enabled (bool)  +  PainterUiEdit::Wash
tool:      wash_brush_enabled() · wash_params() · (reusa fluid_take_dabs + fluid_backdrop)
crate:     WashSolver::encode_step (single-submit) + WashCompositor (preview_tex canvas-res)
shell:     painter_wash_bridge.rs { thread_local WASH_SESSION; drive_wash_gpu(...) }
wiring:    mod.rs → .or(wash_override), ao lado de drive_fluid_gpu, mutuamente exclusivo
```

**Princípio de isolamento:** nenhuma função do v2 é tocada. Reusamos só **helpers genéricos**
(`ensure_preview_slot`, `copy_preview_into_slot`, `union_bbox`, `grow_bbox` de
`painter_fluid_support.rs`) e tipos neutros (`FluidDab` = geometria+cor+massa, `fluid_backdrop`
= snapshot do canvas). Se algum helper for `pub(super)` privado demais, promovo a um
`painter_paint_support` compartilhado (refactor mecânico, sem mudar o v2).

## 3. Seleção (brush + tool + UI) — cabe nos caps congelados

- **`RenderingParams.wash_enabled: bool`** — hoje 13 campos, cap **≤14** → cabe (13→14).
- **`PainterUiEdit::Wash(bool)`** — hoje 21 variantes, cap **≤24** → cabe (21→22). Aplicado em
  `trait_impls.rs` espelhando `P::Fluid`.
- **Mutuamente exclusivo:** ligar `wash` desliga `fluid` (e vice-versa) no handler de aplicação
  — um nunca roda com o outro. O shell chama `drive_wash_gpu` primeiro; se ativo, pula
  `drive_fluid_gpu`.
- **Tool:** `wash_brush_enabled() -> bool` (espelha `fluid_brush_enabled`).
- **UI:** um toggle "Wash (mini)" no sidebar/brush-studio ao lado do "Fluid", emitindo
  `PainterUiEdit::Wash`. (Reusa o padrão de toggle existente; sem novo widget.)
- **NÃO** mexe em `RenderingMode` (exato 6, gate trava), nem `Tool`/`PanelEvent` (congelados).

## 4. Parâmetros — reusa a UI da aquarela, zero campo novo de artista

O wash usa só ~4 dos 21 controles. Em vez de criar `WashControls` (gastaria a última vaga de
`RenderingParams` e um cap novo), o tool **mapeia o subconjunto** do `WatercolorParams`
existente → `WashParams`:

| Controle existente | → WashParams |
|---|---|
| Diffusivity | `diffusivity` (clamp ≤0.25) |
| Bleed | `flow_outward` |
| Evaporation | `evaporation` |
| Wet Gate Lo/Hi | `w_lo`/`w_hi` |

Os outros ~17 controles (capillary, velocity, lift, branching, K–M…) são **ignorados** pelo
wash. `wash_params() -> WashParams` lê esse subconjunto. **`WatercolorParams` fica intacto**
(cap 21 não muda). *(Se depois quisermos esconder os controles irrelevantes quando wash está
ligado, é um filtro de UI — fase 3, opcional.)*

## 5. Dabs — reusa o pipeline do tool, converte cor→absorbância no bridge

- Reusa `FluidDab { cx,cy,r,water,color:[f32;3],mass,staining }` + `fluid_take_dabs()` +
  `wet_pigment_envelope` (já produzidos no stamp loop). O wash ignora `staining`.
- No bridge, `FluidDab → wash::Dab { cx,cy,r, water_add, pig:(absorb.rgb, mass) }`:
  **Beer–Lambert** — um pigmento de cor (refletância) `c` tem absorbância `a = −ln(clamp(c,
  0.02, 1))` por canal. `Dab.pig = (a·deposit, deposit)`, `water_add = water`. Composição
  `exp(−Σabsorb)` reproduz a cor sobre branco; empilhar escurece (absorbâncias somam) —
  subtrativo "muddy" (limitação RGB conhecida, ADR-0086 §5; K–M é add-on).

## 6. Adições na crate `ph2d-painter-wash` (o gap entre teste e app)

Hoje o `WashSolver` compõe num buffer `u32` grid-res com fundo branco (ótimo p/ teste, não p/
o app). Adicionar:

1. **`WashCompositor`** (espelha o contrato do `FluidCompositor`):
   - `begin_stroke(device, gw, gh, cw, ch, scale, backdrop_rgba, coverage_k)` — aloca
     `preview_tex` **Rgba8Unorm** canvas-res (`STORAGE|TEXTURE|COPY_SRC|COPY_DST`).
   - `cs_composite` canvas-res: amostra **bicúbica** do campo de pigmento (grid→pixel),
     `rgb = backdrop · exp(−absorb)`, **premultiplica** e escreve no `preview_tex` (mesmo
     contrato byte do `cs_premul_tex` v2 → paridade grátis com o sprite shader).
   - `encode_frame_to_texture(queue, &mut enc, region)` + `preview_texture() -> &wgpu::Texture`.
2. **`WashSolver::encode_step(device, queue, enc, dabs, substeps, region) -> enc`** —
   single-submit: `cs_splat(dabs)` + `substeps × cs_step` (region-scoped) + normalização p/ `*_a`.
   (Sync wrapper `step_resident` p/ o lane não-hot.)
3. **Region-scope no `cs_splat`** (hoje full-grid) — barato, mas alinha com o region do step.
4. **`WashParams.coverage_k`** já existe no composite; expor no `set_params`.

Tudo isso é **interno à crate nova** (sem superfície congelada). Teste headless novo:
`wash_preview_texture_premul` (compõe num `preview_tex` e confere premul + Beer–Lambert).

## 7. Shell — `painter_wash_bridge.rs` + wiring

- `thread_local! WASH_SESSION: RefCell<Option<WashSession>>` { WashSolver, WashCompositor,
  dims, epoch, preview_slot, wet_bbox, idle_frames }.
- `drive_wash_gpu(tools, gpu, renderer, override_entity) -> Option<PreviewOverride>`: copia o
  laço do `drive_fluid_gpu` — gate por `wash_brush_enabled()`+capable (senão dropa sessão e
  retorna `None`); rebuild on dims-change; per-stroke reset + `begin_stroke(backdrop)`;
  `set_params(wash_params())`; drena dabs + region (`grow_bbox`/`union_bbox`); single-submit
  (`encode_step` + `encode_frame_to_texture` + slot copy + `submit`); retorna `PreviewOverride`.
- `mod.rs`: `let wash = drive_wash_gpu(...); ... .or(wash).or(fluid_override)...` e pular o fluid
  quando wash ativo.

## 8. Contratos tocados (mínimo) + deps

- `+1` bool em `RenderingParams` (13→14, dentro do cap) · `+1` `PainterUiEdit::Wash` (21→22) —
  atualizar os comentários dos gates `painter_ui_edit_variant_count_is_capped` e o sub-cap de
  `RenderingParams`. **Sem bump de cap.**
- **Sem** mudança em `RenderingMode`/`Tool`/`PanelEvent`/`WatercolorParams`/UBO do v2.
- **Deps novas:** `ph2d-tool-painter` e `shells/desktop` passam a depender de
  `ph2d-painter-wash` (com feature `gpu` ligada pelo shell, como faz com `ph2d-painter-fluid/fluid`).

## 9. Fases (cada uma revisável + testável; commits locais)

1. **Crate** (`ph2d-painter-wash`): `WashCompositor` + `encode_step` single-submit + dab
   cor→absorbância + teste headless `wash_preview_texture_premul`. **Não toca o app.**
2. **Brush+tool:** `wash_enabled` + `PainterUiEdit::Wash` + `wash_brush_enabled` + `wash_params`
   + reuso do dab-drain + atualizar gate de contrato. (`cargo check` + gate verde.)
3. **Shell:** `painter_wash_bridge.rs` + wiring em `mod.rs` + toggle de UI.
4. **Smoke do Enio:** pinta com Wash, compara com Fluid lado-a-lado.

## 10. Riscos / notas

- O hot-path single-submit + o slot de preview são o pulo do gato do v2; replicá-los dá ao wash
  a mesma latência baixa. Se algum helper de `painter_fluid_support` for privado, promovê-lo é
  refactor mecânico (não muda comportamento v2).
- Mutual-exclusão é a única regra de coerência; o resto é aditivo.

---

*Apêndice (pontos de integração v2, verificados):* solver `solver.rs:287`/`:414`; sessão
`painter_fluid_support.rs:292`/`:346`; thread-local `painter_fluid_bridge.rs:70`; entry
`drive_fluid_gpu` `painter_fluid_bridge.rs:138`; single-submit `painter_fluid_drive.rs:140`;
preview_tex `composite.rs:357`/`preview_texture()` `:821`; slot `painter_fluid_support.rs:112`;
seleção `fluid_enabled` `rendering.rs:43` / `fluid_brush_enabled` `lifecycle.rs:456`; dabs
`mod.rs:272` (`FluidDab`) / `lifecycle.rs:793` (produção) / `:628` (drain) / `DabGpu`
`solver.rs:138`; params `watercolor.rs:352` (`to_diffusion`) / `lifecycle.rs:523`
(`fluid_diffusion_params`) / `solver.rs:1288` (`set_from_diffusion`); backdrop `lifecycle.rs:591`;
heartbeat `mod.rs:261/277`; gates `architecture_painter_contract_surface.rs:308` (PainterUiEdit
≤24) / `:371` (RenderingParams ≤14) / `:478` (RenderingMode exato 6).
