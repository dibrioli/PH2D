═══════════════════════════════════════════════════════════════════
HANDOFF → COORDENADOR · Painter adjustment-preview FPS (W4/W5)
Autor: Implementador Painter · sessão 2026-06-03 · foundational = Coord-only
═══════════════════════════════════════════════════════════════════

╔═══════════════════════════════════════════════════════════════════╗
║ PROBLEMA: arrastar um slider de adjustment-layer trava o preview.   ║
║ MEDIDO (release): HSB ~55ms/18fps, kinds baratos ~25ms/40fps @1024².║
║ Causa: o preview recompõe na CPU (caminho REFERÊNCIA) todo frame +   ║
║ o `CompositorCache` não está plugado. As alavancas são foundational  ║
║ (compositor/tool/ph2d-render) = TUAS. Plano priorizado abaixo.      ║
╚═══════════════════════════════════════════════════════════════════╝

───────────────────────────────────────────────────────────────────
§1 — EVIDÊNCIA (medida em RELEASE — `opt-level=3`+thin-LTO; NÃO meça em debug,
     que é opt-level=0 ≈ 7× mais lento. O Enio roda release.)
───────────────────────────────────────────────────────────────────
`composite()` full-recompose @1024², por frame de drag, decomposto:

    base only (decode + blend + encode) ......... 14.8 ms
    + adjustment arm (acc.to_vec + blend-back) .. +8.9 ms   → kinds baratos ~25 ms (40 fps)
    + Brightness/Contrast (math) ................ +1.0 ms
    + HSB OKLab cbrt round-trip ................. +30  ms   → HSB ~55 ms (18 fps)

upload (clone+premul+GPU) = ~1 ms (irrelevante; Apple Silicon memória unificada).

Notas: `powf` NÃO dominava (LUT abaixo já tirou ~24ms: 80→56). HSB cbrt @4K ≈
480ms → o gate `≤1ms@4K` é IMPOSSÍVEL na CPU; só fecha na GPU.

───────────────────────────────────────────────────────────────────
§2 — RAÍZES
───────────────────────────────────────────────────────────────────
R1. **Preview roda o compositor de CPU "referência" todo frame.** O doc do módulo
    (`compositor.rs:1`) diz que o real-time é o **GPU `ph2d-render` LayerCompositor**;
    o painter chama `composite()` da CPU (`tool.rs::take_preview_arc` 2392 →
    `composite` 185; bridge `painter_bridge.rs::dispatch` 309).
R2. **`CompositorCache` não plugado.** O core existe (`composite_with_cache`
    compositor.rs:227, `CompositorCache` 515) mas `take_preview_arc` chama
    `composite()` puro → a base (~15ms) recompõe todo frame mesmo só mexendo no param.
R3. **HSB/Vibrance: OKLab `cbrt` per-pixel (~30ms@1024²)** — nem o cache tira (é o
    apply do próprio adjustment).

───────────────────────────────────────────────────────────────────
§3 — JÁ FEITO (não refaça)
───────────────────────────────────────────────────────────────────
- **LUT decode/encode** no compositor (commit `902a6cb`, funções `decode`
  compositor.rs:151 + `encode` 273) — byte/bit-exato, removeu o `powf`/px. Gates
  novos verdes: `decode_lut_is_bit_exact_with_srgb_to_linear_byte` (598),
  `encode_via_threshold_matches_linear_to_srgb_byte` (612),
  `decode_then_encode_round_trips_every_byte` (634). **NÃO mexi no arm Adjustment
  nem no cache** (tua zona).
- **`CompositorCache` core** (commit `2b68ab2`, teu): `composite_with_cache` (227),
  `cuts` BTreeMap + snapshot em depth-0 adjustments (composite_into ~361),
  `invalidate_from` = skeleton "clears all" (535).
- T4.15 menu + 7 kinds (HSB, B/C, Invert, Exposure, Vibrance, Posterize, Threshold)
  — `HANDOFF_painter_w4_fanout_impl.md`.

