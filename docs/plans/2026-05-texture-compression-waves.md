# Plano de waves — Cooked Texture Compression Pipeline (ADR-0055)

**Data:** 2026-05-27 (atualizado noite — v4 Accepted)
**Status:** **W0 FECHADA 2026-05-27 noite** — ADR-0055-v4 Accepted (strategic-only ≤200 LOC) após 2ª opinião de 3 LLMs externas convergir em Opção 4 (ADR enxuto + plano vivo canônico). v3 Round 3+4 (660 LOC com snippets de código) arquivada em [`docs/archive/adrs-rounds-history/0055-v3-round-3-and-4-superseded.md`](../archive/adrs-rounds-history/0055-v3-round-3-and-4-superseded.md). Tabela canon de 22 símbolos migrada para §Symbol Registry deste plano. 13 vapor dependencies (E1..E13) catalogadas em §Open Issues. **W1.T0 destrancada** — próximo: `cargo add ctt` + sweep-grep E1..E13 + audit do source de `ctt`.
**Arquitetura:** [ADR-0055-v4](../architecture/decisions/0055-cooked-texture-compression-pipeline.md) (Accepted strategic).
**Consome (não toca):** Fase 1 codec puro `crates/ph2d-asset-ktx2/` (4 commits f30e225..b276cef, 1207 LOC, 26 tests, ✅ frozen).
**Substrato:** mesmo padrão drop-crate + arch-gate de ADR-0040 (tools) / ADR-0054 (imageio), mas com particularidade: cooker mora em `tools/asset-cooker` (FROZEN 2026-05-22), não em crate satélite isolado.

Tags `W0.Tx` / `W1.Tx` / `W2.Tx` / `W3.Tx` em commits/comentários referenciam este doc.

---

## Forma: foundation cooker → renderer wire-up → painter integration

Diferente de imageio (drop-crate satélites) ou nodes (registry codegen), texture compression tem **3 sub-componentes acoplados** (cooker + asset variant + render pipeline) que precisam fechar coordenadamente. Por isso:

- **W0 = foundational design** (ADR ratificada + plano + SKILL update + HR-1 §2.7.1 codification).
- **W1 = cooker offline + asset variant** (foundational; `tools/asset-cooker` + `crates/ph2d-asset`).
- **W2 = renderer pipeline** (foundational; `crates/ph2d-render` wgpu glue).
- **W3 = painter integration** (Painter consumer; brush atlas BC4 + UI ASTC + Export dialog).
- **W4+ = HDR** (deferido até ecossistema de criação de HDR pronto).

Cada wave fecha com **auditoria 5-lente paralela** + smoke do Enio antes do próximo.

---

## Decisões de design (consolidadas com Enio 2026-05-26 + 2026-05-27)

1. **Opção E ratificada** 2026-05-26 — cooker offline + direct upload runtime, sem Basis runtime, sem `ph2d-color-pipeline`.
2. **`ctt` v0.4.0 cooker** ratificado 2026-05-27 — Rust crate Cargo-installable (multi-encoder unificado bc7e/Intel ISPC/Compressonator/etcpak/astcenc). Fallback A/B/C documentado em ADR §2.7.
3. **Determinismo via canonical-runner CI** (ADR §2.3) — não cross-arch SIMD determinism; cook em GitHub Actions `ubuntu-latest` Linux x86_64 único; KTX2 outputs versionados via Git LFS.
4. **Multi-tier emit per platform** — cooker emite N artefatos (BC7 desktop / ASTC mobile / ETC2 fallback) por source; renderer escolhe via `wgpu::Features` query (ADR §2.6).
5. **iPad Apple7+/M1+ obtém BC** (correção Round 1) — não tratar iOS monolítico; distinguir iPhone (sem BC) vs iPad recent (com BC).
6. **`ph2d-imageio::ColorProfile` é single source-of-truth** (ADR-0054, 8 FROZEN, gate ativo) — `ph2d-color::ColorProfile` da ADR-0051 vapor; esta ADR não depende dele.
7. **`Asset::TextureKtx2` trivial via `#[non_exhaustive]`** — sem cap arch-gate (Round 1 finding).
8. **`SpriteSource::CookedTexture` é breaking change** — W2.T2 bumpa `Sprite::VERSION 3 → 4` + adiciona `#[non_exhaustive]`.
9. **Premultiplied alpha intent** via KTX2 `keyValueData` key `PH2D_PREMUL` (W1 cooker emite; Fase 1 parser **atualmente descarta kvd**; W2.T-pre adiciona kvd preservation + `Ktx2Image::premul_intent() -> PremulIntent { Straight, Premultiplied, Unspecified }` tri-state API; escopo realista ~300 LOC).
10. **HR-1 FFI C/C++ aceitável** sob §2.7.1 critério codificado (offline + ref única + vendored Cargo + license MIT/Apache + maintainer ativo).

---

## §Symbol Registry — verificações executadas pré-W1 (migrado de ADR v3 §1.3)

Tabela canon de 22 símbolos/APIs verificados em código real no repo via `grep`/`cat`/`cargo info`/`ls` em 2026-05-27. Esta tabela é **canônica para implementador de W1+**: símbolos NOVOS introduzidos em waves estão marcados com a wave responsável. Símbolos vapor (não existem ainda mas serão criados) estão flagged.

