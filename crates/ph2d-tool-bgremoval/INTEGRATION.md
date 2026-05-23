# Background Removal — Plano de Integração

> **NOTA (2026-05-20):** o backend **GrabCut / "Smart Cut" foi removido**
> a pedido do Enio (sliders engasgavam; Chroma é o único backend). Tudo
> abaixo que menciona GrabCut / Smart Cut / `BgRemovalMode` / o módulo
> `algorithm::grabcut` / Mode radio / tabs é **histórico** — o pipeline
> atual é Chroma+Flood → Guided Filter (opcional) → compose, mais o
> pincel de proteção (force-keep). Doc não reescrito por completo.

**Status:** Ilha isolada **pronta** — feature core completa, 4 stubs preenchidos pelo Coordenador, host wiring landado em 2026-05-16. 110+ tests verdes.
**Agente Implementador:** Slot #2 — slug `bgremoval`.
**Coordenador (integração):** sessão Coord 2026-05-16.

## 1. O que esta ilha entrega

Bg Removal stateful — Tool ativável via LeftRail com painel Procreate-style. Pipeline em 3 estágios independentes (todos puros, sem deps externas além de `image = "0.25"` default-features off + `rayon = "1"`):

| Estágio | Módulo | Função pública | Saída |
|---|---|---|---|
| Primary segmentation | `algorithm::chroma` | `segment(rgba, w, h, &ChromaParams, &mut Scratch)` | `scratch.mask` (0/255) + `scratch.delta_e` |
| Primary (alt) | `algorithm::grabcut` | `segment(rgba, w, h, &GrabCutParams, &mut Scratch)` | `scratch.mask` (0/255) |
| Refinement (optional) | `algorithm::guided_filter` | `refine(rgba, w, h, &GuidedFilterParams, &mut Scratch)` | `scratch.alpha_f32` |
| Compose | `algorithm::compose` | `write_output(rgba, w, h, &BgRemovalParams, &SegmentResult, did_refine, &mut Scratch)` | `scratch.output_rgba` |
| Orchestrator | `algorithm::run_pipeline` | `run_pipeline(rgba, w, h, &BgRemovalParams, &mut Scratch)` | `scratch.output_rgba` |
| Tool wrapper | `tool::BgRemovalTool` | `impl Tool`, `set_source_snapshot`, `take_pending_apply`, `run_full_resolution` | — |

Ilha **não toca:** *(seção original do briefing pré-ADR-0040; após T2 o feature
inteiro vive aqui em `crates/ph2d-tool-bgremoval/`. Mantido como histórico do
plano de absorção e desativado para futuras releituras.)*

## 2. UX-alvo (v1 — alinhado à Widget Gallery canônica §5.6)

LeftRail novo item entre Brush e Move. Painel Procreate-style com **uma linha de 5 widgets canônicos** (a fileira `controls` do `FloatingPanel`):

| # | Widget canônico | Função |
|--:|---|---|
| 1 | `RadioGroup<String>` | Mode — opções "Chroma" / "Smart Cut" |
| 2 | `Slider` | Tolerance (0..1 → param 0..0.30) |
| 3 | `Slider` | Feather (0..1 → param 0..0.20) |
| 4 | `Slider` | Refine Radius (0..1 → param 0..100 px) |
| 5 | `Toggle` | Apply (one-shot trigger — auto-resets) |

Cada widget vem do `crate::widget::*` (sem widget novo do zero). Tab única `[Chroma]` ou `[Smart Cut]` no topo refletindo o mode atual — apenas indicador visual, sem dispatch próprio.

**Params avançados intencionalmente ocultos do painel v1** (defaults aplicados):
- ColorSwatch override de bg color (auto-detect é default) — futuro.
- Toggles Despill / Color Guide / Boundary Only (defaults bons) — futuro.
- GrabCut Insets T/R/B/L + Iters — defaults 5% / 2 iter — futuro tab "Smart Cut".

(Mockup final do Claude Design ainda não chegou — esta é a estrutura mínima viável usando 100% widgets já canônicos.)

## 3. Contrato de wiring com o host

### 3.1 Trigger de Apply (via Toggle one-shot)

**UX wart documentado:** `PanelControl::Action(PanelAction)` é **paint-only** (sem `NodeId`, dispatcher não roteia click — vide `paint.rs:610-623`). Pra ter um botão Apply funcional no painel reutilizando widgets canônicos, uso `Toggle` como trigger one-shot:

