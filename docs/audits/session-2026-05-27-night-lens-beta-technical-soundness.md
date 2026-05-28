# Session 2026-05-27 (night) — Lens Beta · Technical Soundness Audit

**Scope:** auditor adversarial 3rd-pass sobre o trabalho desta sessão (W0+W1.T0+T1+T2+T2.1+T2.4 do plano KTX2 Fase 2, ADR-0055-v4). Verifica (a) o audit `ctt 0.4.0` de 2 lentes paralelas, (b) os 3 artefatos resultantes (Cargo.toml D1, arch-gate D1, deny.toml D5), (c) se os mitigations escolhidos cobrem o que dizem cobrir.

**Tempo:** ~40min wall-clock. Read-only sobre código produção; só escreveu este markdown.

**Veredito:** **APPROVE — score 8.5/10**. Decisão de adotar `ctt 0.4.0` continua sólida. Os 3 artefatos cumprem D1+D5 funcionalmente. **2 findings HIGH genuínos** sobre fragilidade do arch-gate e cobertura incompleta do `deny.toml` ban; ambos exigem fix barato (≤30min) **antes** de W1.T3 ou ficam débitos invisíveis. Nada CRITICAL muda decisão estratégica.

---

## Findings

### HIGH-1 · `deny.toml` ban cobre nomes incompletos — bypass trivial pelo upstream

**Onde:** `/Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/deny.toml:120-124`

`deny.list` atual banem `basis-universal-sys`, `nvtt`, `nvtt-sys`. `cargo search` (executado nesta auditoria) revela que crates.io expõe AO MENOS estas crates de mesma família:

- `basis-universal` v0.3.1 (high-level wrapper Binomial) — **NÃO banido**
- `basis-universal-sys` v0.3.1 (bindings) — banido ✅
- `basisu_c_sys` v0.7.1 (alternativa C bindings) — **NÃO banido**
- `bevy_basisu_loader_sys` v0.4.4 (wrapper Bevy) — **NÃO banido**
- `nvtt_rs` v0.10.1 (wrapper alto-nível, underscore) — **NÃO banido**
- `nvtt_sys` v0.5.2 (bindings, underscore) — **NÃO banido**; ban atual `nvtt-sys` (hífen) **não casa** o nome real no Index.

**Por que importa para PH2D:** ctt upstream issue #68 fala "integrate with basisu" sem especificar crate-name. Se ctt 0.4.x → 0.5 adicionar feature `encoder-basisu`, a transitive dep pode resolver para `basis-universal` (nome puro, sem `-sys`) ou `basisu_c_sys`, e o `cargo deny check` passa silencioso. Idem para NVTT: ctt poderia depender de `nvtt_rs` (que existe HOJE) e o ban com hífen `nvtt-sys` **não é o nome correto** no crates.io.

**Mitigação (30 min):** ampliar `[bans].deny`:

```toml
deny = [
    { name = "basis-universal" },
    { name = "basis-universal-sys" },
    { name = "basisu_c_sys" },
    { name = "bevy_basisu_loader" },
    { name = "bevy_basisu_loader_sys" },
    { name = "bevy_basisu_saver" },
    { name = "nvtt_rs" },
    { name = "nvtt_sys" },
]
```

Esta sessão escreveu nomes plausíveis sem `cargo search` — exatamente o anti-pattern do feedback [`no-industrial-claims-without-verification`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_no_industrial_claims_without_verification.md) aplicado a INTERNAL state (memory `audit_internal_state_grep`).

---

### HIGH-2 · Arch-gate `architecture_ctt_features_pinned` quebra em 2 formatos TOML válidos

**Onde:** `/Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/tools/asset-cooker/tests/architecture_ctt_features_pinned.rs:31-37`

Testado empiricamente nesta auditoria com `rustc /tmp/test_archgate.rs`. O parser manual `cargo_toml.find("\nctt = {")` + `[start..].find("] }")` falha nestes 2 casos válidos de TOML:

| Caso | Status do teste |
|---|---|
| Dep object inline com features inline ou multi-line array `[...] }` | ✅ PASS |
| Dep object multi-line `ctt = {\n  version = ...,\n  default-features = false,\n  features = ["a"],\n}` (fecha com `\n}` em vez de `] }`) | ❌ FAIL hard (`expect("ctt dep block must close with `] }`")`) |
| Dep object multi-line + features array multi-line `features = [\n  "a"\n]\n}` (fecha com `]\n}`) | ❌ FAIL hard |

**Por que importa para PH2D:** algum `cargo edit` futuro, IDE auto-format, ou edição manual humana pode reformatar a entry para o estilo multi-line (estilo idiomático para listas longas). O teste **panica com `expect`**, não emite assert legível, e o reviewer pode `--no-verify` o commit por "pre-commit quebrou inexplicavelmente". Pior: se alguém **remover deliberadamente** `default-features = false` e reformatar para multi-line, o gate primeiro panica (não falha por motivo correto), ofuscando a regressão real.

**Mitigação (15 min):** trocar parser manual por crate `toml`:

```rust
let manifest: toml::Value = toml::from_str(&cargo_toml).expect("parse Cargo.toml");
let dep = &manifest["dependencies"]["ctt"];
assert_eq!(dep["default-features"].as_bool(), Some(false), ...);
let features: Vec<&str> = dep["features"].as_array().unwrap().iter().map(|v| v.as_str().unwrap()).collect();
for req in REQUIRED_FEATURES { assert!(features.contains(req), ...) }
for fb in FORBIDDEN_FEATURES { assert!(!features.contains(fb), ...) }
```

`toml = "0.8"` já está no workspace (verificável via `cargo tree`). Custo: substituir ~10 LOC. Ganho: robust to TOML re-formatting; falha por motivo correto; comment no test pode dizer "Cargo.toml schema-validated, não string-grep".

---

### HIGH-3 · D2 (canonical runner) está PROMETIDO em ADR mas NÃO existe no `.github/workflows/`

**Onde:** `/Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/.github/workflows/spike.yml` + `/Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/docs/plans/2026-05-texture-compression-waves.md:198` (W1.T10 still pendente)

O consolidated audit (linha 47) afirma "ADR-0055-v4 §2.3 já decidiu: cook em GitHub Actions ubuntu-latest único" — apresentado como mitigação atendida. Mas:

1. `spike.yml` **NÃO tem job `cook`** algum. Test step (linha 257-291) roda `nextest run -p ph2d-asset-cooker` no **matrix completo** (`ubuntu-latest`, `macos-latest`, `windows-latest`). Hoje os testes do cooker são leves (não invocam encoder), então não há problema imediato. Mas no momento que W1.T3 plug em `ctt::Cooker` em qualquer test, o output bytes vai divergir entre macOS ARM (NEON) e Linux x86_64 (AVX2) — e qualquer snapshot test (D4 planejado) vai flapar randomly entre OSes do matrix.
2. `/assets/cooked/` **não existe** + `/.gitattributes` **não existe** (verificado). LFS pipeline = vapor ainda.
3. W1.T10 e W1.T11.5 do plano dizem "novo workflow step + Git LFS setup pendente".

**Por que importa para PH2D:** o consolidated audit declara D2 "MITIGATED por ADR-0055-v4 §2.3". Auditor adversarial classifica esse status como **textual/intencional, não materializado**. Para a coluna "implementado nesta sessão" do tracker, D2 deve estar como `defer` (W1.T10) e NÃO como `decidido pelo ADR`. Esta confusão "decided ≡ implemented" é o que o feedback `pipeline-inject-dont-cap` chama de "promessa vs realização".

**Mitigação:** texto-only, não código. Editar `CONSOLIDATED.md` D2 para dizer "STATUS: defer W1.T10; ADR especifica intent, workflow step ainda não existe; teste de snapshot D4 NÃO pode rodar no matrix multi-OS atual sem D2 primeiro". Reordenar dependency: D4 (snapshot) **bloqueia** em D2 (canonical runner), não só em T3 (wrapper).

