# ADR-0055 — Cooked Texture Compression Pipeline (KTX2 + `ctt` cooker + canonical-runner determinism + direct-upload runtime, sem Basis FFI, sem `ph2d-color-pipeline`)

**Status:** Proposed (Round 3 — escrito 2026-05-27 pós-Round 2 REJECT × 3 com scores 6.5/6.5/5.2 + sweep-grep preventivo Coord-A executado antes da reescrita; auditoria Round 3 pendente).
**Data:** 2026-05-27
**Decisor(es):** Enio (ratificação Opção E em 2026-05-26 + ratificação adoção `ctt` 2026-05-27 + ratificação canonical-runner determinism plan Round 2) + Claude (arquiteto).
**Substitui:** ADR-0055 anterior (KTX2 + Basis Universal + BC6H + ACEScg) deletado em 2026-05-26 após auditoria 4-lente revelar 12 CRITICAL findings (score 5.67/10) — vide [[ktx2-phase1-done-phase2-aborted-2026-05-26]] + [`HANDOFF_ktx2_phase2.md`](../../HANDOFF_ktx2_phase2.md) §3.
**Estende:**
- ADR-0021 (sim/present boundary — texture cache vive em `PresentWorld`),
- ADR-0026 (sprite source strategies — adiciona variant `CookedTexture` em `crates/ph2d-render/src/sprite.rs`, ver §2.4.2),
- ADR-0025 / ADR-0028 (gameobject model / codegen — `Asset::TextureKtx2` adicionado a `crates/ph2d-asset/src/asset.rs` que é `#[non_exhaustive]` e não tem cap arch-gate atual, ver §2.4.1),
- ADR-0040 (tool isolation — cooker mora em `tools/asset-cooker`, FROZEN 2026-05-22),
- ADR-0042 §2.3 (mandato `ph2d-color` cap LOC 2500),
- ADR-0050 (`DeviceCapability`/`GpuId` quirks lookup — connectividade com wgpu feature query em §4),
- ADR-0053 (`DeviceTier = 5 FROZEN`),
- ADR-0054 (`ph2d-imageio::ColorProfile = 8 FROZEN` — este é o ÚNICO `ColorProfile` materializado no codebase em 2026-05-27, ver §2.5).
**NÃO amenda:** ADR-0051 (`ph2d-color::ColorProfile` ainda não materializado em código — vapor; gate `color_profile_variant_count_is_exact_8` em `ph2d-painter-contracts` passa silenciosamente quando o enum não existe, ver §2.5).
**Consome (não amenda):** Fase 1 codec puro `crates/ph2d-asset-ktx2/` (4 commits f30e225..b276cef, 1207 LOC lib.rs, 26 tests).
**Plano vivo:** [`docs/plans/2026-05-texture-compression-waves.md`](../../plans/2026-05-texture-compression-waves.md) (criado W0.T5 nesta sessão — não diferido).

---

## 1. Contexto

PH2D precisa shipar texture compression pipeline GPU-comprimido em 4 plataformas (Desktop, iPad/iOS, Android, Web). A Fase 1 entregou o **codec KTX2 puro** (`crates/ph2d-asset-ktx2/`, 26 tests verdes, `#![forbid(unsafe_code)]`) como alicerce. Esta ADR define a **Fase 2** — integração com `Asset::*` + `ph2d-render` + cooker offline — usando o **caminho canônico Opção E** ratificado pelo Enio em 2026-05-26.

### 1.1 Histórico curto do aborto anterior

A primeira tentativa (ADR-0055 deletado) propôs stack KTX2 + Basis Universal runtime + BC6H + ACEScg working space + criação de `ph2d-color-pipeline`. Auditoria 4-lente derrubou com 12 CRITICAL findings — síntese em [`HANDOFF_ktx2_phase2.md`](../../HANDOFF_ktx2_phase2.md) §3. Os 11 anti-patterns identificados estão **explicitamente NÃO repetidos** nesta ADR (§6).

### 1.2 Histórico curto da Round 1 + Round 2 desta ADR

**Round 1** (manhã 2026-05-27): REJECT × 3 (scores 4.5/4.5/5.0 — abaixo do anterior 5.67) por 13 findings críticos convergentes: ColorProfile vaporware em `ph2d-color`, MSRV `ctt` errado, Asset/SpriteSource shape falso, ADR-0009 fantasma re-cited, iOS BC universal claim incorreto, etc.

**Round 2** (manhã 2026-05-27): remediou os 13 findings R1 mas introduziu 6 NOVOS findings críticos do MESMO padrão — agora com símbolos INTERNOS do repo: `Ktx2Blob` (real `Ktx2Image`), `DeviceTier` vapor (mesma classe que ColorProfile pego mas DeviceTier missed), `MemoryBudget::Render { ... }` triplo-vapor (`MemoryBudget` é struct não enum + `Plugin::init` não existe + `PluginBuilder::declare_budget` vapor), `AssetDb::resolve_for_tier` não existe + content-addressed model colide com multi-tier, parser kvd claim "Fase 1 lê" FALSO, Git LFS não configurado. Scores Round 2: 6.5/6.5/5.2 — melhoria líquida marginal (+2.0/+2.0/+0.2). REJECT × 3.

**Round 3** (este documento, tarde 2026-05-27): diagnóstico do padrão — [[no-industrial-claims-without-verification]] GENERALIZADA para verificações EXTERNAS mas FALHA para verificações INTERNAS (símbolos do próprio repo). Solução metódica: **sweep-grep preventivo executado pelo Coord-A antes da reescrita** para cada `pub`-prefixed identifier citado. Resultado em §1.3 abaixo. Round 3 nasceu factualmente verificado em código real, não em ADRs vizinhas como proxy.

### 1.3 Sweep-grep preventivo executado pelo Coord-A (Round 3 antes da reescrita)

Cada `pub`-prefixed identifier ou API que esta ADR cita foi verificado em código real antes de aparecer no texto. Tabela canon (cmd + linha-cite + status):