- `build_panel()` sempre emite `Toggle::new(APPLY_NODE, "Apply")` com `on = false`.
- Click no Toggle gera `PanelEvent::Toggle(APPLY_NODE, true)` → Tool seta `pending_apply = true`.
- O `on = true` nunca é gravado no model do Tool, então o próximo `build_panel()` já volta com `on = false` e o visual reseta.

Limpeza futura sugerida ao Coord (não bloqueia esta entrega): adicionar `id: NodeId` em `PanelAction` (`floating_panel.rs`) + dispatcher para Action click → permite Apply via `PanelAction` semanticamente correto. Tudo isolado num PR pequeno do Coord, ortogonal a esta ilha.

Host drena:

```rust
if let Some(tool) = active_tool::<BgRemovalTool>() {
    if tool.take_pending_apply() {
        hero_screen.pending_bgremoval = Some(active_sprite_asset_id);
    }
}
```

### 3.2 Drain de `pending_bgremoval` (host frame loop)

Padrão idêntico ao `trim_transparency` (vide `main.rs:2131-2237`):

```rust
if let Some(asset_id) = hero_screen.pending_bgremoval.take() {
    // 1. Reach for the source RGBA (Atlas → asset_db, ou Individual → renderer.readback_individual).
    let (rgba, w, h) = sprite_source_to_rgba(asset_id, &asset_db, &renderer);

    // 2. Run the pipeline at full resolution.
    let mut out = Vec::new();
    let tool = active_tool_mut::<BgRemovalTool>().expect("tool removed mid-apply");
    tool.set_source_snapshot(rgba, w, h);  // (already pushed; this is a no-op refresh)
    let (out_w, out_h) = tool.run_full_resolution(&mut out);

    // 3. Acquire a new Individual texture with the output buffer.
    let new_texture_id = renderer.acquire_individual(out_w, out_h, &out);

    // 4. Swap Sprite.source = SpriteSource::Individual { texture_id: new_texture_id }.
    if let Some(mut sprite) = sim.world_mut().get_mut::<Sprite>(asset_to_entity(asset_id)) {
        sprite.source = SpriteSource::Individual { texture_id: new_texture_id };
    }
    // Note: no pivot reproject — bgremoval does NOT change image dimensions.
}
```

### 3.3 Snapshot push (selection change / tool activation)

Quando o active sprite muda OU `BgRemovalTool` torna-se ativo, o host injeta o RGBA atual no Tool:

```rust
let (rgba, w, h) = sprite_source_to_rgba(active_sprite_asset_id, &asset_db, &renderer);
active_tool_mut::<BgRemovalTool>().unwrap().set_source_snapshot(rgba, w, h);
```

O `set_source_snapshot` rebuilda o thumbnail e re-roda o preview com os params atuais.

## 4. Wiring (concluído 2026-05-16 pelo Coordenador)

- [x] `pub mod bgremoval;` em `crates/ph2d-editor/src/tools/mod.rs`.
- [x] Re-export de `BgRemovalTool` em `lib.rs` para consumo do shells/desktop.
- [x] HeroScreen: novo campo `pub pending_bgremoval: Option<u64>` (asset id token).
- [x] Variant `IconId::BgRemoval` em `crates/ph2d-editor/src/icons.rs` ligado ao `eraser_bezpath()` deste módulo.
- [x] LeftRail entry em `screens/hero/left_rail.rs` apontando para `ToolId::new("bgremoval")` + `IconId::BgRemoval`.
- [x] ToolRegistry register em `shells/desktop/src/main.rs`: `registry.register(Box::new(BgRemovalTool::default()));`.
- [x] Snapshot push handler: chama `tool.set_source_snapshot(rgba, w, h)` no `on_activate` + na transição de seleção ativa.
- [x] Drain handler do `pending_bgremoval` no main.rs (modelo trim_transparency §3.2 acima).
- [ ] Strings via `t!()` quando bundle Fluent existir (atualmente literal "Bg Removal" / "Tolerance" / etc. — HR-15 fallback documentado).

## 5. APIs públicas (para Coord referenciar)

