# Audit W1.T11 + W1.T14 — Lens ι (Iota): Determinism & Fixture Correctness

- **Commit**: `aa6766b` (feat(asset-cooker): W1.T11 + W1.T14 — 7 fixtures canônicos + R8→BC4 proof-of-life)
- **Scope**: `tools/asset-cooker/src/texture/fixtures.rs` + `tools/asset-cooker/tests/sample_cook_brush_atlas.rs` + `tools/asset-cooker/src/texture/cook.rs` (fixture alias) + `tools/asset-cooker/src/texture/mod.rs` (re-export).
- **Lens**: ι — determinismo (HR-6 ready), fixture pattern correctness, cross-platform stability para W1.T10 canonical-runner replay-hash gate.
- **Date**: 2026-05-28
- **Auditor**: Claude Opus 4.7 (adversarial single-pass).

## Score

**7.5 / 10** — APPROVE WITH ONE CRITICAL CAVEAT.

Intra-machine determinism é sólido (8/8 tests passam, incluindo `--include-ignored`).
PNG encoding via `image 0.25.10` + `png 0.18.1` é cross-platform deterministic
(sem `tIME`, sem metadata variável, deflate Balanced reprodutível, adaptive filter
puramente data-driven). PORÉM, **2 fixtures (`normal_map_512`, `brush_atlas_256_r8`)
usam `f32::sin/cos`, que a `std` documenta EXPLICITAMENTE como não-determinístico
cross-platform** — bug latente que vai morder no W1.T10 canonical-runner CI quando
ele entrar online comparando hashes Linux x86_64 ↔ macOS ARM64.

`sdf_font_512` usa só `sqrt` (IEEE 754 garantido) — está OK.

Score não é 9-10 porque a violação `sin/cos` é **documentada na própria std** —
não inventada — e a única razão pra subir é se HR-6 fosse renegociado para
"intra-machine only" (não é, vide ADR-0055-v4 §2.3 + W1.T10 charter).

---

## Findings

### CRITICAL-1 — `f32::sin/cos` em fixtures viola HR-6 cross-machine

- **Severity**: CRITICAL (data-loss-equivalent: AssetId instabilidade cross-platform)
- **Files**:
  - `tools/asset-cooker/src/texture/fixtures.rs:112` (`brush_atlas_256_r8`: `(std::f32::consts::PI * t).cos()`)
  - `tools/asset-cooker/src/texture/fixtures.rs:157-158` (`normal_map_512`: `.sin() * 0.3`, `.cos() * 0.3`)

**Evidência primária** — Rust stdlib documenta literalmente:

```
~/.rustup/toolchains/1.95-aarch64-apple-darwin/lib/rustlib/src/rust/library/std/src/num/f32.rs:683
> "The precision of this function [sin] is non-deterministic. This means it
> varies by platform, Rust version, and can even differ within the same
> execution from one invocation to the next."
```

Idem para `cos` (linha 707). Implementação backing é `intrinsics::sinf32`/`cosf32`,
que o compilador LLVM resolve para `llvm.sin.f32`/`llvm.cos.f32`, que por sua vez
podem cair em qualquer um de: vendor libc (`__sincosf` Darwin vs glibc), libm
fallback, ou hardware-specific (Apple Silicon vs Intel SSE2 SVML vs AMD Bulldozer
sincos). **Bits podem divergir em LSBs** entre macOS ARM64 e Linux x86_64 — exatamente
a matrix do W1.T10 canonical-runner CI.

**Impacto**: `brush_atlas_256_r8` PNG bytes serão potencialmente diferentes em LSB
(quando `intensity_f` cair em região onde 1 LSB de `cos` ↔ atravessa cast `as u8`
boundary, e.g. 127.4999 vs 127.5001). Mesmo padrão em `normal_map_512` (mais
sensível ainda — 2 transcendentais × 512² pixels). PNG encode é determinístico,
mas alimentar bytes diferentes produz PNG diferentes → blake3 hash diferente →
**AssetId não-estável cross-platform** → quebra HR-6 (`AssetId = blake3(bytes)` same
input → same output).

**Cascata**:
1. W1.T14 `sample_cook_brush_atlas_r8_to_bc4_emits_valid_ktx2` passa intra-machine
   (test atual está OK no Mac do dev).
2. W1.T10 canonical-runner CI vai rodar mesma fixture no Linux x86_64 + Windows
   + macOS e comparar `blake3(fixture)` ⟶ pode falhar com "fixture hash mismatch".
