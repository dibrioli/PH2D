# Plano de waves — Image I/O (neck → freeze → fan-out)

**Data:** 2026-05-26
**Status:** W0 quase fechada (T1-T6 ✅ shipped; T7 aguarda ratificação Proposed → Accepted após auditoria 5-lente remediada — 6 commits locais).
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
- **W0.T4** — Arch-gate `crates/ph2d-imageio/tests/architecture_imageio_contract_surface.rs`. ✅ commit `8d8d79e`; caps `Error` 8→11 raised pós-audit `[remediation-pending]`.
- **W0.T5** — Stub PNG (`crates/ph2d-imageio-png/`) decode + encode via `image` 0.25 prova o pipeline end-to-end: `arquivo bytes → DecodedImage::Flat → bytes` round-trip bit-exact. ✅ commit `3db01b6`.
- **W0.T6** — Wiring boot em `shells/desktop/src/init.rs`: chama `register_all_importers` + `register_all_exporters` 1× no startup, após `register_all_tools`. ✅ commit `f002b6a` + smoke do Enio confirmado: `imageio registries built (1 importer(s), 1 exporter(s))`.
- **W0.T6.5** — **Auditoria adversarial 5-lente** (2026-05-26): 4 CRITICAL + 11 HIGH + 9 MEDIUM + 12 LOW. Remediação pré-ratificação em 3 batches (contract data model + HR/quality + docs). ✅ `[remediation-commit-pending]`.
- **W0.T7** — ADR-0054 `Proposed` → `Accepted`. Estende ADRs Painter (0043..0053) + ADR-0040 (tool isolation). Cobre: contrato congelado, color pipeline strategy (sRGB-assumed W1 / ICC W2 / HDR W3 / OKLCH internal sempre), `DecodedImage::Vector` modelado desde W0, EditorAction strategy (defer pra W1.T0 — vide ADR §2.4 amended). ⏳ aguarda smoke ratificação Enio.

## 🔒 FREEZE (gate do fan-out) — após W0.T5 + smoke do Enio

Depois de W0 fixar o contrato e a vertical PNG-stub provar end-to-end: caps do arch-gate apertados ao tamanho atual → qualquer crescimento tripa o gate. Mudanças viram evento raro Coord-A only via amendment ADR-0054. **Fan-out aberto** (W1+).

## WAVE 1 — Universal · PARALELO intra-onda (pós-freeze)

**Política de cor:** sRGB-assumed-no-profile (sem ICC parsing). Pra fixture sintética + foto comum cobre 95% do caso. ICC entra em W2.

Batch A (3 slots paralelos):
- **W1.T1** — `crates/ph2d-imageio-png/`: import + export 8/16-bit + alpha + sBIT chunk. `image` 0.25. ⏳
- **W1.T2** — `crates/ph2d-imageio-jpeg/`: import + export 8-bit RGB, EXIF orientation honrado, quality 0-100. `image` 0.25. ⏳
- **W1.T3** — `crates/ph2d-imageio-webp/`: lossy + lossless + alpha + anim. `image` (decode) + `webp` 0.3 (encode). ⏳

Batch B (2 slots paralelos, após Batch A):
- **W1.T4** — `crates/ph2d-imageio-gif/`: import + export anim → `Animated` ou flat 1ª frame. `image` + `gif` 0.13. ⏳
- **W1.T5** — `crates/ph2d-imageio-ph2d-native/`: postcard + blake3 + `LayerStack` + history + HR-14 migration v1→v1. Round-trip total dos projetos Painter. ⏳

**Aceitação W1:** 5 crates registrados via codegen; smoke Enio abre/salva `.png/.jpg/.webp/.gif/.ph2d` no Painter; CI verde 1× ao fim da onda.

## WAVE 2 — Profissional 2D · PARALELO intra-onda

### W2.0 — Pré-fan-out (Coord-A, 1 sessão)

- **W2.0.1** — ICC profile parsing ativo. `ph2d-imageio::ColorProfile::Custom(IccBytes)` populado por importers que carregam ICC; conversão sRGB/P3/AdobeRGB/ProPhoto ↔ OKLCH internal via `qcms` 0.x (Mozilla puro-Rust) ou `lcms2-rs`. **Decisão**: `qcms` (Mozilla-maintained, mais simples).
- **W2.0.2** — `LayerStack` expansão. Blend modes alinhados a `RenderingMode = 6 FROZEN` (ADR-0044/0046 Painter); opacity, mask, clipping, group, lock.
- **W2.0.3** — ADR-0054 amendment §W2: ICC pipeline ativo; HEIC descartado com rationale.

### W2.1 — Fan-out (4 crates; PSD sozinho por carga)

