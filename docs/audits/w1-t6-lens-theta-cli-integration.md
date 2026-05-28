# W1.T6 — Lens θ (theta) CLI UX, error handling & integration semantics

- **Commit auditado:** `2ab3fac` feat(asset-cooker): W1.T6 — cook_all multi-tier batch (ADR-0055-v4)
- **Auditor:** Lens θ (CLI UX + error handling + W2/W3 consumer future-proofing)
- **Data:** 2026-05-28
- **Tempo-box:** ~35min
- **Escopo:** `tools/asset-cooker/src/main.rs` (CLI surface) · `tools/asset-cooker/src/texture/cook.rs` (`cook_all` lib API) · `tools/asset-cooker/src/texture/mod.rs` · `tools/asset-cooker/Cargo.toml` · `docs/plans/2026-05-texture-compression-waves.md` (W1.T6 row + Symbol Registry + W2/W3 hooks) · `docs/architecture/decisions/0055-cooked-texture-compression-pipeline.md` (§4 lib API consumer = Painter Export)

---

## Score: **8.5 / 10 — APPROVE com follow-ups documentados**

CLI usable, error reporting reasonable, lib API correctly typed para LogicalTextureMap downstream. **5 findings**: 0 CRITICAL, 1 HIGH (partial-write FS cleanup contract não documentado), 3 MEDIUM (W2/W3 consumer ergonomics), 1 LOW (Path semantics edge). Nada bloqueante para shipping W1.T6 — gaps são integration-level follow-ups que W2.T4/W3.T1 vão exercitar naturalmente.

---

## CRITICAL

**Nenhum.** Anti-Goodhart aplicado: revisei especificamente os 4 candidatos do briefing.

- `create_dir_all` em path existente como FILE: testei mentalmente. `std::fs::create_dir_all` retorna `io::Error` com `ErrorKind::NotADirectory` (linux) ou `AlreadyExists` (alguns macOS). `main()` propaga via `eprintln!("✗ {e}")`. Mensagem é razoável ("File exists (os error 17)" ou similar) — não criptografada o suficiente para classificar como CRITICAL. Aceitável.
- `unwrap_or("cooked")` fallback de stem: input `.`/`..`/`/` gera `cooked-desktop.ktx2` etc. Não-ergonômico mas não dataloss — usuário vê o nome esquisito no output e re-roda com path proper. Aceitável.
- `image::ImageReader::with_guessed_format` carrega tudo na heap via `std::io::Cursor::new(source_bytes)`. 1GB PNG → 1GB+ alocado. Para offline-only HR-1 §2.7.1 cooker rodando em dev/CI, aceitável (CI runners ~7GB RAM padrão). Não CRITICAL.

## HIGH

### H1 — Partial-write semântica não documentada (cleanup contract)

**Cite:** `tools/asset-cooker/src/texture/cook.rs:179-180` doc + `tools/asset-cooker/src/main.rs:190-203` loop write.

> `Erros: para-no-primeiro — se cook do tier N falhar, retorna TextureCookError sem tentar tiers subsequentes. Caller pode re-tentar com lista parcial.`

A lib API doc menciona early-return mas **não menciona estado do filesystem após erro** no CLI path. `run_texture_cook_all` chama `cook_all` (memory-only) primeiro, e só depois escreve no FS — então **se o erro vier do `cook` ele NÃO deixa lixo no FS** (positivo!). Mas se erro vier do `std::fs::write` no meio do loop (tier 3 de 5 OK, tier 4 falha por disk full), aí sim deixa 3 ktx2 órfãos em `output_dir` + 2 faltando. Painter Export Cooked Texture (W3) chamando direto a lib API vai bater nessa borda: usuário aborta no meio da exportação → diretório com artifacts parciais → próximo "Cooked Texture" load no asset browser pega arquivo corrupted? Sem tmp+rename atômico, este é o cenário canônico de "cooker output is half-written".

**Recomendação (não-bloqueante para W1.T6, capturar em W1.T6.1 ou W3.T3):**

1. Doc explícito no `run_texture_cook_all`: "Partial state on FS error: artifacts written before failure are NOT cleaned up; caller responsible for `--output-dir` hygiene".
2. (Melhor) Migrar para 2-fase no CLI: cook todos os tiers em memória PRIMEIRO (já é o caso via `cook_all`), depois loop de write atômico — se qualquer write falhar, cleanup dos que já saíram. ~15 LOC.
3. Opcional W3-only: tmp+rename per artifact (`<stem>-<tier>.ktx2.tmp` → `<stem>-<tier>.ktx2`) para que readers nunca vejam half-written KTX2.

