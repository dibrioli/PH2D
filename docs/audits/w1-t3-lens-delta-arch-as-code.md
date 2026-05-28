# W1.T3 — Lente Delta · Architecture-as-Code + Cross-Doc Consistency

**Data:** 2026-05-27 noite (post-W1.T3 close)
**Auditor:** Claude Opus 4.7 (sub-agente adversarial)
**Escopo:** validar que (a) disciplinas D1/D3/D5 declaradas implementadas continuam de fato implementadas pós-W1.T3, (b) docs ADR / plano vivo / audit consolidado / HANDOFF refletem W1.T3 sem drift, (c) gates declarativos sem enforcement real foram banidos no código real.
**Commits auditados:** `971e237` · `db6971c` · `1e516a7` · `a1bb1d2` · `e254271` · `a4a85bf` · `960a56c`.

---

## Veredito: **APPROVE com 2 MEDIUM cross-doc + 1 LOW** — score **8.5/10**

Disciplinas D1/D3/D5 estão **integralmente respeitadas** pelo commit W1.T3 (verificado em código real + `cargo test --no-run`). Implementação da §2 cooker do ADR-0055-v4 entregue conforme prometido (`tools/asset-cooker/` ganhou `texture cook` sub-command + lib API). Anti-pattern #12 (snippets de código em ADR) NÃO re-introduzido. Anti-pattern #14 (defer abuso de `feedback-perfection-no-deferrals`) respeitado — 3 defers (W1.T2.2/W1.T2.3/W1.T4/W1.T6) todos legitimamente adjacentes/posteriores. 0 CRITICAL · 0 HIGH · 2 MEDIUM · 1 LOW.

Score puxado de 9.0 para 8.5 por **doc-drift do tipo "código avançou, plano não acompanhou"** (MEDIUM-1 = plano vivo `W1.T3` ainda no status pré-implementação; MEDIUM-2 = §Symbol Registry sem 5 símbolos novos introduzidos). Nenhum drift afeta runtime nem disciplina técnica; corrigíveis em commit doc-only.

---

## Findings

### MEDIUM-1 — Plano vivo `W1.T3` row sem status `✅` + §HANDOFF §12 não atualizado

**Cite:** `docs/plans/2026-05-texture-compression-waves.md:188`:
```
- **W1.T3** — `tools/asset-cooker/src/texture/mod.rs` + sub-command CLI `asset-cooker texture cook --input X --tier T --output Y`. LOC ~250.
```
Linha continua redigida no infinitivo, sem `✅ 2026-05-27 noite`, sem cite SHA `960a56c`, sem actuals (580 LOC vs ~250 estimate = **2.3× over**). Compare com W1.T0/T1/T2/T2.1/T2.4 que ganharam `✅ 2026-05-27 noite. <SHA>` inline. Padrão estabelecido violado.

**Cross-doc cite:** `docs/HANDOFF_ktx2_phase2.md:310-322` §12 listou W1.T0/T1/T2/T2.1/T2.4 mas **NÃO** menciona W1.T3 closure nem commit `960a56c`. Última linha do §12 ainda diz "Próxima sessão Coord-A: ... Implementar D1/D3/D4/D5 **ANTES** de W1.T3" — temporal-stale.

**Classification:** MEDIUM cross-doc drift. Plano vivo é "specification canônica" (HANDOFF §12 ipsis litteris). Próximo agente lendo só o plano não saberá que W1.T3 fechou.

**Fix:** doc-only commit. (a) Marcar W1.T3 `✅ 960a56c — 580 LOC actual (estimate ~250, 2.3× over por descoberta runtime: encoder tuple-variants + `PipelineOutput` shape)`. (b) §12 do HANDOFF: adicionar bullet `W1.T3 ✅ 960a56c`. (c) Próxima sessão = W1.T4 (Asset::TextureKtx2 + TierIndex + LogicalTextureId) — não mais T3.

---

### MEDIUM-2 — §Symbol Registry sem 5 símbolos NOVOS introduzidos por W1.T3

**Cite:** `docs/plans/2026-05-texture-compression-waves.md:46-74` — tabela canônica de 22 símbolos com regra explícita no §75:
> "ao implementar W1+, antes de citar qualquer símbolo em código novo, re-verificar via `grep`/`cat`. Símbolos podem ter materializado entre sessões"

W1.T3 introduziu 5 símbolos públicos verificáveis via `grep -n "pub " tools/asset-cooker/src/texture/`:
1. `texture::cook::cook` — função pública (lib API)
2. `texture::cook::CookOptions` struct
3. `texture::cook::TextureCookError` enum (4 variants)
4. `texture::target_matrix::Tier` enum (5 variants — shadow de `ph2d_asset::TierIndex` vapor)
5. `texture::target_matrix::AssetClass` enum (4 variants)
6. `texture::target_matrix::target_for(tier, class) -> Option<TargetFormat>` função