- `BgRemovalTool::default() -> Self` — Tool inicial, sem source, params default.
- `BgRemovalTool::set_source_snapshot(rgba, w, h)` — host injeta source quando muda seleção / Tool ativa.
- `BgRemovalTool::has_source() -> bool`.
- `BgRemovalTool::preview_rgba() -> &[u8]` — 160×160 RGBA8, para painel paintar.
- `BgRemovalTool::take_pending_apply() -> bool` — drena flag Apply (true exatamente uma vez).
- `BgRemovalTool::run_full_resolution(&mut out: Vec<u8>) -> (w, h)` — host chama no drain do `pending_bgremoval`.
- `algorithm::run_pipeline(rgba, w, h, &BgRemovalParams, &mut BgRemovalScratch)` — fora do Tool, se o host precisar rodar sem instanciar Tool.
- `eraser_bezpath() -> kurbo::BezPath` — glyph para IconId.

## 6. Deps solicitadas ao Coordenador

| Dep | Status | Justificativa |
|---|---|---|
| `image = "0.25"` (default-features off) | ✅ commit `6d8e9f3` | `imageops::resize` no thumbnail downscale + Fast GF |
| `rayon = "1"` | ✅ commit `6d8e9f3` | parallel guided-filter passes |
| `THIRD_PARTY_LICENSES.md` na raiz | ✅ commit `dca3cd5` | Apache 2.0 + NOTICE OpenCV (para `maxflow.rs` derivado) |

## 7. Atribuição de código de terceiros

O arquivo `algorithm/maxflow.rs` (M2, ainda não escrito) é derivado de **OpenCV `modules/imgproc/src/grabcut.cpp` + `gcgraph.hpp`** (Apache License 2.0, © OpenCV contributors). Header SPDX-style obrigatório no topo do arquivo:

```rust
// Derived from OpenCV modules/imgproc/src/grabcut.cpp and
// modules/imgproc/src/gcgraph.hpp (Apache-2.0).
// © OpenCV contributors. See THIRD_PARTY_LICENSES.md.
```

Tabela `THIRD_PARTY_LICENSES.md` na raiz já lista a entrada para este arquivo.

## 8. Caso de teste manual após integração

1. Carregar sprite PNG com fundo uniforme (greenscreen, branco sólido, gradient suave).
2. Selecionar sprite no Hierarchy.
3. Clicar Bg Removal na LeftRail → painel abre, thumbnail mostra preview vivo.
4. Mexer slider Tolerance → preview atualiza imediatamente.
5. Trocar para Smart Cut tab → mexer Inset sliders → preview atualiza (mais lento, ~300-500 ms).
6. Clicar Apply → spinner brevemente, sprite no canvas perde o fundo.
7. Voltar a clicar Bg Removal (toggle off) → painel fecha.
8. Apply em sprite que estava em Atlas → vira Individual com texture nova (HR padrão Image Tools).
9. Sprite que já era Individual → vira Individual com texture nova também.

## 9. Status por método

- [x] **M1 — Chroma+Flood** — completo + auditado. 35 tests; Oklab + corner k-means + connected flood com border-bg fallback e alpha-aware sampling.
- [x] **M2 — GrabCut** — completo + auditado. 51 tests; downscale Triangle 1024² + iter loop com convergence early-exit, GMM 5-comp × 3×3, BK port (Apache 2.0 OpenCV) com EK oracle de validação.
- [x] **M3 — Guided Filter** — completo + auditado. 17 tests; vanilla gray-guide + separable box filter, var_I floor + ε clamp.
- [x] **Compose** — 5 tests (pós-audit final P0): path-1 refined, path-2 chroma soft band (ΔE² unit fix), path-3 grabcut binary com mode gate.
- [x] **Tool wrapper** — stubs `rebuild_thumbnail` (aspect-fit Triangle 160×160 letterbox) + `rerun_preview` (pipeline na thumbnail) preenchidos em 2026-05-16; eraser_bezpath real port Lucide v0.453.
- [x] **Host wiring** — `pub mod` + `IconId::BgRemoval` + `HeroScreen.pending_bgremoval` + LeftRail entry + ToolRegistry register + snapshot push em on_activate / selection-change + drain handler modelo trim_transparency.

## 10. Out of scope desta ilha (deliberado)

- Preview overlay sobre o sprite no canvas — exige hook canvas → tool que ainda não existe (§5.5 do contrato Agente Periférico).
- Sistema de undo — não existe no PH2D hoje; trim_transparency precedent confirma "novo asset, sem undo".
- Closed-form matting (Method 4) — entrega no Slot #2B futuro (~700 LOC).
- Plug-in de IA (BiRefNet etc.) — entrega futura, depende de infra de plug-in que não existe.