3. Pior: pode passar 99% das execuções e falhar esporadicamente — flaky CI hidden
   bug (vide "can even differ within the same execution" no doc).

**Test atual NÃO pega**: `assert_deterministic` chama o generator 2× sequencialmente
na MESMA thread, MESMO binário, MESMO CPU — captura só drift intra-execução
extremo (improvável dentro do mesmo binário Rust release no mesmo run). Não
captura cross-arch.

**Fix recomendado** (em ordem de menor → maior intrusão):

(a) **Trocar `std::f32::sin/cos` por `libm::sinf`/`cosf` (deterministic crate
math implementation)**. `libm` crate é portable software-only deterministic libm
em pure Rust. Custo: 1 dep mais (`libm = "0.2"`), 5 LOC change em 2 fixtures. **PREFERIDO**.

(b) Reimplementar `brush_atlas_256_r8` com falloff polinomial (e.g., `1 - t²`
suave o suficiente pra brush stamp — perceptualmente indistinguível). `normal_map_512`
trocar `sin/cos` por LUT pré-computada deterministic (ou polinomial Chebyshev). Custo: ~15 LOC, design churn.

(c) Marcar essas 2 fixtures como `#[cfg(test_intra_machine_only)]` e excluir do
W1.T10 canonical-runner gate. Documentar como "exempt". **Não recomendo** —
defeats purpose de fixture canônico.

(d) Snapshot do PNG output em fixture LFS (e.g., `tests/fixtures/snapshots/brush_atlas_256_r8.png`)
e ler em vez de gerar. Custo: LFS infra + commit overhead. Útil só se design
exigir o pattern exato.

**Não fix válido**: tighter `assert_deterministic` cross-thread/cross-process —
não captura cross-arch.

---

### HIGH-1 — `gradient_64x64` byte-equivalence claim implícito não testado

- **Severity**: HIGH (potential snapshot lock break)
- **Files**:
  - `tools/asset-cooker/src/texture/fixtures.rs:42-52` (atual)
  - `tools/asset-cooker/src/texture/cook.rs:213` (alias `use crate::texture::fixtures::gradient_64x64 as fixture_png_64x64;`)
  - **Histórico**: commit `1821f05:tools/asset-cooker/src/texture/cook.rs:212-230` (versão pré-move)

**Verificação de byte-equivalence** (grep + leitura comparativa, este auditor confirmou): o pattern matemático é idêntico (`r = (x * 4) as u8`, `b = ((63 - x) * 4) as u8`, `g = 64`, `a = (y * 4) as u8`) e o wrapper `encode_png` faz `img.write_to(&mut cursor, ImageFormat::Png)` — mesma chamada exata. **Não há regressão de bytes** no momento.

**MAS**: não existe test guard explícito que prove byte-identity. Se um futuro
refactor mudar `encode_png` (e.g., trocar `Cursor` por `BufWriter`, ou trocar
`ImageFormat::Png` por `PngEncoder::new` direto, ou `image` patch-bump 0.25.10 →
0.25.11 mudar default filter) → todos os tests de `cook.rs` (`cook_intra_machine_byte_identity_when_repeated`,
`cook_64x64_desktop_sprite_color_emits_nonempty_ktx2`, etc.) **continuam passando**
(eles não comparam contra hash conhecido), mas snapshot tests W1.T2.3/D4 futuros
vão quebrar silenciosamente.

**Fix recomendado**: adicionar 1 test que faz `assert_eq!(blake3(gradient_64x64()).to_hex(), "<hash_pinado>")`
— stabilize hash atual como guard. Mesmo pattern que `cooker_determinism.rs:73`
faz pra `simple_sprite.json5` (`prefab_cook_hash_is_locked`). Custo: 5 LOC.

---

### HIGH-2 — KTX2 size upper bound em `bc4_smaller_than_uncompressed_baseline` é frouxo demais

- **Severity**: HIGH (regression mask)
- **File**: `tools/asset-cooker/tests/sample_cook_brush_atlas.rs:99-105`

```rust
let upper_bound = raw_r8_bytes + (raw_r8_bytes / 10);  // 64KB + 10% = 70.4KB
assert!(ktx2.len() < upper_bound, ...)
```

256² BC4 raw = 32 KB. KTX2 container overhead typical 200-600 bytes (per Khronos KTX2
spec + plano vivo §Memory Budget Math). Cooked ~32.5 KB esperado. Upper bound de
**70.4 KB** = **2.17× o esperado** — passaria silenciosamente mesmo se overhead
crescer pra 30 KB (e.g., W1.T8 KVD preservation adicionar 30 KB de metadata, ou
ctt bumpear para encoder com header bloat). Test passa, regressão escapa, W1.T2.3
snapshot lock pega depois.

