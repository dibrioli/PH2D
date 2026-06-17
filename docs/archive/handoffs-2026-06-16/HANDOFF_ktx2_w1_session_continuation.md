# HANDOFF — KTX2 Fase 2 W1 continuação (sessão 2026-05-27/28)

**Status:** sessão produtiva de **19 commits locais** entregando W1.T0-T7 + T11 + T14 + T9. Próxima sessão **DEVE começar por auditoria meta-W1.T9** (mandato §4) antes de iniciar nova task.

**Audiência:** próximo agente Coord-A que retomar texture compression.
**Última sessão:** Coord-A (impl-texture) coexistindo com Coord-A Sprite Inspector v2 (impl-sprite) — multi-agente saudável, zero colisões.

---

## §0 Sanity check obrigatório antes de começar

```bash
# 1. Confirmar estado git
git log --oneline -20
# Esperado: HEAD = 9c31822 (W1.T9 kvd preservation)

git status --short
# Esperado: WIP alheio em vários paths (sprite-inspector, painter, vector, imageio).
# CONFIRMAR ZERO arquivos meus pendentes em tools/asset-cooker/ ou crates/ph2d-asset-ktx2/.

# 2. Re-verificar pre-existing failures cross-session (NÃO fixar — vide
#    feedback-audit-scope-discipline; reportar ao owner se mudar):
cargo test -p ph2d-editor-core --test architecture_panel_loc_cap 2>&1 | tail -5
# (hierarchy session pre-existing, panel_loc_cap 388 > 200)
cargo check -p ph2d-host-desktop 2>&1 | tail -5
# (Painter PanelEvent::Activated missing)

# 3. Verificar test infra do MEU crate funcional:
RUST_TEST_THREADS=1 cargo test -p ph2d-asset-cooker --lib 2>&1 | tail -5
# Esperado: 42 lib tests + 1 ignored passing.
RUST_TEST_THREADS=1 cargo test -p ph2d-asset-ktx2 2>&1 | tail -5
# Esperado: 32 lib + 2 doctests passing.

# ⚠️ NUNCA RODAR `cargo test -p ph2d-asset-cooker` SEM RUST_TEST_THREADS=1
# (SIGBUS/SIGTRAP determinístico — ISPC encoders globally thread-unsafe, vide §6)

# 4. Memórias-âncora — relê antes de qualquer ação:
#    - [[project-ktx2-phase2-v4-accepted-2026-05-27]]
#    - [[feedback-perfection-no-deferrals]] (refinada com escopo decisão-atual vs adjacente)
#    - [[feedback-audit-internal-state-grep]]
#    - [[feedback-audit-lens-diversity]]
#    - [[no-industrial-claims-without-verification]]
#    - [[feedback-scoped-commit-shared-index]]
#    - [[feedback-parallel-agent-commit-collision]]
```

Se algo divergir, **PARE e reporte ao Enio** antes de qualquer ação.

---

## §1 TL;DR

Sessão fechou **W1 Batch A** + **W1 Batch B** quase inteiros da KTX2 Fase 2 (ADR-0055-v4). Implementação Rust real do pipeline texture compression:

- **`tools/asset-cooker/src/texture/`** — cook lib API + multi-tier batch + target_matrix + mip pyramid + fixtures (~1.5k LOC novos)
- **`crates/ph2d-asset/`** — `Asset::TextureKtx2` variant + `TierIndex` newtype + `LogicalTextureMap` resolver (~480 LOC novos)
- **`crates/ph2d-asset-ktx2/`** — kvd preservation + `PremulIntent` + helpers (~180 LOC novos no Fase 1 crate, primeira incursão desde entrega original)

5 ciclos audit-then-fix com **10 lentes ortogonais diferentes** (γ δ ε ζ η θ ι κ λ μ) — 1 CRITICAL real encontrado e fixado (sin/cos non-deterministic), todos os demais HIGH/MEDIUM/LOW fechados inline ou diferidos com owner identificado.

**Bloqueio honesto descoberto W1.T8**: `ctt 0.4.0` + `ktx2 0.5` ambos READ-ONLY (zero `pub struct Writer`). Cooker não pode emitir kvd. W1.T9 parser preservation foi entregue; W1.T8 cooker emit DEFERRED com 3 paths possíveis documentados.

---

## §2 Sessão completa — 19 commits locais

