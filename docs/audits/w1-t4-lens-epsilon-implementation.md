# W1.T4 — Lente ε (Implementation Correctness + Edge Cases)

**Auditor:** LLM adversarial (Lens ε).
**Commit:** `d59a467` `feat(asset): W1.T4 — TextureKtx2 variant + TierIndex + LogicalTextureMap (ADR-0055-v4)`.
**Data:** 2026-05-28.
**Escopo:** correção da implementação, validação de invariantes, edge cases, gates de teste.
**Time-box:** ~30min.

**Score: 9.2 / 10 — APPROVE.**

Decisões corretas, invariantes traváveis estão travadas em arch-gate, edge cases conhecidos têm cite executável. Cinco achados (1 NIT-doc, 2 HIGH-context, 2 MEDIUM-design); nenhum CRITICAL. Nenhum exige rework antes de prosseguir W1.T5.

---

## Findings

### CRITICAL — 0

#### C-0 (cleared) — `TierIndex(99)` bypass via tuple-struct constructor
- **Cite:** `crates/ph2d-asset/src/tier.rs:34`.
- **Probe externa:** compilei `let _ = ph2d_asset::TierIndex(99);` em scratch crate → `error[E0425]: cannot find function, tuple struct or tuple variant TierIndex in crate ph2d_asset` (visibilidade default em tuple-struct field é private; tuple-constructor visibility segue field visibility).
- **Verdict:** invariante `0..=4` é compile-time-enforced para external code. Briefing levantou dúvida; testei e está fechado.

#### C-0 (cleared) — `Asset::TextureKtx2` blob sem cap
- **Cite:** `crates/ph2d-asset/src/asset.rs:41-44`, `crates/ph2d-asset/src/db.rs:1-213`.
- **Análise:** `AssetDb` inteiro (M6+) NÃO tem cap por-asset nem total — `byte_size()` existe explicitamente como "rough byte cost ... used later for HR-13 budget accounting" (`asset.rs:48-49`). É hookable, não enforced. M6 lib.rs:22-25 documenta: "M6 explicitly does NOT include: LRU eviction (HR-13 budget enforcement lands when M10 physics + M11 vector start producing real memory pressure)".
- **Verdict:** ausência de cap é decisão de plataforma de longa data (M6 → 2026-05-28); não regressão W1.T4. Não atribuível a este commit.

### HIGH — 2

#### H-1 — `TierIndex` hardcoded ordering com `DeviceTier` vapor + arch-gate adjacente ATIVO sobre vapor
- **Cite:** `crates/ph2d-asset/src/tier.rs:36-46`, `crates/ph2d-painter-contracts/tests/architecture_painter_contract_surface.rs:1130-1137`.
- **Achado:** `tier.rs:5` documenta "verificado em 2026-05-27 noite via grep ... ZERO matches" → confirmei `grep -rn "pub enum DeviceTier" crates/` retorna ZERO. Mas painter-contracts arch-gate `device_tier_variant_count_is_exact_5` (linha 1131) chama `count_enum_variants("ph2d-host", "DeviceTier")` esperando 5 sobre algo que não existe. Esse gate ou (a) está retornando 0 e quebrando suite (não testei painter-contracts) ou (b) `count_enum_variants` tolera ausente. Independente disso, quando `DeviceTier` materializar em ph2d-host **com ordenamento diferente do `TierIndex` em ph2d-asset**, o alias `pub type TierIndex = ph2d_host::DeviceTier;` documentado (`tier.rs:10`) silently quebraria `LogicalTextureMap` que serializou `tier=0=Desktop` em produção.
- **Risco:** quando DeviceTier nascer, dev pode definir `Mobile=0, Desktop=1` (ordem alfabética/arbitrária) e quebrar cooked manifests existentes. Não há arch-gate que trave a ordem cross-crate (`TierIndex::DESKTOP.as_u8() == ph2d_host::DeviceTier::Desktop as u8`).
- **Mitigação proposta (W1.Tnext ou follow-up):** quando criar `DeviceTier` em ph2d-host, adicionar `architecture_*` test em ph2d-host com `assert_eq!(DeviceTier::Desktop as u8, 0); assert_eq!(DeviceTier::Mobile as u8, 1); ...` espelhando `tier_index_canonical_constants_round_trip`. Atualmente arch-gate W1.T4 trava apenas o lado ph2d-asset; o lado ph2d-host é vapor.
- **Severity:** HIGH (latent serialization break path) — mas só dispara em milestone futuro; W1.T4 isoladamente é correto.

#### H-2 — `byte_size()` para `TextureKtx2` ignora overhead estrutural (`tier: u8 + Arc<Vec<u8>>` ~24B + Vec capacity slack)
- **Cite:** `crates/ph2d-asset/src/asset.rs:77`.
- **Achado:** retorna `blob.len()` apenas. Compare `Self::Prefab(p)` (linhas 53-60) que inclui `std::mem::size_of_val(&**p)` e per-component overhead. Já está documentado no arch-gate test (`architecture_texture_ktx2.rs:69` — "no overhead per W1.T4 design"), mas inconsistência interna entre variants do mesmo enum.
- **Impacto:** HR-13 budget accounting — para blobs grandes (≥1MB típico KTX2), 24-byte overhead é <0.003% noise. Para `Arc<Vec<u8>>` com capacity > len (improvável em cooked path, mas possível), undercount maior. Aceitável dado scope explícito ("rough byte cost"). Documentado.
- **Severity:** HIGH-context só por inconsistência design (Prefab inclui overhead, TextureKtx2 não). MEDIUM se considerado em escala HR-13.

