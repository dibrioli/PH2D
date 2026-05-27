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

### 2.3.1 W2.0.1 ICC pipeline — viability gate (audit D-M) ✅ RESOLVIDO 2026-05-26

**Gate W2.0.0 executado 2026-05-26 (Coord-A, ≤15min):**

| Crate | Versão | Repo | Status | Decisão |
|---|---|---|---|---|
| `qcms` | 0.3.0 | FirefoxGraphics | versão velha (Firefox internalizou; crate público dormente) | ❌ rejeitado |
| **`moxcms`** | **0.8.1** | awxkee (mantenedor ativo) | puro-Rust, Rust 1.85.0, SIMD AVX/SSE/NEON opt-in | ✅ **ESCOLHIDO** |
| `lcms2` | 6.1.1 | Little CMS binding | C binding (libc + lcms2-sys) | ❌ viola HR-1 |
| `appthere-color` | 0.1.1 | startup | v0.1 imaturo | ❌ deferred |

**Decisão**: ICC pipeline em W2.0.1 usa `moxcms` 0.8.1 puro-Rust como única dep. Mantenedor ativo (v0.8 atual indica releases recentes), Rust 1.85.0 alinha com nosso MSRV 1.92, features SIMD opt-in alinham com HR-3 (off-hot mas perf-friendly quando ativo).

**Fallback** se moxcms travar mid-implementation: implementação local mínima de ICC v2 matrix lookup (sRGB/P3/AdobeRGB hardcoded; `Custom(IccBytes)` → identidade com warning). NÃO trocar para lcms2 sem amendment HR-1.

ICC pipeline é foundational para W2 (sem ele, PSD/TIFF perdem profile preservation = data-loss). Bloqueio aqui = bloqueio de W2 inteira; o viability gate evitou descobrir isso no meio do batch.

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

### 2.6 `DecodedImage` variant policy — **collapse-when-trivial** (W3.0 gate ratificado 2026-05-26)

Quando um format container que **pode** carregar multi-frame / multi-page / multi-layer recebe um arquivo trivial (1 frame / 1 page / 1 layer flat), qual variant de [`DecodedImage`] retornar?

**Decisão**: **collapse-when-trivial**. Single-trivial-content → [`DecodedImage::Flat`] (ou `FlatHdr` para HDR). Multi-content → variant containerizado correspondente.

| Format | Single | Multi |
|---|---|---|
| **PNG / JPEG / WebP / BMP** | `Flat` (não-multi nativo) | n/a |
| **GIF** | `Flat` (1ª frame) | `Animated(Vec<AnimFrame>)` |
| **APNG** | `Flat` (1ª frame se `acTL` ausente OR `acTL.num_frames == 1`) | `Animated` |
| **TIFF** | `Flat` (single page) | `Layered` (multi-page → layers) |
| **ORA** | `Layered` com 1 layer | `Layered` com N layers |
| **PSD** | `Layered` com 1 layer (sempre) | `Layered` |
| **.ph2d-native** | preserva variant do source byte-exact | idem |

**Por quê collapse-when-trivial?** Reduz fricção no caller comum (95% do uso é flat raster); não perde informação (`Layered` com 1 layer e `Flat` carregam mesmo pixel data); rotas multi-* só aparecem quando o source as exigiu.

**Exceção ORA/PSD**: por design carregam **stack semantics** (mesmo single-layer tem layer-name + opacity + blend mode). Manter `Layered` preserva metadata que collapse para `Flat` perderia. Caller que quer flat de ORA/PSD pode chamar helper `decoded.flatten()` (W3+ amendment se cliente real demandar).

**Audit-aware**: a nova auditoria (Lens C HR-3) flaggava inconsistência entre formats. Esta amendment **alinha o invariant** sem refactor de código já shipado (a tabela acima descreve o status quo dos 9 format crates).

### 2.6.1 Golden blake3 hash scope — **single-platform pin** (W3.T0 amendment ratificado 2026-05-26)

Os 4 testes `export_golden_blake3_local_drift_pinned_macos_silicon`
(PNG/TIFF/ORA/APNG) **NÃO** são gates cross-platform. Por quê:

- `image` + `png` + `tiff` crates dispatcham paths SIMD DEFLATE per-target
  → bytes finais (still-valid) diferem entre `aarch64-apple-darwin`,
  `x86_64-unknown-linux-gnu`, `x86_64-pc-windows-msvc`.
- Pinar um único hash + rodar em matrix = "CI sempre vermelho até cap"
  (anti-padrão; inverte o "loud divergence" que o gate promete).

**Escopo atual (W3.T0)**: cada teste tem
`#[cfg(all(target_os = "macos", target_arch = "aarch64"))]`. Roda
apenas no host de dev (Mac Silicon do Enio). Função: pegar drift
silencioso quando dependências de codec bumpam local + mudam bytes
sem mudar major version.

**Escopo futuro (W3+ ou primeira divergência observada)**: substituir
o const único por tabela `&[(target, hash)]` cobrindo os 3 targets de
CI. Captura inicial = primeira run verde de cada platform; hashes
viram contratos. Entry-point por crate:
`crates/ph2d-imageio-<fmt>/src/lib.rs::export_golden_blake3_local_drift_pinned_macos_silicon`.

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
| **W1.T6** auditoria 5-lente Onda 1 | ✅ | `9127011` | 4 CRITICAL + 13 HIGH + 13 MEDIUM + 15 LOW; remediação inline (memory `feedback-perfection-no-deferrals`) — vide §5.1 |
| **W0.T6.5** auditoria 5-lente Onda 0 | ✅ | `51ea6f6` | 4C+11H+9M+12L pré-W0.T7; remediação inline (substituído `[ratification-commit]` placeholders por commits reais) |
| **W2.0.0** qcms viability gate | ✅ | `5d44e70` | Pivot moxcms 0.8.1 (puro-Rust, ativo) |
| **W2.0.1+2+3** pre-fan-out | ✅ | `d6ecda5` | ICC per-format inline policy; LayerStack pre-cooked pela audit W1.T6 |
| **W2.T1** ORA | ✅ | `3017a7d` | ZIP+XML+PNG layers; 15 blend modes; 12 tests |
| **W2.T2** TIFF | ✅ | `689e798` | 8/16-bit + CMYK + multi-page; 9 tests |
| **W2.T3** APNG | ✅ | `4694c18` | Decode multi-frame (acTL/fcTL); single-frame encode; 9 tests |
| **W2.T4** PSD | ✅ | `4f79ba3` | Decode via psd 0.3.5; **export defer W3+ (escape hatch §5.2 W2.5)**; 6 tests |
| **ONDA 2 FECHADA** | ✅ | 4 format crates + 36 tests | 9 format crates total na família imageio |
| **W2.T6** auditoria 5-lente Onda 2 | ✅ | `34605f2` | 1 CRITICAL + 8 HIGH + 13 MEDIUM + 12 LOW; remediação inline — vide §5.3 |
| **W2.T6.1** nova auditoria pós-W2.T6 | ✅ | `354b218` | 1 CRITICAL regression (PSD cap rejeita single-layer leg) + 3 HIGH residuais + 5 HIGH novos + ADR placeholders — remediação inline vide §5.4 |
| **W3 pre-gates 1+2+3** | ✅ | `f71f16a` | (1) ADR §2.6 amendment; (2) hex-baked Tier-1 fixtures (APNG multi-frame, TIFF CMYK/RGBA16, ORA group nesting); (3) golden blake3 hashes (PNG/TIFF/ORA/APNG) — vide §5.5 |
| **W3.T0** auditoria 5-lente pré-W3 | ✅ | `[remediation-pending]` | 1 CRITICAL (golden hashes single-platform vs CI matrix) + 4 HIGH + 9 MEDIUM + 12 LOW — remediação inline vide §5.5 |

### 5.5 Remediação pós-auditoria W3.T0 pré-W3 (2026-05-26)

