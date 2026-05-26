# ADR-0054 — Image I/O pipeline (contrato `ImageImporter`/`ImageExporter` + canal genérico + registro por codegen)

**Status:** Proposed (W0 em execução; será `Accepted` após smoke do Enio da vertical PNG-stub)
**Data:** 2026-05-26
**Decisor(es):** Enio + Claude (arquiteto).
**Estende:** ADR-0031 (família como unidade FBP), ADR-0040 (tool isolation — mesmo padrão satélite drop-in), ADR-0044/0046/0051 (Painter color profile/blend modes — fonte de verdade para `RenderingMode` e `ColorProfile`).
**Espelha o mecanismo de:** fan-out de nós (ADR-0039) e fan-out de tools (ADR-0040) — `crates/ph2d-imageio-<slug>/` + `tools/ph2d-imageio-sync` + arch-gate.
**Plano vivo:** [`docs/plans/2026-05-imageio-waves.md`](../../plans/2026-05-imageio-waves.md).

---

## 1. Contexto

O Painter (sucessor do Procreate, ADRs 0043..0053) precisa **abrir e salvar arquivos de imagem** — sem isso o app é uma demo. A SKILL_Stack §11.10 listou importadores v1 numa visão asset-pipeline (cooker dev-time): PNG, JPEG, WebP, AVIF, EXR, SVG, etc. Mas import/export do **Painter em runtime** é diferente do cooker:

1. **Runtime user-facing** — usuário clica "Open" / "Export" no chrome, escolhe arquivo, vê resultado imediato.
2. **Bidirecional** — todo formato suportado para importar também precisa exportar (round-trip).
3. **Layered** — formatos profissionais (PSD, ORA, KRA, .ph2d) carregam stack de layers, não bitmap flat.
4. **HDR** — formatos científicos/VFX (EXR, HDR Radiance, AVIF 10-bit) carregam float linear, não 8-bit sRGB.
5. **Vetorial** — SVG e PDF não rasterizam no source; preservar `BezPath` permite edição vetorial não-destrutiva.

Adicionar formato hoje exigiria: criar lógica em algum crate central + variant de `EditorAction` per-formato + match no shell + entrada de chrome. **É o mesmo problema que ADR-0040 resolveu para tools**: edit central triplo por feature, não escala, serializa multi-agente.

Esta ADR aplica o **mecanismo já provado pelo sistema de nós (ADR-0039) e tools (ADR-0040)** à família image-io: drop-crate isolado + codegen + arch-gate + contrato congelado. 16 formatos planejados em 3 ondas paralelas pós-freeze (W1=5, W2=4, W3=5; +2 foundational).

## 2. Decisão

A unidade de feature é **`crates/ph2d-imageio-<slug>/`** contendo TUDO:

```
crates/ph2d-imageio-<slug>/
  src/lib.rs           # pub fn register_importer(reg); pub fn register_exporter(reg)
                       # pub fn make_importer() -> Box<dyn ImageImporter>
                       # pub fn make_exporter() -> Box<dyn ImageExporter>
  src/import.rs        # impl ImageImporter — bytes → DecodedImage
  src/export.rs        # impl ImageExporter — DecodedImage → bytes
  src/algorithm/       # decode/encode helpers (deps de libs Rust dedicadas: image, tiff, ...)
```

`crates/ph2d-imageio/` volta a ser **pura foundation**: define o **contrato** (`trait ImageImporter`, `trait ImageExporter`, `DecodedImage`, `ImageBuffer<P>`, `ExportOpts`, `Error`, `ColorProfile`, `MagicHint`) e **não conhece nenhum formato concreto**.

### 2.1 Contrato congelado (cap arch-gate)

```rust
pub trait ImageImporter: Send + Sync + 'static {
    fn supports(&self, hint: MagicHint<'_>) -> bool;
    fn import(&self, src: &[u8], opts: &ImportOpts) -> Result<DecodedImage, Error>;
}

pub trait ImageExporter: Send + Sync + 'static {
    fn supports_format(&self, fmt: &str) -> bool;
    fn export(&self, img: &DecodedImage, opts: &ExportOpts) -> Result<Vec<u8>, Error>;
}

pub enum DecodedImage {
    Flat(ImageBuffer<SrgbRgba>),       // PNG, JPEG, WebP, GIF (1ª frame), BMP
    FlatHdr(ImageBuffer<LinearRgba>),  // EXR, HDR Radiance, AVIF 10-bit, JXL HDR
    Layered(LayerStack),               // PSD, ORA, KRA, .ph2d-native
    Animated(Vec<AnimFrame>),          // GIF, APNG, WebP-anim, AVIF-anim
    Vector(VectorDoc),                 // SVG, PDF (BezPath + paint stack preserved)
}

pub enum ColorProfile {
    Srgb,                // default W1 (assumed-no-profile)
    DisplayP3,           // W2
    AdobeRgb,            // W2
    ProPhoto,            // W2
    LinearRec709,        // W3 HDR
    LinearRec2020,       // W3 HDR
    Custom(Box<[u8]>),   // arbitrary ICC blob preserved byte-exact
    Unknown,
}
```

