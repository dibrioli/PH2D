# W1.T6 audit — Lens η (multi-tier semantic correctness + target matrix design)

**Commit:** `2ab3fac` (feat(asset-cooker): W1.T6 — cook_all multi-tier batch (ADR-0055-v4))
**Auditor:** Lens η (multi-tier semantics + target matrix design)
**Data:** 2026-05-28
**Time-box:** ~35min
**Veredito:** **9.0/10 — APPROVE**

## Sumário executivo

W1.T6 adiciona `cook_all(source, asset_class) -> BTreeMap<Tier, Vec<u8>>` em `tools/asset-cooker/src/texture/cook.rs`. ~100 LOC reais (estimate ~200 over-provisioned; aproveita `cook` + `target_for` existentes). 3 tests + CLI `cooker texture cook-all`. Dois fixes notáveis (Tier::Constrained Uncompressed + ASTC collapse Mobile=Web) corretos, validados via ctt source. Design intentional (Mobile=Web sharing ASTC) bem documentado em test + matrix doc. Color-space propagation pelo passthrough Uncompressed verificada na ctt internals (`vk_format::denormalize`). Nenhum CRITICAL ou HIGH bloqueador. 5 findings LOW/MED de polimento, todos non-blocking.

---

## CRITICAL findings

**Nenhum.** Hipóteses do briefing investigadas e refutadas:

### Hipótese refutada 1 — Constrained perde sRGB tag

Hipótese: `TargetFormat::Uncompressed(R8G8B8A8_UNORM)` passthrough perde a flag sRGB no KTX2 header.

