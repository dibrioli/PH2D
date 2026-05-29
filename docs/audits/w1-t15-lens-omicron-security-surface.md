# W1.T15 — Lente ο (omicron): superfície de ataque / input não-confiável

- **Data:** 2026-05-28
- **Auditor:** Claude Opus 4.8 (orquestração + verificação) com sub-agente general-purpose no sweep.
- **Escopo:** TODO o resto da superfície de ataque que a lente ξ (W1.T9) NÃO cobriu —
  ξ só auditou os bounds do kvd. Aqui: o resto do parser runtime + o decode de PNG
  do cooker.
- **Lente nova:** ο, não usada antes. Round único anti-Goodhart.

## Threat model (2 superfícies, trust distinto)
- **(A) `decode_ktx2_bytes`** — roda em **runtime de jogo** sobre assets shipados que
  podem estar corrompidos/adulterados → DEVE ser hardened.
- **(B) `cook()`** — decodifica PNG arbitrário via crate `image`, roda **offline** sobre
  assets confiáveis do dev (HR-1: nunca no bundle shipado) → trust maior, mas DoS/panic
  ainda vale notar.

## Veredito: **PASS_WITH_FINDINGS — 8.5 → 9.5/10 pós-fix**

Surface A (runtime) é **genuinamente hardened**: shift de mip bounded, casts não-truncantes,
toda alocação gated por bound ANTES de alocar, MAX_TOTAL_BYTES incremental, supercompression/
DFD/format/2D rejeitados sem panic. **Zero CRITICAL/HIGH.** Os 2 LOW (cooker sem cap explícito;
invariante de shift não-documentada) **fechados inline**.

## Findings

| ID | Sev | Surf | file:line | Descrição | Estado |
|---|---|---|---|---|---|
| ο-O1 | LOW | B offline | `cook.rs` step 1 | Decode do source PNG não tinha cap explícito de dimensão — proteção de decompression-bomb dependia 100% do default `max_alloc=512MiB` do crate `image` (que docs do upstream avisam que pode mudar em major bump). | **FECHADO inline**: `source_decode_limits()` (espelha `ph2d-asset/loader.rs`) seta `max_image_width/height = 8192` + `max_alloc = 512MiB`, decoder-agnóstico. Bônus: garante que todo artefato cookado é ≤ MAX_DIMENSION do parser → sempre decodificável. |
| ο-O2 | LOW | A runtime | `lib.rs` mip loop | Segurança do shift `pixel_width >> i` (não-panic) dependia de invariante cross-crate não-documentado (`level_count > MAX_LEVELS` reject + `levels()` yield = `level_count.max(1)`). | **FECHADO inline**: comentário ancora `i < 32` ao reject + ao `const _: assert!(MAX_LEVELS < 32)` (já existente, line ~99) que garante o bound em compile-time mesmo se MAX_LEVELS subir. |
| ο-O3 | NIT | A | `lib.rs` `let level_idx = i as u32` | Truncation usize→u32, inalcançável (`i ≤ 15`), só pra error reporting. | Sem ação. |

## Detalhe verificado (não confiando em comentários)
1. **Shift/panic/cast/unwrap (A) — SAFE.** `i ∈ 0..=15` (reject line ~602 + upstream `levels()`=`chunks_exact(24)` count = `level_count.max(1)`). Nenhum `.unwrap()/.expect()/index` em código não-test no parser. (O único `.expect` non-test é `mip_gen.rs` `chain.last()`, provavelmente não-vazio + offline.) Slicing upstream em `levels()` é pré-validado em `Reader::new` (`checked_add` + reject), então índice de level forjado não causa OOB.
2. **Memória — alloc gated antes, bound incremental.** `total_bytes` via `saturating_add`, checado ANTES de cada `Arc::<[u8]>::from(payload)`. kvd order COUNT→KEY→VALUE→ALLOC (ξ). ✓
3. **Cooker PNG (B):** `image 0.25` honra `max_alloc` no PngDecoder (bomb → `Err(Limits)`, não OOM) — agora **auto-asserido** via ο-O1 fix.
4. **mip_gen:** loop `/2`-até-1, sem overflow, alloc bounded. Offline.
5. **Supercompression/DFD/format/2D:** todos rejeitam limpo (`UnsupportedSupercompression`/`MissingFormat`/`Unsupported(raw)`/`UnsupportedDimensionality`/`ZeroDimension`/`BoundsExceeded`/`InvalidContainer`). ✓

**Método:** sub-agente leu in-scope + fonte upstream `ktx2 0.5` + `image 0.25` (limits.rs) + rodou grep `unwrap/expect/as/<</>>` + `cargo build` (exit 0). Verifiquei independentemente: o `const _: assert!(MAX_LEVELS < 32)` existe (line 99), confirmando que ο-O2 já tinha guarda compile-time (o agente subestimou — eu confirmei e ancorei com comentário). Sem fuzzer/PoC (raciocínio estático nos pontos de bomb).
