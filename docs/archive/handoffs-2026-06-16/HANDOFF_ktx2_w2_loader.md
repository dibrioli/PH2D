═══════════════════════════════════════════════════════════════════
HANDOFF — KTX2 Fase 2 · W2.T4 (loader/render) + W2.T5 + W2.T6 + W1 CI bundle
Autor: Coordenador (sessão 2026-05-31) · para a PRÓXIMA janela de contexto (Coord-led)
Regras: docs/IntegracaoMultiAgente/DIRETRIZ.md · plano vivo: docs/plans/2026-05-texture-compression-waves.md
═══════════════════════════════════════════════════════════════════

CONTEXTO (1 tela): a W2 do KTX2 está com **T1 ✅ T1.5 ✅ T2 ✅ T3 ✅** e **auditada
(3 lentes adversariais → APPROVE consolidado, zero CRITICAL/HIGH/MEDIUM, 3 LOW
remediados)**. Falta o **W2.T4 (loader path)** — é ele que faz `SpriteSource::CookedTexture`
de fato RENDERIZAR (hoje o extract a PULA). Depois W2.T5 (budget) + W2.T6 (audit fecha a W2).
Tudo é **commit local** (modo acumular — zero push nesta jornada; o Coord faz ship 1× no fim).

───────────────────────────────────────────────────────────────────
§0 — ESTADO (verifique no git; tudo local, não-pushado)
───────────────────────────────────────────────────────────────────
  git log --oneline origin/main..HEAD    # ~18 commits locais (KTX2 W2 + Painter interleaved)
  Commits-chave KTX2 W2:
    d72e751  W2.T1   — wgpu_format_from_ktx2_format (27 variants → wgpu::TextureFormat + Features)
    c23e01e  W2.T1.5 — Renderer::detect_supported_compressions() + CompressionFeatureSet
    29defc6  W2.T3   — compressed_pipeline.rs (1 pipeline compartilhado, block-aligned upload)
    ca538e4  W2.T2   — SpriteSource::CookedTexture { logical_id: LogicalTextureId } (aditivo)
    4b48b07  W2 audit remediation (3 LOW: checked_mul, doc, UX)
  ⚠️ HÁ commits Painter interleaved (janela paralela: sidebar swatch, undo/redo) + um
  fmt drift NÃO-resolvido em `crates/ph2d-panel-painter-sidebar/src/paint.rs` (crate do
  impl Painter — NÃO é teu; o Coord pega no ship). NÃO comite arquivos painter.

───────────────────────────────────────────────────────────────────
SUA PASTA (W2.T4 é FOUNDATIONAL — Coord-led, §3.C)
───────────────────────────────────────────────────────────────────
  crates/ph2d-render/  (extract integration, texture cache, pipeline wiring)
  crates/ph2d-asset/   (logical_texture_resolve helper — W1.T4 já tem LogicalTextureMap::resolve)
  shells/desktop/src/render_loop/  (sim_extract.rs — REMOVER o skip-guard quando o upload existir)
  É foundational/serial; NÃO paraleliza com a janela Painter (que está em ph2d-tool-painter +
  ph2d-panel-painter-sidebar — write-set disjunto, OK rodarem juntas).

───────────────────────────────────────────────────────────────────
O QUE JÁ EXISTE (REUSE — não reinvente; auditado verde)
───────────────────────────────────────────────────────────────────
- **W2.T1** `ph2d_render::{wgpu_format_from_ktx2_format, CompressionFeatureSet, FormatError}`
  (`src/ktx2_format.rs`): Ktx2Format → (wgpu::TextureFormat, required Features).
- **W2.T1.5** `Renderer::detect_supported_compressions() -> CompressionFeatureSet` +
  `.supports(Ktx2Format)`. Base do FALLBACK de tier (device sem BC → não pede BC).
- **W2.T3** `ph2d_render::compressed_pipeline::CompressedTexturePipeline`
  (`new/with_layout/upload/upload_parts/resolve_format/material_bind_group_layout/feature_set`)
  + `UploadedCompressedTexture { texture, view, bind_group }` pronto p/ `@group(1)`.
  `upload(&Ktx2Image)` decodifica os mips e faz `write_texture` block-aligned. **Rgba32Float
  rejeitado** (filterable:false) via `NotFilterableFloat` — não tente subir.
- **W1.T4** `ph2d_asset::LogicalTextureMap::resolve(logical_id, tier: TierIndex) -> Option<AssetId>`
  + `Asset::TextureKtx2 { tier, blob: Arc<Vec<u8>> }`.
- **W2.T2** `SpriteSource::CookedTexture { logical_id }` + `Sprite::cooked_texture(logical_id, size, tint)`.
  Parser: `ph2d_asset_ktx2::decode_ktx2_bytes(&blob) -> Result<Ktx2Image, _>`.

───────────────────────────────────────────────────────────────────
TASK — W2.T4: loader path (plano §6 Batch C) — ACENDER a CookedTexture
───────────────────────────────────────────────────────────────────
Objetivo: um sprite `CookedTexture { logical_id }` renderiza a textura KTX2 do tier do device.