**Severity HIGH** porque Painter Export é o uso anunciado da lib API (ADR-0055 §4) e exposição direta de artifacts parciais quebra UX dialog ("Failed at 60%, your output dir agora tem 3 arquivos parciais — recomenda apagar manualmente"). Não CRITICAL porque W1.T6 fechou só CLI; consumer real (W3) materializa em ciclo futuro e absorve o fix.

## MEDIUM

### M1 — Progress callback ausente — Painter Export W3 freeze risk

**Cite:** `cook.rs:181-199` (`cook_all` síncrono, sem callback) + `plans/2026-05-texture-compression-waves.md:81` (Symbol Registry diz "W1.T6 ✅ criou" sem mencionar consumer ergonomics).

`cook_all` é puro síncrono e CPU-bound (ASTC encode em 4K sprite ~500ms-2s per tier × 5 tiers = 2.5-10s blocking). Chamado da UI thread do Painter dialog congela a janela. **W3.T3 dialog vai precisar:**

- Spawn em worker thread (Painter sabe ou vai descobrir runtime?).
- Progress (tier 3/5) para barra de progresso na dialog.
- Cancelamento mid-batch (usuário fecha dialog).

**Recomendação:** capturar em W1.T6.1 ou W3.T3-pre:

```rust
pub fn cook_all_with_progress(
    source_bytes: &[u8],
    asset_class: AssetClass,
    mut on_progress: impl FnMut(Tier, usize, usize),  // (tier, idx, total)
) -> Result<BTreeMap<Tier, Vec<u8>>, TextureCookError>
```

Manter `cook_all` atual como wrapper de `cook_all_with_progress(.., |_,_,_| {})`. **~15 LOC**. Aceitação: smoke onde callback é exercitado por test.

Doc de plano não menciona — adicionar bullet em "Follow-ups W3.T3-pre" no plano vivo.

### M2 — Helper `cook_all_into_map(.., &mut LogicalTextureMap)` ausente

**Cite:** `cook.rs:170-172` doc menciona "caller pode então hashar cada artefato para `AssetId` (HR-6 content-addressed) e registrar no `ph2d_asset::LogicalTextureMap` externo" — mas não fornece helper.

Cada consumer (CLI no `main.rs:194-200`, Painter dialog W3.T3, asset-pipeline W3.T1 brush atlas wire, etc.) vai duplicar:

```rust
let asset_id = AssetId::from_bytes(&ktx2_bytes);
logical_map.insert(logical_id, tier.into(), asset_id);  // Tier→TierIndex conversão
```

A conversão `Tier → TierIndex` também não está visível (busquei: `target_matrix.rs:19-28` define `Tier` como shadow newtype mas sem `impl From<Tier> for TierIndex`). **Cada consumer vai reimplementar essa conversão** — quebra HR-6 BTree ordering claim se um deles errar a ordering.

**Recomendação:** adicionar `impl From<Tier> for ph2d_asset::TierIndex` (1 match arm, ~10 LOC) + helper:

```rust
pub fn cook_all_into_map(
    source_bytes: &[u8],
    asset_class: AssetClass,
    logical_id: ph2d_asset::LogicalTextureId,
    map: &mut ph2d_asset::LogicalTextureMap,
) -> Result<(), TextureCookError>
```

Diferir explicitamente como **W1.T6.1** no plano (linha 201 já abre essa porta: "LogicalTextureMap integration ... fica como follow-up W1.T6.1") — bom. Mas adicionar a conversão `Tier→TierIndex` deveria ser mover de **W1.T6.1 → W1.T6** dado que o plano linha 81 do Symbol Registry afirma "BTree ordering determinístico (HR-6)" sem que exista o impl. **Não-bloqueante porque o CLI atual não usa LogicalTextureMap**, mas é débito imediato que W2.T4 + W3.T1 vão exercitar nas próximas 2 waves.

### M3 — Exit code uniforme (1=FAILURE) ignora unix conventions

**Cite:** `main.rs:142-147` — todo erro → `ExitCode::FAILURE` (1).

CI scripts e Painter Export dialog não conseguem distinguir:
- Input corrupted (user error, retry inútil)
- FS permission denied (env error, retry depois de chmod)
- ctt encoder bug (internal error, file issue)

Unix convention: 2 = misuse, 64-78 = sysexits.h (`EX_DATAERR=65`, `EX_NOPERM=77`, `EX_SOFTWARE=70`, `EX_IOERR=74`).