| Símbolo / API | Comando verificação | Estado real | Wave responsável |
|---|---|---|---|
| `ph2d_asset_ktx2::Ktx2Image` | `grep -n "pub struct Ktx2" crates/ph2d-asset-ktx2/src/lib.rs` | linha 365: `pub struct Ktx2Image { format, width, height, mip_levels }` | Fase 1 ✓ |
| `Ktx2Image::premul_intent()` API | `grep "premul_intent" crates/ph2d-asset-ktx2/src/lib.rs` | NÃO existe | **W2.T-pre cria** |
| `Ktx2Image::byte_size_estimate()` API | `grep "fn byte_size" crates/ph2d-asset-ktx2/src/lib.rs` | NÃO existe | **W1.T9 cria** |
| `Ktx2Image::kvd: BTreeMap<String, Vec<u8>>` field | parser atual ignora `keyValueData` | NÃO existe (kvd descartado) | **W2.T-pre adiciona** |
| `PremulIntent { Straight, Premultiplied, Unspecified }` enum | grep | NÃO existe | **W2.T-pre cria** |
| `ph2d_core::MemoryBudget` | `cat crates/ph2d-core/src/budget.rs` | STRUCT `{ vram_mb: u32, ram_mb: u32, heap_script_mb: u32 }` + `MemoryBudget::new(...)`. **NÃO é enum**. | existe ✓ |
| `trait Plugin` | `grep -rn "trait Plugin\b" crates/ --include="*.rs"` | ZERO matches — **VAPOR** (SKILL §HR-13 pattern aspirational) | **E3 §Open Issues** |
| `ph2d_host::DeviceTier` enum | `grep -rn "pub enum DeviceTier" crates/` | ZERO matches — **VAPOR** (ADR-0053 cita mas não materializado; gate silently-passing) | **slot futuro** |
| `ph2d_asset::TierIndex(u8)` newtype | grep | NÃO existe | **W1.T4 cria** (host-agnostic; alias-target quando DeviceTier materializar) |
| `ph2d_asset::LogicalTextureId` mapping | grep | NÃO existe | **W1.T4 cria** (`LogicalTextureId → BTreeMap<TierIndex, AssetId>`) |
| `ph2d_asset::AssetId([u8; 32])` | `cat crates/ph2d-asset/src/id.rs` | linha 17: existe, `Copy + Eq + Serialize` | existe ✓ |
| `ph2d_asset::AssetDb` API | `grep "pub fn" crates/ph2d-asset/src/db.rs` | só `pub fn get(&self, id: &AssetId) -> Option<Arc<Asset>>` (linha 145). Content-addressed. | existe ✓ |
| `Asset` enum shape | `grep -A 10 "pub enum Asset" crates/ph2d-asset/src/asset.rs` | 3 variants `ImageRgba8 \| Prefab \| Scene` + `#[non_exhaustive]`. Sem cap arch-gate. | **W1.T4 adiciona `TextureKtx2`** |
| `SpriteSource` enum shape | `grep -A 8 "pub enum SpriteSource" crates/ph2d-render/src/sprite.rs` | 2 variants `Atlas \| Individual` + `Copy + Eq + Serialize`, NÃO `non_exhaustive`. `Sprite::VERSION = 3`. | **W2.T2 adiciona `CookedTexture` (breaking)** |
| `InspectorSpriteSource` mirror | linha 231 `crates/ph2d-editor-core/src/screens/hero.rs` | 3 variants `Atlas \| Individual \| HandPacked` | **W2.T2 mirror sync** |
| `RequestedSpriteStrategy` mirror | linha 307 idem | 3 variants idem | **W2.T2 mirror sync** |
| `INSP_RENDER_STRATEGY_*` constants | linhas 418-420 `crates/ph2d-editor-core/src/ids.rs` | `_ATLAS`, `_INDIVIDUAL`, `_HANDPACKED` | **W2.T2 adiciona `_COOKED_TEXTURE`** |
| `Ktx2Format` variants count | `awk '/pub enum Ktx2Format/,/^}/' lib.rs \| grep -E "^    [A-Z]" \| wc -l` | **28**: 5 uncompressed + 10 BC + 8 ASTC + 4 ETC2 + 1 Unsupported | Fase 1 ✓ |
| `wgpu::PollType::wait_indefinitely()` | `grep "PollType" crates/ph2d-render/src/` | existe em `individual.rs:448` e `vello_pass.rs:222` | existe ✓ |
| `wgpu = 28.0.0` | `cat Cargo.lock \| grep -A 1 '^name = "wgpu"'` | 28.0.0 confirmado | existe ✓ |
| `ph2d_color::ColorProfile` enum | `grep -rn "pub enum ColorProfile" crates/ph2d-color/` | ZERO matches — **VAPOR** (ADR-0051 não materializou) | **fora deste escopo** |
| `ph2d_imageio::ColorProfile` enum | `grep -A 30 "pub enum ColorProfile" crates/ph2d-imageio/src/color.rs:18` | 8 variants ✓ FROZEN gate ativo | existe ✓ |
| `ctt = 0.4.0` crate | `cargo info ctt` | confirmed: 0.4.0, MSRV 1.90, license MIT/Apache-2.0/Zlib, repo cwfitzgerald/ctt | crates.io ✓ |
| `.gitattributes` (Git LFS) | `ls .gitattributes` | NÃO existe — repo não LFS-ready | **W1.T11.5 setup** |
| `ph2d-painter-brush` deps inclui `ph2d-asset` | `cat crates/ph2d-painter-brush/Cargo.toml` | ZERO `ph2d-asset` dep | **W3.T0 pre-task adiciona** |
| `ph2d-painter-brush::atlas.rs` LOC | `wc -l crates/ph2d-painter-brush/src/atlas.rs` | 60 LOC stub (`AtlasStub` placeholder) | **W3.T1 substitui** (LOC estimate 600-800) |

**Regra do registry**: ao implementar W1+, antes de citar qualquer símbolo em código novo, re-verificar via `grep`/`cat`. Símbolos podem ter materializado entre sessões (especialmente E3 Plugin trait, E5 ph2d-i18n). Memória [[feedback-audit-internal-state-grep]] é a versão expandida desta regra.

---

## §Anti-patterns — NÃO repetir (migrado de ADR v3 §6)