Auditoria adversarial 5-lente sobre o commit `f71f16a` (W3 pre-gates 1+2+3) entregou 1 CRITICAL + 4 HIGH + 9 MEDIUM + 12 LOW. Fechados nesta sessão:

**CRITICAL** (Lens A + B convergem):
- Golden blake3 hashes pinados em PNG/TIFF/ORA/APNG eram single-platform Mac-Silicon mas vinculados a "HR-9 cross-platform determinism". CI matrix Linux/Windows falharia 4+ jobs por design (SIMD DEFLATE divergente). Fix: `#[cfg(all(target_os = "macos", target_arch = "aarch64"))]` + renomeados pra `export_golden_blake3_local_drift_pinned_macos_silicon` + docstrings honestas + ADR §2.6.1 amendment formaliza scope single-platform. Multi-platform pinning deferido (entry: novo function name).

**HIGH** (Lens B + E):
- APNG `set_animated(2, 0)` sem comentário sobre `num_plays=0` ↔ infinite loop. Fix: comentário explícito + nota de que loop count é irrelevante pras assertions de extração de delay/offset/blend.
- Lens E vagueness no defer de exporters lossy JPEG/WebP/GIF/BMP. Decisão registrada: gold hash de encoder lossy não é gate apropriado (encoder pode otimizar bytes sem mudar pixels) — defer permanente, não W3.

**MEDIUM**:
- TIFF CMYK test só cobria K=0 (pure-cyan). Fix: arm 2 com K=128 + C=M=Y=0 verificando que K-attenuation funciona; assertions ±1 pra rounding flavour.
- TIFF RGBA16 test só cobria mid-value 0x8080. Fix: sweep de 3 arms (endpoint-low 0x0000, mid 0x8080, endpoint-high 0xFFFF) com ±1 tolerance per channel.
- ADR §5 sem linha pra `f71f16a` (W3 pre-gates). Fix: linha nova + nova linha W3.T0 (auditoria atual).
- ADR §5 W2.T6.1 placeholder `[remediation-pending]` corrigido pra `354b218`.
- ADR §2.6 não disclosure-ava "single-platform pin, CI red expected". Fix: §2.6.1 amendment dedicada acima.
- Plan `2026-05-imageio-waves.md` W3 status não atualizado pós-gates. Fix: §5.5.1 atualiza plan.

**LOW**:
- `let _ = layer.visible` em PSD fixture test era dead-code-as-validation. Fix: substituído por comment explicando que o campo é populado mas o valor exato da fixture upstream não é nosso contrato.
- Outros LOWs (cosméticos: typos, naming) considerados sub-threshold pra remediação inline.

**Defers permanentes (não-bloqueantes)**:
- Multi-platform golden hash pin: até primeira divergência cross-OS observada na CI motivar custo de captura por target.
- Lossy exporter goldens (JPEG/WebP/GIF/BMP): encoders lossy podem reotimizar bytes sem regredir pixels — gold hash não é semanticamente apropriado.
- Lens C Tier-1 fixture expansion (APNG dispose_op variants, TIFF Gray8/A8/Gray16/GrayA16, ORA stack.xml malformed branches): expansion incremental, agendada pra W3.T1+ como tickets vivos (não pré-W3 gates).

**Total Onda 2 pós-W3.T0**: 41+ tests verdes em PNG/TIFF/ORA/APNG/PSD (incluindo Mac-only goldens).

### 5.4 Remediação pós-nova-auditoria W2.T6.1 (2026-05-26)

Nova auditoria adversarial sobre o commit W2.T6 (`34605f2`) entregou 1 CRITICAL regression + 5 HIGH (3 residuais + 2 novos) + 8 MEDIUM + 6 LOW. Fechados nesta sessão:

**CRITICAL regression:**
- **H-N1** PSD cap `MAX_PSD_TOTAL_BYTES = 2 GiB` rejeitava single-layer PSD legítimo no canvas máximo (32K² × 4 = 4 GiB > 2 GiB). Fix: cap aplicado APENAS quando `n_layers > 1`; renomeado `MAX_PSD_MULTI_LAYER_TOTAL_BYTES = 4 GiB`. Single-layer continua bounded por `MAX_RASTER_DIMENSION` alone.

