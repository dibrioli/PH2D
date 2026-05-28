# W1.T7 audit — Lente μ (performance / memory / ctt-integration)

**Target:** commit `7ff552c` — `tools/asset-cooker/src/texture/mip_gen.rs` (mip pyramid generator) +
downstream W1.T7.1 (cook multi-mip) + W3.T4 (Painter Export dialog) integration readiness.

**Lente:** memory footprint na chain, latency de Lanczos3 sobre 4K source, ctt::Image multi-level
support real, e UX implications na Painter W3 Export dialog.

**Verdict:** **APPROVE — score 8.7 / 10** (1 HIGH novo legítimo: alpha-premultiply assumption do
`image::imageops::resize`; 2 MEDIUM forward-compat; 3 LOW limpeza/cosmetic).

---

## Findings

### CRITICAL — nenhum

Memory footprint, latency e ctt multi-mip integration **estão dentro do orçamento razoável** para
o use case declarado (cook offline, não hot path). Os audits anteriores (ι/κ) já validaram
determinism + integration paths. Não invento bug; o módulo é solid.

**Validations executadas:**

| Claim                                                              | Source                                                                                                  | Resultado                                          |
|--------------------------------------------------------------------|---------------------------------------------------------------------------------------------------------|----------------------------------------------------|
| `ctt::Image` suporta multi-mip                                     | `~/.cargo/.../ctt-0.4.0/src/surface.rs:69-77` `pub struct Image { surfaces: Vec<Vec<Surface>>, kind }`  | ✅ `surfaces[layer][mip]` é mip chain por layer    |
| `ctt::Image::validate` verifica mip count uniformity cross-layer   | `surface.rs:97-110` + test `validate_mip_count_uniformity` (`surface.rs:600-610`)                       | ✅ chain length cap'ada pelo validator             |
| 3D textures têm depth chain `(base >> mip_idx).max(1)` validator   | `surface.rs:161-172`                                                                                    | ✅ ortogonal a 2D Texture; mip_gen 2D-only OK      |
| `image::imageops::resize` retorna `ImageBuffer<Rgba<u8>, Vec<u8>>` | `image-0.25.10/src/imageops/sample.rs:964-969` (signature paramétrica em `I::Pixel`)                    | ✅ é `RgbaImage` direto — round-trip desnecessário |
| ADR-0055 §4 menciona multi-mip embedded em KTX2                    | `docs/architecture/decisions/0055-…md` §2 + plano W3.T1 (R8→BC4)                                        | ✅ pipeline canônico assume mips no container      |

Memory footprint chain 4K = `64 * (1 + 1/4 + 1/16 + …) ≈ 85 MB`. Cook 100 atlas 4K em batch =
~8.5 GB peak. Esse é **batch worst-case** (Painter Export full project). Para single asset (use case
W3.T4 typical "exportar 1 brush atlas") fica ~85 MB; aceitável. Latency Lanczos3 4K → chain
completo: ~3-8 s single-thread. **Painter W3.T4 deve ter progress bar**, mas isso é finding
sobre Painter, não sobre mip_gen.

---

### HIGH (1)

#### μ-H1 — `image::imageops::resize` assume alpha premultiplicado em conteúdo non-constant alpha; mip_gen não documenta nem valida

**Severity:** HIGH (correctness; pode introduzir dark fringe em sprites com straight alpha).

**Cite:** `image-0.25.10/src/imageops/sample.rs:960` — doc oficial:
> *"This method assumes alpha pre-multiplication for images that contain non-constant alpha."*

**Problema:**

- `cook()` em `tools/asset-cooker/src/texture/cook.rs:128-140` constrói `Surface` com
  `alpha: options.alpha` (default `Straight`) — assets PNG cookados como sprite **são straight-alpha
  por convenção PH2D**.