**Fix recomendado**: tighter bound. Substituir `raw_r8_bytes + 10%` por
`raw_r8_bytes / 2 + 2048` (32 KB BC4 + 2 KB header generous). Custo: 1 LOC.
Bonus: split em 2 asserts:
1. "BC4 payload ≥ 30 KB ≤ 34 KB" (compression contract).
2. "KTX2 container overhead ≤ 2 KB" (header contract).

---

### MEDIUM-1 — `cook_all_emits_5_artifacts_for_sprite_color` coverage gap em NormalMap/CriticalUi

- **Severity**: MEDIUM
- **File**: `tools/asset-cooker/src/texture/cook.rs` (existente) + nova `cook_all_emits_single_channel_per_tier` (`tests/sample_cook_brush_atlas.rs:108-133`)

W1.T14 adicionou cobertura `cook_all` × `AssetClass::SingleChannel`. Bom progresso.
**Mas faltam ainda**: `cook_all` × `AssetClass::NormalMap` e `cook_all` × `AssetClass::CriticalUi`.
Plano vivo (per CLAUDE.md ADR-0055-v4 §2) tem 4 asset classes — só 2 cobertas (`SpriteColor`
do test pré-existente, `SingleChannel` do novo). Coverage tier ASTC 4×4 pra CriticalUi
(que `critical_ui_16` fixture foi DESENHADA pra exercitar) não é verificado end-to-end.

**Fix recomendado**: 2 novos integration tests análogos:
- `cook_all_emits_normal_map_per_tier` usando `fixtures::normal_map_512`.
- `cook_all_emits_critical_ui_per_tier` usando `fixtures::critical_ui_16`.

Custo: ~30 LOC. Pega regressão real (e.g., W1.T6 audit M3 fix removeu retry — se
target_matrix tiver bug em NormalMap × Constrained tier, ninguém pega).

---

### MEDIUM-2 — `brush_atlas_256_r8` retorna RGBA, não R8 real — claim "R8 source" misleading

- **Severity**: MEDIUM (semântica + downstream cook pipeline assumption)
- **File**: `tools/asset-cooker/src/texture/fixtures.rs:91-118`

Doc-comment: `"256×256 single-channel R8 brush atlas (representado como RGBA com R=intensidade, G=B=R, A=255 — sample cook BC4 lê só canal R)"`.

Implementation linha 115: `*px = Rgba([v, v, v, 255]);` — RGBA real, não R8.
Cook pipeline (`cook.rs:122 to_rgba8()`) sempre decodes PNG → RGBA8, então R8 source
PNG seria automaticamente expandido. **OK funcionalmente**, mas:

1. **PNG output será MAIOR** que se fosse R8 real (4× bytes, mesmo após deflate)
   — não importa pra correctness, mas significa que fixture não é "R8 native",
   é "RGBA grayscale que cook trata como R8".
2. **W1.T14 test bc4_smaller_than_uncompressed_baseline** compara contra
   `raw_r8_bytes = 256*256` (64 KB), assumindo a comparação narrativa "R8 → BC4
   -50%". Mas o source PNG é RGBA (decodificado pra 256 KB), então a economia
   real reportável é 256 KB → 32 KB ≈ -87.5%, não -50%. ADR-0055-v4 -50% claim
   é o que o cook OUTPUT economiza vs hypotético R8 raw — não vs source real.
   **Comentário do test (linhas 77-80) é correto narratively mas conceitualmente
   confuso**.
3. **Não há assert** que `r == g == b` no fixture output. Se um futuro refactor
   quebrar a grayscale invariant (e.g., colorize falloff acidentalmente), fixture
   muda silenciosamente e o "R8 source" claim vira falsidade.

**Fix recomendado**:
(a) Adicionar 1 unit test `brush_atlas_256_r8_is_grayscale` que decoda PNG,
itera pixels e assert `r == g && g == b`. Custo: 10 LOC.
(b) (Opcional) Reimplementar usando `image::GrayImage` (LumaA8) → PNG R8 real,
encoda mais compacto. Custo: 15 LOC + revisar test assertions. Defensivo se
quiser provar pipeline R8 source → BC4 path (cook.rs:122 to_rgba8 expandiria de
R8 → RGBA dentro do cook). **Recomendo a longo prazo**.

