# Plano de waves — Image I/O (neck → freeze → fan-out)

**Data:** 2026-05-26
**Status:** **W0 + W1 + W2 FECHADAS + W3 pre-gates + W3.T0..W3.T0.6 audit-remediations CONVERGÊNCIA RATIFICADA + W3.T1.0 wire-up + W3.T1..T5 FAN-OUT SHIPPED** 2026-05-27 — ADR-0054 `Accepted`; **14 format crates** wired (apng/avif/exr/gif/hdr-radiance/jpeg/jxl/ora/ph2d-native/png/psd/svg/tiff/webp — 5 W3 são magic-only-stubs exceto SVG que faz real parse); 16 imageio crates registrados no `spike.yml` nextest CI matrix; **176 tests verdes Mac aarch64** (141 W2 + 35 W3); 34+ commits locais; ship.sh real-time GREEN isoladamente para os 16 crates.
**Arquitetura:** ADR-0054 (Proposed; ratifica em T7).
**Substrato multi-agente:** mesmo de [`docs/plans/2026-05-node-waves.md`](2026-05-node-waves.md) — drop-crate + codegen + arch-gate.

Os tags `W0.Tx` / `W1.Tx` / `W2.Tx` / `W3.Tx` que aparecem em commits / comentários referenciam este doc.

## Forma: funil (mesmo do sistema de nós + tools)

Um **neck serial** (contrato `ImageImporter`/`ImageExporter`/`DecodedImage` + codegen + arch-gate, Coord-A only — nenhum agente extra acelera) → **FREEZE** → **fan-out paralelo** em 3 ondas sequenciais (16 crates satélite, batches intra-onda ≤ 3 slots RAM). Cada formato vira `crates/ph2d-imageio-<slug>/`, espelhando ADR-0040 (tool isolation).

## Decisões de design (consolidadas com Enio 2026-05-26)

1. **Puro-Rust only.** Sem libs C — HEIC (HEVC patent-heavy, sem decoder puro Rust maduro) **descartado** da v1.
2. **PSD read + write** mantido na Onda 2 (write é o gargalo da onda; encode binário compatível com PS real).
3. **`DecodedImage::Vector` modelado desde W0** — SVG/PDF preservam BezPath vetorial (cap `DecodedImage ≤ 5 variants` desde o início).
4. **Execução sequencial 0 → 1 → 2 → 3.** Paralelismo só **intra-onda** (até 3 slots cargo simultâneos por RAM 8 GiB).
5. **ADR único** (0054) na W0; cada onda subsequente faz amendment in-place (espelha como ADR-0040 evoluiu por TG-A..TG-E).

## WAVE 0 — NECK (contrato compartilhado) · SERIAL · COORD-A ONLY

- **W0.T1** — `crates/ph2d-imageio/`: contrato (`trait ImageImporter`, `trait ImageExporter`, `DecodedImage`, `ImageBuffer<P>`, `ExportOpts`, `ImportOpts`, `Error`, `ColorProfile`, `MagicHint`). Reusa `ph2d-color::{SrgbRgba, LinearRgba, OklchColor}`. ✅ commit `8d8d79e`.
- **W0.T2** — `tools/ph2d-imageio-sync/`: codegen (lib+bin) espelhando `tools/ph2d-tool-sync/`. Scan de `crates/ph2d-imageio-*` por `pub fn register_importer` e `pub fn register_exporter`; regenera entre marcadores. ✅ commit `262a414`.
- **W0.T3** — `crates/ph2d-imageio-registry-init/`: corpo codegen'd com `register_all_importers` + `register_all_exporters`. Staleness gate em `tests/staleness.rs`. ✅ commit `262a414`.
- **W0.T4** — Arch-gate `crates/ph2d-imageio/tests/architecture_imageio_contract_surface.rs`. ✅ commit `8d8d79e`; caps `Error` 8→11 raised pós-audit `51ea6f6`.
- **W0.T5** — Stub PNG (`crates/ph2d-imageio-png/`) decode + encode via `image` 0.25 prova o pipeline end-to-end: `arquivo bytes → DecodedImage::Flat → bytes` round-trip bit-exact. ✅ commit `3db01b6`.
- **W0.T6** — Wiring boot em `shells/desktop/src/init.rs`: chama `register_all_importers` + `register_all_exporters` 1× no startup, após `register_all_tools`. ✅ commit `f002b6a` + smoke do Enio confirmado: `imageio registries built (1 importer(s), 1 exporter(s))`.
- **W0.T6.5** — **Auditoria adversarial 5-lente** (2026-05-26): 4 CRITICAL + 11 HIGH + 9 MEDIUM + 12 LOW. Remediação pré-ratificação em 3 batches (contract data model + HR/quality + docs). ✅ commit `51ea6f6`.
- **W0.T7** — ADR-0054 `Proposed` → `Accepted`. ✅ ratificado 2026-05-26. Smoke pós-remediação confirmado pelo Enio: `[553ms] ADR-0054 W0.T6: imageio registries built (1 importer(s), 1 exporter(s))`. Contrato CONGELADO; mudanças daqui em diante = amendment Coord-A.

