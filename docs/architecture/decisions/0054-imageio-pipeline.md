# ADR-0054 — Image I/O pipeline (contrato `ImageImporter`/`ImageExporter` + canal genérico + registro por codegen)

**Status:** Accepted (W0 fechada 2026-05-26 — T1-T6 ✅ shipped + auditoria 5-lente remediada + smoke pós-remediação confirmado pelo Enio: `[553ms] ADR-0054 W0.T6: imageio registries built (1 importer(s), 1 exporter(s))`)
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

### 1.1 HEIC explicitly out-of-scope (audit D-M follow-up)

A decisão "puro-Rust only" (decisão #1 do Enio em 2026-05-26) **descarta HEIC v1**. Crates pesquisados em 2026-05:

- **`libheif-rs`** (binding C de libheif) — viola HR-1 (não-puro-Rust core).
- **`heif`** crate (crates.io) — apenas decode parcial sem maintenance ativo.
- **HEVC decoders puros Rust** — não há implementação madura. HEVC é patent-heavy (royalty para MPEG-LA), e o esforço da Rust community concentrou-se em AV1 (`rav1d` puro-Rust). ChromeOS internamente decodifica HEVC via hardware; sem fallback software puro-Rust disponível.

Implicação UI: usuário iPad que tira foto em HEIC precisa converter offline para JPEG/PNG antes de abrir no Painter. Documentar essa restrição em UI error message via `imageio.error.unsupported` Fluent key. Reabre como W4 quando decoder HEVC puro Rust maduro emergir.

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
| `ImageImporter` métodos | ≤ 3 | `supports` + `import` (2 — folga 1 reservada p/ W1.T1 chrome derivation, audit C-H1) |
| `ImageExporter` métodos | ≤ 3 | `supports_format` + `export` (2 — folga 1) |
| `DecodedImage` variants | ≤ 5 | `Flat` + `FlatHdr` + `Layered` + `Animated` + `Vector` (5 — frozen; SVG SMIL rasteriza para `Animated`) |
| `ExportOpts` campos | ≤ 6 | `format` + `quality` + `color_profile` + `preserve_layers` + `tone_map` + `metadata` (6, todos enums type-safe) |
| `ColorProfile` variants | ≤ 8 | `Srgb`/`DisplayP3`/`AdobeRgb`/`ProPhoto`/`LinearRec709`/`LinearRec2020`/`Custom`/`Unknown` (8 — frozen) |
| `Error` variants | ≤ 11 | `Decode`/`Encode`/`Unsupported`/`Truncated`/`IccCorrupted`/`MissingLayer`/`HdrUnsupported`/`OutOfMemory`/`DimensionExceedsLimit`/`Cancelled`/`Custom` (11 — raised 8→11 by audit A-H4 para cobrir OOM + decompression-bomb defence + cooperative cancellation) |

Mudar qualquer cap = evento raro Coord-A only + amendment ADR-0054.

### 2.1.1 Data-model types NÃO arch-gated (audit clarification)

Os tipos abaixo são **data model** (every encoder/decoder reads, no implementer trait): podem crescer sem amendment. O arch-gate cobre apenas trait surfaces que **todo format crate implementa**:

- **`BlendMode`** — 24 variants + `Custom(u16)`. PSD/ORA têm 27+ blend modes; o `Custom(u16)` preserva opcode PSD para round-trip byte-exact em modos desconhecidos.
- **`Layer`** — campos (kind, pixels, opacity, blend_mode, visible, mask, effects, color_profile, version).
- **`LayerKind`** — `Pixel/Group/Adjustment/Text/Smart` (modela PSD layer richness; audit A-C2).
- **`LayerEffect` + `LayerEffectKind`** — DropShadow/Glow/Stroke/Bevel/ColorOverlay/GradientOverlay/PatternOverlay/Custom.
- **`LayerStack`** — campos (`version: u32` HR-14, canvas dims, layers, color_profile).
- **`AnimFrame`** — campos (image, delay_ms, offset_xy, dispose_op, blend_op, transparent_index — audit A-C3 para GIF/APNG fidelity).
- **`DisposeOp` / `AnimBlendOp`** — variants modelam GIF/APNG disposal/blend.
- **`VectorDoc`** — campos (W3 expand sem amendment).
- **`ExportFormat`** — 14 variants (cobre W1+W2+W3 + `Ph2dNative`); cresce com cada formato adicionado.

Estes tipos têm `Custom(_)` escape hatches onde aplicável para garantir round-trip byte-exact mesmo quando a versão atual não modela o opcode/feature.

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
| **W2** Profissional | ICC parsing ativo via `qcms` 0.x **OU fallback `moxcms`** (vide §W2.0.1) — sRGB/P3/AdobeRGB/ProPhoto/Custom preservados byte-exact | PSD/TIFF profissional carregam profile; descartar = corromper output do usuário |
| **W3** HDR + vetor | linear-f32 + Rec709/Rec2020; tone-map fallback (Reinhard/ACES) pra LDR export | EXR/AVIF-10bit/JXL-HDR são float linear; tone-map só se exportador target é sRGB |

**Internal canonical:** OKLCH/linear-f32 (alinha ADR-0044 Painter brush engine + ADR-0051 ColorProfile FROZEN 8B). Conversão na fronteira: import → linear → OKLCH (Painter native); export → linear → encoded color space do formato target.

### 2.3.1 W2.0.1 ICC pipeline — viability gate (audit D-M)

Antes de abrir W2.0.1, Coord-A executa: `cargo search qcms` + verificação de last-release date no crates.io + scan do CHANGELOG. `qcms` é projeto Mozilla histórico; se o crate estiver dormente (>12 meses sem release), W2.0.1 pivota para:

- **`moxcms` 0.5+** (puro-Rust, mantido ativamente em 2026, suporte a ICC v2 + v4) — fallback primário.
- **Implementação local** de ICC v2 lookup matricial (sRGB/P3/AdobeRGB hardcoded; Custom → identidade) — fallback se ambos crates falharem.

O ICC pipeline é foundational para W2 (sem ele, PSD/TIFF perdem profile preservation = data-loss). Bloqueio aqui = bloqueio de W2 inteira; o viability gate evita descobrir isso no meio do batch.

### 2.4 Import/export é I/O direto via registry — NÃO atravessa `EditorAction`

**Recalibração 2026-05-26 (pós-survey de engines de mercado).** A formulação original desta seção (e a auditoria C-C1 que derivou dela) partia da premissa errada: "import/export deve passar pelo canal genérico de `EditorAction` reusando `OneShotImageOp` com payload". Pesquisa de patterns nas engines de referência mostra que **nenhuma** delas faz isso. O padrão dominante é **chamada direta polimórfica via registry**, independente do action bus do editor.

#### Padrão observado nas engines de mercado

| Engine | Mecanismo de import/export |
|---|---|
| **Unity** | `AssetDatabase.LoadAssetAtPath<T>(path)` + `AssetImporter` subclass per format. Chamada direta de método estático. |
| **Unreal** | `UFactory` polimórfico per format (PNGFactory, JPEGFactory, …); `FAssetToolsModule::ImportAssetsWithDialog` chama a factory direta. |
| **Godot** | `ResourceImporter` virtual class + `ResourceImporterManager` registry com `import_threaded_request(path)`. Singleton method call. |
| **Bevy** (Rust, mais próximo) | `AssetLoader` trait + `AssetServer::load(path)` retorna `Handle<T>`. Method call em singleton global. |
| **Krita** (raster 2D pro) | `KisImportExportManager::importDocument(path)` com filter polimórfico. Manager method call. |
| **Blender** | `bpy.ops.import_*` operators per format. Operator system (direto, não enum genérico). |

**Conclusão**: action enums (estilo `EditorAction`) servem para **ações simuladas undoable** — "mover sprite", "mudar opacidade", "deletar layer" — coisas que vivem no mundo do jogo, são serializáveis para replay/network/MCP, e têm efeito uniforme. **Import/export é categoria diferente**: I/O com o sistema operacional, dados heterogêneos por formato (PSD ≠ JPEG opts), receptor específico (o importer do formato detectado, não "qualquer tool ativo"), e não-undoable da mesma forma (você fecha o documento; não "desfaz abrir").

#### Padrão canônico no PH2D (alinhado ao mercado)

```rust
// Quando o usuário clica "Open…" no menu:
let path = host.file_dialog().pick().await?;                    // shell pede ao OS
let bytes = std::fs::read(&path)?;                              // shell lê
let importer = gfx.imageio_importers
    .find_for(MagicHint::Bytes(&bytes))?;                        // registry escolhe
let img = importer.import(&bytes, &ImportOpts::default())?;     // chamada direta
spawn_sprite_from(img, gfx);                                     // resultado vai pro ECS
```

**Zero `EditorAction` envolvido.** Mesma forma do Bevy `AssetServer::load()`. O canal `EditorAction` (ADR-0040 frozen) permanece intocado para o que ele foi feito — ações simuladas. Import/export é função do **shell direto via a registry** que esta ADR já entrega em W0.T6.

#### Implicações

- **ADR-0040 §7 frozen continua intocado.** Não há amendment necessário.
- **`OneShotImageOp` permanece `{ tool_id, entity_bits }`** — sem campo payload novo.
- **Audit C-C1 é resolvido**, não deferred: a "contradição" desaparece quando reconhecemos que import/export não é assunto do `EditorAction`.
- **Audit C-H1 + C-H3 (chrome derivation)** ainda valem como tarefas reais de W1.T1 (file dialog filter + io_menu items derivados da registry), mas são UX surface, não action-bus.
- **W1 fan-out abre sem decision gate** — o W1.T0 que esta seção criava na formulação anterior fica obsoleto.

ADR-0040 §4 "sem variant per-feature" continua o norte. Import/export simplesmente nunca foi candidato a variant.

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

W0 abriu 2026-05-26. Espelha o nível de rigor do ADR-0040 §7.

| Task | Estado | Commit | Notas |
|---|---|---|---|
| W0.T1 contrato `ph2d-imageio` | ✅ | `8d8d79e` | 10 arquivos (lib + 7 módulos + arch-gate + Cargo); 13 testes verdes |
| W0.T2 `tools/ph2d-imageio-sync` | ✅ | `262a414` | Codegen lib+bin pure-std mirror de tool-sync; 7 unit tests |
| W0.T3 `ph2d-imageio-registry-init` | ✅ | `262a414` | Aggregation point + staleness gate (3 tests) |
| W0.T4 arch-gate `architecture_imageio_contract_surface` | ✅ | `8d8d79e` + remediação | 6 cap tests (Error cap raised 8→11 pós-audit A-H4) |
| W0.T5 stub `ph2d-imageio-png` | ✅ | `3db01b6` | PngImporter + PngExporter; round-trip bit-exact + byte-exact determinism (HR-5) |
| W0.T6 wiring shell desktop | ✅ | `f002b6a` | `AppGfx.imageio_{importers,exporters}`; smoke do Enio `1 importer(s), 1 exporter(s)` confirmado |
| W0.T6.5 auditoria 5-lente paralela | ✅ | `[pending]` | 4 CRITICAL + 11 HIGH + 9 MEDIUM + 12 LOW; remediação inline (3 batches) |
| W0.T7 ratificação Proposed→Accepted | ✅ | `2011fe6` + `80f3174` | Smoke pós-remediação Enio: `[553ms] imageio registries built (1 importer, 1 exporter)`; W0 FECHADA; C-C1 resolvido via market-pattern survey |
| **W1.T1** PNG-full | ✅ | `ac50809` | Multi-color-type decode, 32K bomb defence, 12 tests |
| **W1.T2** JPEG | ✅ | `4b2b657` | 8-bit RGB + quality clamp, 8 tests |
| **W1.T3** WebP | ✅ | `4423367` | Lossy decode + lossless encode (pure-Rust), 6 tests |
| **W1.T4** GIF | ✅ | `3a84b1b` | Single→Flat / multi→Animated, 7 tests |
| **W1.T5** .ph2d-native | ✅ | `7270d40` | All 5 DecodedImage variants lossless; HR-5+6+14, 13 tests |
| **ONDA 1 FECHADA** | ✅ | 5 commits | 5 format crates registrados; 81 testes verdes na família |

### Remediação pós-auditoria (2026-05-26)

Findings actionable fechados antes de ratificar (`feedback-perfection-no-deferrals` ativo):

- **A-C1** (CRITICAL) `BlendMode` 6→24 variants + `Custom(u16)` opcode-preserving — documentado data-model não-arch-gated.
- **A-C2** (CRITICAL) `Layer` ganhou `kind: LayerKind`, `effects: Vec<LayerEffect>`, `color_profile: Option<ColorProfile>`. `LayerKind` modela Pixel/Group/Adjustment/Text/Smart.
- **A-C3** (CRITICAL) `AnimFrame` ganhou `offset_xy`, `dispose_op`, `blend_op`, `transparent_index` — GIF/APNG round-trip fidelidade.
- **A-H1** `ExportOpts.format: String` → `ExportFormat` enum (14 variants + `extension()` + `mime_type()`).
- **A-H3** `DecodedImage::Vector` SMIL decision: rasteriza para `Animated` em W3 (documentado no doc do `Animated` variant).
- **A-H4** `Error` cap 8→11: `OutOfMemory`/`DimensionExceedsLimit`/`Cancelled`.
- **A-H5** `ImportOpts.color_profile_strictness` enum (Lenient/Strict).
- **A-M1** `MagicMatch::{Strong, Weak, None}` substitui `bool` em `ImageImporter::supports`; `ImporterRegistry::find_for` dispatcha por confidence (Strong > Weak).
- **A-M2** Per-layer `color_profile: Option<ColorProfile>` (PSD smart objects).
- **A-M3** `MetadataPolicy::{All, StripPrivacy, None}` (era `bool`).
- **A-L2** `MagicHint::Bytes` documenta "≥ 32 bytes ou EOF".
- **C-C1** Defer explícito W1.T0 decision gate para `EditorAction` payload (vide §2.4 revisada).
- **C-H2** `tests/architecture_register_all_alphabetical.rs` portado (3 tests: importers + exporters + Cargo deps).
- **D-H/D-M** Plan tracker atualizado com SHAs + status; PSD write estimate revisado; HEIC + qcms viability gates documentados.
- **E-H1** PNG byte-exact determinism test adicionado (HR-5).
- **E-H2** `LayerStack` + `Layer` ganharam `version: u32` (HR-14 save format migration).
- **E-M1** `Error::fluent_key(&self) -> &'static str` (HR-15 i18n surface).
- **A-L1** `impl From<std::io::Error> for Error` para `?` propagation nos satélites.

**Resolved post-Accepted (2026-05-26 market-pattern survey):**

- **C-C1** "EditorAction payload" — **RESOLVIDO via pattern survey** das engines de mercado (Unity / Unreal / Godot / Bevy / Krita / Blender). Import/export NUNCA é candidato a `EditorAction` — é chamada direta via `ImporterRegistry::find_for` no shell. ADR-0040 §7 intocado. §2.4 reescrita. **W1.T0 decision gate eliminado** — W1 abre direto.

**Deferred com decisão registrada** (não-blocker para `Accepted`):

- **C-H1, C-H3** Chrome derivation (file dialog filter + io_menu items derivados da registry) — W1.T1 follow-up (slot reservado no headroom do contrato).
- **D-M** qcms vs moxcms — W2.0.1 viability gate (§2.3.1).
- **D-M** HEIC — out-of-scope v1 com rationale + alternativas pesquisadas (§1.1).
- **C-M2** Cargo.lock merge hotspot — herdado de ADR-0040; mitigation documentada em plano.
- **C-M3** Per-format feature gate — W2/W3 follow-up (door aberta em §2.2).