Nenhum desses 6 aparece na tabela §Symbol Registry. Linha 55-56 ainda registra `TierIndex` como "**W1.T4 cria**" sem nota de que `texture::target_matrix::Tier` já foi materializado como newtype shadow no escopo do cooker. Próxima sessão fazendo W1.T4 (ph2d-asset TierIndex real) **pode duplicar/conflitar** sem entender o relacionamento alias.

**Classification:** MEDIUM doc consistency. Registry foi vendido como single-source-of-truth no §75 com regra grep+update. W1.T3 violou o write-half do contrato (read-half foi respeitado — commit reusa `Tier` em vez de inventar `DeviceTier` vapor).

**Fix:** appendar 6 linhas à tabela §Symbol Registry com cite `tools/asset-cooker/src/texture/{mod,cook,target_matrix}.rs` + `W1.T3 ✓` em "Wave responsável".

---

### LOW-1 — `cook_is_deterministic_for_same_input_same_cpu` rodaria em CI multi-arch matrix sem canonical-runner D2

**Cite:** `tools/asset-cooker/src/texture/cook.rs:204-215` test + `tools/asset-cooker/src/texture/mod.rs:23-25` declara D2 "⏳ aguardam W1.T10".

**Análise:** test em si é robusto (compara 2 cooks **na mesma máquina** — only relies em single-CPU stability, não cross-CPU). Audit consolidado HIGH#A1 (ISA dispatch runtime astcenc) afetaria cross-machine snapshot tests (D4) mas **NÃO** este test. Test name explicitly diz `same_cpu`. Test **passa corretamente** em qualquer runner — não há false negative.

**Risco residual:** `.github/workflows/spike.yml` matrix `[ubuntu-latest, macos-latest, windows-latest]` rodará este test em 3 CPU classes diferentes (cada uma compara internamente, sempre passa). Isso é correto. Nenhum drift. Apenas vale documentar para o futuro implementer de D4 (W1.T2.3) que o assertion shape de D4 será diferente (compara contra snapshot file) e exige D2 antes.

**Classification:** LOW — nota de continuidade, não bug.

**Fix:** opcional — adicionar 1-linha comment no test apontando que D4 NÃO pode reusar este shape.

---

## Disciplinas operacionais — re-verificação pós-W1.T3

### D1 — `ctt` features pinadas (arch-gate `architecture_ctt_features_pinned`)

**Status: PASS** ✓

Verificado via `cargo test -p ph2d-asset-cooker --tests --no-run`: o test binary `architecture_ctt_features_pinned-50fd739259be07a7` compilou sem warning. Re-leitura de `tools/asset-cooker/Cargo.toml:48-54`:

```toml
ctt = { version = "0.4.0", default-features = false, features = [
    "encoder-bc7enc", "encoder-astcenc", "encoder-etcpak",
    "encoder-intel", "ispc-prebuilt",
] }
```

- `default-features = false` ✓
- `REQUIRED_FEATURES` set match ✓
- `encoder-amd` ausente (D3 enforcement) ✓
- W1.T3 adicionou `image = "0.25"` mas **NÃO** tocou bloco `ctt = {...}` — arch-gate isolated de `image` dep.
- Outro agente paralelo adicionou `[package.metadata.cargo-machete] ignored=["ctt"]` (linhas 67-69) — arch-gate parser usa `toml` crate properly, lê apenas `[dependencies].ctt` table, **não é afetado** por metadata.cargo-machete table irrelevante.

Comprovação cross-grep: `grep -rn "ctt-compressonator\|encoder-amd" Cargo.lock` retorna ZERO matches (commit `e254271` build validation continua válido pós-W1.T3).

### D3 — Compressonator BC7+UltraFast bug evitado

**Status: PASS** ✓ via duas camadas:

1. **Build-level**: `encoder-amd` feature ausente → variant `Encoder::Amd` nem está compilado (zero runtime exposure).
2. **Source-level**: `target_matrix.rs:191-208` test `bc7_paths_never_dispatch_via_auto` ITERA todos `Tier × AssetClass` (4×4 = 16 combinações) e assert que toda combinação retornando `Format::BC7_UNORM_BLOCK` dispatcha via `Encoder::Bc7enc`. Inspeção manual confirma 2 BC7 paths reais: `(Desktop, SpriteColor)` e `(Desktop, CriticalUi)` ambos em `bc7enc_encoder()` (linhas 72-73). Test NÃO é vacuous — ele realmente itera matrix e asserts a condição.

**Caveat documentado**: invariante depende de `Encoder::Auto` NUNCA aparecer em row BC7. Atual matrix tem `Encoder::Auto` só em `(Constrained, _)` que retorna `R8G8B8A8_UNORM` (não-BC). W2.T2.2 wrapper guard (defer) continua válido como defense-in-depth se `encoder-amd` voltar.