**HIGH residuais (gaps na remediação anterior):**
- **C-1 residual** APNG substring match `windows(4).any(|w| w == b"acTL")` false-positive em PNG com tEXt/iTXt contendo "acTL" string. Fix: chunk parser proper length-prefixed walk `[length:u32 BE][type:4][data][crc:4]` até IDAT. Test sentinel cobre tEXt-com-"acTL" case explicitamente.
- **H-7 residual** PSD cap só checava pós-`from_bytes` que já alocou. Fix: `src.len() > MAX_PSD_INPUT_BYTES = 2 GiB` cap pre-parse + ainda OOM check em multi-layer fan-out pós-parse.
- **H-8 residual** PSD pin `=0.3.5` sem assertion-guard. Fix: unit-test `map_psd_blend_mode_via_debug_covers_24_variants` direto com 24 strings literais — quebra LOUDLY se Debug repr drift.

**HIGH novos:**
- **H-N2** PSD cap error usava `DimensionExceedsLimit` quando o problema é layer count × canvas (não canvas dim). Fix: trocado para `OutOfMemory` (semanticamente correto).

**MEDIUM novos (e ORA pendente):**
- **M-N3** TIFF BigTIFF magic 4-byte sem verify bytes 4-7. Fix: `is_bigtiff_magic` strict — checa offset-size 0x0008 + reserved 0x0000 conforme BigTIFF spec.
- **M-N4** ORA spec version `"0.0.6"` deve ser `"0.0.5"` (openraster.org/baseline atual). Fix: `ORA_SPEC_VERSION = "0.0.5"`.
- **M-N2** ORA control char strip sem regression-guard test. Fix: `xml_escape_strips_control_chars_preserves_whitespace` test.

**ADR/plan placeholders** (audit Lens E):
- ADR §5 row `W2.T6 [remediation-commit]` → `34605f2`.
- ADR §5 row `W1.T6` (era `[remediation-commit]`) → `9127011`.
- Plan `W0.T4 [remediation-pending]` → `51ea6f6`.
- Plan `W0.T6.5 [remediation-commit-pending]` → `51ea6f6`.

**Deferred com concrete entry-points** (audit Lens E):
- **HR-3** `DecodedImage` variant policy → **W3.0 pre-fan-out adiciona §2.6 amendment** com policy global (collapse-to-Flat-when-trivial vs preserve-container-type). Hard deadline.
- **HR-9** cross-platform golden bytes → **antes de W3.T1 ratificar — bloqueia ratificação Onda 3** (matriz CI Linux/macOS/Windows).
- **HR-15** detail strings inglês → **bloqueia W4 abertura**; `Error::user_facing() -> FluentMessage { key, args }` separa dev-log de UI.
- **HR-17** examples Luau → **rótulo `won't-fix v1`** explícito (entram quando scripting integration milestone abrir, sem deadline).
- **Test coverage fixtures real** → **antes de W3.T1 abrir** (hex-baked APNG 2-frame + PSD 1-layer fixture + TIFF CMYK).

**Test count post-nova-remediação:**
- ORA: 14 (era 12; +2: xml escape control chars + spec version pin)
- TIFF: 10 (BigTIFF spec strict — substituiu test sem mudar contagem)
- APNG: 9 (chunk parser proper — substituiu test)
- PSD: 8 (era 7; +1: map_psd_blend_mode_via_debug coverage)
- Total Onda 2 pós-W2.T6.1: 41 tests verdes

### 5.3 Remediação pós-auditoria Onda 2 (2026-05-26)

Auditoria adversarial 5-lente sobre Onda 2 entregou 34 findings. Fechados nesta sessão (memory `feedback-perfection-no-deferrals`):

**CRITICAL:**
- **C-1** APNG vs PNG magic collision — APNG `supports(Bytes)` agora sniffa chunk `acTL` (busca window 64 KB). Sem `acTL` → `None`; com → `Strong`. Plain PNG dispatch volta ao decoder dedicado. Sentinel test `supports_only_strong_when_actl_chunk_present`.

