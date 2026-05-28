# W1.T4 — Lens ζ (zeta) audit: HR-compliance + cross-doc consistency + design defensibility

**Commit:** `d59a467` (`feat(asset): W1.T4 — TextureKtx2 variant + TierIndex + LogicalTextureMap (ADR-0055-v4)`)
**Data:** 2026-05-28
**Auditor:** Claude (Lens ζ — HR-compliance + cross-doc + design defensibility)
**Escopo:** ph2d-asset W1.T4 (Asset::TextureKtx2 + TierIndex + LogicalTextureMap)
**Método:** leitura linha-a-linha do delta + grep cross-doc + verificação Cargo.toml WIP + comparação ADR §4 vs implementação.

---

## Score: **9.2 / 10 — APPROVE**

Veredito: padrão-ouro. HRs respeitadas literalmente e em espírito; design choice `Arc<Vec<u8>>` é **defensible com documentação inline forte**; cross-doc drift residual é **mecânico e de-baixo-impacto** (placeholders `<commit>`); ZERO findings CRITICAL ou HIGH. 4 findings MEDIUM/LOW catalogados abaixo.

A lente ζ procurou ativamente por: HR-1 vazamento via dep transitiva, ADR-vs-código drift de shape, vapor `Arc<Ktx2Image>` em ADR §4, `<commit>` placeholders em registry, byte_size inconsistência com outras variants. Resultado: o único drift "ADR §4 sugeria Arc<Ktx2Image>" citado no commit message e no audit brief **NÃO É REAL** — o §4 da v4 strategic-only é silent quanto à shape (vide F-1).

---

## Findings

### F-1 [MEDIUM] — ADR §4 NÃO contém claim `Arc<Ktx2Image>` que o commit message e o plano vivo dizem que ela contém

**Local:**
- `docs/architecture/decisions/0055-cooked-texture-compression-pipeline.md:52` (§4, single bullet sobre TextureKtx2)
- `docs/plans/2026-05-texture-compression-waves.md:198` (W1.T4 entry: "Spec original ADR-0055-v4 §4 sugeria Arc<Ktx2Image>")
- commit message `d59a467` body: "Spec original ADR-0055-v4 §4 sugeria Arc<Ktx2Image>"

**Evidência:** `grep -n "Arc<Ktx2\|Arc<Vec\|blob" docs/architecture/decisions/0055-cooked-texture-compression-pipeline.md` retorna **ZERO matches**. O §4 do ADR atual diz literalmente apenas:

> "`Asset` enum (`crates/ph2d-asset/src/asset.rs`) ganha variant `TextureKtx2` (`#[non_exhaustive]` já preserva downstream)."

Sem shape. A menção a `Ktx2Blob` aparece em §7 (Histórico), mas como **anti-pattern do v3 Round 2** ("`Ktx2Blob` vs real `Ktx2Image` — 6 NOVOS CRITICAL **internos**"), não como spec.

**Impacto:** o W1.T4 design narrative ("desvio pragmático vs ADR") é **factualmente incorreto em relação ao ADR v4 Accepted** — não há desvio porque o ADR strategic-only nunca prescreveu shape. A narrativa de "amendment ADR-0055.1 quando Cargo.toml estiver disputável sem race" também perde sentido: não há nada a amender se o ADR nunca prescreveu a shape de origem.