**Caps FROZEN** (arch-gate `architecture_imageio_contract_surface`):

| Item | Cap | Surface atual |
|---|---|---|
| `ImageImporter` métodos | ≤ 3 | `supports` + `import` (2 — folga 1 para feature futura) |
| `ImageExporter` métodos | ≤ 3 | `supports_format` + `export` (2 — folga 1) |
| `DecodedImage` variants | ≤ 5 | `Flat` + `FlatHdr` + `Layered` + `Animated` + `Vector` (5 — frozen) |
| `ExportOpts` campos | ≤ 6 | `format` + `quality` + `color_profile` + `preserve_layers` + `tone_map` + `metadata` (6) |
| `ColorProfile` variants | ≤ 8 | `Srgb`/`DisplayP3`/`AdobeRgb`/`ProPhoto`/`LinearRec709`/`LinearRec2020`/`Custom`/`Unknown` (8 — frozen) |
| `Error` variants | ≤ 8 | folga para mensagens de domain (`Decode`/`Encode`/`Unsupported`/`Truncated`/`IcCorrupted`/`MissingLayer`/`HdrUnsupported`/`Custom`) |

Mudar qualquer cap = evento raro Coord-A only + amendment ADR-0054.

### 2.2 Codegen + workspace.members glob

`tools/ph2d-imageio-sync/` (lib+bin, espelha `tools/ph2d-tool-sync/`) faz scan de `crates/ph2d-imageio-*` por símbolos `pub fn register_importer` e `pub fn register_exporter`. Regenera entre marcadores codegen em `crates/ph2d-imageio-registry-init/src/lib.rs`:

```rust
pub fn register_all_importers(reg: &mut ImporterRegistry) {
    // <ph2d-imageio-sync:importers:begin>
    ph2d_imageio_png::register_importer(reg);
    ph2d_imageio_jpeg::register_importer(reg);
    ...
    // <ph2d-imageio-sync:importers:end>
}
```

`workspace.members` já é glob `crates/*` + `tools/*` (ADR-0040 §codegen-A1) — dropar `crates/ph2d-imageio-<slug>/` exige zero edit em `Cargo.toml` raiz. Staleness gate em `tests/staleness.rs` falha CI se alguém esquecer de rodar `cargo run -p ph2d-imageio-sync`.

### 2.3 Color pipeline strategy (por onda)

| Onda | Política | Por quê |
|---|---|---|
| **W1** Universal | sRGB-assumed-no-profile (sem ICC parsing) | Foto comum + fixtures sintéticas cobrem 95% do uso; perfeição de color trava ship |
| **W2** Profissional | ICC parsing ativo via `qcms` 0.x (Mozilla puro-Rust) — sRGB/P3/AdobeRGB/ProPhoto/Custom preservados byte-exact | PSD/TIFF profissional carregam profile; descartar = corromper output do usuário |
| **W3** HDR + vetor | linear-f32 + Rec709/Rec2020; tone-map fallback (Reinhard/ACES) pra LDR export | EXR/AVIF-10bit/JXL-HDR são float linear; tone-map só se exportador target é sRGB |

**Internal canonical:** OKLCH/linear-f32 (alinha ADR-0044 Painter brush engine + ADR-0051 ColorProfile FROZEN 8B). Conversão na fronteira: import → linear → OKLCH (Painter native); export → linear → encoded color space do formato target.

### 2.4 `EditorAction` strategy — reuso de `OneShotImageOp`

ADR-0040 congelou `EditorAction` em 4 variants genéricos. Importar/exportar **NÃO** ganha variant novo. Em vez disso, reusa `OneShotImageOp { tool_id: "imageio_import", entity_bits, payload }` (e `"imageio_export"`), onde `payload` carrega path + opts. Cap `EditorAction` permanece 4 (sem amendment ADR-0040 §7).