```
9c31822 feat(asset-ktx2): W1.T9 — kvd preservation + PremulIntent + byte_size_estimate
38fe458 fix(asset-cooker): W1.T7 meta-audit λ+μ — off-by-one + alpha doc + cleanup
7ff552c feat(asset-cooker): W1.T7 — mip pyramid generation
d4644ff fix(asset-cooker): W1.T11+T14 meta-audit ι+κ — libm fix + bound tighten + plano
aa6766b feat(asset-cooker): W1.T11 + W1.T14 — 7 fixtures + R8→BC4 proof-of-life
1821f05 fix(asset-cooker): W1.T6 meta-audit η+θ — inline fixes + ISPC parallel issue
2ab3fac feat(asset-cooker): W1.T6 — cook_all multi-tier batch
8ef8a07 fix(asset): W1.T4 meta-audit ε+ζ — inline fixes byte_size + narrativa + SHA
d59a467 feat(asset): W1.T4 — TextureKtx2 variant + TierIndex + LogicalTextureMap
a6a6695 fix(asset-cooker): W1.T3 meta-audit γ+δ — inline fixes H1/H2/H3 + M1/M2
960a56c feat(asset-cooker): W1.T3 — texture cook sub-command + lib API
a4a85bf docs(audits): meta-session audit + inline fixes H1/H2/H3
e254271 feat(asset-cooker): W1.T2.1+T2.4 — ctt features pinned + deny.toml bans
a1bb1d2 docs(audits): W1.T2 — ctt 0.4.0 source audit APPROVE_WITH_CAVEATS
1e516a7 docs(session-active): Coord-A INATIVO
db6971c chore(asset-cooker): W1.T0 — add ctt 0.4.0 + sweep-grep §Open Issues
971e237 docs(adr-0055): v4 enxuta strategic-only Accepted; v3 660 LOC arquivada
```

### Estado de cada task W1

| Task | Estado | Notas |
|---|---|---|
| W1.T0 | ✅ `db6971c` | `cargo add ctt = "0.4.0"` |
| W1.T1 | ✅ | cargo check passou (4m18s) |
| W1.T2 | ✅ `a1bb1d2` | audit ctt 2 lentes APPROVE_WITH_CAVEATS (8/8 sub-crates HR-1) |
| W1.T2.1 (D1 features pinned) | ✅ `e254271` | encoder-amd OMITIDO + arch-gate |
| W1.T2.2 (D3 wrapper guard) | ⏳ defer | encoder-amd já ausente do build |
| W1.T2.3 (D4 snapshot test) | ⏳ defer | aguarda D2/W1.T10 canonical runner |
| W1.T2.4 (D5 deny.toml bans) | ✅ `e254271` | basis-universal + nvtt families banned |
| W1.T2.5 (D6 PR upstream criterion) | ⏳ defer | opcional |
| W1.T3 | ✅ `960a56c` + `a6a6695` audit | texture cook sub-command + lib API + 8 tests |
| W1.T4 | ✅ `d59a467` + `8ef8a07` audit | TextureKtx2 variant + TierIndex + LogicalTextureMap + 17 tests |
| W1.T5 (target matrix) | ✅ (junto W1.T3) | exhaustivo Tier×AssetClass + arch-gate |
| W1.T6 | ✅ `2ab3fac` + `1821f05` audit | cook_all multi-tier batch + 3 tests + E14 §Open Issue |
| W1.T7 | ✅ `7ff552c` + `38fe458` audit | mip pyramid generation + 16 tests |
| **W1.T8** | ⏳ **DEFERRED** | ctt 0.4.0 + ktx2 0.5 ambos READ-ONLY; vide §6 |
| W1.T9 | ✅ `9c31822` | kvd preservation + PremulIntent + 8 tests |
| W1.T10 + T11.5 (canonical runner CI + LFS) | ⏳ | **ALTO RISCO multi-agente** (.github/workflows/) |
| W1.T11 | ✅ `aa6766b` + `d4644ff` audit | 7 fixtures canônicos + 8 tests |
| W1.T12 (cooked-hashes.lock) | ⏳ | depende T10 |
| W1.T13 (CLI smoke CI) | ⏳ | depende T10 |
| W1.T14 | ✅ `aa6766b` | sample cook R8→BC4 proof-of-life + 4 tests integration |
| W1.T15 | ⏳ | audit 5-lente final gate antes W2 |

---

## §3 Audits já feitas (10 lentes ortogonais)