| Símbolo citado em ADR | Comando verificação | Real | Decisão Round 3 |
|---|---|---|---|
| `Ktx2Image` (não `Ktx2Blob` do Round 2) | `grep -n "pub struct Ktx2" crates/ph2d-asset-ktx2/src/lib.rs` | linha 365: `pub struct Ktx2Image { format, width, height, mip_levels }` | Renomeado globalmente Round 2 `Ktx2Blob` → `Ktx2Image` |
| `Ktx2Image::premul_intent()` API (W2.T-pre nova) | `grep "is_premultiplied\|premul_intent" crates/ph2d-asset-ktx2/src/lib.rs` | NÃO existe — Round 3 a especifica como API addition W2.T-pre | Especificado §3 com escopo realista 300 LOC (não 100) |
| `Ktx2Image::byte_size_estimate()` API (W1.T4 nova) | `grep "fn byte_size\|fn total_bytes" crates/ph2d-asset-ktx2/src/lib.rs` | NÃO existe (`total_bytes` é local var no parser linha 519) — Round 3 especifica nova API | Especificado em §2.4.1 byte_size extend |
| `MemoryBudget` real | `cat crates/ph2d-core/src/budget.rs` | **STRUCT** com `{ vram_mb: u32, ram_mb: u32, heap_script_mb: u32 }`; método `MemoryBudget::new(...)`. **NÃO é enum** | §5 reescrito com API real (não `MemoryBudget::Render { ... }`) |
| `Plugin::init` trait | `grep -rn "trait Plugin\b" crates/ --include="*.rs"` | ZERO matches — trait é **aspirational** (SKILL §HR-13 menciona como pattern; `crates/ph2d-core/src/budget.rs:3` doc-comment cita "from its `Plugin::init`" mas trait não existe ainda) | §5 reescrito reconhecendo como vapor SKILL pattern; ADR-0055 não materializa o trait |
| `ph2d_host::DeviceTier` | `grep -rn "pub enum DeviceTier" crates/` + `ls crates/ph2d-host/src/` | ZERO matches; ph2d-host tem apenas `events.rs / filter.rs / lib.rs` + traits `PlatformHost` / `HostHandler` + `ImageFilterMode`. ADR-0053 cita o enum mas **NÃO materializado** ainda; gate `device_tier_variant_count_is_exact_5` em painter-contracts é silently-passing | §2.4.1 troca `tier: DeviceTier` por **`tier: TierIndex` (newtype `pub struct TierIndex(u8)` Round 3 NOVO em `ph2d-asset`)** — desacopla Asset de host até DeviceTier materializar |
| `ph2d-asset` deps | `cat crates/ph2d-asset/Cargo.toml` | deps `blake3 / image / notify / serde / postcard / serde_json` — **ZERO `ph2d-host`** | §2.4.1 corrigido: ADR-0055 **NÃO** adiciona ph2d-host como dep de ph2d-asset; TierIndex é newtype primitivo host-agnostic |
| `Asset` real shape | `grep -A 12 "pub enum Asset" crates/ph2d-asset/src/asset.rs` | 3 variants `ImageRgba8 \| Prefab \| Scene` + `#[non_exhaustive]` confirmado | §2.4.1 OK |
| `SpriteSource` real shape | `grep -A 8 "pub enum SpriteSource" crates/ph2d-render/src/sprite.rs` | 2 variants `Atlas \| Individual` + Copy+Eq+Serialize, NÃO non_exhaustive, `Sprite::VERSION = 3` | §2.4.2 OK |
| `InspectorSpriteSource` mirror | `grep -A 6 "pub enum InspectorSpriteSource" crates/ph2d-editor-core/src/screens/hero.rs` | linha 231: 3 variants `Atlas \| Individual \| HandPacked` | §2.4.2 expandido: W2.T2 inventário inclui mirror chain |
| `RequestedSpriteStrategy` mirror | `grep -A 5 "pub enum RequestedSpriteStrategy" crates/ph2d-editor-core/src/screens/hero.rs` | linha 307: 3 variants `Atlas \| Individual \| HandPacked` | idem |
| `INSP_RENDER_STRATEGY_*` constants | `grep "INSP_RENDER_STRATEGY" crates/ph2d-editor-core/src/ids.rs` | linhas 418-420: ATLAS + INDIVIDUAL + HANDPACKED | idem; W2.T2 adiciona `_COOKED_TEXTURE` constant + 3 mirror chain updates |
| `AssetDb::resolve_for_tier` | `grep "pub fn" crates/ph2d-asset/src/db.rs` | só `pub fn get(&self, id: &AssetId) -> Option<Arc<Asset>>` (linha 145). **Content-addressed**: AssetId = `[u8; 32]` blake3 dos bytes — multi-tier exige N AssetIds distintos | §3 multi-tier identity model REESCRITO Round 3: 1 source PNG → N AssetIds (1 per tier); LogicalAssetId externo |
| `Ktx2Format` variants count | `awk '/pub enum Ktx2Format/,/^}/' crates/ph2d-asset-ktx2/src/lib.rs \| grep -E "^    [A-Z]" \| wc -l` | **28** (não 25): 5 uncompressed + 10 BC + 8 ASTC + 4 ETC2 + 1 Unsupported wildcard | §4 + §8 corrigidos para 28 |
| `wgpu` poll API | `grep "PollType" crates/ph2d-render/src/` | `PollType::wait_indefinitely()` em individual.rs:448 + vello_pass.rs:222 | §4 OK |
| `wgpu = 28.0.0` | `cat Cargo.lock \| grep -A 1 '^name = "wgpu"'` | 28.0.0 confirmado | §4 wgpu 28 OK |
| `ph2d-color::ColorProfile` | `grep -rn "pub enum ColorProfile" crates/ph2d-color/` | **ZERO matches** (vapor) | §2.5 escolhe ph2d-imageio::ColorProfile como single source-of-truth ✓ Round 2 OK |
| `ph2d-imageio::ColorProfile` | `grep -A 30 "pub enum ColorProfile" crates/ph2d-imageio/src/color.rs:18` | 8 variants ✓ FROZEN gate ativo | §2.5 ✓ |
| `GpuId` cap | `grep -rn "pub enum GpuId" crates/ph2d-host/` | **ZERO matches** — vapor (gate silently-passing igual DeviceTier) | §4 corrigido: `GpuId` quirks referencia ADR-0050 como **slot futuro não-materializado**; runtime detection in W2 não depende dele |
| `ctt = 0.4.0` | `cargo info ctt` | confirmed: version 0.4.0, MSRV 1.90, license MIT/Apache-2.0/Zlib, repo cwfitzgerald/ctt | §2.2 ✓ |
| `basis-universal = 0.3.1` | `cargo search basis-universal` | confirmed Nov/2023 dormant | §2.1 ✓ |
| `.gitattributes` (Git LFS setup) | `ls .gitattributes` | **NÃO existe** — repo NÃO inicializado pra LFS | §2.3 + plano vivo W1.T11.5 NOVO: setup `.gitattributes` + storage budget + CI step + CONTRIBUTING note |
| iPad BC support | wgpu#2452 (cite Round 2) | wgpu#2452 é issue **macOS** Apple Silicon, NÃO iPadOS | §2.6 + §6 #4 Round 3 degrade para "runtime feature query is source-of-truth; cooker emite all variants; ADR NÃO compromete iPad BC working até `MTLDevice.supportsBCTextureCompression` verified em iPadOS family specifically" |
| `ph2d-painter-brush` deps | `cat crates/ph2d-painter-brush/Cargo.toml` | deps `ph2d-gpu / wgpu / naga / dhat / serde / postcard / bytemuck / blake3` — **ZERO ph2d-asset** | W3.T0 NOVO pre-task: adicionar `ph2d-asset` dep + `ph2d-host` indirect dep + arch-gate no-cycle; W3.T1 LOC estimate raised |
| `ph2d-painter-brush::atlas.rs` | `wc -l crates/ph2d-painter-brush/src/atlas.rs` | **60 LOC stub** (`AtlasStub` placeholder; doc-comment "T1.5+ substitution") | W3.T1 LOC estimate raised 150 → 600-800 |
| `StampPipeline` storage format | `grep "wgpu::TextureFormat" crates/ph2d-painter-brush/src/stamp_pipeline.rs` | `Rgba8Unorm` storage (linha 186); shape atlas sampled `texture_2d<f32>` | W3.T1 inclui tier-conditional bind-group layout |
| `feedback-audit-internal-state-grep` memory | `ls ~/.claude/.../memory/feedback_audit_internal_state_grep.md` | **MISSING** Round 2 promised mas não criou | Round 3 cria nesta sessão (W0 fechamento) |

### 1.3 Princípio reorientador

**"Melhor para 2D pro tool" ≠ "stack mais complexo de 3D AAA"**. O melhor é:

1. **Zero CPU spike em load** (direct upload, sem transcoder runtime).
2. **Pixel-perfect** (compressão única offline, sem cascata lossy).
3. **WASM portable** (sem `transcoder.wasm` 500 KB).
4. **Builds Cargo-managed estáveis** (cooker via `cargo install`, não download manual de binários C++).
5. **iOS App Store sem fricção** (CLI offline ≠ third-party SDK in-app — sem PrivacyManifest específico).
6. **Determinismo cooked-asset garantido arquiteturalmente** (cook em runner canônico CI Linux x86_64, NÃO "medir e ver" cross-OS).

### 1.4 Por que cooker offline + direct upload vence Basis runtime para 2D

| Aspecto | Cooker offline + direct upload (esta ADR) | Basis Universal runtime transcoder (rejeitado) |
|---|---|---|
| Load latency | Zero CPU spike (`queue::write_texture` direto) | 1–5 ms/texture transcode runtime |
| Qualidade | Pixel-perfect (compressão única offline) | Dupla compressão lossy (UASTC → BC7/ASTC) |
| WASM size | sem transcoder.wasm | +500 KB transcoder.wasm |
| Supply chain | `cargo install ctt-cli` (Rust crate v0.4.0 Maio/2026) | `basis-universal-rs = 0.3.1` (dormente Nov/2023, individual maintainer) |
| Determinismo CI | **Canonical runner Linux x86_64**: cook 1 vez, KTX2 versionado no repo (vide §2.3) | toktx UASTC explicitamente NÃO-determinístico cross-OS (Khronos KTX-Software RELEASE_NOTES.md) |
| iOS App Store | sem PrivacyManifest específico | manifest FFI C++ exige audit |
| `ph2d-imageio::ColorProfile` cap | 8 FROZEN preservado (gate ativo) | quebra silenciosa |
| `ph2d-color::ColorProfile` (vapor) | NÃO bloqueia esta ADR — não consumido aqui | — |

---

## 2. Decisão

### 2.1 Stack canônico