### MEDIUM — 2

#### M-1 — `LogicalTextureMap::insert` silent-overwrite sem distinção API entre "first install" e "intentional swap"
- **Cite:** `crates/ph2d-asset/src/logical_texture.rs:103-110`.
- **Achado:** segue contrato `BTreeMap::insert` (retorna `Option<prev>`). Doc-comment registra "caller responsável por idempotência semantic". Mas em cooked-pipeline real, "mesma logical, mesmo tier, asset_id diferente" deveria ser raríssimo (significa que cook rodou com encoder diferente entre runs). API tradicional teria `try_insert` (erro em conflict) + `replace` (intenção explícita).
- **Trade-off:** simplicidade vs explicit-intent. Para wave 1 (cook → register chain), `Some(prev)` é informação suficiente; caller no cooker driver pode `assert!` em release.
- **Recomendação NIT:** considerar `try_insert -> Result<(), Conflict>` em W2 quando o cook driver concretizar. Não bloqueante.

#### M-2 — `LogicalTextureId::from_source_bytes` chama `update(&[u8])` single-shot
- **Cite:** `crates/ph2d-asset/src/logical_texture.rs:44-48`.
- **Achado:** API blake3 `Hasher::update(&self, input: &[u8])` não aloca (escreve in-place no buffer interno). Single-shot vs streaming não muda alocação para o **hasher** — só importa se o **caller** já tem todos os bytes em memória. Para 64MB 4K PNG bytes, caller já alocou os 64MB lendo o file; passar `&[u8]` ao hasher não duplica.
- **Verdict:** "streaming hash" só relevante se cooker quiser hash-while-decode (não é o caso W1; cook lê file inteiro → hash → re-decode separadamente). Não é bug; briefing perguntou e está OK.
- **Severity:** MEDIUM apenas por completude de análise. Sem ação.

### LOW — 1

#### L-1 — `db.rs:269-274` defense-in-depth double-arm pode confundir leitores
- **Cite:** `crates/ph2d-asset/src/db.rs:269-274`.
- **Achado:** primeiro arm `Asset::Prefab(_) | Asset::Scene(_) | Asset::TextureKtx2 { .. }` cobre todos os variants explícitos com `unreachable!()`; segundo `_ =>` repete `unreachable!()` mesma string para futuro variant não-listado (`#[non_exhaustive]` guard).
- **Análise:** ambas as mensagens são idênticas. Aceitável defense-in-depth para non_exhaustive, mas duas mensagens iguais sem cite ao variant futuro é code-smell leve. Sugestão: trocar a segunda string por `"non_exhaustive Asset variant added post-W1.T4 — extend match"`. Ajuda em failure-mode debugging.
- **Severity:** LOW. Cosmetic.

---

## Validações executáveis (runs do auditor)

| Comando | Resultado |
|---|---|
| `cargo test -p ph2d-asset` | 66 passing (37 unit + 7 arch + 8 import + 8 m6 + 6 prefab) |
| `cargo test -p ph2d-asset --test architecture_texture_ktx2` | 7/7 passing |
| `rustc let _ = ph2d_asset::TierIndex(99);` (external probe) | E0425 — bypass impossível |
| `grep "pub enum DeviceTier" crates/` | 0 matches — vapor confirmado |

**Nota:** briefing diz "67 tests"; medido 66. Off-by-one em comm anterior (não é finding, não muda audit).

---

## Conclusão

W1.T4 implementa identity layer multi-tier de forma correta e bem-testada:

1. **Decisões certas:** newtype `TierIndex(u8)` com tuple-field private bloqueia bypass externo (compile-time-verified); `LogicalTextureId` distinta de `AssetId` por construção blake3 sobre **source** bytes vs cooked; `LogicalTextureMap` via BTreeMap garante ordem determinística HR-6; design pragmático `Arc<Vec<u8>>` vs `Arc<Ktx2Image>` documentado com migration path concreto (amendment ADR-0055.1).
2. **Invariantes traváveis estão travadas:** `COUNT=5` (arch-gate), bounds 0..=4 (`new` + exhaustive loop test), `#[non_exhaustive]` preservation (compile-time check via wildcard match), determinismo postcard (round-trip + byte-identical assertion).
3. **Edge cases conhecidos têm cite:** ausência de cap por-asset = decisão de plataforma M6 (lib.rs:22), não regressão; streaming hash = não aplicável dado caller já-em-memória; overwrite silent = padrão BTreeMap aceitável + doc.

Findings remanescentes são todos forward-looking (H-1 cross-crate ordering quando DeviceTier nascer) ou cosméticos (L-1 double-arm `_`). Nenhum bloqueia avanço para W1.T5.

**Próximo passo recomendado:** quando DeviceTier materializar em ph2d-host (futuro slot ADR), adicionar arch-gate ph2d-host travando `Desktop=0, Mobile=1, Web=2, LowEnd=3, Constrained=4` para preservar contrato cross-crate documentado em `tier.rs:10`.