---

### MEDIUM-1 · D3 defer adequado SE arch-gate cobrir features list (cobre — confirmado)

**Onde:** `tools/asset-cooker/tests/architecture_ctt_features_pinned.rs:24,53-61`

O defer de D3 (wrapper guard ban Compressonator+UltraFast+BC7) está justificado pelo argumento "`encoder-amd` ausente da feature list elimina `ctt-compressonator` do dep graph". Verifiquei via `cargo tree -p ph2d-asset-cooker --depth=4 | grep ctt` (executado nesta auditoria): `ctt-compressonator` **NÃO aparece** na árvore. O encoder está fisicamente removido do binário. ✅

E o arch-gate `FORBIDDEN_FEATURES = &["encoder-amd"]` (linha 24) garante que se um agente futuro re-adicionar a feature por engano, o pre-commit/CI gate falha imediato. ✅

**Veredito:** defer de D3 é defensável **dado que** o arch-gate fica robusto (ver HIGH-2). Se HIGH-2 não for fixado, agente futuro com TOML auto-format pode bypassar o gate inadvertidamente.

---

### MEDIUM-2 · D4 (snapshot test) poderia rodar AGORA sem wrapper PH2D

**Onde:** plano vivo linha 182, `texture-compression-waves.md`

D4 está deferido com motivo "aguarda T3 — precisa de wrapper code que invoca ctt::Cooker antes". Mas isso é defer cosmético: o snapshot test pode invocar `ctt::processing::encode_all` direto (API pública do `ctt 0.4.0`) com 1 fixture 64×64 RGBA8 in-memory + assert primeiros 64 bytes hashed. ~80 LOC standalone, zero dep em código PH2D ainda inexistente.

**Por que importa:** o valor de D4 é detectar drift cross-version DO PRÓPRIO `ctt` (e.g., `ctt 0.4.0 → 0.4.1` muda dispatch silenciosamente). Esse drift pode aparecer ANTES de W1.T3 estar pronto. Deferir D4 para depois de T3 perde a janela onde D4 é mais valioso.

**Mitigação:** considerar adiantar versão minimal de D4 como W1.T2.6 (paralelo a T3). Não-blocker; só otimização.

---

### MEDIUM-3 · Lente A MEDIUM-2 (silent unwrap_or fallback) merecia mais peso

**Onde:** confirmado em `~/.cargo/registry/src/.../ctt-0.4.0/src/processing/encode.rs:59-60`

```rust
let bpp_block = step.target_format.bytes_per_block().unwrap_or(16) as u32;
let (bw, _bh) = step.target_format.block_size().unwrap_or((4, 4));
```

Lente A classificou como MEDIUM ("spec gap, não active bug; futuro Vulkan format addition could regress silently"). Re-li o ctt source: o `unwrap_or(16)` + `unwrap_or((4,4))` produz uma `Surface` com `stride = blocks_x * 16` mesmo quando o formato real é BC1 (8 bytes/block). Se isso for usado num path KTX2 emit, o `keyValueData` ou byte layout fica corrompido por fator 2 — não "regressão futura", **bug latente HOJE para qualquer formato fora da lista hardcoded de `vk_format.rs`**.

Mas: as features que PH2D habilitou (`encoder-bc7enc, encoder-astcenc, encoder-etcpak, encoder-intel`) cobrem **só formatos onde `vk_format.rs` resolve `Some(...)`**, então o fallback é unreachable na superfície PH2D. Lente A está correta em classificar MEDIUM. ✅

**Sem ação requerida** desta sessão. Tracker para W2 quando D4 snapshot test for materializado: incluir um round-trip per formato suportado para detectar qualquer silent stride miscomputation (já listado em plano linha 60-63).

---

### LOW-1 · License `ctt-etcpak` confere com tabela Lente B