---

### MEDIUM-3 — `sdf_font_512` nome implica SDF font real; é radial distance

- **Severity**: MEDIUM (semantic misalignment)
- **File**: `tools/asset-cooker/src/texture/fixtures.rs:120-142`

Nome "SDF font" sugere distance field de glifos (letras renderizadas com signed
distance). Implementation é euclidean distance radial de um círculo (centro
256,256, raio 200). **Não é SDF de fonte**. Pra encoding test (BC4/R8 compression
characteristics), radial é OK — gradient suave estresses compression similarmente.
Mas o nome **mentirá** pra quem ler o test code procurando entender o que está
sendo testado.

**Fix recomendado** (escolher um):
(a) Renomear para `radial_distance_512` ou `circular_sdf_512`. Custo: 1-line +
update 1 test name + docstring. **PREFERIDO** (honest naming).
(b) Reimplementar com sample real (pré-shaped glyph "A" via cosmic-text + SDF
generation). Custo: ~80 LOC + dep ph2d-text. **Overkill** pra fixture de
compression test.
(c) Documentar como "radial distance approximation of SDF font characteristics".
Custo: 2-line doc update. **Mais barato mas menos honest**.

---

### LOW-1 — `normal_map_512` Z channel range muito estreito (~243-255)

- **Severity**: LOW
- **File**: `tools/asset-cooker/src/texture/fixtures.rs:160-164`

`nx, ny ∈ [-0.3, 0.3]` → `nx² + ny² ≤ 0.18`. Então `nz = sqrt(1 - nx² - ny²) ∈ [sqrt(0.82), 1] ≈ [0.905, 1]`. Mapping `(nz + 1) * 127.5` → `[~243, 255]`. Channel B
sempre quase-saturado.

Pra normal map padrão tangent-space (Z aponta para fora), B≈255 é correto. Mas
**range só 12 valores** (243-255) é compression-test fraco para B channel
(BC5/RG_compression nem usa B; BC1/BC7 vão comprimir trivialmente sem stress).
Acceptable pra fixture sintética, mas se W1.T11 pretende stress-test compression,
considere `nx, ny ∈ [-0.7, 0.7]` (still unit normal valid) pra range B mais largo.

**Fix recomendado**: opcional bump `let nx = (...).sin() * 0.7;` e `ny = ... * 0.7`.
Custo: 2 LOC. Bonus: mais "bumpy" visualmente. **Defer** se compression test wave
não precisa.

---

### LOW-2 — Comentários "~4MB raw" misleading (raw vs compressed)

- **Severity**: LOW
- **Files**:
  - `tools/asset-cooker/src/texture/fixtures.rs:19` (`photo_like_1024` "~4MB raw")
  - `tools/asset-cooker/src/texture/fixtures.rs:20` (`atlas_packed_4096` "~64MB raw")
  - `tools/asset-cooker/src/texture/fixtures.rs:171, 192` (mesmas claims em doc-comments)

"4MB raw" e "64MB raw" são pixel-size (`1024*1024*4 = 4 MiB`, `4096*4096*4 = 64 MiB`).
PNG output após deflate é tipicamente 30-70% disso para conteúdo sintético.
`atlas_packed_4096` gera PNG ~30 MB (não 64 MB). Pra alguém estimando custo de
LFS ou CI bandwidth lendo o comment, é off-by-2×. Trivial.

**Fix recomendado**: trocar "~4MB raw" por "1024² RGBA8 (4 MiB uncompressed pixel
size; PNG output ~1.5-3 MiB compressed)". Custo: 4 LOC de doc-comment update.
**Defer** — não bloqueia ninguém.

---

### LOW-3 — `PNG_MAGIC` const duplicada (fixtures.rs vs sample_cook_brush_atlas.rs KTX2_MAGIC)

- **Severity**: LOW (DRY mild)
- **Files**: `fixtures.rs:214`, `sample_cook_brush_atlas.rs:19-21`, `cook.rs:230` (KTX2_MAGIC inline)

Duas constantes magic-header repetidas (PNG_MAGIC em fixtures.rs unit tests;
KTX2_MAGIC em 2 sites: integration test linha 19 + cook.rs unit test linha 230
inline). Cross-crate isolation justifica não compartilhar. Mas 2 copies de
KTX2_MAGIC dentro do mesmo crate (cook.rs unit + sample_cook_brush_atlas.rs
integration) é DRY violation evitável.