Pipeline de resolução (por sprite, com CACHE — NÃO re-upload por frame):
  1. **Tier do device:** decidir o `TierIndex` ativo a partir de
     `detect_supported_compressions()` (BC→Desktop, ASTC→Mobile/Apple, ETC2→Android,
     nenhum→Constrained/Web RGBA8). **Design point a resolver:** onde o DeviceTier vive
     (ADR-0053) — provável um campo no Renderer setado no init via feature query. Documente.
  2. **Resolver:** `logical_id + tier → AssetId` via `LogicalTextureMap::resolve` (ou um
     novo `logical_texture_resolve(logical_id, tier, db) -> Option<Arc<Asset>>` em
     `ph2d-asset/src/logical_texture.rs` se for mais limpo — o plano §6 W2.T4 cita esse helper).
     Fallback de tier: se o tier ideal não tem asset, descer pro próximo suportado.
  3. **Decode + upload (1×, cacheado por AssetId):** `decode_ktx2_bytes(blob) -> Ktx2Image`
     → `CompressedTexturePipeline::upload(&img) -> UploadedCompressedTexture`. Cachear o
     resultado num `BTreeMap<AssetId, UploadedCompressedTexture>` (HR-5 BTree determinístico;
     espelhe o `IndividualTextureStore`). Construir o `CompressedTexturePipeline` 1× passando
     o `SpritePipeline::material_bgl` via `with_layout` (garante bind-group compatível).
  4. **Bind no draw:** o sprite CookedTexture emite um RenderInstance com o bind group da
     textura uploadada (espelhe o caminho `Individual` — texture_id → bind group no batcher).
  5. **REMOVER o skip-guard** em `sim_extract.rs` (`&& !matches!(spr.source, CookedTexture{..})`,
     comentário "W2.T2") e os 3 arms fallback dead — substituir pelo binding real. Esse guard
     foi DELIBERADO p/ W2.T2 (invisível até o loader existir); agora ele sai.

DoD (W2.T4): cook um PNG → `cook-all` (5 tiers) → carrega → sprite CookedTexture renderiza
  BC7 no desktop / ASTC no iPad sim / RGBA8 no Web. Aceitação W2 (plano §6): end-to-end smoke.

───────────────────────────────────────────────────────────────────
DEPOIS (mesma janela, sequencial)
───────────────────────────────────────────────────────────────────
- **W2.T5** (plano §6 Batch D) — `Renderer::Plugin::init` declara `compressed_texture_cache_mb`
  budget (HR-13) + `compressed_size_per_format(format, w, h, mip_count)` helper. **REUSE** a
  block-math do `compressed_pipeline` (`MipUploadLayout::for_mip` já computa `total_bytes`/mip;
  some sobre os mips). LOC ~150.
- **W2.T6** — auditoria 5-lente paralela (lentes ROTACIONADAS — NÃO reuse as 3 desta sessão:
  WGSL/ABI · HR-ADR · cross-GPU realism · regressão · benchmark-vs-claim). Target ≥ 8.5/10.
  **Smoke do Enio** end-to-end antes do APPROVE.
- **W1 CI bundle (trabalho de Coord, deferido p/ o ship):** W1.T10 canonical runner
  (`determinism.rs` + workflow `spike-texture-cook.yml` separado) + W1.T11.5 Git LFS
  (`.gitattributes` p/ `assets/cooked/**/*.ktx2`) + W1.T12 `cooked-hashes.lock`. Enio delegou
  ("faça o que achar melhor") — montar quando o cook precisar de gate de determinismo em CI.

───────────────────────────────────────────────────────────────────
CONTRATOS + ARMADILHAS (auditado — respeite)
───────────────────────────────────────────────────────────────────
- **Sprite schema CONGELADO (ADR-0070):** `Sprite::VERSION = 4` · field count = 20 · RenderInstance
  POD 184B/16 attrs. Gate `architecture_sprite_inspector_surface`. **NÃO bumpe** — W2.T4 não muda
  schema (só consome o `logical_id` já no contrato). Bumpar = ADR amendment + gate vermelho.
- **SpriteSource SEM `#[non_exhaustive]`** (decisão W2.T2, segue precedente SpriteVersioned).
  Qualquer novo match exaustivo de SpriteSource DEVE tratar CookedTexture explícito (o compilador
  força — é o que queremos; NÃO adicione `_ =>` que silencie).
- **compressed_pipeline contrato** (auditado): `for_mip` usa `checked_mul` (overflow→None→clean
  error). `upload_parts` NÃO checa dims vs `max_texture_dimension_2d` antes do `create_texture`
  (delega ao decoder, que cap'a em MAX_DIMENSION=8192) — se o loader passar dims cruas acima do
  cap, considere um guard de dim no entry (L3 LOW deixado como follow-up não-bloqueante).
- **1 LOW remanescente (aceito, documentado):** a exaustividade do `ktx2_format` é assert manual
  de contagem (`len()==27`) — limitação do `#[non_exhaustive]` do Ktx2Format; variant futuro cai
  em `Err(UnmappedKtx2Format)` (loud, não silencioso). Não-bloqueante.
- **asset-cooker `RUST_TEST_THREADS=1`** sempre (ISPC SIGBUS) — via nextest já serializa.

───────────────────────────────────────────────────────────────────
VELOCIDADE (DIRETRIZ §6.6) + GIT + GATES
───────────────────────────────────────────────────────────────────
  Slot warm CoW (`bash scripts/slot-seed.sh <slot>`, prefixe CARGO_TARGET_DIR). Inner loop =
  `cargo check -p ph2d-render`. ≤3 cargos (RAM 8 GiB) — a janela Painter também compila.
  Gate 1× no fechamento: `cargo nextest run -p ph2d-render -p ph2d-asset` + clippy `--all-targets
  -D warnings` + `architecture_sprite_inspector_surface` + as 5 lentes do W2.T6.
  Git: `git status` antes de stage; `git add -- <só teus paths>` (NUNCA -A/-a/.); commit
  `--no-verify` escopado; mensagem longa via `-F` (heredoc) p/ não quebrar no shell. NÃO pusha.
  Ao fechar cada task reporte commit local + gates verdes + (W2.T4) smoke do Enio.
═══════════════════════════════════════════════════════════════════