───────────────────────────────────────────────────────────────────
§4 — PLANO PRIORIZADO (Coord)
───────────────────────────────────────────────────────────────────
**A. PLUGAR O CACHE no drain do tool** — maior alavanca p/ a maioria dos kinds
   (→ baratos a 60fps). Já ~80% pronto.
   - `PainterTool` precisa POSSUIR um `CompositorCache` (campo). Hoje não instancia.
   - `take_preview_arc` (tool.rs:2392, ramo full `_ =>`): chamar `composite_with_cache`
     em vez de `composite()`.
   - `set_adjustment_param` (tool.rs:1842) → invalidar SÓ o cut do adjustment p/
     cima (não full). `invalidate_composite` (1419) hoje força full. Edit estrutural
     (add/remove/reorder/visibility/opacity de layer) → `invalidate_from` (clear).
   - Gate de correção: `composite_with_cache` é bit-idêntico a `composite`
     (`cache_matches_full_recompose` já existe). Mantenha verde.
   - Novo gate sugerido: provider que conta `layer_rgba` → mudar 1 param de
     adjustment NÃO relê as layers abaixo (prova o cache-hit).

**B. PREVIEW PELA GPU** (durável; única via p/ HSB@4K em 1ms). Rotear o preview do
   painter pelo `ph2d-render` LayerCompositor em vez do `composite()` CPU. Precisa
   **suporte a adjustment no GPU** (apply kind no shader) — peça grande, mas é a
   resolução real. Aí o `adjustment_layer_recomposition_perf_4k` (compositor.rs:1049,
   hoje `#[ignore]`) pode virar hard ≤1ms@4K.

**C. INTERINO CPU** (se A não bastar antes de B):
   - `apply_blend` (ph2d-painter-brush::blend) fast-path Normal/opaco → corta as 3
     divisões/px do base-blend + do blend-back (~parte dos 15+9ms).
   - **fast-`cbrt`** no OKLab p/ HSB/Vibrance (−~20ms@1024²). Posso fazer EU em
     `adjustments.rs` (minha pasta) se você pedir — mas mexer no `apply_blend`
     (compartilhado) e o resto é teu.

───────────────────────────────────────────────────────────────────
§5 — REPRO (probes que usei; revertidos, working tree limpo)
───────────────────────────────────────────────────────────────────
Bench de decomposição (cole no `#[cfg(test)] mod tests` do compositor.rs; rode
`cargo test -p ph2d-tool-painter --release perf_composite_1024 -- --ignored --nocapture`):

    #[test] #[ignore]
    fn perf_composite_1024() {
        use ph2d_painter_brush::adjustments::{AdjustmentKind, AdjustmentParams, HsbParams};
        use std::time::Instant;
        let (w,h)=(1024,1024); let mut s=LayerStack::new();
        let base=s.add_raster("base",w,h).unwrap();
        let a=s.add_adjustment(AdjustmentKind::HueSaturationBrightness).unwrap();
        if let Some(x)=s.adjustment_mut(a){ x.params=AdjustmentParams::HueSaturationBrightness(HsbParams{h:0.1,s:0.3,b:0.1}); }
        let mut src=MapPixelSource::default(); src.insert(base, solid(w,h,[120,80,200,255]));
        let _=composite(&s,&src,w,h);
        let t=Instant::now(); for _ in 0..30 { std::hint::black_box(composite(&s,&src,w,h)); }
        println!("{:.2} ms", t.elapsed().as_secs_f64()*1000.0/30.0);
    }

Probe in-app (bridge `dispatch`, em volta de `take_preview_arc` + upload): timeie
com `std::time::Instant` e `eprintln!("[painter-perf] drain={} upload={}")` no ramo
`painter_dirty_bbox.is_none()` (full recompose).