Lista canônica dos anti-patterns identificados em ADR-0055 v1 (deletada 2026-05-26) que afundaram a versão original. Cada um vem com a verificação que detecta tentativas de re-introdução:

1. ❌ **NÃO criar `ph2d-asset-basisu`** (runtime transcoder C++ FFI). Verificação: `grep -rn "ph2d-asset-basisu" --include="*.toml"` deve retornar zero.
2. ❌ **NÃO criar `ph2d-color-pipeline`** crate paralelo. ADR-0042 mandato expande `ph2d-color` (cap 2500 LOC).
3. ❌ **NÃO afirmar `basis-universal-rs >= 0.4`** sem `cargo search`. Real: 0.3.1 dormente Nov/2023, maintainer individual `aclysma`.
4. ❌ **NÃO assumir BC universal em iOS**. iPhone (todas gens) sem BC; iPad Apple7+/M1+ tem BC opcional (`MTLDevice.supportsBCTextureCompression`); runtime feature query é source-of-truth.
5. ❌ **NÃO escrever `-50% VRAM` (BC7 vs RGBA8)** sem mostrar a conta. Real: `BC7 8 bpp ÷ RGBA8 32 bpp = 0.25 → -75%`. Plano de saving canônico vide §5 v3 archived ou §Memory Budget abaixo.
6. ❌ **NÃO amendar `ColorProfile` cap**. Dois ColorProfile distintos (ADR-0051 vapor + ADR-0054 FROZEN). Esta ADR não amenda nem materializa.
7. ❌ **NÃO override HR-1 "pure-Rust"** sem critério objetivo. SKILL §HR-1 §2.7.1 codifica 6 critérios FFI C/C++ aceitáveis (offline-only, ref impl única, vendored Cargo, license compatible, maintainer ativo, NÃO patent-encumbered).
8. ❌ **NÃO citar ADR-0009 como existente** (slot reservado SKILL §16, Holographic Radiance Cascades, ainda não escrito). Usar "slot futuro de ADR — ainda não numerado".
9. ❌ **NÃO afirmar adoção industrial sem WebFetch oficial**. Cada claim sobre Unity/Unreal/Houdini/etc. precisa cite verificável.
10. ❌ **NÃO confundir ACES tonemap com ACEScg working space**. PH2D = Linear sRGB working + ACES tonemap output.
11. ❌ **NÃO modelar HDR sprite pipeline sem ecossistema de criação**. Procreate/PSD/Krita não exportam HDR mainstream. HDR (W4+) deferido até Painter export HDR real.