5 ciclos audit-then-fix; nenhum recriou padrão R1→R4 do v3 (anti-Goodhart sustained):

| Wave | Lentes | CRITICAL real | Fixes inline | Defers honestos |
|---|---|---|---|---|
| W1.T2/T3 | γ + δ | 0 | 3 HIGH | 5 (scope-out W3/etc.) |
| W1.T4 | ε + ζ | 0 | 3 HIGH + 1 MED | 1 (audit-adjacent) |
| W1.T6 | η + θ | 0 | 1 HIGH + 2 MED + 1 LOW | 4 (W3/W1.T15) |
| W1.T11+T14 | ι + κ | 1 (`sin/cos` non-determ) | 1 CRIT + 3 HIGH | 5 |
| W1.T7 | λ + μ | 0 | 2 HIGH + 2 MED + 1 LOW | 4 |
| **W1.T9** | ??? + ??? | **NÃO AUDITADO** | — | — |

Deliverables em `docs/audits/`:
- `ctt-source-audit-2026-05-27-CONSOLIDATED.md` (ctt 0.4.0 source 16k LOC)
- `session-2026-05-27-night-lens-{alpha,beta}-*.md`
- `w1-t3-lens-{gamma,delta}-*.md`
- `w1-t4-lens-{epsilon,zeta}-*.md`
- `w1-t6-lens-{eta,theta}-*.md`
- `w1-t11-t14-lens-{iota,kappa}-*.md`
- `w1-t7-lens-{lambda,mu}-*.md`

**MISSING:** audit deliverables W1.T9 — esse é o mandato §4.

---

## §4 MANDATO INICIAL — Auditoria meta-W1.T9 PRIMEIRO

**Antes de qualquer nova task**, rode auditoria meta-W1.T9 conforme padrão estabelecido (5 ciclos anteriores).

### Sequência exata

1. **2 lentes paralelas ortogonais** (round único, anti-Goodhart) — escolha 2 das próximas letras gregas NÃO usadas: **ν (nu), ξ (xi), ο (omicron), π (pi)**.

2. Lentes recomendadas pra W1.T9:
   - **ν (nu) — Fase 1 contract preservation**: W1.T9 é primeira mudança em `crates/ph2d-asset-ktx2/` (Fase 1, "intocável" desde entrega original). Audit (a) Cargo.toml zero changes confirmado, (b) public API additions são purely additive (zero breaking changes em existing fields/methods/types), (c) postcard wire format compatible com fixtures existentes (Ktx2Image agora tem `kvd` field — qualquer call site que faz struct literal sem `..Default::default()` quebra; verificar via grep).
   - **ξ (xi) — Bounds + DOS attack surface**: `MAX_KVD_ENTRIES = 64` + `MAX_KVD_VALUE_BYTES = 4 KiB` — esses bounds protegem contra qual attack vector? Hostile KTX2 file com 100 entries / 10 MB value: parser para no count check OR no value size check? Order matters (count before insert vs value check). Test coverage real desses paths? PH2D_PREMUL_KEY semantic correctness (tri-state Unspecified wildcard vs explicit error pra malformed value?). UTF-8 validation de keys (`reader.key_value_data()` retorna `&str` — já é validated upstream, mas vale spot-check).

3. **Lentes spawn via `Agent` tool** (subagent_type: general-purpose), paralelo (single message com 2 Agent calls). Prompts auto-contidos com:
   - Contexto W1.T9 escopo
   - Arquivos a auditar (path absolutos)
   - Anti-Goodhart explicit
   - DELIVERABLE path em `docs/audits/w1-t9-lens-{nu,xi}-*.md`
   - Tempo-box ~30-45min

4. **Pós-audit consolidation**: aplicar fixes INLINE dentro do escopo W1.T9 per regra refinada `feedback-perfection-no-deferrals` (decisão-atual). Defers a decisões-adjacentes (W1.T8 cooker emit, ASTC blocks W2, etc.) preservados com owner identificado.

5. **Re-validate**: `RUST_TEST_THREADS=1 cargo test -p ph2d-asset-ktx2`.

6. **Commit escopado** dos fixes + audit deliverables.

### Anti-padrões a NÃO recriar (lições gravadas nas memórias)