**Spot-check:** Lente B tabela linha 25 declara `ctt-etcpak` license = `(MIT OR Apache-2.0 OR Zlib) AND BSD-3-Clause`. Verificado em `~/.cargo/registry/src/.../ctt-etcpak-0.4.0/Cargo.toml`:

```
license = "(MIT OR Apache-2.0 OR Zlib) AND BSD-3-Clause"
```

Match exato. ✅ Spot-check passou; alta confiança nas outras 7 linhas.

---

### LOW-2 · MEDIUM/LOW Lente A — preenchimento vs genuíno

Re-leitura da Lente A:

| Finding | Genuíno? | Comentário |
|---|---|---|
| MEDIUM-1 (`*const → *mut` cast em astcenc wrapper) | Genuíno | Soundness Rust real; mitigação "single-threaded cook" alinha com design intent PH2D |
| MEDIUM-2 (encode.rs silent fallback) | Genuíno (ver MEDIUM-3 desta lente) | merece mais peso que recebeu |
| LOW-1 (`tight_data()` panics on unknown format) | Genuíno-baixo | gatekeeper claim correto na prática |
| LOW-2 (`unpremultiply_f32` uncapped) | Genuíno-baixo | doc-only, defensável |

Nenhum é preenchimento de relatório. Score 7.5/10 da Lente A é honesto, não inflado.

---

## CRITICAL findings (procurados, 0 achados)

Re-leitura focada de `encode.rs` (referenciado no prompt) procurando off-by-one / padding bugs missed by both lenses:

- Buffer math `blocks_x * bpp_block` (linha 61, 68) usa `div_ceil` para `blocks_x`, conserva right semântica ✅
- `compress_with` (linha 178-214) é dispatch puro, sem aritmética ✅
- Iterator `for (layer_idx, layer) in image.surfaces.iter().enumerate()` é o pipeline expected ✅
- Não há `unsafe` block, `unwrap()` sem context, `assert!` debug-only, `let _ = ...` suspeito

**0 CRITICAL achados que mudem o veredito de APPROVE.** Lente A está correta em "0 CRITICAL".

---

## Score 8.5/10 — APPROVE

**Por que 8.5 e não 9-10:**

- HIGH-1 (deny.toml nomes incompletos) e HIGH-2 (arch-gate fragil) são reais e corrigíveis em ≤45 min combinados; se não forem fixados antes de W1.T3, viram débitos silenciosos.
- HIGH-3 (D2 vapor textual) é cosmético do CONSOLIDATED.md, mas reflete um padrão recorrente "decidido ≡ implementado" que merece fix.

**Por que APPROVE e não BLOCK:**

- ctt source é estruturalmente sólido (Lente A confirmada: 0 CRITICAL); o defer de D3 é robusto pelo arch-gate; D1 funciona; D5 funciona parcialmente (cobre 3 dos ~8 nomes de bypass possíveis, mas o cenário de bypass requer ctt upstream futuro adicionar feature explícita — risco contido).
- Nenhum finding muda decisão estratégica.

**Recomendação de continuação:** aplicar fix HIGH-1 + HIGH-2 (≤45 min combinados) na próxima sessão; reclassificar D2 status para `defer W1.T10` no CONSOLIDATED.md; opcional adiantar D4 minimal sem wrapper. Após isso, W1.T3 desbloqueado com confiança 9.5/10.

---

**Auditor:** Claude Opus 4.7 — lens beta (technical soundness 3rd-pass).
**Cite paths absolutos:**

- `/Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/deny.toml:120-124`
- `/Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/tools/asset-cooker/tests/architecture_ctt_features_pinned.rs:31-37`
- `/Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/.github/workflows/spike.yml:140-191`
- `/Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/docs/audits/ctt-source-audit-2026-05-27-CONSOLIDATED.md:47`
- `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ctt-0.4.0/src/processing/encode.rs:59-60`
- `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ctt-etcpak-0.4.0/Cargo.toml:license`
