═══════════════════════════════════════════════════════════════════
HANDOFF → Coordenador Vector · W10 variable fonts (Inovação P6, ADR-0066) — crate core fechada
Autor: Implementador Vector (jornada 2026-06-05)
═══════════════════════════════════════════════════════════════════

## §0 — TL;DR
1. **Inovação P6 (variable fonts) VIVA no core.** Crate nova **`ph2d-vector-font`** (ADR-0066): um glifo de
   variable font **É um `VectorNetwork` nativo**, com os eixos OTF (`wght`/`wdth`/`slnt`/…) expostos como
   parâmetros animáveis do graph. **20/20 testes verdes**, clippy zero warnings, **os 3 gates do ADR cobertos**.
2. **Provado contra fonte REAL:** smoke contra `InterVariable.ttf` (já vendorada em `ph2d-text/fonts/`):
   parse do eixo `wght`, glifo 'A' → `VectorNetwork` válido, e o killer **`weight_axis_changes_the_outline`**
   (weight 100 vs 900 → networks diferentes) — a tese da inovação provada: **o eixo dirige a geometria vetor**,
   não um raster.
3. **Drop-crate isolada (ADR-0075):** lê só contratos congelados (`ph2d-vector-doc`/`ph2d-vector-traits`).
   **skrifa não é dep nova** (já no workspace via parley/vello, 0.40, puro-Rust). Um glifo emite `VectorNetwork`
   standard → o renderer **já** desenha, **zero wiring novo**. O fast-path skrifa→Vello direto (§2.3) é
   otimização do renderer (TEU).

## §1 — O QUE LANDOU (`crates/ph2d-vector-font/`)
- **`axis.rs`** — `AxisTag` (tag OT 4-byte + consts wght/wdth/slnt/opsz/ital/GRAD), trait `VariableFontAxis`,
  `FontAxis` (set estrito que erra fora de range + `set_clamped` p/ animação + `normalized()` f2dot14).
- **`glyph_to_network.rs`** — o coração: `GlyphOutline`/`PathCommand` (boundary neutra, desacoplada de skrifa)
  → `VectorNetwork`. **Region por contorno fechado, NonZero** (resolve holes O/B/8); **quad→cubic exato**
  (`out=⅔(Q−S)`, `in=⅔(Q−E)`); **Y-flip** font(y-up)→screen(y-down); fusão do ponto de fechamento.
  [gate `variable_font_glyph_to_network_golden`].
- **`lib.rs`** — `GlyphVectorNetwork` (network + `GlyphId` próprio + `axes` SmallVec≤8 + `current_axis_values`
  BTreeMap determinista) + crate doc.
- **`axis_animation.rs`** — `VariableFontAxisCurve : AttributeEvaluator` (samplea curva → clamp ao range →
  `AnimValue::Float`); axis change = UBO update, zero recompile (§2.4). [gate `variable_font_axis_interpolation_smooth`].
- **`fallback_chain.rs`** — `PlatformHost` trait + `resolve_glyph_font` locale-aware (HR-15); `Locale`/`FontFamily`/
  `CoverageRanges` + MockHost (CJK roteia por locale: mesmo ideograma → JP vs SC). [gate `variable_font_fallback_chain_locale_aware`].
- **`skrifa_bridge.rs`** — ÚNICO módulo que toca skrifa: `VariableFont::new(bytes)` + `axes()` + `glyph_for_char` +
  `outline(gid, &[(AxisTag,f32)])` via `OutlinePen` → `GlyphOutline`. Smoke vs InterVariable.

## §2 — DECISÕES (reporto)
- **`GlyphId` próprio (não `skrifa::GlyphId`):** o ADR §2.2 diz `skrifa::GlyphId`, mas usei um newtype p/ manter
  `lib`/`glyph_to_network` skrifa-free (só o bridge toca skrifa) — padrão anti-coupling da casa (ph2d-vector-traits
  mocks). ADR-0066 não tem gate de surface congelado (gates são comportamentais), então OK. Sinalizo p/ ratificação.
- **Boundary `GlyphOutline`/`PathCommand`:** conversão glyph→network testável com outlines à mão (golden), sem
  fonte — mata vaporware-coupling com skrifa, espelha o split de arquivos do ADR §2.1.
- **Difusão de cor / espaço:** N/A aqui (fonts são geometria); cor de fill vem do StyleTable (teu).

## §3 — O QUE FICA (deferido / teu)
- **Fast-path render skrifa→Vello direto (§2.3):** otimização (pular o network, ir direto a `BezPath`). O caminho
  canônico (glifo = VectorNetwork) já renderiza pelo renderer existente — então isto é só perf, teu, não-bloqueante.
- **`PlatformHost` impl real no shell:** eu entrego a trait + a lógica de resolução; o shell pluga `system_fonts()`/
  `fallback_chain(locale)` com fontes reais do SO (cmap real em vez de `CoverageRanges`).
- **Gradient-descent em axis-space (Differentiable VF):** explicitamente V2.0+ no ADR §3.3, fora de escopo.
- **Glyph cache LRU 100MB (§2.7) + axis name via name table:** polimentos v2 (uso nome por tag conhecido hoje).

## §4 — GIT / POSSE
- Commit scoped local: `crates/ph2d-vector-font/**` + `Cargo.lock` (só a entrada da minha crate; skrifa 0.40.0
  já-presente, zero churn alheio) + este handoff. `--no-verify`, sem push. `git status` conferido: WIP sujo é
  todo alheio (Painter/render), não toquei. Cap de axes (8) respeitado; contratos congelados intactos.
═══════════════════════════════════════════════════════════════════