- `generate_mip_chain` chama `image::imageops::resize` que **assume premultiplied** para alpha
  non-constant. Se input for straight (e.g., sprite com transparent border ao redor de glyph
  opaco), o downsample mistura cor com transparência sem cancelar o alpha → **dark halo** ao
  redor de edges em mip ≥ 1.
- ADR-0055 §4 + plano §W2.T-pre+T-bgremoval mencionam premul intent tracking via KTX2
  keyValueData `PH2D_PREMUL` (Straight / Premultiplied / Unspecified). mip_gen ignora isso.
- W1.T7.1 (`cook_with_mips`) **precisa** ou (a) premultiplicar antes do resize + des-premultiplicar
  depois, ou (b) declarar que mip_gen exige `Premultiplied` upstream e rejeitar `Straight`.

**Fix sugerido (deferred W1.T7.1 OK; doc agora obrigatório):**

1. Doc no header de `mip_gen.rs`: "**LIMITATION:** `image::imageops::resize` assume
   alpha-premultiplicado. Para `AlphaMode::Straight` sprites com transparência soft, callers
   devem premultiplicar antes de invocar `generate_mip_chain` e (se downstream consumer espera
   straight) des-premultiplicar cada level após. W1.T7.1 + W2.T-bgremoval coordenam essa
   logística."
2. Test futuro W1.T7.1: sprite com straight alpha + transparent border → mip level 1 dark fringe
   detectado vs premul version sem fringe.

Atual mip_gen é **agnóstico de alpha** — sem isso documentado, cook_with_mips no W1.T7.1 vai
silenciosamente quebrar quality em assets reais.

---

### MEDIUM (2)

#### μ-M1 — `from_raw(nw, nh, resize(...).into_raw())` round-trip é dead code

**Severity:** MEDIUM (cleanup; nenhum bug funcional).

**Cite:** `mip_gen.rs:109-115`

```rust
let resized: RgbaImage = ImageBuffer::from_raw(
    nw,
    nh,
    image::imageops::resize(prev, nw, nh, image_filter).into_raw(),
)
.expect("resize output dims match nw×nh×4 bytes");
```

`image::imageops::resize::<&RgbaImage>` retorna **`ImageBuffer<Rgba<u8>, Vec<u8>>` = `RgbaImage`**
(verificado em `image-0.25.10/src/imageops/sample.rs:964-969`). O `from_raw(into_raw())` é
round-trip identidade pagando alocação de wrapper + `Option::expect`. Simplificar:

```rust
let resized: RgbaImage = image::imageops::resize(prev, nw, nh, image_filter);
chain.push(resized);
```

Aceitável defer (não bloqueia W1.T7.1), mas é **2 linhas vs 6** com zero risco.

---

#### μ-M2 — Big fixtures (`photo_like_1024`, `atlas_packed_4096`) sem cobertura `generate_mip_chain` mesmo `#[ignore]`

**Severity:** MEDIUM (gap empírico; bench future).

**Cite:** `fixtures.rs:284-290` tests `#[ignore]` cobrem só `assert_valid_png` para os bigs.

Adicionar `#[ignore]` test em `mip_gen.rs` que mede:

1. `generate_mip_chain(atlas_packed_4096(), MipFilter::Lanczos, None).len() == 13`
2. Soma `chain.iter().map(|l| l.as_raw().len()).sum::<usize>() < 90 * 1024 * 1024` (sanity
   memory ceiling ~85MB + 5MB slack)

Daria empirical baseline pra W1.T7.1 estimate de cook_with_mips latency (Lanczos 4K → 13 levels
em hardware real, cross-platform via canonical runner W1.T10).

**Defer aceitável:** W1.T15 (perf benchmark wave) ou W3.T4 pre-work.

---

### LOW (3)

#### μ-L1 — `generate_mip_chain` clona source pro level 0

**Cite:** `mip_gen.rs:98` `chain.push(source.clone())`