**Recomendação:** match em `TextureCookError` variant + std::io::Error kind para selecionar exit code. **~20 LOC**. Aceitação: doc no `main.rs` linkando sysexits convention. Severity MEDIUM porque Painter dialog (W3.T3) vai chamar lib API direto, não via process spawn — exit code só importa para asset-pipeline scripts (W3.T1 wire de atlas). Aceitável diferir para W1.T6.2.

## LOW

### L1 — Multi-dot stems → estranho mas válido

**Cite:** `main.rs:186-189`.

`Path::file_stem` em Rust 1.x retorna tudo até o **último** `.`: `hero.tar.gz.png` → `hero.tar.gz`. Output: `hero.tar.gz-desktop.ktx2`. Funciona, válido em todos os FS, mas estranho. Usuários com source files versionados (`logo_v2.1.png`) vão ver `logo_v2.1-desktop.ktx2` — passável. **Sem ação.**

### L2 — `tier_filename` lowercase vs `Tier::Desktop` CamelCase

**Cite:** `main.rs:206-214` define `tier_filename(Tier::Desktop) → "desktop"`.

`TierIndex::name()` em `ph2d-asset/src/tier.rs` provavelmente retorna "Desktop" (verificar — não li). Inconsistência: file paths usam lowercase (convenção POSIX-friendly), logs/inspector usam CamelCase. **Aceitável** porque file paths são convencionalmente lowercase. Tag se virar bug "case-sensitivity FS na CI Windows" no futuro.

### L3 — Println unicode `✓` em Windows console

**Cite:** `main.rs:144,167,195,202,234`.

Windows 10+ console default agora é UTF-8. Pre-Win10 ou WSL com terminal antigo pode garblear. Não-bloqueante para asset-cooker (rodado em dev/CI Mac+Linux primários). **Sem ação.**

### L4 — Help text expõe ticket reference

**Cite:** `main.rs:67-69`: `/// W1.T6 — cook one input PNG into N KTX2 outputs ...`

Usuário final lendo `cooker texture cook-all --help` vê "W1.T6" no help. Útil pra grep/blame interno; alguma fricção pra usuário externo. Convenção PH2D existing já faz isso (linha 47 `Texture` cmd diz "W1.T3 (ADR-0055-v4)"). **Aceitável — alinhado ao precedente.**

---

## Validação automática (não rodada — workspace quebrado)

Tentei `cargo run -p ph2d-asset-cooker -- texture cook-all --help` mas workspace está com manifest broken (`crates/ph2d-tool-vector-pen/Cargo.toml` ausente — WIP de outro agente paralelo). Não-bloqueante para este audit: review estático cobre os pontos críticos.

**Confirmação clap default kebab-case via grep:**
`/Users/dibrioli/.cargo/registry/.../clap_derive-4.6.1/src/item.rs:27`:
```rust
pub(crate) const DEFAULT_CASING: CasingStyle = CasingStyle::Kebab;
```

→ `TextureCmd::CookAll` materializa como subcommand `cook-all` (válido); `AssetClassArg::SpriteColor` materializa como `--asset-class sprite-color` (válido); `--asset-class invalid-class` clap rejeita com mensagem "possible values: sprite-color, critical-ui, single-channel, normal-map" (verificado mentalmente pela API `ValueEnum`).

---

## Follow-ups consolidados (para incorporação em plano vivo)

| # | Severity | Ação | Wave-target |
|---|---|---|---|
| H1 | HIGH | 2-fase write atômico (cook-to-mem-then-write-all) + doc partial-state contract | W1.T6.1 OU W3.T3-pre |
| M1 | MED | `cook_all_with_progress` callback + `cook_all` wrapper | W3.T3-pre |
| M2a | MED | `impl From<Tier> for TierIndex` (estrutural, não W1.T6.1) | W1.T6 amend |
| M2b | MED | `cook_all_into_map(.., &mut LogicalTextureMap)` helper | W1.T6.1 (já no plano) |
| M3 | MED | Exit code unix-conventions per error variant | W1.T6.2 |
| L1-L4 | LOW | Documentar como design choice OU sem ação | — |

---

## Conclusão

`cook_all` lib API e CLI surface são solidamente construídos: tipos corretos (`BTreeMap<Tier, Vec<u8>>` ordering determinístico), erros bem propagados, defaults sãos, tests cobrindo HR-6 byte-identity + AssetId distinctness + 5-artifact emit. Os gaps são todos integration-level (consumer ergonomics para W2/W3) e não bloqueiam ship do W1.T6 atual. Score 8.5/10 reflete: implementação core sólida (10/10) menos 1.5 pontos de débito documentado-but-not-resolved (partial-write semantics + progress callback + tier conversion). **APPROVE.**