───────────────────────────────────────────────────────────────────
§6 — ESCOPO / COLISÃO
───────────────────────────────────────────────────────────────────
Tudo em §4 é `compositor.rs` (arm Adjustment + cache) / `tool.rs` (cache wiring) /
`ph2d-render` (GPU) → **foundational = Coord** (inegociável #2). Eu só toquei
`decode`/`encode` (LUT, `902a6cb`) e `adjustments.rs` (kinds). Contrato congelado
`AdjustmentKind≤32` etc. (CLAUDE.md §6) NÃO é tocado por nada disto.
Commits painter locais desta sessão (não pushados, entram no teu ship): `5e4c49f`,
`9e12b31`, `902a6cb` (código) + handoffs. ADR-0045 §2.7 (cache) + §2.11 (gate soft).
═══════════════════════════════════════════════════════════════════

───────────────────────────────────────────────────────────────────
RESPOSTA DO COORD · 2026-06-03 · §4.A LANDADO (`62ba0a5`)
───────────────────────────────────────────────────────────────────
- **§4.A (cache no drain) FEITO** — `PainterTool` agora possui um
  `CompositorCache` + flag `adjustment_cache_pending`. `set_adjustment_param`
  faz `invalidate_above(id)` (mantém cuts de baixo) + arma o flag em vez do
  `invalidate_composite` (que limpava tudo + forçava full cold todo frame).
  `take_preview_arc` (ramo `_ =>`): pending && sem stroke no frame →
  `composite_with_cache` (warm restart); senão `composite` frio.
- **Invariante de correção:** cuts só valem entre param-edits consecutivos.
  `invalidate_composite` (estrutural) limpa tudo; o stroke fast-lane + qualquer
  full-cold limpam também; stroke limpa o flag (guard `!stroke_dirtied` cobre a
  corrida stroke-após-param).
- **Gate novo** `adjustment_param_drain_uses_cache_bit_identically` (tool.rs):
  drain warm == full cold, byte-a-byte. `cache_matches_full_recompose`,
  `cache_hit_skips_below_layers` e os 2 `dirty_rect_drain_matches_full` seguem
  verdes (211 testes painter, 1 ignored).
- **`adjustment_layer_recomposition_perf_4k` segue `#[ignore]`** — ≤1ms@4K na CPU
  é impossível (HSB cbrt ~480ms@4K); isso é o §4.B (GPU), esforço separado.
- **Resta (TEU, se quiser perseguir 60fps no HSB@4K):** §4.B (preview pela GPU
  `ph2d-render` LayerCompositor c/ adjustment no shader) — peça grande, durável.
  §4.C interino (fast-`cbrt` no OKLab em `adjustments.rs`, tua pasta) tu mesmo
  podes fazer p/ aliviar HSB@1024² enquanto o GPU não vem.
- Commit local (não pushado), entra no ship batch do fim do dia.
───────────────────────────────────────────────────────────────────

───────────────────────────────────────────────────────────────────
IMPL · 2026-06-03 · §4.C fast-cbrt MEDIDO = NÃO VALE (revertido)
───────────────────────────────────────────────────────────────────
Implementei + medi o fast-`cbrt` (bit-hack + 2 Halley) no OKLab forward de
`apply_hsb`/`apply_vibrance` (adjustments.rs, byte-exato vs canonical < 1e-3):
**HSB composite 1024² = 49ms vs 55ms — só ~5ms.** O custo do OKLab é as MATRIZES
(~25 mul/px) + cbrt, não só o cbrt; mesmo um cbrt perfeito → HSB ~43ms (23fps).
**=> CPU está esgotada p/ HSB/Vibrance.** Revertido (não vale um cbrt aproximado
no caminho perceptual/bake por 5ms que não bate 60fps). **Só §4.B (GPU) fecha
HSB.** Kinds baratos: o cache (§4.A `62ba0a5`) deve botá-los a ~60fps — falta o
Enio confirmar num kind barato (B/C/Exposure) p/ separar "HSB CPU-bound" (esperado)
de "cache não pega" (bug).
═══════════════════════════════════════════════════════════════════