**Anti-patterns introduzidos no próprio v3 (e NÃO repetir em planos futuros):**
12. ❌ **NÃO escrever ADR com snippets de código** `pub fn foo()`. ADRs strategic-level documentam decisão; código vai no plano vivo ou no próprio código. Snippets em ADR são vapor verificável.
13. ❌ **NÃO rodar 4 rounds de audit consecutivos** sem mudar método. Padrão R1→R4 do v3 trocou classe de drift por round sem convergir (Goodhart's Law).
14. ❌ **NÃO aplicar `[[feedback-perfection-no-deferrals]]` a dependências adjacentes** (Plugin trait em outra crate, runtime em outra ADR). Regra é para gaps *dentro* do escopo da decisão atual.

---

## §Memory Budget Math (migrado de ADR v3 §5)

Contas explícitas de saving — todas mostradas (anti-pattern #5):

- **BC7 vs RGBA8 (desktop sprite)**: `BC7 8 bpp ÷ RGBA8 32 bpp = 0.25 → -75% saving`
- **ASTC 6×6 vs RGBA8 (mobile sprite)**: `ASTC 6×6 3.56 bpp ÷ RGBA8 32 bpp = 0.111 → -89% saving`
- **ASTC 4×4 vs RGBA8 (critical UI sprite)**: `ASTC 4×4 8 bpp ÷ RGBA8 32 bpp = 0.25 → -75% saving`
- **ETC2 RGBA vs RGBA8 (Android fallback)**: `ETC2 RGBA 8 bpp ÷ RGBA8 32 bpp = 0.25 → -75% saving`
- **BC4 vs R8 (brush atlas single-channel)**: `BC4 4 bpp ÷ R8 8 bpp = 0.5 → -50% saving` (não 4× — 4× só vale se source fosse RGBA8)
- **BC6H vs RGBA16Float (desktop HDR sprite, W4+)**: `BC6H 8 bpp ÷ RGBA16F 64 bpp = 0.125 → -87.5% saving`

**Projeção provisional W2+ (audit real W2.T5):**

| Subsistema | Antes | Com texture compression | Assumption |
|---|---|---|---|
| Render textures+meshes iPad | 350 MB | ~200 MB | ASTC 6×6 = -89% sobre RGBA8 em 60% das texturas; meshes inalterados (~140MB fixed) |
| Render textures+meshes Desktop | 1200 MB | ~500 MB | BC7 = -75% sobre RGBA8 em 80% das texturas; meshes inalterados (~240MB fixed) |
| Painter brush atlas | Shape 4 MB + Grain 32-64 MB R8 = 36-68 MB | Shape 2 MB + Grain 16-32 MB BC4 = **18-34 MB (-50%)** | R8 source → BC4 cooked |

**VRAM measurement API** (W2.T5):
- `device.poll(PollType::wait_indefinitely())` NÃO mede VRAM (só drives command-completion).
- Backend-specific introspection (Metal/D3D12/Vulkan) não cross-vendor via wgpu.
- W2.T5 strategy: contar bytes via `compressed_size_per_format(format, w, h, mip_count) × num_textures` (deterministic, suficiente HR-13).

---

## WAVE 0 — Foundation · COORD-A ONLY · em andamento

| Task | Escopo | Status |
|---|---|---|
| **W0.T0** | Pesquisa fresca §6 do HANDOFF (supply chain `ctt`/basis/intel-tex; platforms iOS BC/Android ASTC HDR/WebGPU; Painter atlas) | ✅ 2026-05-27 — 25+ verificações executadas (cargo search/info, WebFetch, GitHub profile, Apple Metal docs) |
| **W0.T1** | Escrever ADR-0055 Round 1 — 7 pontos da recomendação + 11 anti-patterns NÃO-repetidos + pre-flight executavel | ✅ 2026-05-27 — 600 LOC docs |
| **W0.T2 Round 1** | Auditoria 3 lentes paralelas (A supply-chain, B HR-ADR compliance, C WGSL-ABI/test-coverage) | ✅ 2026-05-27 — REJECT × 3, scores 4.5/4.5/5.0 (abaixo predecessor 5.67) |
| **W0.T2 Phase A+B (R2)** | Reescrever ADR Round 2 com TODAS as remediações CRITICAL+HIGH inline (per [[feedback-perfection-no-deferrals]]) | ✅ 2026-05-27 — ADR Round 2 ~900 LOC, 13 findings R1 convergentes endereçados |
| **W0.T2 Round 2** | Auditoria 3 lentes A2/B2/C2 paralelas pós-R2 | ✅ 2026-05-27 — REJECT × 3, scores 6.5/6.5/5.2; 6 novos findings críticos INTERNOS (Ktx2Blob vapor, DeviceTier vapor, MemoryBudget shape, Plugin trait, AssetDb API, kvd parser) |
| **W0.T2 Round 3 (sweep-grep + 12 fixes)** | Coord-A executa sweep-grep preventivo em código real ANTES de reescrita; 12 fixes inline (Ktx2Image rename, TierIndex newtype, LogicalTextureId, MemoryBudget API real, kvd 300 LOC, etc.) | ✅ 2026-05-27 — ADR Round 3 ~1100 LOC; nova memória [[feedback-audit-internal-state-grep]] |
| **W0.T2 Round 3 audit** | Lentes D + E (sense-check + gates enforceability) | ✅ 2026-05-27 — REJECT × 2, scores 6.5/6.2; Lente D = drift cross-doc (Round 4 fix); Lente E = gates declarativos sem enforcement real (deferido §Open Issues) |
| **W0.T2 Round 4 mecânico** | Sync drift cross-doc (Lente D): plan header + status ticks + 25→28 variants + premul_intent unify + W1.T11.5 LFS + W2.T4 + mirror chain + Open Issues section | 🔄 em andamento 2026-05-27 |
| **W0.T5** | Este plano vivo (era citado em ADR Round 1 mas não existia — Lente B C1 finding) | ✅ 2026-05-27 — este doc |
| **W0 — HANDOFF §4 + §6.10 patch** | Corrigir "BC4 (4× saving)" → "(-50% real, R8→BC4)" + atlas size canonical | ✅ 2026-05-27 (HANDOFF_ktx2_phase2.md linhas 108 + 193) |
| **W0.T2 Round 5+** | NÃO executado — diagnóstico 2ª opinião externa: Round 5 trocaria classe de drift sem convergir (Goodhart's Law). Substituído por v4 reescrita. | ❌ cancelado |
| **W0.T3** | SKILL §11.10 update — reconciliar texto canon "Texture compression" com ADR-0055 (cooker `ctt`, canonical-runner, iPad-BC distinction) + HR-1 §2.7.1 critério FFI codificado (6 critérios pós Round 4 split) | ✅ 2026-05-27 (Round 4 incluindo iPad BC hedge consistente) |
| **W0.T4** | SKILL §12.1 memory budget table — adicionar provisional W2+ + `compressed_size_per_format` accounting | ✅ 2026-05-27 |
| **W0.T6** (opcional) | Se ACES tonemap helper for materializado em ph2d-color: `cooked_texture_aces.rs` ≤ 200 LOC sob cap 2500 | ⏳ defer if not blocker (W2 shader pode inline) |
| **W0.T7-v4** | ADR-0055-v4 enxuto (≤200 LOC strategic-only, sem snippets) `Proposed` → `Accepted`. v3 arquivada em `docs/archive/adrs-rounds-history/`. Tabela canon + anti-patterns + memory math migrados para §Symbol Registry / §Anti-patterns / §Memory Budget Math deste plano. Regra `feedback-perfection-no-deferrals` refinada com escopo decisão-atual vs decisões-adjacentes. | ✅ 2026-05-27 noite |

**Aceitação W0:** ✅ Fechada 2026-05-27 noite. ADR-0055-v4 Accepted; SKILL §11.10/§12.1 atualizadas; plano vivo populado + §Symbol Registry / §Anti-Patterns / §Memory Budget Math / §Open Issues; HANDOFF §12 atualizado; W1.T0 destrancada.

---

## 🔒 FREEZE (gate do fan-out) — após W0.T7

Caps congelados pós-W0:
- `Asset::TextureKtx2` — variant adicionada, governance ADR-0025/0028 (não cap arch-gate).
- `SpriteSource::CookedTexture` — variant adicionada via `#[non_exhaustive]` migration + bump VERSION 3→4 + mirror chain sync (InspectorSpriteSource / RequestedSpriteStrategy / `INSP_RENDER_STRATEGY_COOKED_TEXTURE`).
- `wgpu_format_from_ktx2_format` — **28 variants** × wgpu feature flag enumerados em arch-gate (5 uncompressed + 10 BC + 8 ASTC + 4 ETC2 + 1 Unsupported wildcard).
- KTX2 keyValueData key `PH2D_PREMUL` — convenção PH2D documentada.
- Canonical runner Linux x86_64 = source-of-truth para todos cooked artifacts.
- `TierIndex(u8)` newtype em `ph2d-asset/src/tier.rs` (NOVO R3) — host-agnostic; migration alias quando ADR-0053 `DeviceTier` materializar.
- `LogicalTextureId → BTreeMap<TierIndex, AssetId>` em `ph2d-asset/src/logical_texture.rs` (NOVO R3) — multi-tier resolution sem AssetDb amendment.
- `Ktx2Image::kvd: BTreeMap<String, Vec<u8>>` field + `Ktx2Image::byte_size_estimate()` + `Ktx2Image::premul_intent() -> PremulIntent` (W1.T9 + W2.T-pre APIs em Fase 1).
- `PremulIntent { Straight, Premultiplied, Unspecified }` enum (NOVO R3) — tri-state semantics.
- `RENDER_BUDGET_DELTA_W2: MemoryBudget` constante (NOVO R3 em `ph2d-render/src/lib.rs`) — HR-13 budget delta.

Mudanças futuras = amendment ADR-0055.

---

## WAVE 1 — Cooker offline + Asset variant · COORD-A ONLY

**Política**: cooker é foundational cross-cutting; toca `tools/asset-cooker` + `crates/ph2d-asset`. Coord-A only (não fan-out — não há paralelismo natural).

### Batch A — `ctt` integration foundation

- **W1.T1** — `tools/asset-cooker/Cargo.toml` adiciona `ctt = "0.4.0"` (lib API) ou shells out para `ctt-cli` v0.4.0 via `std::process::Command`. **Decisão pré-W1**: lib API preferred (mais determinístico, evita process spawn overhead, integra com cancellation). LOC ~80 (Cargo deps + workspace bump if needed).
- **W1.T1.5** — Opcional: verify GitHub Artifact Attestation no install step. LOC ~50 + script.
- **W1.T2** — **`ctt` source audit** — ler 100% do código wrapper Rust (sub-crates ctt-astcenc/ctt-bc7enc-rdo/ctt-compressonator/ctt-etcpak/ctt-intel-texture-compressor); triage de 13 open issues com checklist (data-loss/security/non-determinism CRITICAL?). Deliverable: `docs/audits/ctt-source-audit-2026-05-XX.md`. Esforço ~3h leitura.

### Batch B — Cooker module

- **W1.T3** — `tools/asset-cooker/src/texture/mod.rs` + sub-command CLI `asset-cooker texture cook --input X --tier T --output Y`. LOC ~250.
- **W1.T4** — `Asset::TextureKtx2 { tier, blob: Arc<Ktx2Image> }` variant em `crates/ph2d-asset/src/asset.rs`; extend `byte_size()` HR-13. Trivial via `#[non_exhaustive]`. LOC ~80.
- **W1.T5** — `tools/asset-cooker/src/texture/target_matrix.rs` — tabela §2.6 do ADR codificada (input + DeviceTier → ctt format + encoder + quality + flags). LOC ~250.
- **W1.T6** — `tools/asset-cooker/src/texture/multi_tier.rs` — source → N artifacts (5 per tier do §2.6). LOC ~200.
- **W1.T7** — `tools/asset-cooker/src/texture/mip_gen.rs` — mip pyramid (box/Lanczos/point pre-ctt). LOC ~200.
- **W1.T8** — `tools/asset-cooker/src/texture/premul_tracking.rs` — emit KTX2 keyValueData `PH2D_PREMUL` byte. LOC ~80.
- **W1.T9** — `crates/ph2d-asset-ktx2/src/lib.rs` API additions (W2.T-pre): adicionar campo `kvd: BTreeMap<String, Vec<u8>>` ao `Ktx2Image` + reparse kvd via `ktx2::Reader::key_value_data()` (Round 3 fix: parser atual DESCARTA kvd, NÃO "lê via API"); `Ktx2Image::byte_size_estimate()` HR-13 helper; `Ktx2Image::premul_intent() -> PremulIntent` (tri-state Straight/Premultiplied/Unspecified); arch-gate `premul_kv_round_trips` + `byte_size_estimate_matches_mip_sum`. **LOC ~300 (Round 3 fix: era ~100 — kvd reparse + bounds + DOS defence + tri-state tests + benchmarks são reais)**.

### Batch C — Determinism gate

- **W1.T10** — `tools/asset-cooker/src/texture/determinism.rs` + GitHub Actions workflow step que (a) snapshot `ctt-cli --version`, (b) cook 8 fixtures canônicos do ADR §2.3, (c) blake3 cada output, (d) compare com `assets/cooked-hashes.lock`, (e) falha = ctt-cli upgrade ou flag mudou → human review. LOC ~300 + workflow YAML.
- **W1.T11** — 8 fixtures canônicos em `tools/asset-cooker/src/texture/fixtures/`: 256² gradient · 1024² photo · 4096² atlas-packed · 256² R8 brush atlas · 1024² SDF font · 16² critical UI · 512² normal map · (EXR HDR deferred W4+). LOC ~150 (fixtures source) + ~100 (golden hashes).
- **W1.T11.5** (Round 4 NOVO) — **Git LFS setup** (Round 3 finding: `.gitattributes` missing, repo NÃO inicializado pra LFS). Tasks: (a) `git lfs install` no clone canon, (b) criar `.gitattributes` com `assets/cooked/**/*.ktx2 filter=lfs diff=lfs merge=lfs -text`, (c) decisão entre GitHub LFS / Cloudflare R2 / self-hosted (GitHub free tier 1GB/month bandwidth — pode esgotar com cooked texture volume), (d) docs `CONTRIBUTING.md` adicionar `git lfs install` step, (e) `.github/workflows/spike.yml` adicionar `git lfs pull` step antes de testes. LOC ~30 (config) + setup-cost docs.
- **W1.T12** — Adicionar `assets/cooked-hashes.lock` + bootstrap initial hashes from canonical CI run (depende W1.T11.5 LFS setup). LOC = 0 (data file via Git LFS).

### Batch D — Wire-up + audit

- **W1.T13** — `tools/asset-cooker` CLI sub-command listed em `--help`. Smoke test via CI workflow step.
- **W1.T14** — Sample cook: convert `crates/ph2d-painter-brush` shape atlas R8 → BC4 KTX2 (proof-of-life, não consumir ainda — W3 consume).
- **W1.T15** — **Auditoria 5-lente paralela**: WGSL/ABI · HR-ADR compliance · cross-GPU realism · regression vs verbal · test-coverage. Target ≥ 8.5/10.

**Aceitação W1:** ✅ `cargo install ctt-cli` em CI; cooker sub-command working; 8 fixtures cookados; `assets/cooked-hashes.lock` populado; W1.T2 audit publicado; W1.T15 5-lente APPROVE.

**LOC estimate W1:** ~1800-2400 (Round 2 calibrado).

---

## WAVE 2 — Runtime renderer pipeline · COORD-A ONLY

### Batch A — wgpu format mapping (Fase 1 gap)

- **W2.T1** — `crates/ph2d-render/src/ktx2_format.rs` (NOVO) — `pub fn wgpu_format_from_ktx2_format(fmt: Ktx2Format) -> Result<(wgpu::TextureFormat, wgpu::Features), FormatError>`. Enumera **28 `Ktx2Format` variants** × wgpu mapping (Round 4 correção R3: real count = 5 uncompressed + 10 BC + 8 ASTC + 4 ETC2 + 1 Unsupported = 28; plan anterior dizia 25). LOC ~150. Arch-gate `ktx2_format_exhaustive_mapping` — **NÃO usar `_ =>` wildcard** no match (compile-time exhaustiveness).
- **W2.T1.5** — wgpu feature query helper: `Renderer::detect_supported_compressions() -> CompressionFeatureSet`. LOC ~100.

### Batch B — SpriteSource breaking migration

- **W2.T2.pre** — `grep -rn "match .* SpriteSource" crates/` — inventário de match sites que precisam `_ =>` ou non-exhaustive accommodation.
- **W2.T2** — `SpriteSource::CookedTexture { asset_id: AssetId }` variant em `crates/ph2d-render/src/sprite.rs` + adiciona `#[non_exhaustive]` + bump `Sprite::VERSION 3 → 4`. **Mirror chain sync** (Round 3+4 expansion): adicionar variant `CookedTexture` em `InspectorSpriteSource` ([`crates/ph2d-editor-core/src/screens/hero.rs:231`](../../crates/ph2d-editor-core/src/screens/hero.rs#L231)) + `RequestedSpriteStrategy` ([linha 307](../../crates/ph2d-editor-core/src/screens/hero.rs#L307)) + constant `INSP_RENDER_STRATEGY_COOKED_TEXTURE` em [`ids.rs`](../../crates/ph2d-editor-core/src/ids.rs#L418) + routing em `action_bus.rs` + panel-inspector sections. **HandPacked landing precedence**: HandPacked já em 3/4 mirror sites (não em SpriteSource real); coordinator de W2.T2 escolhe via grep state — landing junto ou independente. LOC ~300 (Round 4 raised de 200).
- **W2.T2.test** — Fixture cook-hash churn: 1 test fixture per Wave já feito sob VERSION 3 precisa re-bake. LOC ~50.

### Batch C — Pipeline-per-format renderer

- **W2.T3** — `crates/ph2d-render/src/compressed_pipeline.rs` — pipeline-per-format selection (BC7/BC6H/ASTC/ETC2/RGBA8 paths); bind group differ por format. LOC ~600.
- **W2.T4** — Loader path (Round 4 fix: `AssetDb::resolve_for_tier` NÃO existe — content-addressed AssetDb tem só `fn get(&AssetId)`): **LogicalTextureId resolution via mapping externo** em `crates/ph2d-asset/src/logical_texture.rs` (NOVO W1.T4): `pub fn logical_texture_resolve(logical_id: LogicalTextureId, tier: TierIndex, db: &AssetDb) -> Option<Arc<Asset>>` → resolve LogicalId→AssetId→AssetDb.get. Renderer reads `Asset::TextureKtx2` → `wgpu::queue::write_texture` direct. LOC ~250.

### Batch D — Memory budget + audit

- **W2.T5** — HR-13 budget declaration + accounting: `crates/ph2d-render/src/plugin.rs::Plugin::init` declara `compressed_texture_cache_mb` budget; `compressed_size_per_format(format, w, h, mip_count)` helper. LOC ~150.
- **W2.T6** — **Auditoria 5-lente paralela**: WGSL/ABI · HR-ADR · cross-GPU realism · regression vs verbal · benchmark-vs-claim. Target ≥ 8.5/10.

**Aceitação W2:** end-to-end smoke: cook → ship → load → upload → sample renderiza BC7 sprite no desktop, ASTC no iPad simulator (se accessible), RGBA8 fallback no Web. W2.T6 APPROVE.

**LOC estimate W2:** ~1200-1500.

---

## WAVE 3 — Painter integration · COORD-A scaffold + Implementador

### Batch A — Brush atlas BC4 (high-priority mobile VRAM)

- **W3.T1** — Cook brush shape atlas (64×256² R8) → BC4 KTX2 via `asset-cooker texture cook --input <atlas> --format bc4 --tier all`. Wire in `crates/ph2d-painter-brush/src/atlas.rs` — switch from raw R8 upload to KTX2 load via `AssetDb`. LOC ~150.
- **W3.T2** — Grain atlas R8 → BC4 (same pattern, Grain 32-64 MB → 16-32 MB). LOC ~100.

### Batch B — UI asset ASTC LDR

- **W3.T3** — Cook UI assets (chrome icons, panel backgrounds) → ASTC LDR multi-tier. Wire em chrome render path. LOC ~200.

### Batch C — Painter Export Cooked Texture UX

- **W3.T4** — Painter "Export to Cooked Texture" dialog: target picker (Desktop/Mobile/Web/All), quality preset (Fast/Balanced/HighQuality), async progress bar. HR-7 editor-feature-gated (`--features release-game` test removes). HR-15 Fluent strings (no inline literals). HR-17 example em `docs/scripting/examples/cooked_texture.luau`. LOC ~500-800.

### Batch D — Audit

- **W3.T5** — **Auditoria 5-lente paralela** — incluindo lente UX (Painter dialog flow).

**Aceitação W3:** Painter shipa "Export to Cooked Texture" feature; brush atlas footprint reduzido -50%; W3.T5 APPROVE.

**LOC estimate W3:** ~900-1300.

---

## WAVE 4+ — HDR (DEFERIDO)

Defer até **ambos** condições:

1. Painter ter export HDR real (EXR/HDR Radiance) shippado (atual: not started — defer indeterminado).
2. Slot futuro de ADR Radiance Cascades materializado em código (atual: slot reservado SKILL §16, sem número ADR ainda).

Quando destrancado:
- `BC6H` desktop + iPad M1+
- `ASTC HDR` iOS 16.4+ + Android Vulkan 1.3 ASTC HDR extension
- Painter color profile workflow EXR import → cooked HDR
- W4.T0 ADR amendment ADR-0055 → ADR-0055.1 (HDR Wave)

---

## §Open Issues / Vapor Dependencies (Round 3 Lente E findings deferidos para W1+)

Round 3 Lente E (test-coverage gates + enforceability) identificou que múltiplos gates prometidos no ADR-0055 §9 **dependem de runtimes/abstractions vapor** ou são **declarativos sem enforcement real**. Round 4 (esta sessão) NÃO resolve estes — são endereçados wave-by-wave quando dependências materializarem. ADR-0055 fica **Proposed** (não Accepted) até resolução.

### E1 — `count_enum_variants` silently-passing pattern (foundational)
**Problema**: Helper em `painter-contracts/tests/architecture_painter_contract_surface.rs` retorna `Option<usize>`; gates como `color_profile_variant_count_is_exact_8` / `device_tier_variant_count_is_exact_5` usam `if let Some(n) = ...` → **silenciosamente passam quando enum não existe**. ADR-0055 introduz mais gates desta família (`device_tier_variant_count_is_exact_5` quando DeviceTier materializar, futuros gates ao redor de TierIndex/PremulIntent).
**Wave/ADR resolution**: foundational; melhor endereçado em ADR separada `architecture_gate_enforceability_polishing` (slot futuro). Resolução proposta: helper `assert_existing_enum<E>` que **falha** quando enum ausente.
**Mitigation interim**: ADR-0055 documenta o anti-pattern em `feedback-audit-internal-state-grep` mas NÃO corrige a raiz nesta ADR.

### E2 — `RENDER_BUDGET_DELTA_W2` constante órfã (HR-13 sem aggregator)
**Problema**: ADR §5 propõe constante em `ph2d-render/src/lib.rs` mas `crates/ph2d-tool-registry-init/tests/registry_budget_aggregate.rs` só soma tool manifests + `SYNTHETIC_CORE_BASELINE` — **não conta `ph2d-render`**.
**Wave/ADR resolution**: W2.T5 cria `crates/ph2d-render/tests/architecture_render_budget_registered.rs` que **agrega** `RENDER_BUDGET_DELTA_W2` + tools + core baseline contra `Platform::max_total_mb()`.
**Mitigation interim**: declarar constante mesmo sem aggregator — futura W2.T5 plug-in.

### E3 — `Plugin` trait + `PluginBuilder::declare_budget` vapor
**Problema**: SKILL §HR-13 prescreve "Plugin::init declara budget" mas trait NÃO existe (`grep -rn "trait Plugin\b" crates/` = 0 matches). Round 2 ADR inventou enum variant `MemoryBudget::Render { ... }` apoiando-se no vapor.
**Wave/ADR resolution**: slot futuro ADR `plugin_trait_materialization` (não numerado ainda). Quando trait materializar, `RENDER_BUDGET_DELTA_W2` constante vira `impl Plugin for RenderPlugin { fn budget(&self) -> MemoryBudget { ... } }`.
**Mitigation interim**: usar `MemoryBudget::new(vram, ram, heap)` API real até trait existir.

### E4 — HR-7 release-game feature gate `#[ignore]`-skeleton
**Problema**: `crates/ph2d-tool-registry-init/tests/registry_no_tool_symbols_in_release.rs` é `#[ignore = "PR 10 — un-ignore after release-game feature lands"]`. ADR §9 promete arch-gate `architecture/editor_feature_isolation.rs` que **NÃO existe** (`find` = 0 matches).
**Wave/ADR resolution**: W3.T3 Painter Export dialog deve esperar release-game feature wired + un-ignore desse gate. **W3 BLOCKED** se release-game feature não materializar antes.
**Mitigation interim**: W3 começa em status "blocked by ADR-X (release-game feature)" — não promete enforcement.

### E5 — HR-15 i18n: `ph2d-i18n` Fluent runtime + `t!()` macro vapor
**Problema**: `find . -name "*.ftl"` = 0 matches; `grep "fluent\|FluentBundle"` = 0 matches. Gate `hr15_no_hardcoded_ui_strings` doc explicita: "Until Fluent runtime (`ph2d-i18n`) is wired and `t!(...)` exists, enforces frozen baseline". Novo Painter Export dialog (W3.T4) usaria strings → adicionar à BASELINE perpetua débito.
**Wave/ADR resolution**: W3 BLOCKED até ADR-X (ph2d-i18n materialization, não numerado) ratificar.
**Mitigation interim**: W3 status "blocked by ph2d-i18n".

### E6 — HR-17 examples: Luau `Asset.LoadCookedTexture` binding ausente
**Problema**: `grep "Asset\." docs/scripting/examples/spike/llm-tests/ph2d.d.luau` = 0 matches. ADR §9 promete `cooked_texture.luau` example, mas API binding Luau não existe.
**Wave/ADR resolution**: W3.T4 OU ADR-X (Luau Asset surface) ratificar binding antes.
**Mitigation interim**: W3 status "blocked by Luau Asset binding"; example pode usar mock binding até real existir.

### E7 — W1.T5 canonical-runner gate tautológico
**Problema**: Lente E flagou que "Linux ≡ Linux mesma SHA do ctt-cli" só pega flag/version drift, não determinismo real (cooked-hashes.lock populado pelo MESMO runner que valida).
**Wave/ADR resolution**: W1.T5 plan addendum: além de hash-lock compare, adicionar **5 cooks consecutivos do mesmo input dentro do same CI job** + assert blake3 igualdade (detecta multi-threading non-determinism real).
**Mitigation interim**: deixar W1.T5 sem teste real; W1.T15 5-lente audit valida durante execution.

### E8 — Hot reload `.ktx2` extension ausente
**Problema**: `crates/ph2d-asset/src/watcher.rs:143` é hardcoded `is_png_extension`. ADR-0055 introduz `Asset::TextureKtx2` mas **NÃO menciona** watcher extension. Dev workflow "Painter export → preview live" = quebrado silenciosamente.
**Wave/ADR resolution**: W1.T8.5 (NOVO task plano): estender `AssetWatcher` para `.ktx2` files.
**Mitigation interim**: aceitar gap — dev workflow re-cook é manual via CLI até W1.T8.5.

### E9 — `premul_kv_round_trips` tampering test silenciosamente degrada
**Problema**: ADR §3 propõe `Ktx2Image::premul_intent()` retornando `Unspecified` em wildcard match → tampering 1-byte vira `Unspecified` (degraded), não Erro. ADR diz "tampering test deferred to W2.T3" mas W2.T3 escopo é pipeline-per-format.
**Wave/ADR resolution**: W2.T-pre adicionar magic-byte (e.g., `[0xPE, 0xMU, premul_byte]`) + CRC verify; load com bad CRC → `Err(Ktx2Error::CorruptedKeyValue)`.
**Mitigation interim**: deixar como wildcard `Unspecified` (não-quebra); tampering vira W2.T-pre stretch goal.

### E10 — CI matrix Linux-only canonical step
**Problema**: `.github/workflows/spike.yml` matrix `[ubuntu-latest, macos-latest, windows-latest]`. W1.T10 canonical-runner step roda apenas Linux. **macOS + Windows CI runners**: que rodam? Plano não decide.
**Wave/ADR resolution**: W1.T10 plan addendum: `if: matrix.os == 'ubuntu-latest'` condicional explícito; macOS/Windows skip cook gate mas mantêm test runs (cook outputs vêm de Linux runner via Git LFS).
**Mitigation interim**: W1.T10 implementer escolhe.

### E11 — Cook failure recovery negative-path testing ausente
**Problema**: Plano não inventoriou testes para OOM (input 16K×16K), disk full, ctt-cli crash mid-process, partial output cleanup, fixture corrompido.
**Wave/ADR resolution**: W1.T15 5-lente audit explicit lens "negative-path coverage".
**Mitigation interim**: cook errors atualmente abortam CI silenciosamente; W1.T15 adiciona explicit fixtures.

### E12 — Tier-mismatch resolution failure path indefinido
**Problema**: `logical_texture_resolve(logical_id, tier=Mobile, db)` quando AssetDb só tem Desktop cookado? Fallback ladder não especificado.
**Wave/ADR resolution**: W2.T4 plan addendum: fallback ladder concreto (tier-down: Desktop → Mobile → Web → LowEnd → RGBA8 source); logging via `log::warn`; magenta-sprite fallback se uncook source ausente.
**Mitigation interim**: aceitar None retorno; renderer renders Atlas fallback silently.

### E13 — Painter `ph2d-color` LOC cap 4% headroom colide com W3 raised estimates
**Problema**: cap 2500 LOC; atual 1003 + ADR-0051 expansões previstas (~1200) + ACES (~200) = ~2400; margem real ~100 LOC. W3.T1 raised estimate 600-800 LOC (de 150).
**Wave/ADR resolution**: W3 entry gate `cap_overflow_blocks_w3` — falha CI se `ph2d-color` LOC > 2400 antes W3 começar. Se atingir, amendment ADR-0051 §2.1 cap (major event).
**Mitigation interim**: W3.T1 LOC raised explicitly; pré-W3 audit checa cap status.

---

## Memórias relacionadas

- [[ktx2-phase1-done-phase2-aborted-2026-05-26]] — Fase 1 OK + ADR-0055 anterior abortado
- [[no-industrial-claims-without-verification]] — pre-flight obrigatório a executar EM CÓDIGO real, não em ADR
- [[feedback-perfection-no-deferrals]] — gaps W0.T2 R1 → remediation W0.T2 R2 (não diferir)
- [[feedback-audit-lens-diversity]] — rotacionar lentes entre rounds

---

## Métricas de progresso (W0 → W3)

- W0: ~900 LOC docs cumulative (ADR Round 1+2 + este plano + SKILL updates + HR-1 §2.7.1 + HANDOFF patch).
- W1: ~1800-2400 LOC cooker + asset variant + determinism gate + 8 fixtures + audit.
- W2: ~1200-1500 LOC wgpu mapping + SpriteSource breaking migration + pipeline-per-format + budget.
- W3: ~900-1300 LOC Painter brush/UI/Export dialog.

Total Phase 2: **~4800-6100 LOC** (compare Fase 1: 1229 LOC).

---

**Coord-A: edite este plano ao iniciar/fechar cada wave; status canônico no fim da sessão.**