**Fix recomendado** (opcional): mover `KTX2_MAGIC` para `pub(crate) const` em
`texture::cook` (ou nova `texture::magic` module). Integration test pode `use
ph2d_asset_cooker::texture::KTX2_MAGIC;` se promovido a `pub`. Custo: 5 LOC. **Defer**.

---

## Cross-platform PNG encoding — externally verified

Investigated `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/png-0.18.1/`
+ `image-0.25.10/`:

- **tIME chunk**: NÃO escrito por default. `grep tIME` em ambos: zero call sites
  no encoder path. `image::ImageFormat::Png` → `PngEncoder::new` → `set_color`/`set_depth`/
  `set_compression`/`set_filter` only (`image-0.25.10/src/codecs/png.rs:708-714`).
  `png-0.18.1/src/encoder.rs:282 write_header` → IHDR + (PLTE/tRNS conditional) +
  IDAT + IEND. **Sem timestamps, sem metadata variável**. ✓
- **sRGB/gAMA/cHRM/cICP**: NÃO escritos por default. Mesmo `grep`: zero call sites.
  Encoder só emite o que o caller setou. `image` crate `RgbaImage::write_to` não
  seta nenhum desses. ✓
- **Compression default**: `image-0.25.10/src/codecs/png.rs:672 CompressionType::Default
  → png::Compression::Balanced` → `DeflateCompression::from_simple(Balanced)` →
  zlib default level (6) via `flate2`. flate2 default é cross-platform deterministic
  (mesmo deflate stream para mesmo input + level). ✓
- **Adaptive filter**: `png-0.18.1/src/filter/mod.rs:500-528 adaptive_filter` —
  itera 4 filtros fixos (Up, Sub, Avg, Paeth), escolhe o que minimiza `sum_buffer`
  cost. Decisão é 100% data-driven, sem floats, sem platform-specific code paths.
  Tie-break determinístico (`<=` favorece último). ✓
- **flate2 backend**: cargo-tree default = `miniz_oxide` pure-Rust. Cross-platform
  bit-exact (não usa libz dinâmica que poderia variar). ✓ Verificar se features
  do projeto não fazem `flate2 + zlib-sys` — se sim, libz versão pode importar.

**Verdict**: PNG encoding está safe cross-platform DESDE QUE os bytes alimentados
ao encoder sejam idênticos. O CRITICAL-1 quebra essa precondição em 2 fixtures.

---

## Test result summary

```
running 8 tests (texture::fixtures, --include-ignored)
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; finished in 2.72s

running 4 tests (sample_cook_brush_atlas)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; finished in 0.43s
```

Big fixtures (`photo_like_1024`, `atlas_packed_4096`) executam OK com `--include-ignored`.

---

## Priorização para fix

1. **CRITICAL-1** (sin/cos non-determinism) — bloqueia W1.T10 canonical-runner.
   Fix antes do W1.T10 wire-up. Recomendo trocar `std::f32::sin/cos` por
   `libm::sinf/cosf` (1 dep, 5 LOC change). **Fix nesta sessão se W1.T10 vier
   próximo; defer se W1.T10 está a 2+ waves de distância — mas documentar como
   pré-requisito explícito**.
2. **HIGH-1** (hash lock para `gradient_64x64`) — 5 LOC, pega regressão silenciosa.
3. **HIGH-2** (KTX2 upper bound apertar) — 1 LOC, pega regressão silenciosa.
4. **MEDIUM-1** (cook_all coverage NormalMap/CriticalUi) — 30 LOC, defendível.
5. **MEDIUM-2/3, LOW-1/2/3** — defer ou batch num polish round.

## Anti-Goodhart check

Considerei rejeitar findings como "purist over-engineering":
- CRITICAL-1 NÃO é purist — é literalmente documentado pela std como
  non-deterministic; e ADR-0055-v4 + W1.T10 charter explicitamente exigem
  cross-machine determinism. Fica.
- HIGH-1 NÃO é purist — `cooker_determinism.rs:64-76` já tem precedente de
  hash-pinning. Fica.
- HIGH-2 NÃO é purist — 2.17× upper-bound é frouxo concreto; pode escapar
  regressão real. Fica.
- MEDIUM/LOW são judgement calls; deixei marcados claramente como "defer
  acceptable".

NÃO inventei findings. Total: 1 CRITICAL + 2 HIGH + 3 MEDIUM + 3 LOW = 9 findings.
Trabalho real existe (sin/cos é bug latente verdadeiro), mas implementação é
sólida intra-machine e a maioria dos findings é polish.