- ❌ **NÃO rodar 4 rounds de audit consecutivos** sem mudar método (R1→R4 padrão v3 = Goodhart). Round único + lentes ortogonais = healthy.
- ❌ **NÃO usar f32::sin/cos/transcendentals** sem `libm` (HR-6 non-deterministic cross-platform — vide W1.T11+T14 ι-CRITICAL-1).
- ❌ **NÃO escrever ADR com snippets de código** que vão se tornar vapor (anti-pattern v3 #12).
- ❌ **NÃO commitar Cargo.lock** sem coordenação (mix WIP alheio é a norma multi-agente).
- ❌ **NÃO aplicar `feedback-perfection-no-deferrals` a dependências adjacentes** (Plugin trait em outra crate, runtimes em outras ADRs — anti-pattern v3 #14).

---

## §5 Pastas reservadas / safe paths

### MINHA pasta de trabalho (KTX2 Fase 2):
- `tools/asset-cooker/` (texture/ + tests/)
- `crates/ph2d-asset/` (Asset::TextureKtx2 + tier.rs + logical_texture.rs)
- `crates/ph2d-asset-ktx2/` (parser API — primeira incursão desde Fase 1)
- `docs/architecture/decisions/0055-*.md`
- `docs/plans/2026-05-texture-compression-waves.md`
- `docs/audits/ctt-source-audit-*.md` + `w1-t*-lens-*.md` + `session-*-night-lens-*.md`
- `docs/HANDOFF_ktx2_phase2.md` (Fase 2 W0 entrega; status canônico)
- `docs/HANDOFF_ktx2_w1_session_continuation.md` (este arquivo)

### NÃO TOCAR (outros agentes ativos):
- `crates/ph2d-render/` (Coord-A Sprite Inspector v2 W1.T1.x próxima)
- `crates/ph2d-ecs/` + `crates/ph2d-editor-core/` (Sprite Inspector T1.3.5 R2 spread)
- `crates/ph2d-tool-painter/` (Painter T1.9 pausado, mas tem WIP)
- `crates/ph2d-painter-stroke/` + `crates/ph2d-painter-contracts/` (Painter pausado)
- `crates/ph2d-tool-vector-pen/` (Vector Module W1 audit pausado per session-end handoff)
- `crates/ph2d-imageio-*` (imageio W3 pausado)
- `crates/ph2d-panel-painter-sidebar/` (WIP crate sem src/lib.rs — quebra workspace check)
- `.github/workflows/` (compartilhado — alto risco)

### Pre-existing failures (vide `feedback-audit-scope-discipline` — NÃO fixar):
1. `cargo test -p ph2d-editor-core --test architecture_panel_loc_cap` → hierarchy session
2. `cargo check -p ph2d-host-desktop` → Painter PanelEvent::Activated missing
3. `crates/ph2d-panel-painter-sidebar` workspace member sem lib.rs

---

## §6 Known issues + workarounds

### §6.1 ISPC parallel SIGBUS (E14 §Open Issue)

`cargo test -p ph2d-asset-cooker` em paralelo **crasha determinísticamente** (SIGTRAP/SIGBUS) — vendored ISPC encoders (Intel, astcenc, bc7enc-rdo) têm global state não-thread-safe.

**Workaround obrigatório:** `RUST_TEST_THREADS=1 cargo test -p ph2d-asset-cooker`

**Notas:**
- `.cargo/config.toml [env] RUST_TEST_THREADS = "1"` testado e **NÃO funciona** (cargo `[env]` aplica build-time, não runtime para libtest).
- W1.T10 CI workflow DEVE incluir essa env var.
- Fix definitivo: `cargo nextest` (memory-isolated test runner) OR `serial_test` crate. Avaliação em W1.T15.

### §6.2 W1.T8 (cooker emit kvd) DEFERRED

`ktx2 = "0.5"` crate é READ-ONLY (zero `pub struct Writer`) + `ctt::convert` não suporta kvd. Para emit `PH2D_PREMUL` em cooked KTX2, precisa:

- **(a)** PH2D patcher post-hoc (bytes-level, complexo — header offset rewrites + alignment)
- **(b)** Upstream PR em `ctt` expondo `kvd: BTreeMap<String, Vec<u8>>` em `ConvertSettings`
- **(c)** Custom KTX2 writer puro-Rust (~500-1000 LOC; reescreve container)

Por ora `Ktx2Image::premul_intent()` sempre retorna `Unspecified` em cooked KTX2 — API existe pra future cooker integration (W1.T8.1 OR Painter W3 bg-removal cookado) sem regressão.

### §6.3 ISPC cache corruption pós-build incremental

Quando outro agente paralelo está buildando, `cargo test -p ph2d-asset-cooker` incremental pode SIGBUS por cache ISPC corrompido. **Fix:** `cargo clean -p ph2d-asset-cooker && RUST_TEST_THREADS=1 cargo test ...` (rebuild ~3-7min). Sintoma: SIGBUS em test pré-existente que passou em run anterior.

### §6.4 Pin hash `gradient_64x64` desligado

Test `gradient_64x64_pin_hash` em fixtures.rs imprime hash actual mas assert está desligado (`// assert_eq!(...)`) — esperar W1.T10 canonical runner estabelecer valor cross-platform pinned. Após W1.T10 entregue, próxima sessão deve:
1. Rodar test no canonical runner
2. Copiar hash actual reportado em `EXPECTED_HEX` const
3. Re-enable assert
4. Commit

---

## §7 Próximas tasks W1 (pós-audit W1.T9)

Em ordem de valor + risco:

### Baixo risco (mesmo crate, isolado):
1. **W1.T15** — audit 5-lente final W1 (gate antes W2). ~3-4h sessão. Catalogue todos os ciclos audit já feitos + final integration check. Recomendado APÓS audit W1.T9 + qualquer fix gap descoberto.
2. **W1.T8.1** (NOVO) — implementar patcher post-hoc PH2D que insere PH2D_PREMUL key em cooked KTX2 bytes. Complexidade ~200-400 LOC (header offset rewrites + alignment). Destrava bg-removal premul tag W3.

### Alto risco multi-agente:
3. **W1.T10 + W1.T11.5** — canonical runner CI workflow + Git LFS setup. TOCA `.github/workflows/spike.yml` compartilhado com outras sessions. **Renegociar com Enio antes** — provável criar `spike-texture-cook.yml` workflow separado em vez de modificar spike.yml. ~300 LOC YAML + setup.

### Pendentes pré-W2:
4. **W1.T12** — `assets/cooked-hashes.lock` populado via canonical runner. Depende T10.
5. **W1.T13** — CLI smoke test CI. Depende T10.

### W2 (próxima wave após W1.T15 gate):
- `ph2d-render` `ktx2_format.rs` + pipeline-per-format + `SpriteSource::CookedTexture` variant
- Mirror chain sync em editor-core
- Bump `Sprite::VERSION 3 → 4` (potencial conflito com Sprite Inspector v2 que também faz 3→4 — coordenar com Coord-A Sprite)

---

## §8 Memórias para criar pós-audit W1.T9 (opcional)

Quando W1 fechar inteiro (pós-T15), criar memória `project-ktx2-phase2-w1-complete-2026-05-XX` resumindo:
- 19+ commits locais
- 10+ lentes audit ortogonais
- ISPC parallel issue resolution (cargo nextest? serial_test?)
- W1.T8 path escolhido (patcher / upstream PR / custom writer)
- Total LOC entregue por crate
- Pré-requisitos W2 cumpridos

---

## §9 Boundaries — o que próximo agente decide vs PERGUNTA Enio

**Próximo agente decide:**
- Lentes específicas pra audit W1.T9 (escolher 2 das ν/ξ/ο/π)
- Detalhes de implementação dentro de W1.T15
- Fixes inline dentro-do-escopo per regra refinada
- Commit cadence (escopado per memory `feedback-scoped-commit-shared-index`)

**Próximo agente PERGUNTA Enio:**
- W1.T10 workflow strategy (modificar spike.yml vs criar separado)
- Push para origin/main (sessão atual NÃO pushed — 19 commits locais)
- Decisão entre 3 paths para W1.T8.1 (patcher/upstream/writer)
- Abandono ou pivot de W1 (improvável dado progresso)
- Amendments a contratos FROZEN

---

## §10 Mandato de brevidade

Enio prefere updates curtos. Mantenha:
- Status em 1-2 sentenças, não headers e tabelas
- Audit no máximo 2 lentes paralelas por round (não 3+)
- Commits a cada task lógica fechada (não acumular toda sessão num bloco)
- Se um caminho está gerando ciclo (audit-fix-audit-fix do mesmo finding), PARE e reporte ao Enio

---

**Boa sessão. W1 está 80% completo, codebase saudável, padrão sustentável validado em 5 ciclos. Não pule a auditoria W1.T9 — é o último audit pendente do batch B.**