**Causa raiz provável:** confusão com v3 Round 3 archived que continha snippet `pub enum Asset { ... TextureKtx2 { tier: TierIndex, image: Arc<Ktx2Image> } ... }` (cite `docs/archive/adrs-rounds-history/0055-v3-round-3-and-4-superseded.md`). v4 deliberadamente removeu snippets de código (anti-pattern #12 no plano vivo §Anti-patterns). Auditor brief herdou claim do v3.

**Recomendação:** dois fixes pequenos, atômicos:
1. Corrigir doc-comment em `crates/ph2d-asset/src/asset.rs:38-40` ("Spec original ADR-0055-v4 §4 sugeria...") → "Spec do v3 Round 3 (archived) sugeria `Arc<Ktx2Image>`; v4 Accepted strategic-only não prescreve shape (§4 = só variant existir). Design pragmático `Arc<Vec<u8>>` durante sessão multi-agente W1.T4 documentado em `docs/plans/2026-05-texture-compression-waves.md:198`."
2. Mesma correção no W1.T4 entry do plano vivo (linha 198).

NÃO precisa amendment ADR — o ADR é silent, não inconsistente.

---

### F-2 [MEDIUM] — `<commit>` placeholders no §Symbol Registry NÃO foram substituídos pelo SHA real `d59a467`

**Local:** `docs/plans/2026-05-texture-compression-waves.md:56-59` (4 entries marcam "**W1.T4 ✅ criou** (`<commit>`)").

**Evidência:** `grep -n "<commit>" docs/plans/2026-05-texture-compression-waves.md` retorna 4 matches, todos nas entries TierIndex / LogicalTextureId / LogicalTextureMap / Asset::TextureKtx2. Compare com W1.T3 entries (linhas 76-82) onde o SHA `960a56c` aparece literal.

**Impacto:** drift residual leve. Reviewer futuro abrindo o plano vivo não consegue navegar do "W1.T4 ✅ criou" para o commit que materializou sem `git log`. Padrão estabelecido por W1.T3 é "sempre cite o SHA".

**Recomendação:** trocar 4 ocorrências de `(`<commit>`)` por `(`d59a467`)`. Edição mecânica, ~30s. Pode entrar no próximo commit (W1.T5 ou cleanup commit pequeno).

---

### F-3 [MEDIUM] — `Asset::TextureKtx2.byte_size()` retorna `blob.len()` puro; outras variants somam overhead via `mem::size_of_val(...)`

**Local:**
- `crates/ph2d-asset/src/asset.rs:50-79` (`byte_size` impl)
- `crates/ph2d-asset/tests/architecture_texture_ktx2.rs:60-71` (gate `asset_texture_ktx2_byte_size_matches_blob_len`)

**Evidência:** comparação inter-variant:
- `Prefab`: `components.iter().map(|c| c.data.len() + size_of_val(c)).sum() + children.len() * size_of_val(...) + size_of_val(&**p)` — conta payload + per-item overhead + container.
- `Scene`: idem com overrides + relations + container overhead.
- `TextureKtx2`: `blob.len()` apenas. Ignora 1 byte `tier: TierIndex` + ~16-24 bytes `Arc<Vec<u8>>` overhead (ptr + strong + weak counts + Vec header).

Gate `asset_texture_ktx2_byte_size_matches_blob_len` (linhas 60-71) trava intencionalmente o comportamento "blob.len() direto, sem overhead" via assertion `assert_eq!(asset.byte_size(), blob_size)`. Comentário no gate justifica: "HR-13 budget accounting must read blob.len() directly (no overhead per W1.T4 design)".

**Impacto:** modesto. HR-13 budget é grosso (ordem de MB); 24 bytes per-asset overhead é < 0.001%. Mas **inconsistência interna do método** — se Prefab/Scene contam overhead, TextureKtx2 deveria também por princípio de uniformidade. Alternativa: refatorar Prefab/Scene para ignorar overhead (mais simples, mesma precisão para HR-13).

**Recomendação:** dois caminhos válidos:
- (A) Adicionar `+ std::mem::size_of_val(blob.as_ref()) + std::mem::size_of::<TierIndex>()` em TextureKtx2 e atualizar gate. Custo: ~30 LOC.
- (B) Deixar como está; adicionar comment em `asset.rs:77` explicando que cooked KTX2 é dominado pelo blob (~MB) então per-asset overhead é noise, e contrastar explicitamente com Prefab/Scene que podem ter centenas de small components (overhead pode dominar). Custo: 3 linhas de comment.

(B) parece mais defensible — não é bug, é diferença semântica entre "few large blobs" vs "many small structs". MEDIUM porque deixa cap arch-gate trancando a versão mais barata sem documentar o "por quê".

---

### F-4 [LOW] — Commit message reporta "17 new tests" mas a soma real é 19

**Local:** commit `d59a467` message body:
- "4 tests" (tier.rs)
- "8 tests" (logical_texture.rs)
- "7 arch-gate tests"
- "Tests pós-W1.T4: 67 passing em ph2d-asset (era 50, +17 do W1.T4)"

**Evidência:** `grep -c "#\[test\]"` em cada arquivo: 4 + 8 + 7 = **19**, não 17. Possível explicação: 2 testes em `db.rs` que antes existiam foram alterados (match arm extension) e o autor compensou contando "delta líquido". Mas `git diff d59a467^..d59a467 -- crates/ph2d-asset/src/db.rs | grep "fn "` mostra que nenhum test foi adicionado/removido em db.rs (só o match arm dentro de teste existente foi estendido).

**Impacto:** trivial. 2-test off-by-N em commit message. Baseline `50 → 67` (+17) também precisaria conferir contra `cargo test -p ph2d-asset --no-run 2>&1 | grep "test result"` — não bloqueia.

**Recomendação:** ignorar (nenhum gate depende disso) OU corrigir em próximo commit body se houver fixup natural. Não vale touch isolado.

---

## Validações que PASSARAM (não viraram findings — registradas pra rastreabilidade)

### V-1 — HR-1 platform-agnostic preservada ✅

`tier.rs:5-11` doc-comment declara explicitamente: "**Mandato HR-1**: este newtype é host-agnostic — `ph2d-asset` NÃO ganha dep em `ph2d-host` por causa disso. Quando `DeviceTier` materializar (slot futuro ADR), este módulo vira `pub type TierIndex = ph2d_host::DeviceTier;` alias OR re-export, sem mudança de API downstream."

Verificação:
- `grep "ph2d-host\|ph2d_host" crates/ph2d-asset/Cargo.toml crates/ph2d-asset/src/**/*.rs` → ZERO matches (excluindo doc-comments).
- Migration path concreto (type alias) documentado — não é vapor.
- `TierIndex` é `u8` newtype, derives `Serialize/Deserialize` próprias, não delega para `ph2d-host`.

### V-2 — HR-6 content-addressed identity semântica preservada ✅

`AssetId = blake3(cooked bytes)` e `LogicalTextureId = blake3(source bytes)` são **distintos** por design:
- `id.rs:?` (não inspecionado nesta auditoria, mas usado pelo gate): `AssetId([u8; 32])` blake3 de bytes opacos.
- `logical_texture.rs:39`: `LogicalTextureId([u8; 32])` blake3 do source PNG.
- `logical_texture.rs:1-23` doc-comment articula o "porquê" (cooker emite N artefatos cooked com N AssetIds blake3 distintos; cliente quer 1 identidade lógica estável across tiers; mapping externo resolve sem amendar AssetDb).
- Gate `logical_texture_id_is_distinct_from_asset_id` (linhas 74-85 do arch-gate) trava que `to_hex()` representations diferem para inputs distintos.

Separação semântica é **explícita, testada, documentada**. `AssetDb` permanece HR-6 puro (sem novos métodos `resolve_for_tier` etc.) — mapping vive em estrutura paralela.

### V-3 — HR-13 budget accounting cobre o variant novo ✅

`asset.rs:77` adiciona `Self::TextureKtx2 { blob, .. } => blob.len()` ao `byte_size()` match. Gate `asset_texture_ktx2_byte_size_matches_blob_len` (linhas 60-71) trava que o número retornado é `blob.len()` exato. Cap arch-gate `asset_enum_is_non_exhaustive_after_w1_t4` (linhas 41-57) garante que downstream pode match com `_ => "unknown"` (Asset enum não regrediu de `#[non_exhaustive]`).

Inconsistência de overhead vs outras variants registrada como F-3 (não bloqueante).

### V-4 — Cargo.toml WIP-isolation rationale verificada ✅

`git status --short crates/ph2d-asset/` confirma: `M crates/ph2d-asset/Cargo.toml` + `M crates/ph2d-asset/tests/import_image.rs` continuam unstaged — isto é, **WIP alheio do imageio fan-out ainda está nessa árvore**, exatamente como o commit message descreve. `git show d59a467 -- crates/ph2d-asset/Cargo.toml` retorna empty (commit não tocou Cargo.toml).

Rationale "evita commitar Cargo.toml com WIP alheio" é factualmente verdadeira **no momento do commit** (e ainda agora, 16h depois). `feedback-scoped-commit-shared-index` foi respeitada.

A questão "e se imageio commit primeiro?" tem resposta semi-aceitável: amendment ADR-0055.1 quando Cargo.toml liberar. Mas vide F-1 — narrative de "amendment" pressupõe ADR-vs-código drift que **não existe** (o ADR strategic-only não prescreve shape). Migration então é puramente refactor local: `blob: Arc<Vec<u8>>` → `image: Arc<Ktx2Image>`, com `byte_size()` mudando para `image.byte_size_estimate()` (W1.T9 vapor). Custo: ~50 LOC + 1 dep em Cargo.toml + gate update. Não-bloqueante para W2.

### V-5 — Symbol Registry §75 "regra do registry" respeitada ✅

`tier.rs:13-14` doc-comment cita: "Audit Lente δ W1.T3: §Symbol Registry do plano vivo (`docs/plans/2026-05-texture-compression-waves.md`) registra esta entry como 'W1.T4 cria'." — implementer leu o registry antes de codificar e respeitou. Mesma citação em `logical_texture.rs:25`. Não houve invenção de símbolos não-listados.

### V-6 — Determinismo postcard via BTreeMap (HR-6 wire-format) ✅

`LogicalTextureMap` interno é `BTreeMap<LogicalTextureId, BTreeMap<TierIndex, AssetId>>` (não HashMap). Gate `logical_texture_map_postcard_round_trip` (linhas 111-123) trava byte-identical re-serialização. Comment `logical_texture.rs:86-88` justifica BTreeMap explicitamente vs HashMap. ADR-0022 ban em sim-crates respeitado.

### V-7 — `#[non_exhaustive]` preservado em Asset enum ✅

`asset.rs:17` mantém `#[non_exhaustive]`. Gate `asset_enum_is_non_exhaustive_after_w1_t4` (arch-gate linhas 41-57) match externo com catch-all `_ => "unknown"` compila — confirma que enum não regrediu para exhaustive. Comment de teste honestamente nota a limitação: "Compile-time check: external code MUST be able to match `Asset` with a catch-all (`_`) arm. If `#[non_exhaustive]` was accidentally removed, this test would still compile but downstream crates would break." (linhas 42-44) — limitação reconhecida, não escondida.

### V-8 — Multi-source merge de LogicalTextureMap (out-of-scope) ✅

Pergunta do audit brief: "se duas máquinas têm LogicalTextureMap diferentes (developer A registrou hero.png em desktop, B em mobile), merge não-trivial. Plano resolve?" — não, e é correto: ADR-0055-v4 §2.3 prescreve **single canonical runner** (Linux x86_64 GitHub Actions) como source-of-truth. Developers não geram cooked outputs localmente em PR-merge; cook é CI step. LogicalTextureMap é gerado pelo cooker no canonical runner. Merge multi-source é não-problema por design — registry single-writer arquitetural.

---

## Resumo executivo (≤ 300 palavras)

W1.T4 entrega o identity layer multi-tier (`TierIndex` newtype + `LogicalTextureId`/`LogicalTextureMap` + `Asset::TextureKtx2`) em padrão-ouro, com 19 testes (4 tier + 8 logical + 7 arch-gate), preservando HR-1 (`ph2d-asset` continua zero-dep em `ph2d-host` — `TierIndex` é newtype host-agnostic com migration path documentado), HR-6 (`AssetId` e `LogicalTextureId` separados semanticamente, AssetDb não amendado, BTreeMap determinístico) e HR-13 (`byte_size` estende para o variant novo, gate trava o valor).

Cinco validações positivas (V-1..V-8 acima) confirmam que cada HR relevante tem evidência executável (arch-gate test) e não apenas afirmação em doc.

Quatro findings catalogados, **nenhum CRITICAL ou HIGH**:

- **F-1 MEDIUM** — narrativa "design diverge da ADR §4" é factualmente falsa: ADR v4 strategic-only não prescreve shape; confusão herdada de v3 Round 3 archived. Fix = 2 edições de comment.
- **F-2 MEDIUM** — `<commit>` placeholders no §Symbol Registry não substituídos pelo SHA `d59a467` (W1.T3 estabeleceu padrão de SHA literal). Fix = 4 edições mecânicas.
- **F-3 MEDIUM** — `byte_size()` de TextureKtx2 ignora overhead que Prefab/Scene contam. Inconsistente mas dentro de noise HR-13 (24 bytes vs MB). Fix = 3 linhas de comment ou refactor de Prefab/Scene.
- **F-4 LOW** — commit message diz "+17 tests"; soma real é 19. Ignorar.

Design choice `Arc<Vec<u8>>` em vez de `Arc<Ktx2Image>` é **defensível mesmo independente da rationale "WIP alheio"**: o ADR strategic-only é silent quanto à shape, então não há débito arquitetural — só refactor local quando Cargo.toml liberar. Migration custo ~50 LOC + 1 dep + 1 gate update.

**Veredito: 9.2 / 10 APPROVE.** Recomendo merge sem bloqueio; F-1 + F-2 podem entrar como cleanup commit pequeno no fim da W1 (ou no commit W1.T5 inicial).