**HIGH:**
- **H-1** ORA ZIP magic Strong → Weak. ZIP signature é genérico (KRA/EPUB/JAR/ODT). Strong only via Extension match.
- **H-2** TIFF BigTIFF magic detection (`II+\0` / `MM\0+`) → Strong + import retorna `Error::Unsupported` actionable apontando classic TIFF / `.ph2d-native` para > 4 GB.
- **H-6** PSD PSB magic (`8BPS\0\x02`) → Strong + import retorna `Error::Unsupported` actionable apontando PSD save / `.ph2d-native`.
- **H-7** PSD OOM defense: `MAX_PSD_TOTAL_BYTES = 2 GiB` cap em `n_layers × canvas_w × canvas_h × 4`. PSD com 16K × 16K × 200 layers (= 200 GB) refusada antes do iterate.
- **H-8** PSD `Cargo.toml` pin `=0.3.5` exato (Debug-name match hack fragilidade).

**MEDIUM:**
- ORA: `xml_escape` agora strips XML 1.0-illegal control chars (U+0001..U+001F exceto `\t \n \r`). Layer names com BEL/ESC produziam XML inválido.
- ORA: `ORA_SPEC_VERSION` const promovido do magic-string inline; reutilizável em test.

**Deferred com decisão registrada** (não-blocker; entry-point identificado):

- **H-3** TIFF multi-page diff dims (max() canvas) — documentado, W3+ amendment se cliente real aparecer.
- **H-4** APNG canvas dims dropped (sub-frame rect) — exige amendment `DecodedImage::Animated { canvas, frames }`; W3+ quando timeline UI primeiro cliente.
- **H-5** APNG palette+tRNS expansion — edge case raro; W3+ se feedback real.
- **HR-1** `ImportOpts::Strict` ignorado em 9 crates — W2.0.1 ICC pipeline materializa (Strict só faz sentido com ICC ativo).
- **HR-2** `ExportOpts::preserve_layers` lido por zero crates — W2+ flatten path.
- **HR-3** `DecodedImage` variant policy inconsistente — documentar no contrato em ADR amendment quando 1 cliente real reclamar.
- **HR-9** Cross-platform determinism golden bytes — W3+ CI matrix.
- **HR-13** PSD `MemoryBudget` declaração — `MAX_PSD_TOTAL_BYTES` é budget implícito; explicit declaration W3+.
- **HR-15** detail strings inglês — mesma deferral W1 (ADR follow-up).
- **HR-17** examples Luau — scripting integration milestone.
- **Test coverage gaps** (APNG multi-frame fixture, PSD layer fixture, TIFF CMYK/16-bit, ORA malformed XML) — fixtures binárias hex-baked custosas; W2.0.1 ou W3+ com fixtures reais.

### 5.2 W2 abertura (amendment §W2, 2026-05-26)

Onda 2 (profissional 2D) abre com 3 decisões pré-batch:

- **ICC policy per-format inline** (W2.0.1): cada W2 format crate que carrega ICC (TIFF iCCP, PSD ImageResource 1039) adiciona `moxcms` 0.8.1 dep + parse inline. Não há `ph2d_imageio::icc` central crate hoje. Refactor para API unificada acontece quando 2º cliente independente aparecer (TIFF + PSD ambos shipados); evita over-engineering pré-cliente. ColorProfile já contém todos os variants necessários (Srgb/DisplayP3/AdobeRgb/ProPhoto/Custom).
- **LayerStack pré-cooked pela auditoria W1.T6**: `BlendMode 24 + Custom(u16)`, `LayerKind {Pixel/Group/Adjustment/Text/Smart}`, `LayerEffect`, per-layer `color_profile: Option<…>`, `version: u32` (HR-14). PSD/ORA W2.1 chegam encontrando o data model pronto sem amendment adicional.
- **HEIC permanece descartado** §1.1 (HEVC patent-heavy, sem decoder puro-Rust maduro). Reopen W4 quando upstream chegar.