Batch A (3 slots paralelos):
- **W2.T1** — `crates/ph2d-imageio-ora/`: ZIP + `stack.xml` + PNG/layer + thumbnail. **Padrão aberto Krita/MyPaint.** `zip` 2.x + `roxmltree` 0.20 + `image`. ⏳
- **W2.T2** — `crates/ph2d-imageio-tiff/`: 16-bit + CMYK→RGBA via ICC + multi-page. `tiff` 0.10. ⏳
- **W2.T3** — `crates/ph2d-imageio-apng/`: anim lossless + alpha. `apng` 0.3 ou custom sobre `png` crate. ⏳

Batch B (1 slot dedicado, sequencial após A — PSD é gargalo):
- **W2.T4** — `crates/ph2d-imageio-psd/`: import via `psd` 0.x (read-only crates.io); **export binário PS-compatible custom = greenfield** (header + color mode + image resources + layer/mask info + image data + global layer mask). Blend modes ↔ `BlendMode` (24 variants + `Custom(u16)`); `LayerKind {Pixel/Group/Adjustment/Text/Smart}` + `LayerEffect` preservation. Golden contra fixture Photoshop real. **Estimativa revisada pós-audit D-M**: 2-3 sessões solo + audit adversarial dedicada (vs 1 sessão original). Escape hatch: se export PSD slip a 1 semana, fork em ADR-0054.W2.5 com decisão "PSD write defer para W4 ou implementar via Lottie-style pipeline alternativo".

**Aceitação W2:** 4 crates registrados; smoke Enio abre PSD do Photoshop, edita, salva como ORA e re-importa idêntico no Painter; ICC P3 round-trip byte-exact; CI verde.

## WAVE 3 — HDR + next-gen + vetorial · PARALELO intra-onda

### W3.0 — Pré-fan-out (Coord-A, 1 sessão)

- **W3.0.1** — HDR float pipeline ativo. `DecodedImage::FlatHdr(ImageBuffer<RgbaF32>)` populado; tone-map fallback (Reinhard/ACES) pra LDR export quando o exportador é sRGB-only; conversão linear-f32 ↔ OKLCH preserva dinâmica.
- **W3.0.2** — Vector bridge ativo. `DecodedImage::Vector(VectorDoc)` populado: `kurbo::BezPath` + paint stack + transforms + clip paths; bridge `rasterize_at(dpi)` opt-in via Vello no import; export Painter vector layers → SVG/PDF.
- **W3.0.3** — ADR-0054 amendment §W3: HDR pipeline + Vector preservado canonical.

### W3.1 — Fan-out (5 crates, 2 batches paralelos)

Batch A (3 slots paralelos, puro Rust pequenos):
- **W3.T1** — `crates/ph2d-imageio-jxl/`: `jxl-oxide` 0.x (puro Rust!). HDR + lossless + lossy. ⏳
- **W3.T2** — `crates/ph2d-imageio-exr/`: `exr` 1.x (puro Rust). Float32 linear multi-channel; direto → `FlatHdr`. ⏳
- **W3.T3** — `crates/ph2d-imageio-hdr-radiance/`: `image` (`hdr` feature). RGBE encoding; IBL/skybox. ⏳

Batch B (2 slots paralelos):
- **W3.T4** — `crates/ph2d-imageio-avif/`: `avif-decode` (puro Rust decode) + `ravif` 0.11 (rav1e encoder, puro Rust). 10/12-bit HDR via `FlatHdr`. ⏳
- **W3.T5** — `crates/ph2d-imageio-svg/`: import preserva `DecodedImage::Vector(VectorDoc)` via `usvg` 0.43 + `resvg`; rasterize opt-in; export Painter vector layers → kurbo → SVG string via `ph2d-vector` (M11). ⏳

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

- **`EditorAction` strategy** — Em W0.T7 confirmar: import/export rota via `OneShotImageOp { tool_id: "imageio_import", payload }` reusando canal genérico (alinha "sem variant per-feature" ADR-0040), OU amendment ADR-0040 adicionando 2 variants `ImportImage`/`ExportImage`. **Recomendação: reuso** (sem inflar cap `EditorAction`).
- **HEIC futuro** — Quando decoder puro-Rust de HEVC chegar (rust-libheif puro Rust em discussão upstream), reabre como W4.T?.
- **PDF export** — Vello backend PDF (kurbo+peniko) pode entregar export PDF como bônus em W3 via `pdf-writer` 0.10, mas não está priorizado.
- **RAW câmera** (DNG/CR2/NEF/ARW) — fora de escopo v1; entra em W4+ se demanda real aparecer.