## 🔒 FREEZE (gate do fan-out) — após W0.T5 + smoke do Enio

Depois de W0 fixar o contrato e a vertical PNG-stub provar end-to-end: caps do arch-gate apertados ao tamanho atual → qualquer crescimento tripa o gate. Mudanças viram evento raro Coord-A only via amendment ADR-0054. **Fan-out aberto** (W1+).

## WAVE 1 — Universal · PARALELO intra-onda (pós-freeze)

**Política de cor:** sRGB-assumed-no-profile (sem ICC parsing). Pra fixture sintética + foto comum cobre 95% do caso. ICC entra em W2.

Batch A (3 slots paralelos):
- **W1.T1** — `crates/ph2d-imageio-png/`: ✅ commit `ac50809`. Multi-color-type decode (Gray/GrayAlpha/RGB/RGBA/Indexed → 8-bit RGBA); 16-bit quantize-down documented; 32K dimension cap (bomb defence). 12 tests. **16-bit roundtrip + sBIT + ICC ficam W2** (sem cliente real em W1).
- **W1.T2** — `crates/ph2d-imageio-jpeg/`: ✅ commit `4b2b657`. 8-bit RGB decode + encode; alpha dropped on export (JPEG composites against black); quality 1-100 clamped. 8 tests. **EXIF orientation honouring fica W2**.
- **W1.T3** — `crates/ph2d-imageio-webp/`: ✅ commit `4423367`. Lossy + lossless decode; **encoder lossless-only** (pure-Rust image-webp; lossy encode precisa libwebp C = HR-1 violation, defer W2+); alpha preserved. 6 tests.

Batch B (2 slots paralelos, após Batch A):
- **W1.T4** — `crates/ph2d-imageio-gif/`: ✅ commit `3a84b1b`. Single-frame → `Flat`; multi-frame → `Animated(Vec<AnimFrame>)`; per-frame `delay_ms` + `offset_xy` preserved; **`dispose_op` + `transparent_index` ficam W2** (image::Frame opaque-wraps DisposalMethod). 7 tests (inclui anim roundtrip 3-frame).
- **W1.T5** — `crates/ph2d-imageio-ph2d-native/`: ✅ commit `7270d40`. **Native Painter save format.** 52-byte header (magic + version u32 + blake3 32B + payload_len u64) + postcard envelope; preserves all 5 `DecodedImage` variants lossless (Flat/FlatHdr/Layered PSD-grade/Animated full-fidelity/Vector stub); HR-5 + HR-6 + HR-14 todos cobertos; schema mirror em `src/schema.rs`. 13 tests.

**Aceitação W1:** ✅ 5 crates registrados via codegen alphabetical (gif/jpeg/png/webp/ph2d-native); 81 testes verdes; smoke Enio do boot trace `5 importer(s), 5 exporter(s)` ainda pendente quando você quiser rodar `./play.command`.

## WAVE 2 — Profissional 2D · PARALELO intra-onda

### W2.0.0 — qcms viability gate (Coord-A, 15min — HARD BLOCKER de W2.0.1) ✅ RESOLVIDO 2026-05-26