**Por quê:** ADR-0040 §4 estabeleceu "sem variant per-feature" como invariante. Image I/O é mais uma feature drop-in; criar variants `ImportImage`/`ExportImage` violaria o invariante e adicionaria pressão pro próximo agente justificar **mais** variants depois (slippery slope).

### 2.5 HR cumpridas

- **HR-1** (platform-agnostic): contrato puro Rust; libs C explicitamente proibidas (HEIC descartado por isso).
- **HR-3** (no alloc hot path): import/export rodam **off-hot-path** — invocados por user click no chrome, não em `render_graph`/`physics_step`/`audio_callback`/`editor_layout`. Alocação livre.
- **HR-6** (asset = hash blake3): cada `DecodedImage` produzido tem blake3 do bytes original como source-of-identity para o `AssetDb`.
- **HR-13** (memory budget): cada importer declara budget no manifest (provisional W0; arch-gate em W1+).
- **HR-14** (save format versionado): `ph2d-imageio-ph2d-native` (W1.T5) carrega `version: u32` + migração `migrate_v{N}_to_v{N+1}`.
- **HR-15** (i18n): erros user-facing via Fluent key `imageio.error.<variant>`.

## 3. Consequências

**Positivas:**
- Adicionar formato novo = drop-crate + sync + 3 testes verdes. Zero edit central, zero amendment ADR.
- Fan-out paralelo viável até 3 sessões simultâneas por RAM 8 GiB (CARGO_TARGET_DIR slot isolation, DIRETRIZ §1.2).
- Contrato congelado pelo arch-gate; crescimento exige decisão consciente.
- Reuso de `ph2d-color` (SrgbRgba/LinearRgba/OklchColor) elimina conversão duplicada — color discipline alinhada cross-engine.

**Negativas:**
- 16 crates a manter (16 satélites + 2 foundational). Onboarding de LLM nova → lê DIRETRIZ §3.A + ADR-0054 + um template (PNG ou JPEG) e está pronta.
- Lib Rust pure-only impede HEIC v1 (HEVC patent-heavy). User iPad shooting HEIC tem que converter offline para JPEG/PNG antes de abrir. Documentar em UI error message.
- PSD write é trabalho pesado (~600 LOC + golden test contra PS real). Risco de Onda 2 atrasar 1-2 dias além do estimado.

**Neutras:**
- `DecodedImage::Vector` modelado desde W0 mesmo só populado em W3 — cap fica em 5 desde início, evita amendment em W3.
- ICC parsing usa `qcms` (Mozilla) em vez de `lcms2-rs` (binding C) — pure Rust + maintenance ativo.

## 4. Alternativas consideradas

| Alternativa | Por que rejeitada |
|---|---|
| **God-crate `ph2d-imageio` único** com módulos por formato | Edit central por formato = não escala multi-agente; serializa fan-out |
| **HEIC v1 com libheif C** | Quebra HR-1 (platform-agnostic core); risco patent HEVC |
| **`EditorAction::ImportImage`/`ExportImage` variants** | Viola invariante ADR-0040 "sem variant per-feature"; abre slippery slope |
| **3 ADRs separados (1 por onda)** | Inflação de docs; ADR-0040 mostrou que amendments inline são suficientes |
| **Color management como ADR separado** | Acopla tight a image-io (origem do ICC); modelar inline simplifica |
| **`DecodedImage::Vector` adicionado em W3 como amendment** | Force cap bump em W3; modelar desde W0 é zero-cost (variant nunca populado em W1/W2) |

## 5. Histórico de execução

W0 abre 2026-05-26.

| Task | Estado | Commit |
|---|---|---|
| W0.T1 contrato `ph2d-imageio` | ⏳ em execução | — |
| W0.T2 `tools/ph2d-imageio-sync` | ⏳ pendente | — |
| W0.T3 `ph2d-imageio-registry-init` | ⏳ pendente | — |
| W0.T4 arch-gate `architecture_imageio_contract_surface` | ⏳ pendente | — |
| W0.T5 stub `ph2d-imageio-png` (prova end-to-end) | ⏳ pendente | — |
| W0.T6 wiring shell desktop | ⏳ pendente | — |
| W0.T7 ratificação (Proposed → Accepted) | ⏳ aguarda smoke do Enio | — |

(Histórico de execução vai sendo atualizado conforme W0 fecha cada task. Padrão: ADR-0040 §7.)