W2.1 batches:
- **Batch A** (3 slots paralelos): `ph2d-imageio-ora` + `ph2d-imageio-tiff` + `ph2d-imageio-apng`.
- **Batch B** (1 slot dedicado, sequencial): `ph2d-imageio-psd` (PSD write greenfield; 2-3 sessões + escape hatch ADR-0054.W2.5 se slip > 1 semana).

### 5.1 Remediação pós-auditoria Onda 1 (2026-05-26)

Auditoria adversarial 5-lente sobre Onda 1 entregou 45 findings actionable. Fechados nesta sessão pré-W2:

**CRITICAL:**
- **E-1** inner version drift silenciosa em `LayerStackV1`/`LayerV1` — `schema::validate_v1_inner_versions` walk após postcard decode + test cobre `version=7` inner.
- **A-Crit1** APNG silent first-frame em PNG — documentado em crate doc + test sentinel + amendment do plan (W2.T3 ships `ph2d-imageio-apng`).
- **A-Crit2** GIF `collect_frames` sem cap — `MAX_FRAMES = 1024` cap antes do `into_frames` collect; documentado streaming W2+.
- **C-C1** Cargo.lock uncommitted — commitado junto com a remediação (débito anterior dos 5 commits W1 documentado: bisect quebrado nesse range; daqui em diante OK).

**HIGH:**
- **B-H2** `MAX_DIMENSION` redeclarada 4× — hoisted para `ph2d_imageio::MAX_RASTER_DIMENSION` (single source of truth + compile-time sanity).
- **B-M1** `From<std::io::Error> for Error` unused — removido (era encouragement para refactor errado).
- **D-E2** HR-5 byte-exact tests missing em JPEG/WebP/GIF — 3 tests adicionados (~10 LOC cada).
- **D-E4** `Vec::with_capacity(pixels.len() * 4)` sem overflow guard — `checked_mul` em 4 encode paths (PNG/JPEG/WebP/GIF) + 1 helper GIF (`anim_frame_to_image_frame`).
- **`.ph2d-native` L1** `MAX_PAYLOAD_LEN = 4 GiB` exato overflow 32-bit — hoisted para `MAX_PH2D_PAYLOAD_LEN = u32::MAX as u64`.
- **`.ph2d-native` L1** ICC profile sem cap — `MAX_ICC_PROFILE_LEN = 4 MiB` validado em `validate_color_profile_v1`.
- **`.ph2d-native` L1** version=0 vs future schema — arm dedicado `0 => Error::Decode("uninitialized")` + test.

**Deferred com decisão registrada** (não bloqueador):
- **A-H1** JPEG sRGB-assumed sem ICC parsing — W2.0.1 quando ICC pipeline lands.
- **A-H2** WebP lossy → lossless re-export warning — W2+ junto com lossy encoder.
- **A-H3** GIF `dispose_op` semantic loss — W2 (image 0.26 ou `image-gif` direct dep).
- **B-H1** Truncated heuristic por substring — aceitável até `image` 0.26 surface `ImageError::IoError(kind)`.
- **B-M2** `ImportOpts.color_profile_strictness` Strict ignorado — W2.0.1 ICC pipeline materializa.
- **B-M3** GIF single-frame round-trip Flat-vs-Animated ambiguidade — W2 spec choice.
- **B-M4** WebP animated first-frame fallback silent — W2 amendment.
- **B-M5** `ExportFormat::from_extension(&str)` — W1.T7 chrome integration helper.
- **D-E3** HR-15 detail strings inglês hardcoded em `Display` — ADR follow-up: `Error::user_facing()` separa dev-log de UI.
- **D-E5** HR-17 examples Luau import/export — W1+ scripting integration milestone.
- **D-E6** HR-13 memory budget surface — re-export caps via `ph2d_imageio::limits` parcial; ADR §addendum ratifica "imageio off-hot, budget vive no shell AssetDb".
- **E-H5** qcms hard deadline — adicionar W2.0.0 viability gate ao plan (faz parte deste commit).
- **C-M3** per-format feature gate — W2/W3.

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