Executado 2026-05-26 (Coord-A): `cargo search qcms / moxcms / lcms2 / appthere-color` + `cargo info` detalhes.

**Decisão: `moxcms` 0.8.1** (puro-Rust, mantenedor ativo, Rust 1.85.0 + SIMD opt-in). Veredicto detalhado em ADR-0054 §2.3.1. `qcms` 0.3.0 rejeitado (versão velha, Firefox internalizou). `lcms2` rejeitado (C binding viola HR-1).

**W2.0.1 ICC pipeline abre** com moxcms 0.8.1 como única dep. Fallback documentado: implementação local de ICC v2 matrix lookup mínima se moxcms travar mid-implementation.

### W2.0 — Pré-fan-out (Coord-A, 1 sessão)

- **W2.0.1** — ICC pipeline policy: **per-format inline** com `moxcms` 0.8.1 (gate W2.0.0). Cada format crate W2 que carrega ICC (TIFF iCCP, PSD ImageResource 1039) adiciona `moxcms` dep e parseia inline. Refactor para `ph2d_imageio::icc` central API acontece quando 2º cliente independente chega. ✅ policy 2026-05-26.
- **W2.0.2** — `LayerStack` expansão **já realizada pela audit W1.T6**: `BlendMode` (24 variants + `Custom(u16)` preservando opcodes PSD); `Layer` com `kind: LayerKind {Pixel/Group/Adjustment/Text/Smart}` + `effects: Vec<LayerEffect>` + `color_profile: Option<ColorProfile>` + `version: u32` (HR-14). PSD/ORA chegam encontrando shape pronto. ✅ pre-cooked.
- **W2.0.3** — ADR-0054 amendment §5.2 W2 abertura: 3 decisões ratificadas + batches anunciados. ✅ 2026-05-26.

### W2.1 — Fan-out (4 crates; PSD sozinho por carga)

Batch A (3 slots paralelos):
- **W2.T1** — `crates/ph2d-imageio-ora/`: ZIP + `stack.xml` + PNG/layer + thumbnail. **Padrão aberto Krita/MyPaint.** `zip` 2.x + `roxmltree` 0.20 + `image`. ⏳
- **W2.T2** — `crates/ph2d-imageio-tiff/`: 16-bit + CMYK→RGBA via ICC + multi-page. `tiff` 0.10. ⏳
- **W2.T3** — `crates/ph2d-imageio-apng/`: anim lossless + alpha. `apng` 0.3 ou custom sobre `png` crate. ⏳

Batch B (1 slot dedicado, sequencial após A — PSD é gargalo):
- **W2.T4** — `crates/ph2d-imageio-psd/`: ✅ commit `4f79ba3` (import-only; **escape hatch §5.2 W2.5 invocado por antecipação**). Import via `psd` 0.3.5 (read-only crates.io); export retorna `Error::Unsupported` com mensagem citando ADR-0054 §5.2 + `.ph2d-native` alternative. 7 tests (pós-audit W2.T6 com PSB magic Strong + Unsupported actionable + n_layers × canvas cap defense). PSD write greenfield (~600 LOC + golden contra Photoshop real) fica W3+ quando UI Save As PSD virar primeiro cliente real.

**Aceitação W2:** 4 crates registrados; smoke Enio abre PSD do Photoshop, edita, salva como ORA e re-importa idêntico no Painter; ICC P3 round-trip byte-exact; CI verde.

## WAVE 3 — HDR + next-gen + vetorial · PARALELO intra-onda

### W3 pre-gates (Coord-A, 1 sessão) ✅ FECHADOS 2026-05-26 commit `f71f16a` + audit-remediation `35cc149`

3 gates pré-fan-out aplicados ao código já shipado de W1/W2 antes de abrir W3:

- **Gate 1 — ADR §2.6 amendment** (DecodedImage variant policy): collapse-when-trivial policy formalizada, alinhando 9 format crates. Vide ADR §2.6.
- **Gate 2 — Hex-baked Tier-1 fixtures**: APNG multi-frame, TIFF CMYK (K=0 + K>0 arms) + RGBA16 (endpoints + mid), ORA group nesting com 2 children. Real decoder paths agora exercitados; substring matches eliminados.
- **Gate 3 — Golden blake3 hashes (single-platform Mac aarch64)**: PNG/TIFF/ORA/APNG têm `export_golden_blake3_local_drift_pinned_macos_silicon` gateados por `#[cfg]`. Scope = LOCAL drift guard apenas; CI matrix passa por `#[cfg]` gate. Multi-platform pinning deferido até primeira divergência observada. Vide ADR §2.6.1.

**Auditoria W3.T0** (5-lens 2026-05-26): 1 CRITICAL (golden hashes single-platform vs CI matrix) + 4 HIGH + 9 MEDIUM + 12 LOW; fechados inline em commit `35cc149`. Vide ADR §5.5.

**Auditoria W3.T0.1** (5-lens 2026-05-26): 1 CRITICAL (imageio FORA da CI matrix) + 7 HIGH (HR-9→HR-5 nomenclatura + tolerâncias TIFF off-by-one + clippy convenção) + 11 MEDIUM + 13 LOW; fechados em commit `fd34240`. Vide ADR §5.6.

**Auditoria W3.T0.2** (5-lens 2026-05-26): 8 CRITICAL (6 meus 3 ship-blockers F-B1/B2/B3 + 3 OOM J-1/J-2/J-3; 2 não-meus B-4/M-5 flaggados) + 11 HIGH (ORA `<stack>` spec + opts dead window + EOF helper + non_exhaustive + PSD catch_unwind + 5 absorvidos) + 13 MEDIUM + 15 LOW; fechados em commit `108a623`. Vide ADR §5.7.

**Auditoria W3.T0.3** (6-lens 2026-05-26 — rotacionadas K/L/M/N/O/P): 0 CRITICAL + 6 HIGH (ColorProfile::Custom vapor + GIF semantica + EOF helper adoption + APNG multi-frame test gap + docs drift + P-mine fmt) + 17 MEDIUM (test coverage de 5 fixes + caps hoist + EOF gaps + variant context) + 12 LOW + 1 P0 doc; fechados em commit `2a41a0b`+`64f54d9`. Vide ADR §5.8.