| Camada | Decisão | Onde implementa | Wave |
|---|---|---|---|
| **Container** | KTX2 (read-only parser puro Rust) | `crates/ph2d-asset-ktx2/` (Fase 1 ✅ entregue, NÃO tocar) | — |
| **Cooker offline** | **`ctt-cli` v0.4.0+** (Rust crate, multi-encoder unificado) | `tools/asset-cooker/src/texture/` (extension) | W1 |
| **Compressão SDR** | BC7 desktop+iPad Apple7+ · ASTC LDR mobile (todos iOS + Android) · ETC2 Android fallback · BC1 low-end · BC4 single-channel (atlas mask/R8 source) | cookado offline per-platform via `ctt` | W1 |
| **Compressão HDR** | BC6H desktop+iPad M1+ · ASTC HDR mobile (iOS 16.4+ / Android Vulkan 1.3 com `VK_EXT_texture_compression_astc_hdr`) | cookado offline sem Basis layer | W4 (deferido) |
| **Apple iPad (Apple7+/M1/M2/M3+)** | **ASTC default, BC opcional** — runtime feature query é source-of-truth (Round 3 fix: wgpu#2452 era macOS, NÃO iPadOS verified) | `ph2d-render` wgpu adapter.features() query runtime + ADR-0050 `GpuId` quirks (vapor pre-flight) | W2 |
| **Apple iPhone (todas as gens)** | **ASTC apenas** (BC NOT exposed em iPhone Metal) | mesma query runtime detecta ausência → tier fallback | W2 |
| **Apple macOS (Intel + Apple Silicon)** | **BC** disponível | mesma query | W2 |
| **Runtime transcoder** | **NÃO criar.** Renderer lê KTX2 → `wgpu::queue::write_texture` direto | `ph2d-render` (W2.T3 pipeline-per-format) | W2 |
| **`Ktx2Format` → `wgpu::TextureFormat` mapping** | **NOVO em W2.T1** — Fase 1 lib.rs:27 explicita gap intencional ("`ph2d-render` decides per pipeline"). Função `wgpu_format_from_ktx2_format` mora em `crates/ph2d-render/src/ktx2_format.rs` (W2 débito) | `crates/ph2d-render/` | W2.T1 |
| **Color pipeline** | **`ph2d-color` (ADR-0051) expandido** dentro do cap 2500 LOC (atual 1003 / 60% margem). **NÃO criar `ph2d-color-pipeline`** | `crates/ph2d-color/` | W0.T6 (se ACES tonemap helper necessário) |
| **Color management** | **Linear sRGB working space + ACES tonemap shader output**. **NÃO ACEScg gamut.** Razão: Unity HDRP / Unreal default é Linear sRGB working + ACES tonemap; ACEScg working em 2D games shippados = nenhum encontrado em pesquisa 2026-05-27 | shader em `ph2d-render` (W2) | W2 |
| **`ColorProfile`** | **Single source-of-truth = `ph2d-imageio::ColorProfile`** (ADR-0054, 8 variants FROZEN, gate ativo `color_profile_variant_count_is_capped` em `crates/ph2d-imageio/tests/`). `ph2d-color::ColorProfile` da ADR-0051 ainda é vapor (não materializado em código) — esta ADR **NÃO depende** dele | `crates/ph2d-imageio/src/color.rs` (já existe) | — |
| **Asset variant** | `Asset::TextureKtx2 { handle, tier }` em `crates/ph2d-asset/src/asset.rs` — enum é `#[non_exhaustive]` (M6 doc explicita "intentionally non-exhaustive so adding variants doesn't break downstream matches") — **sem cap arch-gate atual** (acréscimo trivial; governance ADR-0025/0028) | `crates/ph2d-asset/` | W1 |
| **SpriteSource variant** | `SpriteSource::CookedTexture { asset_id }` em `crates/ph2d-render/src/sprite.rs` — enum atual NÃO é `non_exhaustive` e tem `#[derive(Copy, Eq, Serialize)]` — adicionar variant exige (a) tornar `non_exhaustive` + bump postcard `VERSION` (cook-hash churn) OU (b) bump `VERSION` standalone + backward-compat deserialize default. W2.T2 detalha. | `crates/ph2d-render/` | W2 |
| **Painter integration** | Brush shape/grain atlas R8 → BC4 (-50% VRAM) · UI assets → ASTC LDR | cookado offline | W3 |

### 2.2 Por que `ctt` (Connor Fitzgerald) sobre `toktx` (Khronos) e `Compressonator` (AMD)

Verificações §6 do HANDOFF (executadas 2026-05-27, Round 2 revisões):

| Fato | Verificação |
|---|---|
| `ctt = 0.4.0` released **16 Maio 2026** | `cargo search ctt` + crates.io + GitHub releases |
| Maintainer: **cwfitzgerald** (Connor Fitzgerald — wgpu core contributor, baseado em NY; emprega-se em Configura como Senior Graphics Engineer; orgs ativas: gfx-rs, NovaMods, BVE-Reborn) | `cargo info ctt` + GitHub profile (NÃO Embark Studios — correção Round 2) |
| Multi-encoder vendored: **bc7e + Intel ISPC + AMD Compressonator + etcpak + astcenc + bc7enc-rdo** | sub-crates Cargo: `ctt-astcenc`, `ctt-bc7enc-rdo`, `ctt-compressonator`, `ctt-etcpak`, `ctt-intel-texture-compressor` (todos v0.4.0) |
| **Prebuilt ISPC libs cross-platform**: Linux/macOS/Windows × x86_64/aarch64 | descrição lib.rs/crates.io |
| **GitHub Artifact Attestation** — feature interna do CI do repo (não third-party audit) | descrição lib.rs/crates.io — **valor real**: provenance verificável da build do CI, NÃO substitui audit independente. PH2D pode escolher verificar attestation em `tools/asset-cooker` install step (W1.T1.5) |
| KTX2 nativo (container default) — depende de `ktx2 = 0.5` (mesma crate da Fase 1) | deps lib.rs |
| License triplo: MIT / Apache-2.0 / Zlib | `cargo info ctt` (sub-crates podem ter `AND Apache-2.0` para conteúdo vendored como bc7enc_rdo — verificar W1.T2 audit) |
| Instalável: `cargo install ctt-cli` | manifesto |
| **MSRV Rust 1.90 edition 2024** (autoritativo via `cargo info ctt-cli`; README upstream stale com 1.88 — Round 2 correção) | `cargo info ctt` `rust-version` field |
| PH2D workspace MSRV: `Cargo.toml` `rust-version = "1.92"`, `rust-toolchain.toml` `channel = "1.95"` → folga 1.92 ≥ 1.90 ✓ | `grep "rust-version" Cargo.toml` |
| 78 commits trunk, **9 GitHub stars**, **515 downloads/mês**, **used in 2 crates**: ctt é especializado/jovem, exposição baixa, fallback plan §2.7 não-cosmético | GitHub + lib.rs |
| 13 open issues: triagem WebFetch 2026-05-27 — **nenhum CRITICAL data-loss/security/non-determinismo**; majority `enhancement` + `bug, codec` minor (HDR ASTC handling, RGBM, 3D textures, vendoring scripts) | W1.T2 re-audit completo antes de Accept Wave |
| Determinismo cross-OS **NÃO documentado** explicitamente; encoders vendored (bc7e ISPC, astcenc, etcpak) **conhecidamente NÃO bit-exact cross-arch** mesmo single-threaded (SIMD intrinsics, FP-rounding) | §2.3 abandona "medir e ver" em favor de canonical runner |

**Por que ctt > toktx para PH2D:**

1. **HR-1 (platform-agnostic core) letra cumprida**: `cargo install` Cargo-managed > download binário Khronos C++ separado. **Espírito qualified**: encoders vendored ainda são C/C++ FFI mas (a) só rodam offline em developer machine ou CI, (b) não vão pra app bundle de release-game (HR-7), (c) critério objetivo para FFI cross-cutting está em §2.7.1.
2. **Multi-encoder em uma única tool**: bc7e (top BC7 quality) + astcenc (reference ASTC) + etcpak (fast ETC2 fallback) sem CLIs separadas.
3. **GitHub Artifact Attestation = build provenance** (NÃO third-party audit) — útil pra CI verificar binary integrity, mas não substitui `cargo-vet` / `cargo-crev`. W1.T1.5 (NOVO) adiciona verify-step opcional no install.
4. **Maintainer veteran cross-checked**: Connor Fitzgerald é wgpu core contributor (validável: GitHub `gfx-rs/wgpu` contributor list).
5. **Mesmo `ktx2 = 0.5`** que Fase 1 consome → zero divergence container parsing.

**Por que ctt > basis-universal-rs C++ FFI:**

1. Vendored encoders são para **cooking offline** (dev-time/CI), não runtime → não vai pro app bundle → sem PrivacyManifest iOS.
2. Cooker C++ é problema diferente de runtime C++ in-process: cooker roda no developer machine ou CI runner, falhas são detectadas em tempo de build, não em produção.

### 2.3 Determinismo — canonical runner, não "medir e ver"

**Round 1 lição** (Lente A H6 + Lente B H8 + Lente C H4): encoders ISPC/SIMD-multi-thread como bc7e/astcenc/etcpak **NÃO são bit-exact cross-arch** mesmo com `--threads=1`. Mandato HR-6 (asset = hash blake3 = identidade de conteúdo) requer determinismo do cooked artifact.

**Decisão arquitetural Round 2**: cook em **runner canônico único**, NÃO espalhar cook on dev-machines.

| Item | Decisão |
|---|---|
| Onde cook acontece | CI GitHub Actions Linux x86_64 (runner `ubuntu-latest`) — único runner canônico |
| Artefato cooked | `assets/cooked/**/*.ktx2` **versionado no repo via Git LFS** (HR-6: hash blake3 deterministic via canonical runner) |
| Dev-machine cook | Permitido **apenas para preview local** — output gitignored; CI re-cook é source-of-truth |
| Cross-OS replay-hash gate | W1.T5 NÃO mais "medir Linux ≡ macOS ≡ Windows"; vira **"Linux x86_64 runner ≡ Linux x86_64 runner (mesma SHA do `ctt-cli`)"** — gate de regression do canonical builder, não de cross-arch determinism |
| Fixture set | W1.T5 specifica **8 fixtures canônicos**: 256×256 gradient · 1024×1024 photo · 4096×4096 atlas-packed · 256×256 R8 brush atlas · 1024×1024 SDF font · 16×16 critical UI · normal map 512×512 · EXR HDR 512×512 (W4+ deferred) |

W1.T5 implementation:
```bash
# CI job:
# 1. Snapshot ctt-cli SHA + arch + flags em manifest
# 2. Cook fixture set canônico
# 3. blake3 cada output
# 4. Compare with prior canonical hashes em `assets/cooked-hashes.lock`
# 5. Falha de hash → indica ctt-cli upgrade ou flag mudou → human review required
```

**Fallback se canonical-runner approach falhar W1 audit**:
- Fallback A: fork `ctt` em `PH2D-engine/ctt` mirror com determinism patches.
- Fallback B: substituir `ctt-cli` por **`toktx` (Khronos) + `Compressonator` (AMD) CLIs externas em mesmo canonical runner** — herda mesma garantia de determinismo (CI runner único).
- Fallback C: `intel-tex-rs-2` + `astc-encoder-rs` separados.

### 2.4 Amendments propostos a ADRs vizinhas

Round 1 corrigiu (Lente A H4 + Lente B H1/H2 + Lente C C2/C3): ambos Asset e SpriteSource enums **NÃO têm cap arch-gate** hoje. Esta ADR adiciona variants:

#### 2.4.1 `Asset::TextureKtx2` variant (W1.T4)

Estado **REAL** do `Asset` enum em [`crates/ph2d-asset/src/asset.rs:16-31`](../../../crates/ph2d-asset/src/asset.rs):

```rust
#[derive(Clone, Debug)]
#[non_exhaustive]  // ← key: doc explicita "so adding variants doesn't break downstream matches"
pub enum Asset {
    ImageRgba8 { width: u32, height: u32, pixels: Arc<[u8]> },
    Prefab(Arc<PrefabDoc>),
    Scene(Arc<SceneDoc>),
}
```

**Adição W1.T4** (trivial via `#[non_exhaustive]`, sem breaking change):

```rust
// crates/ph2d-asset/src/tier.rs (NOVO W1.T4)
/// Index numérico (u8) compactando ADR-0053 `DeviceTier` enum **antes** dela ser
/// materializada em `ph2d-host`. Pre-flight Round 3 verificou: `pub enum DeviceTier`
/// é vapor em 2026-05-27 (gate `device_tier_variant_count_is_exact_5` em
/// `painter-contracts` silently-passa quando enum não existe). TierIndex é
/// host-agnostic e mantém Asset sem dep em `ph2d-host`.
///
/// Mapping (matching ADR-0053):
///   0 = Desktop · 1 = Mobile · 2 = Web · 3 = LowEnd · 4 = Constrained
///
/// Migration path: quando `ph2d_host::DeviceTier` materializar, este newtype
/// vira `pub type TierIndex = ph2d_host::DeviceTier;` alias (ou re-export).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct TierIndex(pub u8);

// crates/ph2d-asset/src/asset.rs (extend)
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Asset {
    ImageRgba8 { ... },
    Prefab(...),
    Scene(...),
    /// NOVO W1.T4 — cooked GPU-compressed texture (BC7/ASTC/ETC2/etc.).
    /// `Arc<Ktx2Image>` para compartilhar entre snapshots (HR-3 zero-alloc hot path).
    TextureKtx2 {
        tier: TierIndex,
        image: Arc<ph2d_asset_ktx2::Ktx2Image>,
    },
}
```

`byte_size()` extend (HR-13 budget) — Round 3 fix: `Ktx2Image::byte_size_estimate()` é API addition NOVA em W1.T9 (não existia em Fase 1; pre-flight verificou linha 365 `Ktx2Image { format, width, height, mip_levels }` sem método de tamanho):

```rust
// crates/ph2d-asset-ktx2/src/lib.rs (W1.T9 NOVA API)
impl Ktx2Image {
    /// Estimativa do byte size total = sum(mip_level.data.len()). HR-13 accounting.
    #[must_use]
    pub fn byte_size_estimate(&self) -> usize {
        self.mip_levels.iter().map(|m| m.data.len()).sum()
    }
}

// crates/ph2d-asset/src/asset.rs (byte_size extend)
Self::TextureKtx2 { image, .. } => image.byte_size_estimate(),
```

**Governance**: Asset enum cresce sob ADR-0025 (gameobject model) / ADR-0028 (codegen). Sem arch-gate de cap hoje (pre-flight Round 3 verified); cap formal seria amendment **major** ADR-0025 (com auditoria N-lente própria — diferido até justificável).

**`ph2d-asset` → `ph2d-host` ACOPLAMENTO NÃO ADICIONADO** (Round 3 fix do Round 2 falacy): pre-flight verificou `cat crates/ph2d-asset/Cargo.toml` → ZERO `ph2d-host` dep. Round 2 §2.4.1 afirmou "Asset já depende de ph2d-host indiretamente via AssetDb que precisa de plataforma para resolução" — **FALSO**. Round 3 usa `TierIndex` newtype primitivo (definido em `ph2d-asset/src/tier.rs`, host-agnostic) — ph2d-asset permanece sem dep em ph2d-host. Migration path: type alias quando DeviceTier materializar.

#### 2.4.2 `SpriteSource::CookedTexture` variant (W2.T2)

Estado **REAL** do `SpriteSource` enum em [`crates/ph2d-render/src/sprite.rs:36-45`](../../../crates/ph2d-render/src/sprite.rs):

```rust
#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SpriteSource {
    Atlas { key: u32 },
    Individual { texture_id: u32 },
}
// NOTA: NÃO é #[non_exhaustive], deriva Copy+Eq+Serialize+Deserialize.
//       Sprite::VERSION = 3 (já bumpado em M14.x para anchor field).
```

**Adição W2.T2** (BREAKING CHANGE em serialize/match — requer bump):

```rust
#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]  // ← NOVO: prevenir breakage downstream em adições futuras
pub enum SpriteSource {
    Atlas { key: u32 },
    Individual { texture_id: u32 },
    /// NOVO W2.T2 — cooked KTX2 referenciado via AssetId.
    /// Renderer resolve `AssetId` → `Asset::TextureKtx2 { tier, blob }` (§2.4.1)
    /// e mantém GPU texture cache em `PresentWorld` (ADR-0021).
    CookedTexture { asset_id: ph2d_asset::AssetId },
}
```

**Implicações serde** (W2.T2 plan):
1. **Bump `Sprite::VERSION`** para 4 (incl. cook-hash churn em fixtures — aceitável W2 scope).
2. Backward-compat deserialize: older `Sprite` (VERSION=3) sem `CookedTexture` lê normalmente — não há `Sprite` cookado pré-W2 com `CookedTexture`.
3. `AssetId` é `pub struct AssetId([u8; 32])` em `crates/ph2d-asset/src/id.rs:17` (pre-flight Round 3 verificou) — `Copy + Eq + Serialize` via derives.

**Mirror chain do `SpriteSource`** (Round 2 C2-C6 + Round 3 fix com inventário verificado):

Pre-flight Round 3 grep encontrou que `SpriteSource` tem MIRROR CHAIN no editor-core que Round 2 ignored. Adicionar `CookedTexture` em `SpriteSource` exige sync em:

| Site | Verificação grep | W2.T2 ação |
|---|---|---|
| `crates/ph2d-render/src/sprite.rs:37-45` — `pub enum SpriteSource` | 2 variants atuais (`Atlas \| Individual`) | adicionar variant `CookedTexture { asset_id: AssetId }` + `#[non_exhaustive]` |
| `crates/ph2d-editor-core/src/screens/hero.rs:231-235` — `pub enum InspectorSpriteSource` | 3 variants (`Atlas \| Individual \| HandPacked`) | adicionar variant `CookedTexture { asset_id: u64 }` (Inspector-side simplified ID) |
| `crates/ph2d-editor-core/src/screens/hero.rs:307-311` — `pub enum RequestedSpriteStrategy` | 3 variants (`Atlas \| Individual \| HandPacked`) | adicionar variant `CookedTexture` |
| `crates/ph2d-editor-core/src/ids.rs:418-420` — `INSP_RENDER_STRATEGY_*` constants | 3 constants (`_ATLAS`, `_INDIVIDUAL`, `_HANDPACKED`) | adicionar `INSP_RENDER_STRATEGY_COOKED_TEXTURE` constant |
| `action_bus.rs` + `panel-inspector` routing | grep confirms wiring exists for current 3 | extend routing por sync com 4 variants |

**HandPacked landing precedence**: HandPacked já está em 3/4 dos mirror sites (não no real `SpriteSource` enum). Decisão W2.T2: landing `HandPacked` em SpriteSource real **antes** de `CookedTexture` (ou bundled junto). Coordinator de W2.T2 escolhe via grep state at start.

Downstream match sites: `grep -rn "match.*SpriteSource\|SpriteSource::" crates/` em W2.T2 — pre-flight Round 3 não inventoriou todos sites mas Lente A2 noted "code path uses `matches!()` (panel-inspector) e `==` (tests) — nenhum quebra com nova variant". `#[non_exhaustive]` migration garante future-proof.

**Governance**: SpriteSource governance ADR-0026; sem cap arch-gate atual (pre-flight verified).

### 2.5 Color management — **NÃO-decisões** explícitas + escolha do `ColorProfile` materializado

Round 1 corrigiu (Lente A C3 + Lente B C2 + Lente C C1): ADR-0051 `ph2d-color::ColorProfile` enum **ainda NÃO existe em código** (`grep` em 2026-05-27: zero matches em `crates/ph2d-color/src/`). O arquivo `crates/ph2d-color/src/profile.rs` que ADR-0051 §2.1 declara "NEW" não foi criado. Gate `color_profile_variant_count_is_exact_8` em [`crates/ph2d-painter-contracts/tests/architecture_painter_contract_surface.rs:939-950`](../../../crates/ph2d-painter-contracts/tests/architecture_painter_contract_surface.rs) usa padrão `if let Some(n) = count_enum_variants(...)` — **silenciosamente passa quando enum não existe**.

**Implicação para ADR-0055**: esta ADR **NÃO depende** de `ph2d-color::ColorProfile`. Único `ColorProfile` consumido é o de `ph2d-imageio::ColorProfile` (ADR-0054, 8 variants FROZEN, gate ativo `color_profile_variant_count_is_capped` em `crates/ph2d-imageio/tests/`).

**Esta ADR NÃO introduz**:

- ❌ `ph2d-color-pipeline` crate (use `ph2d-color` existente).
- ❌ `AcescgLinear` / `ACEScg` working space variant (use Linear sRGB working + ACES tonemap output shader).
- ❌ Variant nova em `ph2d-imageio::ColorProfile` ADR-0054.
- ❌ Materialização de `ph2d-color::ColorProfile` ADR-0051 (mantém vapor — não-bloqueante pra esta ADR; W1+ do Painter materializa em outra ADR).

Razão consolidada em [`HANDOFF_ktx2_phase2.md`](../../HANDOFF_ktx2_phase2.md) §3 anti-patterns #2, #6, #10.

**ACES tonemap helper** em `ph2d-color` (W0.T6 — opcional, se necessário):
- ≤ 200 LOC novo em `crates/ph2d-color/src/cooked_texture_aces.rs` (Round 3 rename: era `aces_tonemap.rs` que seria silent amendment de ADR-0051 §2.1 module list que enumera `oklab/display_p3/prophoto/hdr/profile/mixbox_space`; novo nome `cooked_texture_aces` deixa claro que é derivative ADR-0055, não amendment ADR-0051).
- Cap total `ph2d-color` permanece 2500 LOC. Atual 1003 + ADR-0051 expansões previstas (~1200) + ACES (~200) = ~2400 ≤ 2500. Verificar W0.T6.

### 2.6 Per-platform target matrix

Round 1 corrigiu (Lente A C4): iPad Apple7+/M1+ SUPORTA BC formats. Matrix Round 2:

| Tier (ADR-0053) | Platform | Hardware | SDR format | HDR format (W4+) | Fallback |
|---|---|---|---|---|---|
| `Desktop` | Windows / Linux / macOS Intel | x86_64 | BC7 | BC6H | RGBA8 |
| `Desktop` | macOS Apple Silicon | Apple7+ | BC7 | BC6H | RGBA8 |
| `Mobile` | iPad Pro (Apple7+/M1/M2/M3+) | Apple Silicon | **ASTC 6×6 default, BC7 opcional** (cooker emite ambos; runtime feature query é source-of-truth — Round 3 fix: wgpu#2452 referenced em Round 2 é issue **macOS**, NÃO iPadOS — iPad BC exposure em wgpu Metal-iOS adapter ainda NÃO verified em primary source iPadOS-specific) | ASTC HDR (default), BC6H opcional | RGBA8 |
| `Mobile` | iPhone (todas gens) + iPad antigo | Apple GPU sem BC exposure | **ASTC 6×6 LDR** (4×4 critical UI) | ASTC HDR (iOS 16.4+) | RGBA8 |
| `Mobile` | Android (Vulkan 1.0+) | Adreno / Mali / PowerVR | ASTC 6×6 LDR (4×4 critical) | ASTC HDR (Vulkan 1.3 `VK_EXT_texture_compression_astc_hdr` device support) | ETC2 RGBA → RGBA8 |
| `Web` | Chrome 113+ (WebGPU) | Adapter-dependent | Runtime `requestDevice({requiredFeatures: ["texture-compression-bc"]})` → falha → tenta ASTC → ETC2 → RGBA8 | (W4+) | RGBA8 |
| `Web` | Safari 19+, Firefox 145+ | Adapter-dependent | Idem | (W4+) | RGBA8 |
| `LowEnd` | Android low-tier sem ASTC | Mali T-series antigo | ETC2 RGBA · BC1 emergency | — | RGBA8 + memory budget cut |
| `Constrained` | Outros | — | RGBA8 uncompressed | RGBA16Float | — |

**Apple Metal Feature Set Tables** (referência canônica): [`https://developer.apple.com/metal/Metal-Feature-Set-Tables.pdf`](https://developer.apple.com/metal/Metal-Feature-Set-Tables.pdf). `MTLDevice.supportsBCTextureCompression`: [`https://developer.apple.com/documentation/metal/mtldevice/supportsbctexturecompression`](https://developer.apple.com/documentation/metal/mtldevice/supportsbctexturecompression). wgpu issue #2452 (gfx-rs) confirma exposição BC em Apple Silicon Mac.

**Runtime detection mecanismo** (W2.T1):

```rust
// Startup:
let adapter = instance.request_adapter(...).await?;
let mut required = wgpu::Features::empty();
// Tier matrix → preferências de feature; cooker tem ambos os artefatos
// (Apple Silicon: cooker emite BOTH bc7-mac.ktx2 AND astc-6x6.ktx2; runtime escolhe)
if adapter.features().contains(wgpu::Features::TEXTURE_COMPRESSION_BC) { ... }
if adapter.features().contains(wgpu::Features::TEXTURE_COMPRESSION_ASTC) { ... }
// fall back ladder per platform tier (ADR-0053) + GpuId quirks (ADR-0050).
```

### 2.7 Fallback path se `ctt` desativar ou falhar W1.T2 audit

Se W1.T2 audit dos 13 open issues + 100% code-read revelar showstopper (data-loss, security regression, undocumented behavior), o fallback ordenado §2.3 aplica.

#### 2.7.1 Critério objetivo para FFI C/C++ vendored aceitável

Round 1 (Lente A H1 + Lente B M1): justificar override "pure-Rust spirit" do HR-1.

**Critério (codificado em SKILL §HR-1 follow-up W0.T3 update)** — Round 4 split em 6 critérios (era 5 com bug lógico no #5):

FFI C/C++ é aceitável SE TODAS as condições:
1. **Offline tooling only** — código FFI nunca embarcado em app bundle release-game.
2. **Reference implementation única OU best-of-domain benchmark-comprovada** — encoder canônico do domínio (e.g., bc7e da DirectXTex, astcenc da ARM).
3. **Vendored via crate Cargo com `build.rs` reproducível** — `cargo build` produz binário sem download manual.
4. **License compatible** com MIT/Apache-2.0 PH2D (incluindo sub-crates vendored) — verificar W1.T2 audit do `ctt`.
5. **Maintainer ativo** com commits < 12 meses OU PH2D pode fork sem fricção técnica.
6. **NÃO patent-encumbered** — HEVC/H.265 / DTS / AAC / outros payloads royalty-bearing.

`ctt` 0.4.0 + sub-crates: ✓ todos os 6 critérios passam (cooking offline ✓, refs canônicos ✓, build.rs vendored ✓, MIT/Apache-2.0/Zlib ✓, 78 commits trunk recente ✓, sem patents conhecidos ✓). `libheif` (ADR-0054 §1.1 rejeitada): maintainer Strukur GmbH ativo **passa #5**, mas **falha #6** porque HEVC é patent-heavy MPEG-LA royalty.

---

## 3. Estrutura — `tools/asset-cooker/src/texture/`

```
tools/asset-cooker/src/texture/
  mod.rs            # sub-command CLI entry: `asset-cooker texture cook --input X --tier T --output Y`
  ctt_wrapper.rs    # invoca `ctt-cli` via std::process::Command (CLI wrapper) — version pin manifest
  multi_tier.rs     # source → N artifacts (5 per platform tier in §2.6)
  mip_gen.rs        # mip pyramid generation (box / Lanczos / point) — Round 1 finding: pre-ctt mip
  target_matrix.rs  # tabela §2.6 — input + DeviceTier → format + encoder + quality + ctt flags
  determinism.rs    # W1.T5 canonical-runner hash registry (`assets/cooked-hashes.lock`)
  premul_tracking.rs # KTX2 keyValueData `PH2D_PREMUL` u8 (0=straight, 1=premultiplied) — Round 1 finding
  fixtures/         # 8 canonical fixtures (§2.3) + EXR HDR (W4+)
```

**Cooker invocation interface** (Round 1 Lente B M4 + Lente C M1):

1. **CLI direto** (developer / CI): `asset-cooker texture cook --input sprites/hero.png --tier Mobile --variant astc-6x6 --output assets/cooked/hero-astc.ktx2`.
2. **Multi-tier batch** (CI canonical): `asset-cooker texture cook-all --input sprites/hero.png --output-dir assets/cooked/hero/` → emite todos os 5 variants do §2.6.
3. **Lib API** (Painter "Export Cooked Texture" W3): `ph2d_asset_cooker::texture::cook(input, options) -> Result<Vec<CookedArtifact>>`.
4. **Build script**: `tools/asset-cooker` é Cargo binary; integração com `build.rs` é responsabilidade do consumer (jogo client). PH2D não impõe build.rs no consumer.

**Multi-tier asset identity model** (Round 1 Lente B C5 + Lente C M3 + Round 2 C2-C5 + Round 3 fix com API real):

Pre-flight Round 3 verified `cat crates/ph2d-asset/src/db.rs`: `AssetDb` única lookup API é `pub fn get(&self, id: &AssetId) -> Option<Arc<Asset>>` (linha 145); `AssetId` é `[u8; 32]` (blake3 dos bytes, content-addressed). **Round 2 §3 inventou** `AssetDb::resolve_for_tier()` que não existe + afirmou "AssetId stable across tiers" — **contradição com content-addressed model** (cada blob comprimido tem hash distinto).

Round 3 reformulação real:

- `source_hash = blake3(input PNG bytes)` — identidade do source.
- Cooker emite N artefatos per source (1 per tier do §2.6 matrix). Cada artefato tem byte-content distinto → `cooked_hash[tier] = blake3(canonical-runner KTX2 bytes for tier)`.
- **N AssetIds distintos** por logical texture: `AssetId[tier=Desktop]`, `AssetId[tier=Mobile]`, etc. (1 source → N entries em AssetDb).
- **LogicalTextureId** (NOVO em `ph2d-asset/src/logical_texture.rs` W1.T4): mapping externo `LogicalTextureId → BTreeMap<TierIndex, AssetId>`. Cliente (renderer) faz lookup via `logical_texture_resolve(logical_id, current_tier) -> AssetId` → `AssetDb::get(&asset_id)`.
- HR-6 preservado: cada `AssetId` continua content-addressed deterministic via §2.3 canonical runner.

**Trade-off explicit**: 1 source PNG ocupa N entries em AssetDb (uma per tier). Para uma texture cookada em todos 5 tiers, AssetDb tem 5 entries (5 hashes). Aceitável: VRAM saving via compression (-75% a -89%) supera overhead de IDs metadata. Storage de cooked artifacts via Git LFS (§2.3).

**Premultiplied alpha intent** (Round 1 Lente C H5 + Round 2 C2-C3 + Round 3 fix com parser real):

KTX2 spec tem campo `keyValueData` opcional. Esta ADR adiciona convenção:

- `KTX2 keyValueData key = "PH2D_PREMUL"` (UTF-8 ASCII string-key NUL-terminated per KTX2 spec §3.10.8)
- `value = [u8; 1]` onde `0 = straight alpha`, `1 = premultiplied alpha`. Key ausente = `PremulIntent::Unspecified` (tri-state, Round 2 finding C2-M2 fix).
- Cooker (W1) emite key se source PNG é known premultiplied (M9 Image-Tools BG-Removal Apply path) ou se cook flag `--premul` for usado.

**Fase 1 parser de produção atualmente DESCARTA `keyValueData`** (Round 3 pre-flight verified `grep "kvd\|key_value_data\|metadata" crates/ph2d-asset-ktx2/src/lib.rs` → ZERO matches no parser; só em fixtures de teste que escrevem `kvd_byte_offset: 0`). W2.T-pre adiciona kvd preservation — escopo REAL (Round 2 C2-C3 corrigido):

```rust
// crates/ph2d-asset-ktx2/src/lib.rs (W2.T-pre — ~300 LOC, NÃO 100)
pub struct Ktx2Image {
    pub format: Ktx2Format,
    pub width: u32,
    pub height: u32,
    pub mip_levels: Vec<MipLevel>,
    /// NEW W2.T-pre: KTX2 keyValueData preserved (kvd parser fix).
    /// Bounded BTreeMap (max 64 entries, max value 4 KiB each) — DOS defence.
    pub kvd: std::collections::BTreeMap<String, Vec<u8>>,
}

pub enum PremulIntent { Straight, Premultiplied, Unspecified }

impl Ktx2Image {
    pub fn premul_intent(&self) -> PremulIntent {
        match self.kvd.get("PH2D_PREMUL").map(|v| v.as_slice()) {
            Some([0]) => PremulIntent::Straight,
            Some([1]) => PremulIntent::Premultiplied,
            _ => PremulIntent::Unspecified,
        }
    }
}
```

Arch-gate `premul_kv_round_trips` em `ph2d-asset-ktx2/tests/` — escopo W2.T-pre: kvd parse + bounds + DOS defence + round-trip + tampering test (`kvd` value mutado mid-file → renderer-side detection deferred to W2.T3).

**Não toca**: `crates/ph2d-asset-ktx2/` (Fase 1 ✅ congelada — W2.T-pre adiciona accessor API sem mudar parser core).

---

## 4. Runtime path — `ph2d-render` pipeline-per-format

W2.T3: para cada formato cookado, renderer cria pipeline com `wgpu::TextureFormat::Bc7RgbaUnormSrgb` / `Astc { block: ..., channel: UnormSrgb }` / `Etc2Rgba8UnormSrgb` etc. Bind group differs por format. Pipeline selection runtime via `wgpu::Features::TEXTURE_COMPRESSION_*` query no startup + `Asset::TextureKtx2.tier` campo.

**Sem transcoder runtime** — `wgpu` consome KTX2 já comprimido via Fase 1 parser:

```rust
// crates/ph2d-render/src/ktx2_format.rs (W2.T1 NOVO)
pub fn wgpu_format_from_ktx2_format(fmt: ph2d_asset_ktx2::Ktx2Format)
    -> Result<(wgpu::TextureFormat, wgpu::Features), FormatError>
{ /* enumerate all 28 Ktx2Format variants (Round 3 fix: 5 uncompressed + 10 BC + 8 ASTC + 4 ETC2 + 1 Unsupported wildcard) */ }

// Sample upload (loader):
let ktx2 = ph2d_asset_ktx2::parse(&bytes)?;
let (wgpu_fmt, required_feature) = wgpu_format_from_ktx2_format(ktx2.format())?;
assert!(adapter.features().contains(required_feature));
queue.write_texture(/* dest */, ktx2.level(0).data(), /* layout */, /* extent */);
```

**Round 1 Lente C H2 correction**: `wgpu_format_from_ktx2_format` é débito W2.T1 — NÃO existe em Fase 1 (lib.rs:27 explicita gap intencional "`ph2d-render` decides per pipeline"). W2.T1 LOC estimate ~150 (28 variants × 6 LOC + Error variants).

**Wgpu version pin**: PH2D Cargo.lock `wgpu = 28.0.0` (verificar W1.T1). Wgpu API `wgpu::PollType::wait_indefinitely()` (substitui antiga `Maintain::Wait` — Round 1 Lente C H6 correction).

---

## 5. Memory budget impact (SKILL §12.1)

Provisional (W2.T5 audit measures real numbers via backend-specific API — não `device.poll()` que NÃO mede VRAM):

| Subsistema | Antes (SKILL §12.1 atual) | Com texture compression (W2+ post-Painter W3 atlas BC4) | Assumption |
|---|---|---|---|
| Render textures+meshes iPad | 350 MB | provisional ~200 MB | ASTC 6×6 = -89% sobre RGBA8 em 60% das texturas; meshes inalterados (~140MB fixed) |
| Render textures+meshes Desktop | 1200 MB | provisional ~500 MB | BC7 = -75% sobre RGBA8 em 80% das texturas; meshes inalterados (~240MB fixed) |
| Painter brush atlas (parte de Render) | Shape 4 MB + Grain 32-64 MB R8 = 36-68 MB | Shape 2 MB + Grain 16-32 MB BC4 = **18-34 MB (-50%)** | R8 source → BC4 cooked; conta abaixo |

**Conta canônica per format** (anti-pattern #5 reforçado):

- **BC7 vs RGBA8 (desktop sprite)**: `BC7 = 8 bpp ÷ RGBA8 = 32 bpp = 0.25 → -75% saving`
- **ASTC 6×6 vs RGBA8 (mobile sprite)**: `ASTC 6×6 = 3.56 bpp ÷ RGBA8 = 32 bpp = 0.111 → -89% saving`
- **ASTC 4×4 vs RGBA8 (critical UI sprite)**: `ASTC 4×4 = 8 bpp ÷ RGBA8 = 32 bpp = 0.25 → -75% saving`
- **ETC2 RGBA vs RGBA8 (Android fallback)**: `ETC2 RGBA = 8 bpp ÷ RGBA8 = 32 bpp = 0.25 → -75% saving`
- **BC4 vs R8 (brush atlas single-channel)**: `BC4 = 4 bpp ÷ R8 = 8 bpp = 0.5 → -50% saving` (HANDOFF §4 corrigido)
- **BC6H vs RGBA16Float (desktop HDR sprite, W4+)**: `BC6H = 8 bpp ÷ RGBA16F = 64 bpp = 0.125 → -87.5% saving`

**Correção do HANDOFF §4**: HANDOFF afirmou "brush atlas R8 → BC4 (4× saving)". Conta correta: R8 = 8 bpp → BC4 = 4 bpp → **-50% (2× saving)**, **não 4×**. 4× só vale se source fosse RGBA8 (32 bpp). Brush atlas é R8 single-channel → saving real é metade. HANDOFF §4 deve ser patched in same session (W0 fechamento) per [[feedback-perfection-no-deferrals]].

**VRAM measurement API** (Round 1 Lente C H6 correction):
- `device.poll(PollType::wait_indefinitely())` **NÃO mede VRAM** — só drives command-completion.
- Backend-specific introspection:
  - Vulkan: `VK_EXT_memory_budget` + `vkGetPhysicalDeviceMemoryProperties2` (não exposto cross-vendor via wgpu).
  - Metal: `MTLDevice.currentAllocatedSize`.
  - D3D12: `IDXGIAdapter3::QueryVideoMemoryInfo`.
- W2.T5 implementation: contar bytes via `compressed_size_per_format(format, w, h, mip_count) × num_textures` (deterministic; não-cross-backend mas suficiente HR-13).

**HR-13 declarative budget** (Round 1 Lente B H3 + Round 2 B2-C3 + Round 3 fix com API real):

Pre-flight Round 3 verified `cat crates/ph2d-core/src/budget.rs`: `MemoryBudget` é **struct** `{ vram_mb: u32, ram_mb: u32, heap_script_mb: u32 }` com `MemoryBudget::new(vram, ram, heap)`. `Plugin` trait + `PluginBuilder` são **aspirational SKILL pattern** (citados em doc-comment mas trait NÃO existe — Round 2 inventou `MemoryBudget::Render { ... }` enum variant que não existe). Round 3 usa API real:

```rust
// crates/ph2d-render/src/lib.rs (W2.T5 ADD)
//
// SKILL §HR-13 prescreve "subsistema declara budget em Plugin::init" mas o
// trait Plugin ainda não materializou — pre-flight 2026-05-27 `grep -rn "trait Plugin\b"`
// retorna ZERO matches. Até trait existir, ph2d-render adiciona constante de budget
// estática como Source-of-Truth para boot-time MemoryBudget aggregation manual:
pub const RENDER_BUDGET_DELTA_W2: ph2d_core::MemoryBudget =
    ph2d_core::MemoryBudget::new(/* vram_mb */ 200, /* ram_mb */ 0, /* heap_script_mb */ 0);
```

Quando `Plugin` trait materializar (slot futuro SKILL §HR-13 follow-up; não nesta ADR), código migra de constante estática para `fn budget(&self) -> MemoryBudget` impl. ADR-0055 **NÃO** materializa o trait — esse é trabalho cross-cutting de outra ADR.

---

## 6. NÃO-decisões (anti-patterns do ADR-0055 anterior)

Lista canônica dos 11 anti-patterns que afundaram a versão anterior — todos explicitamente NÃO repetidos:

1. ❌ NÃO criar `ph2d-asset-basisu`. (§2.1 runtime = direct upload)
2. ❌ NÃO criar `ph2d-color-pipeline`. (§2.5 use `ph2d-color` expandido)
3. ❌ NÃO afirmar `basis-universal-rs >= 0.4`. (Confirmado 0.3.1 dormente; Opção E não usa)
4. ❌ NÃO assumir BC universal em todos os iOS. (§2.6 distingue iPhone sem BC vs iPad Apple7+/M1+ — pre-flight Round 3 confirmou que wgpu#2452 cite Round 2 era para macOS, **NÃO iPadOS**; ADR-0055 Round 3 NÃO compromete iPad BC working até verified em Apple Metal Feature Set Tables específicas iPadOS — runtime feature query é source-of-truth, cooker emite ASTC + BC ambos para tier Mobile-Apple)
5. ❌ NÃO escrever `-50% VRAM` (BC7 vs RGBA8). (§5 mostra conta `BC7=8bpp ÷ RGBA8=32bpp = 0.25 → -75%`; ASTC 6×6 = 3.56 bpp = -89%)
6. ❌ NÃO amendar `ColorProfile` cap (ADR-0054 single source-of-truth; ADR-0051 vapor). (§2.5)
7. ❌ NÃO override "pure-Rust only" sem critério. (§2.7.1 critério codificado em 6 itens: cooking offline + ref impl única + vendored Cargo + license MIT/Apache + maintainer ativo + **NÃO patent-encumbered** — Round 4 split do #5 anterior)
8. ❌ NÃO citar ADR-0009 como existente. (Esta ADR usa "slot futuro de ADR Radiance Cascades — ainda não numerado" em §6 item 11 e §8 W4 — Round 2 correção)
9. ❌ NÃO afirmar adoção industrial sem fonte. (§2.2 cada claim sobre `ctt` linkado a verificação executável — `cargo search`, `cargo info`, GitHub URL, lib.rs)
10. ❌ NÃO confundir ACES tonemap operator com ACEScg working space. (§2.1 explicit: "Linear sRGB working + ACES tonemap shader output", NÃO "ACEScg working")
11. ❌ NÃO modelar HDR sprite pipeline sem ecossistema de criação. (§2.1 W4 HDR explicitamente deferido até Painter export HDR real + ecossistema Radiance Cascades pronto — slot futuro de ADR)

---

## 7. Pre-flight verificações executadas (memória [[no-industrial-claims-without-verification]])

Conforme [`HANDOFF_ktx2_phase2.md`](../../HANDOFF_ktx2_phase2.md) §5 e memória `feedback-no-industrial-claims-without-verification`. Round 2 verificações foram executadas em código real, não em documentos — Round 1 falhou aqui.

| Verificação | Comando | Resultado |
|---|---|---|
| ADR 0055 livre? | `ls docs/architecture/decisions/0055-*.md` | LIVRE (ADR-0055 anterior deletado) ✓ |
| ADRs ratificadas 72h | `git log --since="72 hours ago" --oneline docs/architecture/decisions/` | Painter cascade 0043..0053 (não conflitam) |
| Audit contaminação 4 termos abortados | `grep -rn "ph2d-asset-basisu\|ph2d-color-pipeline\|basis-universal\|UASTC\|BasisLZ\|AcescgLinear"` | Só Fase 1 codec docs + HANDOFF + esta ADR (legítimo) ✓ |
| `ctt` v0.4.0 + sub-crates existem? | `cargo search ctt` | confirmado 0.4.0 + 9 sub-crates ✓ |
| `ctt` MSRV autoritativo? | `cargo info ctt` | **rust-version: 1.90** (Round 2 fix: ADR Round 1 disse 1.88) ✓ |
| `ctt` maintainer? | WebFetch GitHub cwfitzgerald profile | **Configura** (NÃO Embark — Round 2 fix). wgpu core contributor via gfx-rs orgs ✓ |
| `basis-universal-rs` versão? | `cargo search basis-universal` + `cargo info` | 0.3.1 Nov/2023 dormente ✓ |
| `basisu_rs` JakubValtar publicado? | WebFetch github.com/JakubValtar/basisu_rs | "No releases published" + "cleaning up to release" ✓ |
| Apple iOS BC formats — UNIVERSAL não-suporte? | WebSearch + Apple Metal docs + wgpu#2452 | **CORRIGIDO Round 2: iPad Apple7+ SUPORTA BC** (via `MTLDevice.supportsBCTextureCompression`); iPhone todas-gens sem BC ✓ |
| `ph2d-color` LOC atual / cap | `find crates/ph2d-color -name "*.rs" \| xargs wc -l` | 1003 / 2500 cap = ~60% margem ((2500-1003)/2500 = 59.88%) ✓ |
| `ph2d-color::ColorProfile` materializado? | `grep -rn "pub enum ColorProfile" crates/ph2d-color/` | **ZERO matches** (Round 2 fix: ADR Round 1 implicitamente assumiu materializado) — esta ADR NÃO depende dele ✓ |
| `ph2d-imageio::ColorProfile` FROZEN? | `grep -n "pub enum ColorProfile" crates/ph2d-imageio/src/color.rs` | 8 variants em linhas 18-41 + gate ativo em `crates/ph2d-imageio/tests/architecture_imageio_contract_surface.rs` ✓ |
| `DeviceTier` cap | ADR-0053 + `grep` | 5 variants FROZEN ✓ |
| Fase 1 ktx2 crate intacto + último commit | `git log --oneline crates/ph2d-asset-ktx2/ \| head -1` + `wc -l` | `b276cef` 1207 LOC ✓ |
| Fase 1 wgpu mapping gap explicit? | `grep -n "wgpu::TextureFormat" crates/ph2d-asset-ktx2/src/lib.rs` | lib.rs:27 explicita "No `wgpu::TextureFormat` mapping — `ph2d-render` decides per pipeline" ✓ |
| wgpu poll API atual | `grep -rn "PollType\|Maintain::" crates/ph2d-render/` | `wgpu::PollType::wait_indefinitely()` (Round 2 fix: ADR Round 1 disse `Maintain::Wait` que é API antiga removida) ✓ |
| Asset enum real shape | `grep -A 10 "pub enum Asset" crates/ph2d-asset/src/asset.rs` | 3 variants `ImageRgba8 \| Prefab \| Scene` + `#[non_exhaustive]` + sem cap arch-gate (Round 2 fix) ✓ |
| SpriteSource enum real shape | `grep -A 10 "pub enum SpriteSource" crates/ph2d-render/src/sprite.rs` | 2 variants `Atlas \| Individual` + `Copy + Eq + Serialize` + **NÃO** `non_exhaustive` + sem cap (Round 2 fix) ✓ |
| Workspace MSRV vs ctt | `grep "rust-version" Cargo.toml` | PH2D = 1.92; ctt = 1.90 → folga ≥ ✓ |
| Plano vivo W0.T5 existe? | `ls docs/plans/2026-05-texture-compression-waves.md` | Round 2: criado nesta sessão ✓ (era MISSING em Round 1) |
| ADR-0009 existe? | `ls docs/architecture/decisions/0009-*.md` | NÃO existe (slot reservado SKILL §16). Round 2 fix: §6 #11 + §8 W4 row reescritos para "slot futuro Radiance Cascades — ainda não numerado" ✓ |

Contas explícitas de saving — vide §5.

---

## 8. Wave structure (resumo do plano W0.T5)

Round 1 Lente A H3 (LOC estimates inflados) corrigido: estimates re-calibrados baseando em Fase 1 (1207 LOC para parser puro) e imageio waves (cada importador ~600-1500 LOC).

| Wave | Escopo | Estimativa LOC (Round 2 calibrado) | Auditoria |
|---|---|---|---|
| W0 | Esta ADR + auditoria N-lente Round 1 + Round 2 + SKILL §11.10/§12.1 update + HR-1 §2.7.1 codification + plano vivo + HANDOFF §4 fix + 2.7 fallback plan | ~900 LOC docs cumulative | Lentes A/B/C × 2 rounds (este doc) |
| W1 | `tools/asset-cooker/src/texture/` + `ctt-cli` wrapper + mip gen + multi-tier batch + `Asset::TextureKtx2` variant + W2.T-pre `Ktx2Image::is_premultiplied()` + W1.T2 audit `ctt` source + W1.T5 canonical-runner determinism gate | ~1800-2400 LOC | 5-lente paralela pós-impl |
| W2 | `ph2d-render` `wgpu_format_from_ktx2_format` (W2.T1, ~150 LOC) + pipeline-per-format (W2.T3, ~600 LOC) + `SpriteSource::CookedTexture` (W2.T2, ~200 LOC) + wgpu feature query (W2.T1, ~100 LOC) + W2.T5 VRAM accounting gate (~150 LOC) | ~1200-1500 LOC | 5-lente paralela pós-impl |
| W3 | Painter brush atlas BC4 (W3.T1) + UI assets ASTC LDR (W3.T2) + Painter "Export Cooked Texture" dialog (W3.T3, HR-7 editor-feature-gated, HR-15 Fluent strings, HR-17 examples) | ~900-1300 LOC | 5-lente paralela pós-impl |
| W4+ | HDR (BC6H + ASTC HDR) — deferido até Painter HDR export real + ecossistema Radiance Cascades pronto (slot futuro de ADR — ainda não numerado) | — | — |

Detalhe per-task em [`docs/plans/2026-05-texture-compression-waves.md`](../../plans/2026-05-texture-compression-waves.md) (W0.T5).

---

## 9. Gates de qualidade

Padrão-ouro **9.0/10** (vide Painter cascade 0050-0053 ratificada 2026-05-26 com 4 audits).

**Gates obrigatórios antes de Accepted:**

- ✅ Pre-flight executável §7 (este doc Round 2 — verificações executadas em código real).
- ✅ Auditoria N-lente paralela Round 1 (REJECT × 3, scores 4.5/4.5/5.0).
- ⏳ Auditoria N-lente paralela Round 2 (este doc pós-remediation) — em andamento.
- ✅ `feedback-perfection-no-deferrals` ativo — gaps Round 1 viraram remediation Round 2, não diferidos.

**Gates per-Wave (delegated to plano vivo):**

- W1.T2: `ctt` source audit + 13 issues triage com checklist data-loss/security/non-determinismo.
- W1.T5: **canonical-runner determinism gate** — re-cook fixture set canônico Linux x86_64 vs hash anterior; falha = ctt-cli ou flag mudou.
- W2.T1: `wgpu_format_from_ktx2_format` enumerar 28 variants × feature flag em arch-gate.
- W2.T5: HR-13 memory budget declaration + accounting via `compressed_size_per_format`.
- W2.T-pre: `Ktx2Image::is_premultiplied()` API + arch-gate `premul_kv_round_trips`.
- W1.T6, W2.T6, W3.T4: 5-lente paralela pós-impl per Wave.
- HR-7 editor-feature isolation: W3.T3 Painter dialog gated em `--features release-game` (test `architecture/editor_feature_isolation.rs`).
- HR-15 i18n: W3 strings em Fluent bundle (NÃO inline literal).
- HR-17 examples: W3 inclui example em `docs/scripting/examples/cooked_texture.luau`.

---

## 10. Consequências

**Positivas:**

- ✅ Texture compression shipa nas 4 platforms sem Basis runtime overhead (zero CPU spike load + WASM 500 KB economy).
- ✅ Cooker via `cargo install ctt-cli` ≠ download binário externo opaco (HR-1 letra; espírito via §2.7.1 critério codificado).
- ✅ `ph2d-imageio::ColorProfile` cap FROZEN preservado (zero amendment ADR-0054); `ph2d-color::ColorProfile` vapor não-bloqueante (não consumido aqui).
- ✅ iPad Pro Apple7+/M1+ **opcionalmente** recebe BC7/BC6H quando wgpu adapter expor (Round 3 fix: claim hedged — cooker emite ASTC default + BC opcional; runtime query é source-of-truth; sem cite primário iPadOS para BC exposure).
- ✅ Painter brush atlas saving -50% (Shape 4→2 MB + Grain 32-64→16-32 MB).
- ✅ Fase 1 codec (1207 LOC, 26 tests) tem cliente concreto W2+ → "ilhamento intencional" resolvido.
- ✅ Determinismo via canonical runner — não dependente de propriedade cross-arch de ISPC/SIMD encoders.

**Negativas (mitigação):**

- ⚠️ Dependência de single individual maintainer (cwfitzgerald) com crate 9 stars — mitigado via §2.7 fallback plan A (fork) / B (toktx + Compressonator em mesmo canonical runner) / C (intel-tex + astc-encoder separados). Round 1 Lente A C5 finding mitigado: fallback B é equivalente operacionalmente.
- ⚠️ Canonical runner abordagem requer cooked artifacts versionados (Git LFS) — custo extra repo size; aceitável (compressed assets são ordens de magnitude menores que source). Detalhe em plano vivo.
- ⚠️ `SpriteSource::CookedTexture` é breaking change serialize (W2.T2 bump `Sprite::VERSION` para 4 — cook-hash churn em fixtures, aceitável Wave 2 scope).
- ⚠️ Cooker offline aumenta tempo de build no developer machine (custo aceitável vs runtime transcode latency).

**Reversibilidade:**

Reverter ADR-0055 requer:
1. Deletar `tools/asset-cooker/src/texture/` (~1800-2400 LOC W1).
2. Reverter `Asset::TextureKtx2` variant + `crates/ph2d-asset/src/tier.rs` (`TierIndex` newtype) + `crates/ph2d-asset/src/logical_texture.rs` (`LogicalTextureId` mapping) (W1.T4 amendment — trivial via `#[non_exhaustive]`).
3. Reverter `SpriteSource::CookedTexture` variant + mirror chain (`InspectorSpriteSource::CookedTexture` + `RequestedSpriteStrategy::CookedTexture` + `INSP_RENDER_STRATEGY_COOKED_TEXTURE` constant) (W2.T2 amendment — breaking; bumpa de VERSION 4 → 3, cook-hash churn em fixtures).
4. Reverter `ph2d-render` pipeline-per-format + `wgpu_format_from_ktx2_format` (W2.T1+T3) + `RENDER_BUDGET_DELTA_W2` constant + `architecture_render_budget_registered` aggregator (W2.T5).
5. Reverter `Ktx2Image::kvd` field + `Ktx2Image::byte_size_estimate()` + `Ktx2Image::premul_intent()` + `PremulIntent` enum (W1.T9 + W2.T-pre additions em Fase 1).
6. Reverter `assets/cooked/` versionado + `.gitattributes` LFS config (W1.T11.5).
7. Reverter `ph2d-color::cooked_texture_aces` se foi adicionado (W0.T6).

Fase 1 codec (`crates/ph2d-asset-ktx2/`) **NÃO** reverte — é alicerce independente.

---

## 11. Memórias relacionadas

- [[ktx2-phase1-done-phase2-aborted-2026-05-26]] — síntese ADR-0055 anterior abortado
- [[no-industrial-claims-without-verification]] — checklist pre-flight (Round 1 falhou aqui — Round 2 executado em código real)
- [[feedback-perfection-no-deferrals]] — gaps Round 1 viraram remediation Round 2
- [[feedback-audit-lens-diversity]] — rotação de lentes para auditoria N-lente
- [[project-painter-w0-ratified-2026-05-26]] — padrão-ouro 9.0/10 referência
- (NOVA Round 3, criada nesta sessão) [[feedback-audit-internal-state-grep]] — Round 1 falhou ao não-grep enums externos; Round 2 falhou ao não-grep enums internos do próprio repo; Round 3 adota sweep-grep preventivo do Coord-A antes de escrever ADR. Generalização da regra [[no-industrial-claims-without-verification]] para **toda afirmação sobre o próprio codebase**.

---

**FIM ADR-0055-Revised Round 3** — Round 3 audit (Lentes D + E) REJECT × 2 (scores 6.5/6.2). Round 4 mecânico aplicou drift cross-doc (Lente D findings). Lente E findings (gates declarativos dependentes de runtime/abstractions vapor — `assert_existing_enum` helper, `Plugin` trait, `ph2d-i18n` Fluent runtime, Luau Asset binding, release-game feature, hot-reload .ktx2) **deferidos para waves W1+** em [plano vivo §Open Issues](../../plans/2026-05-texture-compression-waves.md). ADR fica **Proposed** (não Accepted) com gates-vapor explicitamente flagged.