### D5 — `deny.toml` bans intactos

**Status: PASS** ✓

`deny.toml:120-131` continua com 5 entries:
- `basis-universal` · `basis-universal-sys` · `basisu_c_sys` · `nvtt_rs` · `nvtt_sys`

W1.T3 não tocou `deny.toml`. `grep "name = \"basis|name = \"nvtt|name = \"compressonator" Cargo.lock` retorna ZERO matches → bans não-violados.

### D2 + D4 — corretamente flagged como pendentes

**Status: aguardam W1.T10/W1.T11.5** — declarado em `tools/asset-cooker/src/texture/mod.rs:23-25` + `cook.rs:13-15` + audit consolidado §D2/D4. Snapshot file (audit original sugeria "primeiros 64 bytes per (format, encoder, quality)") **NÃO** pode ser escrito ainda mesmo para 1 combo — sem canonical runner CI workflow materializado, snapshot capturado localmente diverge cross-machine. Owner = W1.T10 (CI workflow) + W1.T11.5 (Git LFS) + W1.T2.3 (test fixture).

**Comentário:** §Open Issues do plano vivo NÃO lista D4 owner explicitamente — `W1.T2.3` é citado em §Wave 1 Batch A linha 182. Próximo agente saberá via plano vivo. Sem drift.

---

## Cross-doc consistency — outras verificações

- **ADR-0055-v4 §4** "tools/asset-cooker/ ganha sub-command `texture cook` + lib API" ✓ confere com `tools/asset-cooker/src/texture/{mod,cook,target_matrix}.rs` + `main.rs` `TextureCmd::Cook { input, output, --tier, --asset-class }`.
- **ADR-0055-v4 §4** "lib API consumível pelo Painter Export Cooked Texture" ✓ — `pub fn cook(source_bytes: &[u8], options: CookOptions) -> Result<Vec<u8>, _>` é o que ADR prometeu.
- **Audit consolidado §D1** linha 39 mentions `format-ktx2` feature, lista exata "em W1.T3 após inventory". W1.T3 NÃO ativou `format-ktx2` feature (não consta no Cargo.toml). Re-verificação: `cargo info ctt --version 0.4.0` mostraria features list real. **Caveat consciente**: ctt v0.4.0 pode emitir KTX2 via `Container::ktx2()` (cook.rs:141) sem precisar de feature flag — `Container::ktx2()` é symbol, não feature gate. Audit consolidado linha 39 foi prescrição pré-inventory, descartada pós-T2.1 quando feature inventory real revelou 5 features (encoder-* + ispc-prebuilt). Nenhum drift real — apenas obsolescência textual em audit.
- **Anti-pattern #12** (snippets em ADR) ✓ — ADR-0055-v4 continua 101 LOC strategic-only, zero `pub fn`.
- **Anti-pattern #14** (defer abuso) ✓ — W1.T3 commit message lista 3 defers (T2.3, W1.T4, W1.T6) todos **adjacentes** (T2.3 bloqueado por D2 vapor; W1.T4 = ph2d-asset variant, próximo sub-task; W1.T6 = multi-tier orchestration, depende de T4). Defers legítimos.

---

## Sessão hygiene

- ✓ 8 commits da sessão respeitaram escopo. W1.T3 commit toca apenas `tools/asset-cooker/{Cargo.toml, src/lib.rs, src/main.rs, src/texture/*}`.
- ✓ W1.T3 commit message reconhece `[package.metadata.cargo-machete] ignored=["ctt"]` adicionado por outro agente paralelo (sem reverter).
- ✓ `Cargo.lock M` continua não-commitado (decisão consistente desde W1.T0).
- ✓ `image = "0.25"` adicionada com `features = ["png"]` apenas (PR description W1.T3 documenta: JPEG/WebP em W4+).

---

## Resumo executivo

W1.T3 entregou padrão-ouro técnico: disciplinas D1/D3/D5 respeitadas, anti-patterns 12/14 evitados, código compila, tests passam, escopo respeitado, defers legítimos. **Single drift class**: doc-side não acompanhou code-side em 2 sites (plano §W1.T3 row + §Symbol Registry + HANDOFF §12 stale). Corrigível em 1 commit doc-only (~30 LOC); não bloqueia W1.T4.

**Recomendação:** APPROVE com fix MEDIUM-1+MEDIUM-2 antes de abrir W1.T4 (para próximo agente ler estado consistente).

---

## Memórias relacionadas

- [[feedback-audit-internal-state-grep]] — re-verificação de internal state foi feita
- [[feedback-perfection-no-deferrals]] — escopo decisão-atual vs adjacente respeitado em W1.T3
- [[no-industrial-claims-without-verification]] — N/A (W1.T3 é code, não doc)
- [[feedback-audit-lens-diversity]] — esta é Lente Δ (delta arch-as-code) após Lentes A/B/α/β
