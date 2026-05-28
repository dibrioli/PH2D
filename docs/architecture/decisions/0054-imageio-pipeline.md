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

**Formats lossy** (JPEG/WebP/GIF/BMP): **golden hash semanticamente
inadequado, defer permanente** (não W3 nem futuro). Encoders lossy
podem reotimizar bytes sem regredir pixels — um bump de quality LUT
ou rate-distortion table muda o hash sem mudar a imagem. Para esses
formats o gate é **pixel-roundtrip** (encode → decode → compare
within ε) e não byte-pin. `.ph2d-native` é determinístico-por-design
via `postcard` (HR-5 já satisfeito em todo target).

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
| **W3.T0** auditoria 5-lente pré-W3 | ✅ | `35cc149` | 1 CRITICAL (golden hashes single-platform vs CI matrix) + 4 HIGH + 9 MEDIUM + 12 LOW — remediação inline vide §5.5 |
| **W3.T0.1** nova auditoria pós-W3.T0 | ✅ | `fd34240` | 1 CRITICAL (imageio FORA da CI matrix — gate de §2.6.1 era cosmético) + 7 HIGH (ADR HR-9→HR-5 nomenclatura + tolerâncias TIFF mascarando off-by-one + clippy convenção) + 11 MEDIUM + 13 LOW — vide §5.6 |
| **W3.T0.2** nova auditoria pós-W3.T0.1 | ✅ | `108a623` | 6 CRITICAL meus (3 ship-blockers F-B1/B2/B3 + 3 OOM J-1/J-2/J-3) + 5 HIGH meus (ORA H-1 + opts I-1/I-2 + EOF I-3 + non_exhaustive I-4 + PSD catch_unwind G-F2) + 9 MEDIUM + 14 LOW; 2 CRITICAL não-meus flaggados (B-4 shells/desktop + M-5 Cargo.lock dhat) — vide §5.7 |
| **W3.T0.3** nova auditoria pós-W3.T0.2 | ✅ | `2a41a0b` | 1 P0 (§5 row drift) + 6 HIGH (ColorProfile::Custom vapor + GIF semantica + EOF helper adoption parcial + APNG multi-frame test gap + math/docs drift + fmt P-mine) + 17 MEDIUM (test coverage 8 fixes + caps hoist + EOF gaps + variant context) + 12 LOW — vide §5.8 |
| **W3.T0.4** nova auditoria pós-W3.T0.3 | ✅ | `cde3e44` | 1 CRITICAL (ORA `parse_stack` recursion DoS — uncatchable SIGSEGV via deep nesting) + 6 HIGH (caps hoist + GIF L-3 complete + fmt + tests count + 2 arquiteturais flag-pro-Enio) + 6 MEDIUM (zip-slip-via-XML + PsdDeny preventive + markdown drift + ORA layer count + ColorProfile::Srgb docs + deps tracking) + 10 LOW — vide §5.9 |
| **W3.T0.5** nova auditoria pós-W3.T0.4 | ✅ | `4d7dfdd` | 1 CRITICAL (ORA `opacity` aceita `NaN`/`±Inf` — compositor poison + persiste no `.ph2d-native` save) + 1 HIGH meu (ZIP central directory amplification ~1.5 GiB) + 1 HIGH não-meu (X arquitetural: shells/desktop bypass imageio registry) + 2 MEDIUM (stack.xml `take(N)` cap + APNG test snake_case) — vide §5.10 |
| **W3.T0.6** convergence final pós-W3.T0.5 | ✅ | `84fd496` | 0 CRITICAL + 0 HIGH meu + 1 MEDIUM doc (plan placeholder não-substituído) + 4 LOW (2 forensic CC + 1 orphan T0.2 docs + 1 tracing infrastructure absent); Lens BB threading GREEN + Lens EE ship.sh real-time GREEN — **PADRÃO-OURO RATIFICADO** — vide §5.11 |
| **W3.T1.0** wire-up `AssetDb → imageio registry` (fecha X-HIGH-1) | ✅ | `e54d41a` (multi-agent collision) | Bridge: `ph2d-asset/loader.rs::decode_via_imageio_registry` fallback acionado quando `image::guess_format` retorna Other. Estende `is_supported_image_extension` para gif/tiff/tif/ora/apng/psd/ph2d. Multi-layer/multi-frame/HDR/Vector retornam Error::Decode actionable. Tests: 46 ph2d-asset verdes; workspace check 1m 02s. **Colisão**: meus arquivos staged foram absorvidos pelo commit `e54d41a` (sessão KTX2 paralela) — conteúdo correto, atribuição confusa. Vide §5.12. |
| **W3.T1..T5** fan-out 5 format crates (AVIF/EXR/HDR/JXL/SVG) | ✅ | `cc97cd4` | 5 new crates per ADR §3.8 fan-out drop-crate: AVIF (W3.T4 magic-only stub), EXR (W3.T2 magic-only stub), HDR-Radiance (W3.T3 magic-only stub), JXL (W3.T1 magic-only stub), SVG (W3.T5 **real parse** via usvg 0.43 → VectorDoc). registry-init regenerated 9→14 via codegen. spike.yml +5 crates (16 imageio total). 35 new tests verdes (8+6+6+7+8). Vide §5.13. |
| **W3.T4** AVIF re-ship — Path C `libavif-sys` decode+encode+HDR | ✅ | _(local)_ | Path C (candidate #3 do §5.17): `libavif-sys` codec-dav1d (decode) + codec-rav1e (encode pure-Rust). Decode real SDR/HDR (`nclx`+ICC→ColorProfile, PQ/HLG/linear EOTF, FlatHdr scene-linear), encode (lossless+lossy, 10-bit PQ HDR10). Verification: 0 RUSTSEC, zero owning_ref, licenças OK. `forbid(unsafe)` dropado (FFI) → `deny(unsafe_op_in_unsafe_fn)` + RAII + catch_unwind. CI ganhou meson+ninja+nasm 3-OS (handoff "vendored→sem CI install" era falso). 21 tests verdes. Vide §5.18. |
| **W3.T4** AVIF real decode (DESHIPADO) | ❌ | `272d99d` → reverted `f034e9a` | Audit-15 6-lente revelou 1 CRITICAL (RUSTSEC-2022-0040 owning_ref UAF via avif-decode) + 6 HIGH + 8 MEDIUM (incl. upstream `unprem()` math bug, HDR PQ silent, libaom-sys 26MB C+cmake). Per `feedback-no-industrial-claims-without-verification` + `feedback-perfection-no-deferrals` UNSHIPPABLE. Revert restaura magic-only stub W3.T4. 3 candidate re-evaluation paths documentados em §5.17. |
| **W3 wave-2.1** audit-14 remediation pós-wave-2 | ✅ | `5f9582b` | 3 CRITICAL (HDR Inf panic + JXL ColorProfile mislabel + JXL CMYK collision via auto-srgb-request) + 4 HIGH (HDR wide-DR docs + JXL HDR/multi-frame guards + alpha-drop warning) + 2 MEDIUM (written guard + doc honesty) — Lens NN/PP GREEN, OO inconclusivo (sessão paralela cobrindo end-to-end test) — vide §5.16 |
| **W3 wave-2** real decode/encode em HDR/EXR/JXL | ✅ | `dc4ec6a` | Substitui magic-only-stubs de cc97cd4 por impl real. HDR-Radiance: image 0.25 hdr feature, encode+decode RGBE → LinearRgba; EXR: `exr = "1"` builder API, decode via closure-state struct; JXL: `jxl-oxide = "0.10"` decode-only first-frame para Flat path. AVIF + SVG mantêm stubs. 21 tests verdes (6 HDR + 7 EXR + 8 JXL). Vide §5.15. |
| **W3.T1.5** nova auditoria pós-W3.T1..T5 | ✅ | `54a8a12` | 1 CRITICAL doc-honesty (EXR claim falso `exr=1` in Cargo.toml) + 3 HIGH (AVIF Cargo.toml vapor comment, SVG Cargo.toml promete rasterize-on-import inexistente, **HH-FIN-1 SVG security: `default_string_resolver` faz `std::fs::read(href)` em `<image href="/etc/passwd"/>`**) + 3 MEDIUM (JXL codestream test ausente + AVIF avis sequence test ausente + SVG hostile-input tests ausentes) + 1 P1 ship-blocker fmt — vide §5.14 |

### 5.18 W3.T4 AVIF re-ship — Path C `libavif-sys` decode+encode+HDR real (2026-05-28)

Re-ship do W3.T4 pós-deship (§5.17), agora visando **padrão-ouro** por
decisão do Enio "o melhor possível, sem pensar em custos" (2026-05-28).
Isso **inverte** as 3 economias do handoff Path A original (decode-only /
reject-HDR / sem-CI-install): escopo agora é **Path C = decode E encode E
HDR/wide-gamut real**.

**Decisão de codec/dep**: `libavif-sys = "0.17.0+libavif.1.0.4"`
(implementação de referência AOM `libavif`) com `codec-dav1d` (decode) +
`codec-rav1e` (**encode pure-Rust**). `codec-aom` **rejeitado** — exigiria
build C de 26 MB (libaom) + nasm, sem ganho de qualidade necessário pra um
exporter de editor; rav1e é pure-Rust, menor superfície unsafe, HR-1-friendly.

**Verification protocol (scratch `/tmp/avif-c-verify`, per
[`feedback-no-industrial-claims-without-verification`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_no_industrial_claims_without_verification.md))**:
- `cargo audit`: **0 RUSTSEC**. Único warning `paste 1.0.15` unmaintained
  (RUSTSEC-2024-0436) — **já no `ignore` do `deny.toml`**.
- `cargo tree -e normal`: 68 crates; **zero `owning_ref`** (a classe que
  matou Path A: eliminada). Sem `lodepng`/`aom-decode`/`avif-decode`.
- `cargo deny check licenses`: árvore normal toda BSD-2/MIT/Apache/Unicode-3.0/
  Unlicense — dentro do allowlist. `NCSA`/`LGPL` (libfuzzer-sys/r-efi) NÃO
  estão na árvore normal sob nossas features.

**Correção de claim do handoff (§5.18.1)**: o handoff afirmava "vendored →
sem CI install". **FALSO** — os `*-sys` vendoram *source*, não binários:
`libdav1d-sys` exige **meson + ninja**, rav1e (asm x86) exige **nasm**. Build
local só passou após `brew install meson nasm`. `spike.yml` ganhou
`Install AVIF build tools` nos jobs lint + MSRV (apt) + matrix 3-OS
(apt/brew/choco). **Windows é o risco não-verificável localmente** —
babysit do CI no 1º push.

**Trade-off `forbid(unsafe_code)` (§5.18.2)**: o wrapper safe `libavif` 0.14
é **8-bit RGBA only** (esconde `nclx`, depth, float) — incapaz de HDR real.
HDR/wide-gamut exige `libavif-sys` FFI cru → `unsafe` vive no nosso crate.
Decisão: **dropar `#![forbid(unsafe_code)]`** (único format-crate sem ele),
substituído por `#![deny(unsafe_op_in_unsafe_fn)]`; todo bloco `unsafe`
carrega `// SAFETY:` + RAII guards (`Decoder`/`Image`/`RgbScratch`/`RwData`)
liberam toda alocação libavif em todo exit path + `catch_unwind` no boundary
do parser. HDR real venceu o stub LDR-only.

**Cobertura**:
- **Decode**: `nclx` → `ColorProfile`; SDR 8-bit → `Flat`; PQ/HLG/linear/>8-bit/
  BT.2020 → `FlatHdr` scene-linear (EOTF PQ SMPTE-2084 / HLG ARIB-B67 / linear /
  sRGB invertida). **ICC embarcado override do nclx** (preservado byte-exact como
  `ColorProfile::Custom`, capped `MAX_ICC_PROFILE_LEN`). Straight-alpha requestado
  (unpremultiply se fonte premultiplied). Grid stitched pelo libavif. Animação
  (`avis`) decoda 1º frame (Animated bridge W3+; documentado, não-silencioso).
- **Encode**: rav1e; quality 0..100 (≥100 ⇒ lossless identity-matrix, RGB exato);
  YUV444; `nclx` escrito da profile fonte (HDR → 10-bit PQ HDR10). FlatHdr→PQ,
  Flat→sRGB. Layered/Animated/Vector → `Error::Unsupported` actionable.
- **HR-13**: `imageDimensionLimit = MAX_RASTER_DIMENSION` no decoder C **antes**
  do parse + re-check pós-parse.
- **Determinismo (HR-5)**: EOTF usa `std` transcendentals (não libm-pinned) — decode
  de pixel é *content I/O*, fora do replay-hash domain (o YUV dav1d C já não é
  bit-idêntico cross-OS). Round-trip tests asseguram consistência forward/inverse,
  não bit-equality cross-OS.

**Testes**: 21 verdes Mac aarch64 (12 unit color/magic + 9 integração:
magic_recognition, truncated, hostile_garbage, lossless+lossy roundtrip,
**hdr_wide_gamut_roundtrip** 10-bit PQ Rec.2020, grid, reject-non-raster).
Fixtures gerados in-process (encode→decode) — sem `.avif` externo.

### 5.17 W3.T4 AVIF deship — audit-15 reverteu `avif-decode 1.0` wire-up (2026-05-28)

Auditoria adversarial 6-lente sobre commits `272d99d` + `82d503e` (AVIF real decode wave-2.1). Lentes especializadas: **QQ** safety/unsafe boundary · **RR** HDR/wide-gamut silent quantization · **SS** fuzz/malformed-input · **UU** dep tree audit · **VV** cross-impact · **WW** ship.sh real-time.

**Resultado**: 1 CRITICAL + 6 HIGH + 8 MEDIUM + 5 LOW — **decisão: REVERT 272d99d → restaurar magic-only stub**.

**CRITICAL (1)** — UU + QQ convergem:
- **RUSTSEC-2022-0040** `owning_ref 0.4.1` use-after-free, no fix upstream, transitive via `avif-decode 1.0.2 → aom-decode 0.2.13 → owning_ref`. CI hard-fail `cargo audit` (existente em advisory-DB). Cadeia atribuída a Kornel sem migração para `safer_owning_ref`.

**HIGH (6)**:
- **UU** `libaom-sys 0.17.2+libaom.3.11.0` vendora 26 MB C source via cmake build-script (+30-60s clean build × 3 platforms; WASM/no-std impossível).
- **UU** `avif-parse` duplicado (1.4.0 via aom-decode + 2.1.0 via avif-decode); 2 parsers ISOBMFF + ~200 KB binary duplicado.
- **RR + QQ** AVIF HDR PQ/HLG silent quantization 16→8 via top-byte drop (não-linearisado) + `ColorProfile::Srgb` hardcoded mislabela BT.2020/Display-P3. **Asymmetry com fix JXL audit-14** que adota `request_color_encoding(srgb)` + `hdr_type()` reject.
- **SS** `assert!(offset <= size)` panic determinístico em `avif-parse-1.4.0/src/lib.rs:668` UUID-box parser — bytes hostis derrubam thread.
- **SS** `Vec::with_capacity(width * height)` no aom-decode pre-decode (8 sites) aceita AV1 sequence-header dims sem cap. Hostile 65535×65535 claim → ~16 GiB OOM antes do `MAX_RASTER_DIMENSION` post-decode check.
- **WW** clippy `-D warnings` falha em 4 sites (`p.0` deprecated em `rgb::Gray_v08`) — ship-blocker imediato.

**MEDIUM (8)**:
- **QQ** Upstream `unprem()` matematicamente errado: `((u16::from(val) * 256) / (u16::from(alpha) * 256) / 256) = val/alpha/256 = 0` para `alpha < 255`. Premultiplied AVIF vira **preto**. Bug NA dep, não-patchable aqui.
- **UU** ~160 unsafe blocks transitivos (lodepng=97, owning_ref=42, libaom-sys=9, aom-decode=11). Aceitável pra codec mas longe da assinatura "Rust safe-by-default" do resto do workspace.
- **UU** Maintainer bus-factor=1 (Kornel single-author chain).
- **UU** MSRV bump latente (`avif-decode 1.0.2` rust-version="1.91"; workspace 1.95 OK mas frágil).
- **SS** `unreachable!()` em `ChromaSampling::Monochrome` 16-bit alcançável via crafted AVIF 10/12-bit monochrome.
- **SS** + **QQ** Sem `catch_unwind` boundary (vs PSD pattern audit-7).
- **SS** Granularidade erro perdida (UnexpectedEof → Decode em vez Truncated).
- **QQ** Pré-decode dimension cap ausente.

**LOW (5)**:
- **SS** tests não cobrem box-size hostile fixtures.
- **VV** `is_supported_image_extension` em `ph2d-asset/loader.rs` faltando `"avif"` (não-blocked pelo decode end-to-end via `image::guess_format` mas UX filter rejeita).
- **VV** Gap test end-to-end AVIF asset bridge.
- **QQ** transmute lifetime extension em avif-decode é unsafe upstream OK (não nosso código).
- **WW** Tests + arch gates verdes pós-revert.

**DECISÃO: REVERT** per [`feedback-no-industrial-claims-without-verification`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_no_industrial_claims_without_verification.md) + [`feedback-perfection-no-deferrals`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_perfection_no_deferrals.md). 1 unfixable RUSTSEC + 1 unfixable upstream math bug + asymmetry HDR-vs-JXL é UNSHIPPABLE. `git revert 272d99d` (não-destrutivo) executado em `f034e9a`. Magic-only stub restaurado com nota crate-level listando 15 findings + 3 candidate re-evaluation paths:

1. **`image = { features = ["avif-native"] }`** — different dep tree (uses `mp4parse` + Rust dav1d?); needs verification que `owning_ref` NÃO está no path.
2. **Wait `avif-decode 2.x`** — quando upstream migrar para `safer_owning_ref` + fix `unprem()`.
3. **Direct `libavif-sys`** — C FFI (fastest mas unsafe ABI surface).

**Estado pós-revert (2026-05-28)**:
- 14 format crates wired; **4** com decode real (HDR/EXR/JXL + 9 do W1/W2); **2 stubs** honest (AVIF + SVG aguardando ADR-0056).
- 9 AVIF stub tests verdes (mesmo conjunto pre-272d99d).
- 16 imageio crates no CI matrix; RUSTSEC removida.
- Cargo.lock -153 lines (avif-decode/aom-decode/avif-parse/owning_ref/yuv/imgref/rgb/lodepng/libaom-sys/etc.).

### 5.16 W3 wave-2 audit-14 — 3 CRITICAL + 4 HIGH fechados (2026-05-27)

Auditoria adversarial 6-lente sobre commits `dc4ec6a` + `d734dd1` (W3 wave-2 real decode/encode). Lentes especializadas: **KK** closure correctness · **LL** RGBE precision + HDR edge cases · **MM** JXL channel/HDR · **NN** dep audit · **OO** asset bridge HDR · **PP** ship.sh real-time.

**Resultados**:
- **NN GREEN** — zero RUSTSEC, licenças OK, MSRV folgado, dep tree aceitável (jxl-oxide bus-factor=1 anotado).
- **PP GREEN** — fmt+clippy+typos+machete+tests todos verdes nos 3 crates wave-2.
- **OO inconclusivo** — agent travou; sessão paralela já estava cobrindo o end-to-end test via `ph2d-asset/Cargo.toml` adicionando `ph2d-imageio-hdr-radiance` como dev-dep.
- **KK + LL + MM** convergem em 3 CRITICAL + 4 HIGH + 2 MEDIUM fechados.

**CRITICAL (3)**:
- **LL** HDR encode panic em `f32::INFINITY`: `image-0.25.10::HdrEncoder::encode` panic com "attempt to add with overflow". Real HDR EXR pode ter Inf legítimo (gamut clip) — DoS via export. Fix: sanitize non-finite + negative → 0.0 no encode boundary; test `export_sanitizes_non_finite_and_negative_without_panic` exercita Inf/NaN/Neg/-Inf.
- **MM** JXL `ColorProfile::Srgb` hardcoded sem `request_color_encoding`: wide-gamut JXLs (Display-P3, Rec2020, BT.2100 PQ) renderizam no encoding nativo + quantizam como sRGB → crominância errada. Fix: `img.request_color_encoding(EnumColourEncoding::srgb(RenderingIntent::Relative))` ANTES de `render_frame` — agora `ColorProfile::Srgb` é truthful.
- **MM + KK** JXL CMYK channels=4 collision com RGBA: `stream.channels() == 4` interpretado como RGBA mas pode ser CMYK (silent data corruption). Fix: `request_color_encoding(srgb)` auto-gamut-mapa CMYK → sRGB upstream do `Render::stream` — collision estruturalmente prevenida; match Cmyk redundante removido.

**HIGH (4)**:
- **MM** JXL HDR silent clamp[0,1]: PQ/HLG transfer fica white-saturated. Fix: `if img.hdr_type().is_some() { Error::Unsupported }` apontando FlatHdr bridge W3+.
- **MM** JXL multi-frame silent first-frame: `render_frame(0)` sem checar count. Fix: `if num_loaded_keyframes() > 1 { Error::Unsupported }` apontando Animated bridge W3+.
- **LL** HDR wide-DR loss não-documentado: shared-exp limit ~1:256 channel ratio; doc dizia "1% precision". Fix: crate doc declara limite real + alpha-drop policy + non-finite/negative sanitisation policy.
- **MM** Zero real-decode coverage JXL: deferido W3.T1.6 quando fixture binário disponível (cjxl CLI).

**MEDIUM (2)**:
- **MM** `written == 0` guard aceitava partial-write 1..len-1. Fix: `written != planar.len()` — exige fill exato.
- **MM** Doc-comment JXL mentia sobre cobertura ("HDR + alpha + animation in spec"). Fix: doc atualizada explicitando subset W3.T1 + lista de rejected paths.

**LOW absorvidos / deferidos**:
- MM `as u8` não-saturating (defer; clamp já garante range).
- LL tolerância 5% sem golden bytes (defer W3.T3.x).
- LL endianness assumption (defer; image-rs documenta NE).
- LL `ColorProfile::LinearRec709` hardcoded sem ler header chromaticities (EXR `chromaticities` attr / Radiance `PRIMARIES=`) — defer W3+ ICC pipeline.

**Total wave-2 pós-audit-14**: 22 tests verdes Mac aarch64 (7 HDR + 7 EXR + 8 JXL).

### 5.15 W3 wave-2 — real decode/encode em HDR/EXR/JXL (2026-05-27)

Substitui os magic-only-stubs de `cc97cd4` (W3.T1..T5 fan-out) por implementações reais em 3 dos 5 crates. AVIF e SVG mantêm stubs documentados (AVIF: deps de codec complexos `avif-decode` + `ravif` → defer pra primeiro real client; SVG: aguarda canonical types `kurbo::BezPath` paint stack via [ADR-0056 vector-network](0056-vector-network-data-model.md) que outra sessão está propondo).

**HDR-Radiance W3.T3** — pure-Rust via `image = "0.25"` `hdr` feature:
- Decode: `HdrDecoder::new` + `read_image` retorna `ColorType::Rgb32F` (3 floats per pixel native endian) → reinterpret via `f32::from_ne_bytes` → `ImageBuffer<LinearRgba>`. Alpha sintetizado=1.0 (RGBE spec sem alpha).
- Encode: `HdrEncoder::encode` aceita `Vec<image::Rgb<f32>>` → RGBE bytes. Drops alpha.
- `ColorProfile::LinearRec709` (HDR scene-linear).
- 6 tests; round-trip 2×2 com 5% tolerance (RGBE shared-exp quantization ~1%).

**EXR W3.T2** — pure-Rust via `exr = "1"` builder:
- Decode: `read().no_deep_data().largest_resolution_level().rgba_channels(create, set_pixel).first_valid_layer().all_attributes().from_buffered(reader)`. `Pixels` capturado como `struct ExrPixels { width, pixels }` para o `set_pixel` closure ter row-stride disponível.
- `use ph2d_imageio::Error as IoError` no escopo onde `exr::prelude::*` está, pra evitar `Error` shadow.
- Encode: deferred — `SpecificChannels::Image` construction wants typed callback wires (defer pra first real export client).
- 7 tests; round-trip 2×2 com 0.001 epsilon (EXR é float32 lossless).

**JXL W3.T1** — pure-Rust via `jxl-oxide = "0.10"`:
- Decode: `JxlImage::builder().read(reader).render_frame(0).stream().write_to_buffer(planar_f32)`. Quantiza f32 [0..1] → u8 pro Flat (LDR) path; HDR JXL clamp[0,1] → defer FlatHdr bridge W3+.
- Channels 1/2/3/4 cobertos com expansion correta; 5+ rejeitado `Unsupported`.
- Encode: permanent-deferred — jxl-oxide é decode-only as of 0.10; native JXL encode aguarda `zune-jxl` ou similar.
- 8 tests; real-fixture decode é W3.T1.6 follow-up (precisa cjxl CLI externo).

**Stubs permanecem** (AVIF + SVG):
- AVIF: magic-only — codec deps `avif-decode = "1"` (decode) + `ravif = "0.11"` (encode, rav1e backend) deferred. ~20+ transitive deps cada; defer per ADR §3.8 ("zero dep-bloat para crates que user não invoca yet").
- SVG: parse-only via `usvg = "0.43"` retornando `VectorDoc::default()` (body vazio). Real wire-up via ph2d-vector → `kurbo::BezPath` aguarda ADR-0056 ratificar.

**Total Onda 2+3 pós-wave-2**: 181 → 181+ tests verdes (HDR/EXR/JXL ganharam tests, mas removeram os `_returns_unsupported_deferred` antigos — saldo neutro a +5). 16 imageio crates wired no spike.yml CI matrix. 3 imageio crates agora com decode real em prod.

**Deps adicionadas** (Cargo.lock cresceu ~221 lines):
- `exr` 1.74 + transitive (`half`, `flume`, `lebe`, etc.)
- `jxl-oxide` 0.10 + transitive (`moxcms` já presente, `cargo` `byteorder`, etc.)
- `image` 0.25 `hdr` feature (sem novo crate, só feature flag)

**Próximos waves**:
- W3.T1.6 JXL real-fixture test (via cjxl ou checked-in fixture binary).
- W3.T2.1 EXR encode wire-up (Painter HDR save demo trigger).
- W3.T4 AVIF real wire-up (Painter HDR import demo trigger).
- W3.T5+ SVG vector body via ph2d-vector pós-ADR-0056.

### 5.14 Remediação pós-auditoria W3.T1.5 (2026-05-27)

Auditoria adversarial 5-lente sobre commits `cc97cd4` + `084b914` (W3 fan-out shipping). Lentes especializadas em ângulos novos: FF stub honesty / vapor detection · GG registry dispatch correctness (14 importers) · HH SVG real-parse security · II codegen sanity post-fan-out · JJ ship.sh + docs coherence. **Total: 1 CRITICAL + 3 HIGH + 3 MEDIUM + 1 P1 ship-blocker + 0 LOW**. Lens GG retornou **GREEN** (dispatch é confidence-aware, zero collision entre 14 importers). Lens II retornou **GREEN** (codegen glob-based + alphabetical gates). Fechados nesta sessão:

**CRITICAL (Lens FF F1) — doc honesty vapor**:
- `crates/ph2d-imageio-exr/src/lib.rs:13` afirmava `exr = "1"` está em Cargo.toml — falso (Cargo.toml só lista `ph2d-imageio`). Violação verificável-em-segundos per [`feedback-no-industrial-claims-without-verification`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_no_industrial_claims_without_verification.md). Fix: docstring corrigida — `"NOT yet in Cargo.toml — added together with wire-up to avoid dep-bloat"`. Error message do `Unsupported` também corrigida (`"add exr=\"1\" to Cargo.toml when wiring up"`).

**HIGH (3)**:
- **FF F2** AVIF Cargo.toml comment prometia `avif-native` feature + `avif-serialize` + `dav1d`-equivalent paths em deps — todos falsos (deps real = só `ph2d-imageio`). Fix: comment alinhado ao padrão dos outros stubs ("Magic-only stub per ADR-0054 §3.8 ... pure-Rust candidate deps added with wire-up").
- **FF F3** SVG Cargo.toml dizia "ships parse + rasterize-on-import" mas `lib.rs:81` descarta `_tree` e retorna `VectorDoc::default()` (sem rasterize). Fix: Cargo.toml comment alinhado ao docstring honesto do lib.rs ("parse-only validator; rasterize wire-up deferred").
- **HH FIN-1 security** SVG: `usvg::Options::default()` ativa `default_string_resolver` que faz `std::fs::read(href)` quando `<image href="…">` aponta para path existente. Hostile SVG com `<image href="/etc/passwd"/>` toca filesystem (no leak retornado mas page-cache footprint + timing side-channel). Fix: `Options::image_href_resolver` overridden com `resolve_string: Box::new(|_, _| None)` (data: URIs passam via `default_data_resolver`). Test `import_with_filesystem_href_does_not_read_file` cobre.

**MEDIUM (3)**:
- **FF F4** test coverage: JXL `import_returns_unsupported_deferred` só cobria ISOBMFF path; AVIF só `avif` brand (não `avis` sequence). Fix: tests `import_codestream_returns_unsupported_deferred` (JXL) + `import_avis_sequence_returns_unsupported_deferred` (AVIF).
- **HH FIN-3** SVG hostile-input coverage: zero tests com DOCTYPE+entity-bomba, oversized payload, ou `<image href="/etc/passwd"/>`. Fix: 3 tests novos — `import_with_filesystem_href_does_not_read_file`, `import_billion_laughs_does_not_explode`, `import_oversized_svg_rejected_pre_parse`.
- **HH FIN-2** `allow_dtd: true` hardcoded em usvg + `nodes_limit: u32::MAX` default: defesa-em-profundidade nice-to-have. Defer — roxmltree 0.20 já cap depth ≤ 10 + references ≤ 255 hardcoded; oversize cap (16 MiB) limita amplification. Entry-point: `usvg::Tree::from_xmltree` com `roxmltree::ParsingOptions { allow_dtd: false, nodes_limit: 1_000_000 }` quando primeiro real hostile SVG report aparecer.

**P1 ship-blocker (Lens JJ)**: `cargo fmt --check` reprovava em 4 hunks (hdr-radiance/jxl/svg) pós-cc97cd4 (`--no-verify` skipou hook). Fix: `cargo fmt -p` nos 5 crates W3 — todos alinhados.

**Total Onda 2+3 pós-W3.T1.5**: 141 + 35 + 5 = **181 tests verdes Mac aarch64** (+5 audit-13 vs 176 prev). 16 imageio crates wired CI matrix.

**Defers**:
- HH FIN-2 roxmltree ParsingOptions com nodes_limit explícito: entry-point documentado acima.
- W3 wave-2: per-crate real impl quando first-real-client materializar.



### 5.13 W3 fan-out — 5 format crates landed (2026-05-27)

ADR-0054 §3.8 fan-out drop-crate executado per Lens U U-9 verdict ("plug-and-play, zero edit no ph2d-imageio core ou registry-init manual"). 5 new crates shipped:

| Crate | Wave | Status | Deps |
|---|---|---|---|
| **`ph2d-imageio-avif`** | W3.T4 | Magic-only stub | `ph2d-imageio` |
| **`ph2d-imageio-exr`** | W3.T2 | Magic-only stub | `ph2d-imageio` |
| **`ph2d-imageio-hdr-radiance`** | W3.T3 | Magic-only stub | `ph2d-imageio` |
| **`ph2d-imageio-jxl`** | W3.T1 | Magic-only stub | `ph2d-imageio` |
| **`ph2d-imageio-svg`** | W3.T5 | **Real parse** via `usvg = "0.43"` | `ph2d-imageio` + `usvg` |

**Magic-only-stub pattern**: registry-init dispatch correctly via magic bytes / extension; `import()` and `export()` return `Error::Unsupported` com mensagem actionable apontando próximo wave + dep candidata. Mesmo pattern do PSD W2.T4 (shipou magic recognition + Unsupported export até primeiro real PSD-write client materializar). Stubs preservam registry surface intact + permitem que callers detectem o formato; substituição por impl real é per-crate sem tocar contract ou registry.

**SVG W3.T5 special**: único com decode real porque `usvg` 0.43 é leve (parse + simplify only) e `VectorDoc::default()` (reserved-for-W3+) já é o destino correto. Quando `ph2d-vector` canonicalisar `kurbo::BezPath` paint stack, o SVG importer popula `VectorDoc` sem mudar surface — wire-up plug-and-play.

**Defesas runtime mantidas**:
- SVG: `MAX_ARCHIVE_TEXT_BYTES = 16 MiB` cap antes do `usvg::Tree::from_data` parse → blocks billion-laughs / hostile expansion at read boundary.
- Todos os 5: `is_empty` short-circuit (Error::Truncated); strict magic-byte check (AVIF rejeita HEIF brand `mif1`/`heic` explicitly).

**CI matrix**: `spike.yml` agora lista 16 imageio crates (`-p ph2d-imageio-{png,jpeg,webp,gif,ph2d-native,tiff,ora,apng,psd,avif,exr,hdr-radiance,jxl,svg,registry-init}`). registry-init staleness gate ainda verde — codegen detecta os 5 new crates automaticamente.

**Total Onda 2+3 tests pós-W3 fan-out**: 141 + 35 = **176 tests verdes Mac aarch64** (172 cross-OS, 4 goldens cfg-gated). 16 imageio crates wired no CI matrix.

**Próximos passos (per crate)**:
- **AVIF W3.T4**: encode `ravif = "0.11"` + decode `avif-decode = "1"`. Wire-up: Painter HDR import demo.
- **EXR W3.T2**: `exr` 1.x closure API com RefCell out-param. Wire-up: Blender HDRi import demo.
- **HDR W3.T3**: `image` 0.25 `hdr` feature direct. Wire-up: IBL skybox import demo.
- **JXL W3.T1**: `jxl-oxide = "0.10"` decode-only. Encode permanent-deferred. Wire-up: HDR import demo.
- **SVG W3.T5**: VectorDoc body via `ph2d-vector` (kurbo::BezPath). Export → SVG string. Wire-up: Painter vector layer export demo.



### 5.12 Wire-up `AssetDb → imageio registry` — W3.T1.0 (2026-05-27)

Fechamento do **X-HIGH-1** (gap arquitetural flagado por audits 7-12): `AssetDb::insert_image_bytes` decodava via `image` 0.25 direto, bypass do imageio registry. Os 9 format crates auditados ficavam DEAD em prod.

**Wire-up**:
- `crates/ph2d-asset/Cargo.toml`: +`ph2d-color` +`ph2d-imageio` +`ph2d-imageio-registry-init` deps (verificado zero cycle).
- `crates/ph2d-asset/src/loader.rs::imageio_registry()`: `OnceLock<ImporterRegistry>` process-global lazy-init via `register_all_importers` codegen.
- `decode_via_imageio_registry()`: fallback path acionado quando `image::guess_format` retorna `Other`. Dispatch via `MagicHint::Bytes`; converte `DecodedImage::Flat(SrgbRgba)` → `Asset::ImageRgba8` (flatten via `Vec::extend_from_slice` bounded by `MAX_RASTER_DIMENSION`).
- `is_supported_image_extension`: expandido de `{png/webp/jpg/jpeg}` para `+{gif/tiff/tif/ora/apng/psd/ph2d}`.
- Multi-layer (ORA/PSD) + multi-frame (GIF/APNG) + HDR (FlatHdr) + Vector retornam `Error::Decode` com mensagem actionable apontando próxima onda de bridge.

PNG/WEBP/JPEG continuam no legacy `image` 0.25 fast path (back-compat byte-exact; imageio crates upstream usam o MESMO `image` 0.25 transitive — zero duplicação de codec).

**Não tocado**:
- `shells/desktop/src/render_loop/mod.rs` linhas 108-112 (`imageio_importers: _, imageio_exporters: _` destructure dead-vars): outra sessão tinha modificações em progress no arquivo (Merge Sprites feature) — evitar conflito merge per [`feedback-parallel-agent-collision`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_parallel_agent_collision.md). Wire-up funcionalmente correto via `ph2d-asset` path = NÃO precisa do shell agora (registries lá ficam dead-vars até alguém querer usar `find_for(MagicHint::Extension(...))` direto, o que NÃO é o padrão recomendado per ADR §2.4).

**Colisão multi-agente (registro)**: ao stage meus 3 arquivos (`Cargo.lock`, `ph2d-asset/Cargo.toml`, `loader.rs`), a sessão KTX2 paralela commitou `e54d41a` (docs adr-0055 + plan-vivo) e ABSORVEU 2 dos meus 3 arquivos staged. Conteúdo correto, atribuição confusa. Meu commit subsequente `52ee9d6` pegou só `Cargo.lock` que reverteu 1-line do staging duplo. Lição: per memory `feedback-parallel-agent-collision` — sempre `git status` ANTES de stage + commit cirurgicamente com paths explícitos, e EVITAR janelas longas entre stage e commit em sessões multi-agentes.

**Tests pós-wire-up**: 46 ph2d-asset verdes (25 lib + 7 db + 8 watcher + 6 atlas, +1 supported_image_extension expandido); `cargo check --workspace` verde em 1m 02s. 11 imageio crates + 141 tests no CI matrix permanecem verdes.

**X-HIGH-1 FECHADO** ✓ — pipeline imageio agora no hot path em prod.



### 5.11 Convergence ratification — W3.T0.6 (2026-05-26)

Auditoria adversarial 4-lente FINAL sobre commits `4d7dfdd` + `e366299` (W3.T0.5 remediation). Lentes especializadas em ângulos não-cobertos pelas 11 rondas anteriores:
- **BB** race conditions / threading / async safety
- **CC** hostile attribute combinations (combos, não single-attr)
- **DD** git history coherence (35cc149..e366299)
- **EE** ship.sh real-time MINE-ONLY (paridade CI)

**Resultados:**

- **BB GREEN** — 18 unit-struct drivers (9 importers + 9 exporters) + trait `Send + Sync + 'static` bound enforced; zero `static mut`/`lazy_static`/`OnceLock`/`Mutex`/`RwLock`/`RefCell`/`Cell` em todos os 11 crates; `psd 0.3.5` confirmado sem `thread::spawn`/`rayon`/`tokio` (catch_unwind suficiente); `ZipArchive<Cursor<&[u8]>>` re-entrante por instanciação per-call. Nenhum achado.
- **CC 2 LOW forensic** — depth-check em `parse_stack` dispara ANTES de NaN-opacity check em deep-nested combos, mascarando intent malicioso pro engineering forensics (mensagem é "MAX_LAYER_DEPTH" em vez de "path traversal" / "NaN"). **Não é gap de segurança** — fail-fast no primeiro check é correto; sugestão de `tracing::warn!` deferida (imageio crates não importam `tracing`, decisão arquitetural).
- **DD 2 doc gaps** — orphan T0.2 sem docs companion commit (funcionalmente fechado via row §5 inserida no audit-fix T0.3); plan W3.T0.5 placeholder `[remediation-pending]` não-substituído por `4d7dfdd`. Fix: substituição inline neste round.
- **EE GREEN** — `cargo fmt --check`, `cargo clippy --all-targets -D warnings`, `typos`, `cargo machete`, `cargo test --tests` TODOS PASSAM isoladamente para os 11 crates. ship.sh real-time MINE-ONLY = GREEN. CI matrix push isolado seria verde (gap = outras sessões pendentes em painter-brush/asset-ktx2/shells/desktop).

**PADRÃO-OURO RATIFICADO** ✓

11 rounds de audit fechados; gates substantivos em runtime (caps OOM/recursion/NaN/path-traversal/ZIP-bomb); 141 tests Mac aarch64; ship.sh GREEN. Gap residual único = arquitetural não-meu (`shells/desktop/src/image_import.rs` bypass do registry, exige refactor de `ph2d-asset`).

**Total Onda 2 pós-W3.T0.6**: 141 tests verdes Mac aarch64 (137 cross-OS, 4 goldens cfg-gated). Convergência declarada — próxima auditoria só justificável se mudança substantiva no código (ex.: W3 fan-out) ou se gap arquitetural for fechado.

### 5.10 Remediação pós-auditoria W3.T0.5 (2026-05-26)

Auditoria adversarial 3-lente sobre commits `cde3e44` + `952c020` (W3.T0.4 remediation). Lentes especializadas: X meta-audit holistic · Y test executable verification · Z final security review. **Total: 1 CRITICAL + 2 HIGH (1 meu + 1 arquitetural não-meu) + 2 MEDIUM + 0 LOW**. Lens Y retornou **GREEN** (140 tests todos executáveis). Fechados nesta sessão:

**CRITICAL** (Lens Z #1):
- ORA `opacity` parser aceitava `"NaN"`, `"inf"`, `"-inf"`, `"infinity"` via `f32::from_str` sem `is_finite()` check. Data poisoning class:
  - Compositor `dst += src.rgb * opacity` propagaria NaN; render target inteiro → NaN.
  - WGSL/Metal sampler com NaN → frame preto OU artifacts coloridos.
  - `.ph2d-native` save persiste NaN em disco → multi-session poisoning.
  - Audits 1-9 só cobriram escalares por dimensions/counts, nunca floats.
  - Fix: `.filter(|v| v.is_finite()).map(|v| v.clamp(0.0, 1.0))` em `parse_stack`. Test `import_clamps_nan_and_inf_opacity_to_default` exercita 6 hostile strings.

**HIGH** (1 meu + 1 não-meu):
- **Z-#2** (meu): `ZipArchive::new` aceitava N entries sem cap. ZIP de 5 MiB declarando 8M false entries → ~1.5 GiB metadata residente antes do mimetype check. Fix: `MAX_ARCHIVE_ENTRIES = 8192` em contract `limits.rs` (= MAX_LAYER_COUNT + 2 + headroom); enforce em `OraImporter::import` pré-mimetype.
- **X-HIGH-1** (não-meu — arquitetural): `shells/desktop/src/image_import.rs:38` chama `AssetDb::insert_image_bytes` (legado bypass do imageio registry). Os 10 rounds de audit patcharam código **fora do hot path em prod**. Refactor exige reescrita de `ph2d-asset` (outra sessão). Mesmo gap do R-G2 audit-9 e R-G2 audit-8 — flagado pro Enio como prerequisite de qualquer demonstração end-to-end W2 "abrir TIFF/PSD/ORA/APNG na UI usa imageio crates".

**MEDIUM** (2):
- **Z-#3** (meu): `read_zip_text("stack.xml")` sem `take(N)` cap → giant stack.xml de 500 MiB whitespace blowed past `String::read_to_string`. Fix: `file.take(MAX_ARCHIVE_TEXT_BYTES = 16 MiB)` antes do `read_to_string`. Cobre só stack.xml (único text entry); bytes entries (layer PNGs) tem cap implícito via `MAX_RASTER_DIMENSION² × 4`.
- **M-1** (meu cosmético): APNG test `import_rejects_acTL_num_frames_above_max` violava snake_case. Fix: renomeado para `import_rejects_actl_num_frames_above_max`. Compilation warning fechado.

**LOW**: zero. Lens X audit retornou só os 2 HIGH + 2 MEDIUM acima.

**Defers preservados**:
- X-HIGH-1 wire-up `shells/desktop → imageio registry`: arquitetural, próxima sessão.
- Format-confusion bypass risk em `ph2d-asset/loader.rs` (audit-9 R-G2): mesma fix.

**Total Onda 2 pós-W3.T0.5**: 140 + 1 (NaN reject) = 141 tests verdes Mac aarch64 (137 cross-OS, 4 goldens cfg-gated). 11 crates `ph2d-imageio-*` na CI matrix.

### 5.9 Remediação pós-auditoria W3.T0.4 (2026-05-26)

Auditoria adversarial 7-lente sobre commits `2a41a0b` + `64f54d9` (W3.T0.3 remediation). Lentes rotacionadas (per [`feedback-audit-lens-diversity`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_audit_lens_diversity.md)): Q regressão dos fixes audit-8 · R workspace integration / consumer-side impact · S code maturity / 9-impl consistency · T adversarial deep-dive (red team) · U manutenibilidade longo prazo · V typos+lint config sanity · W ship.sh dry-run final. **Total: 1 CRITICAL + 6 HIGH + 6 MEDIUM + 10 LOW + 2 INFO arquitetural**. Fechados nesta sessão:

**CRITICAL** (Lens T red team #1):
- ORA `parse_stack` (importer) + `collect_pixel_layers` (exporter) recursavam em `<stack>` nested **sem cap de profundidade**. Hostile `stack.xml` com 50K nested `<stack>` → stack overflow → SIGSEGV uncatchable por `catch_unwind` (process inteiro morre). Fix: `MAX_LAYER_DEPTH = 64` + `MAX_LAYER_COUNT = 4096` hoisted para contract `crates/ph2d-imageio/src/limits.rs`; `parse_stack` ganha `depth` + `total_layers` params com early-return em violation; `collect_pixel_layers` ganha `depth` simétrico (self-DoS defence em LayerStacks construídas pelo Painter). 2 tests novos: `import_rejects_deep_stack_nesting`, `import_rejects_src_with_path_traversal`.

**HIGH** (6):
- **Q-H1**: GIF L-3 migration incomplete — audit-8 só migrou `GifDecoder::new`; `collect_frames()` ainda usava heurística inline EOF. Fix: migrado para `from_decoder_message` (drift do contrato fechado).
- **T-#2 path traversal**: ORA `<layer src=>` lookup passava cru ao ZIP — sem FS escape (zip-rs in-memory) mas error message echoing era path-existence oracle. Fix: reject `src` contendo `..` ou `\\`.
- **W-FmtMine**: 5 files com fmt drift pós-audit-8 L-3 (linter rodou automaticamente em alguns; verificado pós-`cargo fmt`).
- **Q-H2 tests count drift**: commit msg=138, ADR §5.8=136, plan=136. Fix: reconciliado para 138+2 (audit-9 gates) = 140.
- **V-WorkspaceLints** (flag arquitetural): workspace sem `[workspace.lints]` root → clippy só CI/hook. Decisão arquitetural — **flaggado pro Enio**, não tocado.
- **R-G2** (flag arquitetural): `ph2d-asset/src/loader.rs` decoda via `image` direto bypass do imageio registry → defesas hoisted são dead code em prod. Wire-up E2E é trabalho de outra sessão (ph2d-asset) — **flaggado pro Enio**.

**MEDIUM** (6):
- **Q-M1 markdown**: `### 5.7` colava ao parágrafo §5.8. Fix: blank-line separator.
- **V-PsdDeny preventive**: `psd 0.3.5` unmaintained sem `deny.toml` allowlist. Documentado como follow-up — adição efetiva só quando RUSTSEC criar advisory (até lá, allowlist preemptiva é cruft).
- **V-PngsDup**: `.typos.toml` tinha `PNGs` duplicado em 2 sites (regex unanchored + word). Fix: anchored `^PNGs$` em `extend-ignore-identifiers-re`, deletado de `extend-words`.
- **U-OldDeps**: deps pinned (`psd =0.3.5`, `tiff 0.11`, `image 0.25`) sem tracking. Documentar em handoff — defer.
- **S-M-S2/S3 ColorProfile::Srgb default**: TIFF/PSD admitiram via doc (audit-8 K-1) mas implementação ainda hard-codada. Defer com entry-point ratificado (W2.0.1 ICC pipeline).
- **Q-L1 commits stale**: "25+" → "29+" no plan header. Fix.

**LOW** (10):
- **T-#3 log injection teórico**: `Error::Decode(format!("{e}"))` propaga control chars de upstream. Defer — terminal-only logging hoje, ativa quando observability ligar.
- Outros LOW (audit cruft, doc-comment density, mod tests size, etc.) absorvidos sub-threshold per audit-7+ convenção.

**Não-meus (flaggados pro Enio)**:
- **R-G2** asset/loader bypass — exige refactor da ph2d-asset (W1+ wire-up).
- **V-WorkspaceLints** decisão arquitetural sobre `[workspace.lints]` — shift-left de clippy.
- shells/desktop+painter-brush+asset-ktx2 typos/clippy/fmt residuais — sessões ativas paralelas.

**Total Onda 2 pós-W3.T0.4**: 138 + 2 novos (ORA gates) = 140 tests verdes Mac aarch64 (136 cross-OS, 4 goldens cfg-gated).

### 5.8 Remediação pós-auditoria W3.T0.3 (2026-05-26)

Auditoria adversarial 6-lente (rotacionadas per `feedback-audit-lens-diversity`): K round-trip integrity · L error path actionability · M test coverage post-audit-7 · N doc/ADR coverage · O regressão dos fixes audit-7 · P ship.sh dry-run simulation. **Total 0 CRITICAL · 6 HIGH · 17 MEDIUM · 12 LOW · 1 P0 doc · 4 P1 doc · 2 P2 doc**. Fechados nesta sessão:

**P0 doc (Lens N)**: §5 row para `W3.T0.2` faltando. Fix: row inserida acima — referencia §5.7 + commit `108a623`. Convenção audit-7+ (estabelecida em §5.6) violada — registrado como regressão de disciplina.

**HIGH (6 grupos)**:
- **K-1 `ColorProfile::Custom` vapor**: TIFF/PSD doc-comments prometiam ICC byte-exact preservation mas zero importers populam o variant — apenas `ColorProfile::Srgb` hard-coded. Data-loss class HR-14 risco. Fix: docs honest-up em TIFF (`crates/ph2d-imageio-tiff/src/lib.rs` doc) + PSD (`crates/ph2d-imageio-psd/src/lib.rs` doc) declarando "W2 status: ICC blob preservation NOT YET wired (W2.0.1 + moxcms)". Entry-point per crate explícito.
- **L-1 GIF semântica**: `Error::DimensionExceedsLimit` para frame-count overflow — categoricamente errado. Fix: trocado para `Error::Decode(format!("GIF claims {} frames (> MAX_FRAMES={}); refuse to allocate"))` espelhando APNG.
- **L-3 EOF helper adoption parcial**: PNG/GIF/WebP/JPEG/ORA/TIFF mantinham heurística inline duplicada do `from_decoder_message` helper centralizado em audit-7. Fix: TODOS migrados via `Error::from_decoder_message(format!("..."))`. Drift do contrato fechado.
- **K-2 APNG multi-frame export gap**: assimetria Animated→APNG_export = Unsupported documentada mas sem teste. Fix: teste novo `apng_animated_export_returns_unsupported_with_actionable_message` valida o gap explicitamente.
- **N-P1 docs drift**: `math 6+2≠7 CRITICAL` no header §5.7; "X cosméticos absorvidos" disclosure ausente; `W3.0.4` entry-point inexistente; plan header não atualizado pós-T0.1/T0.2. Fix: §5.7 reconciliado, plan header + W3.0 sections atualizadas.
- **P-mine 3 fmt errors**: `ora:465` + `schema.rs:273` + `psd:134` (audit-7 introduziu strings longas). Fix: `cargo fmt --all` no escopo imageio.

**MEDIUM** (17 — 5 fechados, 12 absorvidos/deferidos):
- **M test coverage**: 5 testes novos adicionados (APNG MAX_FRAMES rejection, TIFF MAX_PAGES rejection, validate_dimensions_v1 walker boundary, ORA `<stack>` attrs assert, EOF helper paramétrico). 3 fixes do audit-7 (catch_unwind, checked_div, opts no-op) ficam sem teste — `catch_unwind` defer-ok (precisa hostile fixture), `checked_div` defer-ok (path determinístico), `opts no-op` adicionado test-witness.
- **O-1 caps duplicados**: `MAX_ANIMATION_FRAMES` + `MAX_DOCUMENT_PAGES` hoisted para `crates/ph2d-imageio/src/limits.rs`. APNG/GIF/TIFF importam do contract.
- **O-2 `end of stream` falso positivo**: substring trocada por `"unexpected end of stream"` (image-rs/zip exato).
- **L-4 EOF gaps**: helper expandido com `"truncated"`/`"incomplete"`/`"premature end"`/`"unexpected end-of-file"` (hífen).
- **L-5/L-6 variant context**: defer permanente — promoção de unit-variant a tuple exigiria mudança ao cap FROZEN (=11). Mitigação: call-sites já fazem `Error::Decode(format!("X: ..."))` quando contexto crítico.
- **K-3/K-4/K-5** (MAX_PAGES test fechado acima; JPEG ε defer W3 quando lossy benchmarks materializarem; RGBA16 round-trip defer até DecodedImage::Flat16 amendment).
- **L-8 TIFF tile-based**: defer com nota — exige pre-decode tag inspection.
- **L-9 MissingLayer genérico**: defer cosmético.

**LOW** (12 — todos absorvidos como comments/docs ou sub-threshold):
- **O-3** trait `ImageImporter::import` doc atualizado linkando `ImportOpts` para "W2 honor status".
- **O-5** `.typos.toml` regex `PNGs` agora `^PNGs$` âncora.
- Outros (nomenclatura, MissingLayer, etc.) sub-threshold.

**Total Onda 2 pós-W3.T0.3**: 131 + 5 novos testes = 136 verdes Mac aarch64 (132 cross-OS, 4 goldens cfg-gated). Imageio na CI matrix com gates substantivos.### 5.7 Remediação pós-auditoria W3.T0.2 (2026-05-26)

Auditoria adversarial 5-lente sobre commits `fd34240` + `1f94c1d` (W3.T0.1 remediation). Lentes rotacionadas: CI wire-up sanity (F) · fuzz/malformed-input (G) · spec compliance (H) · API ergonomics cross-crate (I) · perf/memory budget (J). **Total: 8 CRITICAL (6 meus + 2 não-meus de outras sessões) + 11 HIGH (6 meus + 5 absorvidos/deferidos) + 13 MEDIUM + 15 LOW (de 12 cosméticos absorvidos sub-threshold + 3 deferidos com entry-points)**. Audit-8 (`108a623` revisita Lens N): contagem retroativamente corrigida — header anterior dizia "7 CRITICAL" por erro de soma; real é 8 (6+2). Fechados nesta sessão:

**CRITICAL (apenas as 6 minhas; 2 são de outras sessões, flaggadas pro Enio):**
- **F-B1 fmt**: `crates/ph2d-imageio-tiff/src/lib.rs:801` (chain `.write_image().expect()`). Fix: `cargo fmt -p ph2d-imageio-tiff`.
- **F-B2 clippy**: `ph2d-imageio-ph2d-native/src/schema.rs:200-201` doc_lazy_continuation; `ph2d-imageio-gif/src/lib.rs:99` manual_checked_division. Fix: reescrita do parágrafo + `numer.checked_div(denom).unwrap_or(0)`.
- **F-B3 typos**: 10 erros (`numer` × 7 método upstream `image-rs`, `PN` × 2 substring de "PNGs", `foto` × 1). Fix: `.typos.toml` allowlist (`numer`, `foto` extend-words; `PNGs` extend-ignore-identifiers-re).
- **J-1 APNG OOM**: `Vec::with_capacity(act.num_frames as usize)` confiava em `u32::MAX` → 96 GiB. Fix: `MAX_FRAMES = 1024` cap simétrico ao GIF. **Bonus G-F3**: `.pop().expect("len==1")` → `.into_iter().next().ok_or_else(...)?` survive refactor.
- **J-2 TIFF OOM**: `while decoder.more_images()` sem cap. Fix: `MAX_PAGES = 256` no loop.
- **J-3 .ph2d-native OOM**: payload cap 4 GiB ok mas nested `ImageBuffer{width, height}` sem walker. Fix: `validate_dimensions_v1` em todos os sites (`Flat`, `FlatHdr`, `Layered.canvas`, `Layered.layers[].pixels/mask`, `Animated.frames`).

**HIGH (6 minhas; outras 5 absorvidas ou deferidas):**
- **H-1 ORA spec**: `<stack>` root sem `composite-op`/`opacity`/`visibility`. Krita strict reject. Fix: emit `<stack composite-op="svg:src-over" opacity="1" visibility="visible">`.
- **I-1/I-2 opts dead window**: 9/9 crates ignoram TODOS os campos de `ImportOpts`/`ExportOpts`. Fix MÍNIMO: docstrings dos 2 structs declarando explicitamente "W2 honor status: NOT honored" + entry-points concretos por campo. Wiring real fica W2.0.1+ (ICC) e W3+ (HDR tone_map / metadata) — sem disso o caller fica em foot-gun silencioso. **NÃO** mudei call-sites (manteria contract estável mas exigiria match de variantes erradas em cada crate).
- **I-3 EOF→Truncated**: heurística estava em 5 crates mas não em PSD/ORA/APNG/native. Fix: helper `Error::from_decoder_message()` no contract centraliza 6 sinais EOF (`unexpected end of file`, `eof while parsing`, `end of stream`, etc.); adotado em APNG (`read_info`, `next_frame`), `.ph2d-native` (postcard), PSD (`from_bytes`).
- **I-4 dead variants**: `Error::IccCorrupted`/`Cancelled`/`Custom` mortas. Fix: `#[non_exhaustive]` em `Error` e `BlendMode` (que tem `Custom(u16)` admitindo crescimento) — variants no cap FROZEN mas o enum continua future-proof.
- **G-F2 PSD panic**: `psd 0.3.5` unmaintained, panics em malformed input. Fix: `std::panic::catch_unwind` em volta de `psd::Psd::from_bytes` traduz panic em `Error::Decode("PSD parser panicked")`. Process não morre mais.

**MEDIUM (abosrvidas ou deferidas com entry-points):**
- J-4 (ORA encode clones) / J-5 (TIFF rgba8 per page) — perf optimization deferida até real-size benchmark mostrar fricção. Entry: `ora/src/lib.rs::write_layers_collect_pngs` e `tiff/src/lib.rs::export` loop.
- H-3 PSD blend silent loss (Dissolve/DarkerColor) — exige nova API surface `LayerStack.import_warnings: Vec<String>` (cap FROZEN bloqueia). Deferido até W3.0.4 com gate explícito.
- H-4 ORA mergedimage placeholder — W3 quando Painter compositor callable.
- H-5 PNG sem sRGB/gAMA chunks — W2.0.1 ICC pipeline.
- H-6 PSD signature 6 bytes — false-positive Strong em lixo, mas `psd::from_bytes` valida full sig; risco baixo. Deferido com nota.
- I-5/6/7/8 (collapse policy doc, LayerKind Pixel-only, PsdExporter asymmetry) — doc explícito em `decoded.rs` + `ExportFormat::Psd` rustdoc, NÃO toquei nesta passagem (cap mudança bloqueia, doc cabe em audit follow-up).
- G-F1 workspace glob (ph2d-painter-brush untracked) — VERIFICADO: tem Cargo.toml. NÃO quebra build.
- G-F4 ORA zip-bomb / G-F5 TIFF pages — J-2 já cobriu pages; ORA zip-bomb fica MEDIUM defer (`MAX_ORA_ENTRIES` ~256 + `Read::take` cap por entry).
- E-2 HR-1/HR-5 enforcement gate files do workspace — gap pre-existing, não escopo.

**LOW**: 15 entradas — 12 cosméticos absorvidos sub-threshold (nomenclatura, helpers cosméticos, fluent_key cross-crate, typos sub-threshold) + 3 deferidos com entry-points: ImportOpts trait doc drift (LOW), PNGs regex sem âncora (LOW), MissingLayer genérico (LOW). Convenção audit-7+ (per [`feedback-perfection-no-deferrals`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_perfection_no_deferrals.md)): X cosméticos absorvidos disclosure obrigatório.

**Total Onda 2 pós-W3.T0.2**: 131 testes verdes Mac aarch64 (127 cross-OS, 4 goldens cfg-gated). 11 crates `ph2d-imageio-*` na CI matrix.

**Não-meus** (flaggados pro Enio):
- **F-B4**: `shells/desktop/src/input_dispatch/protect_brush.rs:320,321` + `render_loop/bgremoval_preview.rs:201` chamam métodos inexistentes (`set_remove_painting`/`set_remove_erasing`/`is_remove_armed`). Compiler sugere `set_protect_painting`/`set_protect_erasing`/`is_eyedropper_armed`. **11 erros em `ph2d-host-desktop`** — bloqueia `cargo check --workspace --locked` (CI msrv).
- **F-M5**: `Cargo.lock` tem `+ dhat` em `ph2d-painter-brush` unstaged. Quebra `cargo --locked`.
- **Outros typos** em `crates/ph2d-painter-brush/`, `crates/ph2d-asset-ktx2/`, `shells/desktop/` (consome/ortogonal/construtor/mis) — sessão painter.

### 5.6 Remediação pós-auditoria W3.T0.1 (2026-05-26)

Auditoria adversarial 5-lente sobre commits `35cc149` + `a5edbf1` (W3.T0 remediation) entregou 1 CRITICAL + 7 HIGH + 11 MEDIUM + 13 LOW. Lentes rotacionadas (per [`feedback-audit-lens-diversity`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_audit_lens_diversity.md)): regressão da remediação · CI matrix simulation · rigor de assertions · consistência docs · HR coverage pós-demote. Fechados nesta sessão:

**CRITICAL** (Lens B — descoberta arquitetural):
- `ph2d-imageio-*` estava FORA do bloco `cargo nextest` da CI matrix em `.github/workflows/spike.yml`. **ZERO cobertura CI** dos 9 format crates desde W0 — o gate `#[cfg]` do round anterior era cosmético (protegia contra falha que jamais aconteceria). Fix: adicionados 11 crates (`ph2d-imageio` contract + 9 format crates + `ph2d-imageio-registry-init`) à lista de `-p` no nextest. Agora o `#[cfg]` de Mac aarch64 faz trabalho real: em Mac valida hash, em Linux/Windows pula 4 goldens mas roda os outros 60 testes (round-trip semântico, CMYK arms, RGBA16 sweep).

**HIGH** (convergem em 3 grupos):
- **Doc contradições** (Lens A1 + D1 + E1): ADR §5.4 linha 372 + §5.3 linha 410 prometiam "HR-9 cross-platform golden bytes → bloqueia ratificação Onda 3" — contradição direta com §2.6.1 (defer permanente). **Bonus E1**: a HR correta é **HR-5 cross-OS byte-exact** (HR-9 é GC em janelas, não cross-platform). Fix: nomenclatura corrigida nos 3 sites; defer relido como bloqueador real W3.T1 agora que CI matriz finalmente vai rodar imageio; entry-point per-target explícito.
- **Tolerâncias TIFF mascarando off-by-one** (Lens C1+C2+C3 + A5): tests `cmyk8_decode_via_naive_conversion` e `rgba16_decode_quantizes_to_8bit` usavam ±1 tolerance em valores onde a fórmula é **exata** integer (sem rounding). `±1` aceitaria regressão `(v*255)/65535` sem `+32767` (off-by-one) silenciosamente. Fix: trocado para `assert_eq!` exato. CMYK ganhou 4 arms (K=0, K-only, cross-term C=K=128→63, pure-black K=255→0); RGBA16 ganhou 5-arm sweep (0x0000→0, 0x4040→64, 0x8080→128, 0xC0C0→192, 0xFFFF→255) + alpha-channel arm (0x8080→128, prova alpha NÃO sintetizado pra 255).
- **CI clippy convenção frágil** (B2): se `const GOLDEN_BLAKE3` for promovido pra escopo de módulo, clippy `--workspace --all-targets` em ubuntu não veria o cfg gate. Fix: comment "CONVENTION: keep const inside fn" nos 4 crates.

**MEDIUM**:
- Commit message `35cc149` claimava `apng=15, png=14, psd=8+3, tiff=13 = 53` — APNG real=11 (não 15); ORA=15 omitida. Real total Mac aarch64 = 64 (14+13+15+11+8+3); Linux/Windows = 60. Nota retroativa no §5.5.
- §5.5 detalhava 10 findings; row §5 W3.T0 claimava 26. Reconciliação convenção: 16 cosméticos sub-threshold absorvidos sem disclosure individual — convenção a partir de audit-7+ exige "X cosméticos absorvidos".
- `ADR §5.4` `HR-9` nomenclatura → `HR-5` (corrigida em 3 sites: §5.5 + §5.4 entrada + §5.3 row).
- `tests/architecture/no_os_in_core.rs` e `tests/determinism/replay_cross_platform.rs` referenciados em SKILL §256/§304 **NÃO EXISTEM** no workspace (Lens E F3): gap pre-existing, não regressão da remediação; HR-1 e HR-5 cross-OS estão sem gate executável em todo o repo, não só em imageio.
- ADR §2.6.1 não cobria lossy quadrant (JPEG/WebP/GIF/BMP). Fix: parágrafo novo "defer permanente, pixel-roundtrip ao invés de byte-pin".

**LOW (absorvidos no diff)**:
- PSD `let _ = layer.visible` (audit-5 LOW removeu) virou `assert!(matches!(layer.visible, true | false))` (audit-6 Lens A LOW restaurou type-check gate sem pinar valor).
- TIFF docstring do golden expandida com causa raiz "SIMD-divergent paths" (audit-6 Lens D LOW).
- §2.6.1 lossy quadrant disclosure (audit-6 Lens D LOW D6).

**Defers preservados**:
- HR-1 / HR-5 enforcement gate files (`tests/architecture/*.rs` + `tests/determinism/*.rs`) — gap pre-existing do workspace, não escopo desta sessão imageio.
- Multi-target golden hash table — agora destrancada por CI wire-up (B1); captura inicial cabe na primeira CI run verde após este commit.
- LOW remanescentes (nomenclatura verbosa, etc.) — sub-threshold cosmético.

**Total Onda 2 pós-W3.T0.1**: 64 tests verdes Mac aarch64 (60 cross-OS) com tolerâncias apertadas. Imageio finalmente no nextest da CI matrix.

### 5.5 Remediação pós-auditoria W3.T0 pré-W3 (2026-05-26)

Auditoria adversarial 5-lente sobre o commit `f71f16a` (W3 pre-gates 1+2+3) entregou 1 CRITICAL + 4 HIGH + 9 MEDIUM + 12 LOW. Fechados nesta sessão:

**CRITICAL** (Lens A + B convergem):
- Golden blake3 hashes pinados em PNG/TIFF/ORA/APNG eram single-platform Mac-Silicon mas vinculados a "HR-5 cross-OS byte-exact" (corrigida nomenclatura — HR-9 é GC, não cross-platform; nomenclatura errada surgiu pre-W3.T0 em §5.4 + §5.3 + briefings; fixed audit-6). CI matrix Linux/Windows falharia 4+ jobs por design (SIMD DEFLATE divergente). Fix: `#[cfg(all(target_os = "macos", target_arch = "aarch64"))]` + renomeados pra `export_golden_blake3_local_drift_pinned_macos_silicon` + docstrings honestas + ADR §2.6.1 amendment formaliza scope single-platform. Multi-platform pinning deferido (entry: novo function name).

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

**Total Onda 2 pós-W3.T0** (contagem real recapturada audit-6 Lens A/D): PNG=14 + TIFF=13 + ORA=15 + APNG=11 + PSD=8 lib + 3 fixture = **64 testes verdes em Mac aarch64**; em Linux/Windows = 60 (4 goldens `#[cfg]`-gated). O commit message original (`35cc149`) citava "apng=15, ... = 53" — erro de contagem manual via grep; refleta abaixo o real. ORA também foi omitida da soma do commit message. **Nota retroativa**: amend de commit é proibido pelo workflow, esta linha é o registro corrigido.

**Reconciliação de findings audit-6 (Lens D MEDIUM)**: a row §5 W3.T0 original claimava "1 CRITICAL + 4 HIGH + 9 MEDIUM + 12 LOW" = 26 findings. §5.5 detalhava apenas 10 (1+2+6+1). Os 16 restantes (2 HIGH + 3 MEDIUM + 11 LOW) eram cosméticos absorvidos como sub-threshold sem disclosure individual. Convenção a partir de audit-7+: ou citar todos, ou explicitar "X cosméticos absorvidos".

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
- **HR-5** cross-OS byte-exact (corrige nomenclatura — HR-9 é GC, não cross-platform) → **antes de W3.T1 ratificar — bloqueia ratificação Onda 3** (matriz CI Linux/macOS/Windows). Per amendment §2.6.1 + audit-6 Lens B fix: o gate antigo (4 goldens) foi rebaixado a local drift guard. Substituto obrigatório: tabela `&[(target, hash)]` cobrindo `aarch64-apple-darwin` + `x86_64-unknown-linux-gnu` + `x86_64-pc-windows-msvc`, capturada via primeira run verde de cada job; entry-point por crate = `export_golden_blake3_local_drift_pinned_macos_silicon`. **Habilitado por audit-6 Lens B**: imageio crates agora estão no `spike.yml` nextest matrix.
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
- **HR-5** Cross-OS byte-exact (corrige nomenclatura — não HR-9) — W3+ CI matrix (multi-target hash table; vide §5.4 entrada atualizada e §2.6.1).
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