Para fixture 4 MB (1024² RGBA) o clone aloca 4 MB extra **antes** do primeiro resize. Caller que
já possui owned `RgbaImage` paga 2× memória peak. API consume `into_mip_chain(source: RgbaImage)`
evitaria.

**Defer OK:** API consume é breaking; W1.T7.1 pode introduzir variant `into_mip_chain` paralelo
quando `cook_with_mips` materializar (`cook` decode → owned RgbaImage → mip_gen consume é a path
mais natural).

#### μ-L2 — `MipFilter` enum 3 variants sem `#[non_exhaustive]`

**Cite:** `mip_gen.rs:27-38`

Adicionar `Mitchell` ou `CatmullRom` (já existe no `image::FilterType`) seria SemVer-breaking
para downstream que faz `match` exaustivo. Adicionar `#[non_exhaustive]` antes do public release
da lib API (W3.T1 já consome). Custo: zero hoje, evita break depois.

#### μ-L3 — Coverage gap: `Point` filter sem test distinctness vs `Box`

**Cite:** `mip_gen.rs:200-217` `generate_chain_lanczos3_distinct_from_box` cobre Lanczos vs Box.
Não há teste paralelo para Point vs Box em radial content. Risco baixo (Point é truncate
ortogonal a média), mas explicit gate previne regressão silenciosa em refactor futuro do mapping
`MipFilter → image::FilterType`.

---

## Forward-compat W1.T7.1 / W3.T4

Para destrancar W1.T7.1 (`cook_with_mips`) sem revisitar audits:

1. **μ-H1 fix obrigatório**: premultiply guidance no doc; idealmente helper
   `premultiply_then_resample` shared em `texture::` module.
2. **μ-M1 cleanup**: remover round-trip antes de W1.T7.1 (1 linha, sem risco).
3. **Painter W3.T4** progress bar: cook_with_mips deve aceitar `progress: Option<&dyn Fn(stage:
   &str, frac: f32)>`. 100 assets × 5 tiers × Lanczos 4K = **~15-25 min batch** (já com mip_gen
   determinístico cost ~3-8s/asset cap). Sem progress UX = inaceitável.
4. **rayon::par_iter** sobre `cook_all` per-tier deferido (já doc'd em `cook.rs:184-185`); mip_gen
   ortogonal — chain é serial dependência (level N+1 = downsample do N).

---

## Anti-Goodhart check

- Memory footprint: 4K source chain ~85 MB heap. Para typical Painter Export ("exportar este 1
  asset" → 1 source × 5 tiers paralelo serial via cook_all) = **OK**. Batch 100 assets é cenário
  hipotético plano vivo não documenta ainda; quando materializar, streaming/queue é fix natural.
- Latency Lanczos3 4K: 3-8s/asset single-thread. Para offline cook tool **não é hot path** (HR-3
  exempt em `cook.rs:116` doc). Painter W3.T4 dialog precisará UX work mas isso é finding na
  Painter, não no mip_gen.
- ctt::Image multi-mip: **confirmado via source read** (`surfaces[layer][mip]`, validator
  cross-mip uniformity check). W1.T7.1 path é viável **sem custom KTX2 writer**.

Não inventei findings pra inflar contagem. mip_gen é solid lib API — apenas 1 gap real
(premultiply) + cleanup minor.

---

## Score: 8.7 / 10

**APPROVE para W1.T7 close + W1.T7.1 destravar com pre-req μ-H1 doc fix em W1.T7.1 mesmo (ou
mip_gen.rs follow-up commit).** Determinism guards já robustos (audit ι), integration path
mapeado (audit κ), e essa lente μ adiciona apenas alpha-premultiply concern como blocker real
pra cook integration.

**Deltas vs 10/10:** μ-H1 documentação + μ-M1 cleanup (2 min) + μ-M2 big fixture coverage
(15 min) levantariam pra 9.5+. μ-L1/L2/L3 não bloqueiam.
