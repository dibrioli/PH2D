# W1.T15 — Lente π (pi): superfície de API pública / ergonomia / forward-compat

- **Data:** 2026-05-28
- **Auditor:** Claude Opus 4.8 (orquestração + verificação) com sub-agente general-purpose.
- **Escopo:** a superfície PÚBLICA dos 3 crates, nunca auditada como superfície coerente
  (passes anteriores: correctness, determinism, bounds, multi-tier, mip, perf, contract,
  seam, ledger). Error-type completeness, `#[non_exhaustive]` consistency, ergonomia/footguns,
  naming/discoverability, const surface.
- **Lente nova:** π. Round único anti-Goodhart.

## Veredito: **PASS_WITH_FINDINGS — 8.0 → 9.3/10 pós-fix**

Superfície bem-formada e com cobertura de error-reachability incomum (todas as 13 variantes
de `Ktx2Error` construídas + testadas). Achados concentram-se em **hazards de semver**
(`#[non_exhaustive]` ausente em 3 enums públicos com plano de crescimento) + drift de doc.
Nada exige redesign; fixes locais e aditivos — **todos fechados inline** exceto 3 LOW
documentados.

## Findings

| ID | Sev | file:line | Descrição | Estado |
|---|---|---|---|---|
| π-1 | HIGH | `lib.rs` `enum Ktx2Format` | NÃO era `#[non_exhaustive]` apesar de docs+ADR prometerem mais VkFormats em Fase 2 e o design esperar `match` downstream. `Unsupported(u32)` NÃO basta (`match Foo \| Unsupported(_)` quebra ao surgir `Bar`). Assimétrico com ν-7 (que pôs em Ktx2Error/Image). | **FECHADO inline**: `#[non_exhaustive]` adicionado. Verifiquei: único match externo é meu seam test (usa `matches!`, wildcard-safe) + doctest (`other => panic!`). Zero quebra. |
| π-2 | MED | `lib.rs` `enum PremulIntent` | NÃO era `#[non_exhaustive]`; enum de metadata forward-compat. | **FECHADO inline**: `#[non_exhaustive]`. |
| π-4 | MED | `cook.rs` `enum TextureCookError` | NÃO era `#[non_exhaustive]`; vai ganhar variantes (WebP/JPEG W4+). Assimétrico com Ktx2Error. | **FECHADO inline**: `#[non_exhaustive]`. |
| π-5 | MED | `docs/plans/2026-05-texture-compression-waves.md:48-52` | **Symbol-registry canônico para implementadores estava factualmente errado pós-W1.T9**: dizia `Ktx2Image` em "linha 365" com shape de 4 campos, e marcava `premul_intent`/`byte_size_estimate`/`kvd`/`PremulIntent` como "NÃO existe / W2.T-pre cria" — todos **já existem em W1.T9**. | **FECHADO inline**: 5 rows atualizadas (shape com `kvd` + `#[non_exhaustive]`, e marcadas `W1.T9 ✅ criou`). |
| π-3 | LOW→NIT | `cook.rs` `TextureCookError::Io` | Agente alegou "dead variant". **Verifiquei e corrigi a alegação**: `Io` está fiado num `?` vivo (`with_guessed_format()?` → `From<io::Error>`), só é raramente atingido na prática (cursor in-memory). NÃO é variante não-construída; remover seria errado. `#[non_exhaustive]` (π-4) já cobre a preocupação semver. | Mantido + non_exhaustive. Sem remoção. |
| π-6 | LOW | `docs/archive/adrs-rounds-history/0055-v3-...md` | Ref stale `ph2d_asset_ktx2::parse` num doc **arquivado/superseded** (impacto ~zero). As refs `parse` vivas em HANDOFFs descrevem o bug que ν-6 fixou (corretas). | NÃO fixado — doc de história arquivada não se reescreve. |
| π-7 | LOW | `asset.rs:42` `blob: Arc<Vec<u8>>` | Double-indirection vs `Arc<[u8]>` (que `ImageRgba8` usa). Mas o doc (asset.rs:39-41) **já nomeia `Arc<Ktx2Image>` decode-once como a migração canônica futura** — `Arc<[u8]>` seria meio-passo substituído. Único construtor é o arch-gate test (zero site de produção; W2 não existe). | NÃO fixado — wart interino com resolução futura documentada; não churnar o arch-gate por um meio-passo. |
| π-8 | LOW | `lib.rs` `base_level()` | Panic em `mip_levels[0]` alcançável só via struct-literal interno (`#[non_exhaustive]` bloqueia externo; decoder garante ≥1 em line ~663). Zero site vivo. | Aceitável as-is. |
| π-9 | NIT | `logical_texture.rs` `to_hex` | 32 `String` transitórias por call num helper de debug. | Sem ação. |

## Passou limpo (verificado)
- **Error reachability (Ktx2Error):** todas as 13 variantes construídas em código não-test E cobertas por teste. Zero dead variant.
- **Error-message hygiene:** `InvalidContainer(String)` documentado como diagnóstico opaco upstream; `KvdKeyTooLong` deliberadamente NÃO carrega a key (evita o alloc multi-MiB que ela guarda). Sem leak interno.
- **const surface:** MAX_DIMENSION/TOTAL_BYTES/LEVELS/KVD_*/PH2D_PREMUL_KEY todos `pub` + doc + `const _: assert!` sanity. Consumidores raciocinam sobre limites.
- **Tier/AssetClass/TierIndex:** exaustivos por design (TierIndex FROZEN=5 ADR-0053 + arch-gate). `#[non_exhaustive]` seria ERRADO aqui — contratos fechados. ✓
- **seam:** cook→decode exercitado por `seam_cook_decode_ktx2.rs` (dev-dep only — decisão W1.T4 honrada). ✓

**Método:** sub-agente grep de cada construção de variante + testes + `ph2d_asset_ktx2::` ref sweep + arity de `cook`. Verifiquei independentemente: π-1 safety (matches externos só `matches!`), π-3 (Io fiado em `?` vivo — corrigi a alegação de "dead"), π-7 (único construtor = arch-gate test).