**Refutada via ctt source:** [`ctt-0.4.0/src/output/ktx2.rs:19`](file:///Users/dibrioli/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ctt-0.4.0/src/output/ktx2.rs) executa `first.format.denormalize(first.color_space)`. Para input Surface com `format=R8G8B8A8_UNORM + color_space=Srgb` (default SpriteColor via [`for_asset_class`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/tools/asset-cooker/src/texture/cook.rs:90)), `denormalize` emite `R8G8B8A8_SRGB` no KTX2 vk_format header ([`vk_format.rs:545`](file:///Users/dibrioli/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ctt-0.4.0/src/vk_format.rs)). Test interno `roundtrip_rgba8_srgb` ([`ktx2.rs:267`](file:///Users/dibrioli/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ctt-0.4.0/src/output/ktx2.rs)) confirma `header.format == R8G8B8A8_SRGB` + `transfer_function == SRGB`. Constrained NormalMap: Linear → R8G8B8A8_UNORM (correto, linear data). **Path correto end-to-end.**

### Hipótese refutada 2 — `bc7_paths_never_dispatch_via_auto` panic com Constrained

Hipótese: arch-gate W1.T3 itera Constrained agora que retorna `Uncompressed` em vez de `Compressed`.

**Refutada:** [`target_matrix.rs:233`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/tools/asset-cooker/src/texture/target_matrix.rs) literalmente é `for tier in [Tier::Desktop, Tier::Mobile, Tier::Web, Tier::LowEnd]` — **NÃO** itera Constrained. Helper `extract` ([`target_matrix.rs:165`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/tools/asset-cooker/src/texture/target_matrix.rs)) já tem arm para `Uncompressed` (`(format, "Uncompressed")`), então mesmo se iterasse seria graceful. Test `constrained_falls_back_uncompressed` ([`target_matrix.rs:203`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/tools/asset-cooker/src/texture/target_matrix.rs)) cobre esse arm explicitamente.

### Hipótese refutada 3 — Mobile+Web colision é bug

Hipótese: 4 distinct AssetIds em vez de 5 é bug de copy-paste.

**Refutada (design intentional, bem documentado):** doc do enum em [`target_matrix.rs:36-37`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/tools/asset-cooker/src/texture/target_matrix.rs) descreve Web como "adapter-dependent ladder BC → ASTC → ETC2 → RGBA8" e comment em [`target_matrix.rs:99-102`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/tools/asset-cooker/src/texture/target_matrix.rs) explica que cooker emite ASTC como "primary cooked artifact pro tier Web" e renderer faz fallback runtime. Test [`cook.rs:317-350`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/tools/asset-cooker/src/texture/cook.rs) tem 19 linhas de comment explicando porquê 4 (não 5) AssetIds + assert explícito `map[Mobile] == map[Web]` (byte-identical). **Padrão-ouro:** invariante observável + comment + assert + matrix doc reciprocally consistent.

---

## HIGH findings

**Nenhum.**

---

## MEDIUM findings

### M1 — Test coverage assimétrica entre AssetClasses

[`cook.rs:324-350`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/tools/asset-cooker/src/texture/cook.rs) — `cook_all_artifacts_distinct_per_distinct_target` cobre só `SpriteColor` (4 distinct AssetIds). Para CriticalUi/SingleChannel/NormalMap o padrão é diferente:

- **CriticalUi**: Mobile=ASTC_4x4, Web=ASTC_4x4 → também 4 distinct (mesma colisão)
- **SingleChannel**: Desktop=BC4_Intel | Mobile=ASTC_6x6 | Web=ASTC_6x6 | LowEnd=ETC2 | Constrained=RGBA8 → 4 distinct
- **NormalMap**: Desktop=BC5_Intel | Mobile=ASTC_6x6 | Web=ASTC_6x6 | LowEnd=ETC2 | Constrained=RGBA8 → 4 distinct

**Severidade MED não HIGH:** invariante "distinct AssetIds per distinct target" é estrutural; verificada implicitamente via `target_for` + `bc7_paths_never_dispatch_via_auto` arch-gate. Mas: parametrizar test sobre todos 4 asset classes pegaria regressões matrix-shape específicas das outras classes (e.g., alguém alteraria Mobile NormalMap para BC5 e não notaria). **Recomendação:** macro test ou loop iterando os 4 asset classes em follow-up W1.T6.1 (não bloqueador).

### M2 — Sequential cook = 5× tempo wall-clock para fixtures grandes

[`cook.rs:181-199`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/tools/asset-cooker/src/texture/cook.rs) — itera 5 tiers serial. Para fixture 64×64 (~ms cada) ok. Para 4096×4096 production assets, 5× sequential = ~20-30s wall. ctt internals usa rayon parallelism per-block dentro do encoder mas NÃO across tiers.

**Severidade MED:** offline cooker, batch CI; non-hot-path. Mas multi-source × multi-tier ainda compõe quadraticamente. **Recomendação:** TODO comment em `cook_all` apontando que `rayon::par_iter` sobre tiers é trivial (cada tier é independente, source bytes shared via `&[u8]`) quando perf importar — provavelmente W3 (Painter export) com batches grandes.

### M3 — `cook_all` early-return contradiz comment "lista parcial"

[`cook.rs:179-180`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/tools/asset-cooker/src/texture/cook.rs) doc-comment: *"Erros: para-no-primeiro — se cook do tier N falhar, retorna `TextureCookError` sem tentar tiers subsequentes. **Caller pode re-tentar com lista parcial**."*

API atual retorna `Result<BTreeMap, Error>` — sucesso OR erro, nunca parcial. Caller não tem acesso aos tiers que cookaram antes da falha; a `BTreeMap` interna é dropped no `?`. Para implementar "lista parcial" precisaria retornar `(BTreeMap<Tier, Vec<u8>>, Option<TextureCookError>)` ou `BTreeMap<Tier, Result<Vec<u8>, Error>>`.

**Severidade MED:** comment é misleading; não há harm direto (caller só vê o erro), mas seta expectativa falsa. **Recomendação:** soften comment ("erros: para-no-primeiro, lista parcial é dropped — para multi-tier resilient cook, futuro W1.T6.2") OU implementar coleta parcial agora.

---

## LOW findings

### L1 — CLI `tier_filename` duplica `TierIndex::name()` shape

[`main.rs:206-214`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/tools/asset-cooker/src/main.rs) emite `"desktop"/"mobile"/"web"/"lowend"/"constrained"` (lowercase). `TierIndex::name()` em [`crates/ph2d-asset/tests/architecture_texture_ktx2.rs:26-30`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/crates/ph2d-asset/tests/architecture_texture_ktx2.rs) retorna `"Desktop"/"Mobile"/.../"Constrained"` (CamelCase). Inconsistência cosmética: filename é lowercase-kebab convenção, log/Display é CamelCase. Aceitável mas adiciona vapor.

**Recomendação:** quando `TierIndex` materializar como alias-target (W1.T6.x follow-up no plano vivo), add `TierIndex::filename_slug() -> &str` para single source of truth.

### L2 — Symbol Registry doc não cita test de Tier ordering

[`docs/plans/2026-05-texture-compression-waves.md`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/docs/plans/2026-05-texture-compression-waves.md) §Symbol Registry linha 76 menciona `Tier` newtype mas não há arch-gate executável validando `Tier::Desktop < Tier::Constrained` ordering. [`target_matrix.rs:26-28`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/tools/asset-cooker/src/texture/target_matrix.rs) comment afirma alinhamento com `TierIndex::*::as_u8()` mas é claim verbal. Test [`cook_all_emits_5_artifacts_for_sprite_color`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/tools/asset-cooker/src/texture/cook.rs:294) itera 5 tiers e checa presence, mas não checa ordering implícito do BTreeMap ↔ array-literal.

**Severidade LOW:** BTreeMap iteration determinismo é Rust stdlib guarantee; ordering bug seria caught por intra-machine byte identity test indireto. Mas custo de adicionar `assert_eq!(map.keys().collect::<Vec<_>>(), &[&Desktop, &Mobile, &Web, &LowEnd, &Constrained])` é uma linha.

### L3 — `unreachable!` em match arm de `target_for`

[`target_matrix.rs:116`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/tools/asset-cooker/src/texture/target_matrix.rs) — `(Constrained, _) => unreachable!("Constrained handled by early-return above")`. Defensivo correto (compilador match exhaustiveness). Considerei se `#[allow(unreachable_patterns)]` + `_ =>` seria preferível (early-return cobre Constrained 100%), mas `unreachable!` é mais auto-documenting e custo zero runtime (debug_assert em release mode if profile=panic=abort).

**Severidade LOW:** code-quality preference, não bug. Atual está OK.

---

## Itens validados verde

- [x] **Constrained sRGB tag propagation** — ctt `denormalize(Srgb)` preserva via vk_format::R8G8B8A8_SRGB header
- [x] **Mobile=Web byte-identical** — design intentional + assert explícito + 19 linhas de comment
- [x] **BTreeMap deterministic ordering** — stdlib guarantee + intra-machine identity test cobre
- [x] **Tier Ord/PartialOrd derived order** — declaration order Desktop<Mobile<Web<LowEnd<Constrained matches comment + matches array-literal iteration order em `cook_all`
- [x] **extract helper Uncompressed arm tested** — `constrained_falls_back_uncompressed` test atinge
- [x] **bc7_paths_never_dispatch_via_auto** — não regressou (não itera Constrained, helper graceful)
- [x] **CookOptions::for_asset_class auto-derive** — γ-H2 fix preservado, `cook_all` usa-o em vez de `Default::default`
- [x] **HR-3 não aplicável** — offline cooker é declarado non-hot-path no doc do `cook`
- [x] **HR-6 content-addressed** — caller hasha bytes; BTreeMap ordering determinístico + intra-machine byte identity asserted

---

## Veredito final

**9.0 / 10 — APPROVE.** W1.T6 entrega função multi-tier batch sem inventar abstração (coabita com `cook`; aproveita `target_for` + `CookOptions::for_asset_class`). Dois fixes não-óbvios (Constrained Uncompressed + Mobile=Web collapse) são corretos, documentados no test + comment, e validados contra ctt source. Design intent (Web=Mobile ladder) é arquitetural, não cosmético. Findings restantes (M1/M2/M3 + L1/L2/L3) são polimento incremental para W1.T6.1/T6.2 follow-ups, nenhum blocker.

**Não há finding inventado.** Auditoria respeitou anti-Goodhart: hipóteses do briefing investigadas via leitura direta da ctt source + tests existentes; refutadas com path de evidência. Score reflete entrega correta + 3 MED legítimos de hardening incremental.