**Auditoria W3.T0.4** (7-lens 2026-05-26 — rotacionadas Q/R/S/T/U/V/W): 1 CRITICAL (ORA `parse_stack` recursion bomb — stack overflow DoS) + 6 HIGH (Lens T-#1 caps; Q-H1 GIF L-3 incomplete migration; W-FmtMine; V-WorkspaceLints — flag arquitetural; R-G2 asset bypass — flag arquitetural; tests count drift) + 6 MEDIUM + 10 LOW; fechados em commit `cde3e44`. Vide ADR §5.9.

**Auditoria W3.T0.5** (3-lens 2026-05-26 — especializadas X/Y/Z): 1 CRITICAL (ORA `opacity` NaN/Inf data poisoning) + 1 HIGH meu (ZIP central directory amplification) + 1 HIGH não-meu (X arquitetural shells/desktop bypass) + 2 MEDIUM (stack.xml take(N) cap + APNG snake_case rename) + 0 LOW; Lens Y retornou GREEN. Fechados em commit `4d7dfdd`. Vide ADR §5.10.

**Auditoria W3.T0.6** (4-lens 2026-05-26 — convergence final BB/CC/DD/EE): 0 CRITICAL + 0 HIGH meu + 0 MEDIUM (apenas 2 LOW forensic em CC + 2 doc gaps em DD); Lens BB threading **GREEN**, Lens EE ship.sh real-time **GREEN** (11 crates passam fmt+clippy+typos+machete+test isoladamente). **PADRÃO-OURO CONVERGÊNCIA RATIFICADA.** Vide ADR §5.11.

### W3.0 — Pré-fan-out (Coord-A, 1 sessão) — HDR + Vector ativos

- **W3.0.1** — HDR float pipeline ativo. `DecodedImage::FlatHdr(ImageBuffer<RgbaF32>)` populado; tone-map fallback (Reinhard/ACES) pra LDR export quando o exportador é sRGB-only; conversão linear-f32 ↔ OKLCH preserva dinâmica.
- **W3.0.2** — Vector bridge ativo. `DecodedImage::Vector(VectorDoc)` populado: `kurbo::BezPath` + paint stack + transforms + clip paths; bridge `rasterize_at(dpi)` opt-in via Vello no import; export Painter vector layers → SVG/PDF.
- **W3.0.3** — ADR-0054 amendment §W3: HDR pipeline + Vector preservado canonical.
- **W3.0.4** — `LayerStack.import_warnings: Vec<String>` field addition (audit-7 H-3): PSD blend modes (Dissolve, DarkerColor, LighterColor) atualmente mapeiam silenciosamente; este campo expõe perdas pro caller (UI surface "Save As .ph2d-native to preserve"). Cap arch-gate revisão obrigatória (LayerStack hoje cap=5, addition exige re-ratify). ADR amendment + arch-gate update + smoke do Enio.

### W3.1 — Fan-out (5 crates, 2 batches paralelos)

Batch A (3 slots paralelos, puro Rust pequenos):
- **W3.T1** — `crates/ph2d-imageio-jxl/`: ✅ commit `cc97cd4` — magic-only stub (ISOBMFF + codestream); decode `jxl-oxide` 0.10 deferred quando primeiro real client; encode permanent-deferred (no pure-Rust JXL encoder).
- **W3.T2** — `crates/ph2d-imageio-exr/`: ✅ commit `cc97cd4` — magic-only stub; decode/encode `exr` 1.x deferred (closure API + RefCell out-params).
- **W3.T3** — `crates/ph2d-imageio-hdr-radiance/`: ✅ commit `cc97cd4` — magic-only stub (`#?RADIANCE\n` + `#?RGBE\n`); decode/encode `image` 0.25 `hdr` feature deferred.

Batch B (2 slots paralelos):
- **W3.T4** — `crates/ph2d-imageio-avif/`: ✅ commit `cc97cd4` — magic-only stub (ISOBMFF ftyp `avif`/`avis`); decode `avif-decode` + encode `ravif` 0.11 deferred.
- **W3.T5** — `crates/ph2d-imageio-svg/`: ✅ commit `cc97cd4` — **REAL parse** via `usvg = "0.43"` (parse + simplify validation); retorna `DecodedImage::Vector(VectorDoc::default())`. `MAX_ARCHIVE_TEXT_BYTES = 16 MiB` cap blocks billion-laughs. Export deferred (ph2d-vector canonical types W3+).

**Aceitação W3:** 5 crates registrados; smoke Enio: (a) abre EXR HDRi de Blender, edita exposure, salva AVIF 10-bit; (b) importa SVG Lucide, mantém vetorial editável no canvas, exporta SVG válido em Chrome/Safari/Firefox real; CI verde.

## Cronograma sequencial estimado

```
Dia 1-2:   W0 (Coord-A solo)               [neck]
Dia 3-5:   W1 (5 impl, 2 batches paralelos) [universal]
Dia 6-12:  W2 (1 Coord pre + 3+1 impl)      [profissional]
Dia 13-18: W3 (1 Coord pre + 3+2 impl)      [HDR + vetor]
```

Total: 14-18 dias wall-clock; 22-27 sessões Claude.

## Achados em aberto (não-bloqueiam W0)

- ~~**`EditorAction` strategy**~~ — **RESOLVIDO 2026-05-26 via market-pattern survey.** Import/export é chamada direta no shell via `ImporterRegistry::find_for` (padrão Unity/Unreal/Godot/Bevy/Krita/Blender). NUNCA atravessa `EditorAction`. ADR-0054 §2.4 reescrita.
- **HEIC futuro** — Quando decoder puro-Rust de HEVC chegar (rust-libheif puro Rust em discussão upstream), reabre como W4.T?.
- **PDF export** — Vello backend PDF (kurbo+peniko) pode entregar export PDF como bônus em W3 via `pdf-writer` 0.10, mas não está priorizado.
- **RAW câmera** (DNG/CR2/NEF/ARW) — fora de escopo v1; entra em W4+ se demanda real aparecer.
