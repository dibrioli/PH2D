# Handoff + Loop de operação autônoma — Sprite Inspector v2

**Data:** 2026-05-28 (W0 RATIFIED)
**Para:** a LLM que vai construir o Sprite Inspector v2 fase-a-fase, em loop, enquanto o Enio está fora.
**Como usar:** §0 e §1 são INSTRUÇÕES que governam seu comportamento. §2+ é referência. Leia tudo + a spec ratificada ([`docs/Sprite_projeto/`](Sprite_projeto/)) antes de tocar em código.

---

## 0. MANDATO (lê isto primeiro — governa tudo)

> **Padrão-ouro. Puro-sangue. O definitivo. Sem economias, sem gambiarras.**

- Construa **a melhor versão que existe**, não um "v1 que dá pro gasto". Se a forma definitiva é viável agora, é ela que você faz.
- **Proibido:** corner-cut disfarçado de v1; `unwrap`/falha silenciosa onde cabe `Result`/erro real; assumir paridade/correção sem prova; `TODO: depois` em coisa que dá pra fazer certo agora; copiar-colar com gambiarra em vez de extrair a abstração certa.
- **Determinismo (HR-5)** respeitado onde aplica; documentado onde é isento. **Contratos minúsculos e gateados.** Toda superfície pública documentada. Testes cobrem feliz + edge + a classe-de-bug.
- O Enio confia em você. A barra é "melhor que Godot/Unity/Unreal/Paper2D/Defold/GameMaker/Construct/Phaser/Aseprite combinados na seara do Inspector do Sprite". Gambiarra é "obrigado, mas não".

---

## 1. O LOOP (siga fase a fase, sem parar até precisar de smoke)

Para CADA task do plano ([`docs/Sprite_projeto/15_plano_de_implementacao.md`](Sprite_projeto/15_plano_de_implementacao.md)):

1. **Escolha a próxima task** do plano. Atualize o todo list.
2. **Build isolado:** sempre `CARGO_TARGET_DIR="$PWD/target/slot-coord-sprite" cargo ...` (não contende no lock do `target/`).
3. **Implemente no padrão-ouro** (§0). Cite o princípio no código quando ajudar a próxima LLM.
4. **Auto-verifique, tudo verde:** `cargo test -p <crate>` + `cargo clippy -p <crate> --all-targets -- -D warnings` + `cargo fmt -p <crate> -- --check`.
5. **AUDITE — adversarial e independente.** Lance **≥2 auditores em paralelo** com lentes rotacionadas (memory [`feedback-audit-lens-diversity`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_audit_lens_diversity.md)). 5 lentes canônicas: A escopo · B ABI/grep · C determinism/HR-5 · D UX/a11y/i18n · E security/perf/test-coverage. Instrua-os a serem **duros, caçar bugs/lacunas, dar severidade, NÃO validar por cortesia**.
6. **CORRIJA TODOS os achados** (Crítico→Baixo). **Nada adiado** — exceto follow-ups genuinamente não-bloqueantes em §9.
7. **RE-AUDITE até erro zero.**
8. **Commit** (`git commit --no-verify` em background; local; commit limpo por task). **Stage explícito com paths específicos** (`git add -- <my-paths>`) para fence contra reset de outros agentes (memory [`feedback-destructive-reset-collision`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_destructive_reset_collision_2026_05_28.md)).
9. **Próxima task.** Volte ao 1.

### Quando PARAR (e só então)
- A task precisa de **smoke visual** (`./play.command`) — vide fixtures canônicas em [`docs/Sprite_projeto/15_plano_de_implementacao.md §15.8.2`](Sprite_projeto/15_plano_de_implementacao.md).
- Mudança em **foundational fora do escopo** (Painter, Vector, asset cooker) — escala pro Enio.
- Ao parar: relatório curto pro Enio — o que ficou pronto, o que ele precisa olhar.

### NÃO faça autonomamente
- **`git push` / CI** — é o "ship" de fim-de-jornada, sob ordem do Enio. Acumule commits locais.
- Tocar `crates/ph2d-painter-stroke/`, `crates/ph2d-host/`, `crates/ph2d-asset/`, `tools/asset-cooker/` sem necessidade clara (pastas Coord-A; verifique [`docs/SESSION_ACTIVE.md`](SESSION_ACTIVE.md) antes).
- Tocar contratos congelados sem amendment ADR ([ADR-0039 nodegraph](architecture/decisions/0039-nodegraph-contract-freeze-w2t4.md), [ADR-0040 tool](architecture/decisions/0040-tool-as-isolated-feature-crate.md)).

---

## 1.5 Auditoria de continuação — briefing canônico para o próximo agente

> **Quando rodar:** ANTES da primeira modificação de código quando você assume a sessão (independente de quem você é — outra LLM, mesma LLM em contexto novo, ou continuação humana). A sessão anterior shippou 4 commits locais; VERIFIQUE antes de construir em cima.

### O que verificar (4 commits)

| Commit | Tarefa | Claim a verificar | Como verificar |
|---|---|---|---|
| `cef1959` | W0.T0.12 + T0.13 | "21 tests em sprite_versioned_postcard + 5 fixtures 35B + 1 ADR amendment" | `cargo test -p ph2d-render --test sprite_versioned_postcard` + `ls -l crates/ph2d-render/tests/fixtures/*.postcard` + `ls docs/architecture/decisions/0070-amendment-2.md` |
| `e3ad19f` | W0 R3 follow-up | "ADR renamed -1→-2 + DX panic msgs + postcard exact-pin gate" | `git log --oneline | grep amendment-2` + `cargo test -p ph2d-render --test sprite_versioned_postcard postcard_exact_version_pin_enforced_in_cargo_toml` |
| `5974a84` | T1.3.5 libm sweep v1 (incompleto) | "9 sites swept" — **CLAIM FALSA, foi corrigida em `f9850bf`** | `git show 5974a84 --stat` |
| `f9850bf` | T1.3.5 R2 follow-up | "16 missed split sin/cos sites + libm pin gate" | `cargo test -p ph2d-ecs --test transform_determinism` (4 tests verde incluindo `libm_exact_version_pin_enforced_in_workspace`) |

### Lentes obrigatórias (mínimo 2 em paralelo, lens-rotation discipline)

A sessão anterior cobriu: B+C (R1 T0.12+T0.13), E+A (R2 T0.12+T0.13), D+meta (R3 W0), B+C (R1 T1.3.5), E+A+meta (R2 T1.3.5). Para auditoria de continuação, **as 2 lentes mínimas são B (com grep AMPLIADO) + C (executar os arch-gates)** — isso pega as 2 classes de erro que vazaram entre R1 e R2 do T1.3.5.

```bash
# Lens B — grep ampliado workspace-wide
grep -rn '\.\(sin\|cos\|tan\|atan2\|sqrt\|exp\|pow\)()' \
  crates/ tools/ shells/ 2>/dev/null

# Expected: only paint-overlay (gizmo/paint.rs), debug oscillator
# (render_loop/mod.rs:278-280), color animation, sprite_merge.rs
# (foreign WIP — NÃO swept per anti-collision discipline).
# Zero hits em paths transform-write.

# Lens C — todos os arch-gates de determinismo
CARGO_TARGET_DIR="$PWD/target/slot-coord-sprite" \
  cargo test -p ph2d-ecs --test transform_determinism
CARGO_TARGET_DIR="$PWD/target/slot-coord-sprite" \
  cargo test -p ph2d-render --tests
```

### Memory canônica a consultar antes do audit

- [`feedback-determinism-sweep-grep-all-transcendentals`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_determinism_sweep_grep_all_transcendentals.md) — por que o grep R1 do `5974a84` falhou.
- [`feedback-exact-pin-needs-substring-gate`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_exact_pin_needs_substring_gate.md) — disciplina de pinagem.
- [`feedback-audit-commit-msg-claim-verification`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_audit_commit_msg_claim_verification.md) — re-verificar claims numéricos contra grep ampliado.
- [`feedback-audit-lens-diversity`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_audit_lens_diversity.md) — rotação de lentes (B/C/D/E/A + meta).
- [`feedback-audit-scope-discipline`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_audit_scope_discipline.md) — bug em crate adjacent → handoff/issue pro owner, NÃO fixo.

### Saída esperada

Relatório markdown estruturado por severity:

```
## Audit-Zero Continuation Report (2026-MM-DD)
### Methodology
Lenses run: <list>. Total findings: N CRIT + M HIGH + K MED + L LOW.

### Verification of prior session claims
- cef1959: <PASS|FAIL com detalhes>
- e3ad19f: <PASS|FAIL>
- 5974a84: <PASS|FAIL — esperado HISTORICAL via superseded por f9850bf>
- f9850bf: <PASS|FAIL>

### New findings (if any)
[SEVERITY] <title>
File: <path>:<line>
Issue: <para>
Fix: <concrete>

### GO / NO-GO decision for T1.1
- GO se zero ≥HIGH e claims verificáveis.
- NO-GO + fix-up commit antes de T1.1 caso contrário.
```

### Após GO

Comece T1.1 (Sprite struct v3→v4 expansion) seguindo §2 PONTO DE ENTRADA. **Não comece em outra task** sem completar T1.1 — schema bump é foundational; T1.2..T1.14 dependem.

---

## 2. Estado atual (TL;DR)

**W0 RATIFICADA 2026-05-28.** 7 ADRs Accepted (6 novos 0069..0074 + ADR-0025-amendment-1 Skew) pós **5 lentes adversariais rotacionadas** (147 findings; 31 CRITICALs únicos fechados a erro-zero, sem deferral).

Spec normativa: [`docs/Sprite_projeto/`](Sprite_projeto/) (17 arquivos: README + 14 sections + `16_i18n_catalog.md` novo).

> ### ⏯ PONTO DE ENTRADA (próximo agente — comece AQUI)
>
> **Status 2026-05-28 noite:** W0 carry-over (T0.12 + T0.13) + T1.3.5 (libm cross-OS sweep) **entregues em 4 commits locais não-pushados:** `cef1959` + `e3ad19f` + `5974a84` + `f9850bf`. Próxima task canônica = **T1.1** (`Sprite` struct expansion v3→v4 com 14 novos campos). Você está em **Coord-A foundational mode**. Releia §0 (mandato) e §1 (loop) antes de qualquer código.
>
> ### 🛑 PRIMEIRO PASSO OBRIGATÓRIO — auditoria de continuação fresca
>
> **NÃO comece T1.1 antes de fechar a auditoria abaixo.** A sessão anterior shipou 4 commits com claim "audit-zero pós 4 rounds × 8 lentes (B+C+E+A+D+meta+E+A+meta) + 69 findings closed". Você é a verificação independente. A sessão anterior MENTIU sem querer no R1 do `5974a84` ("9 sites swept" — havia 25 sites; descoberto em R2) — assuma postura adversarial.
>
> **Lentes obrigatórias (mínimo 2 lentes rotacionadas em paralelo per memory `feedback-audit-lens-diversity`):**
> - **Lens B (ABI/grep)** com GREP AMPLIADO `\.\(sin\|cos\|tan\|atan2\|sqrt\|exp\|pow\)\b` (não só `sin_cos`!) — per memory `feedback-determinism-sweep-grep-all-transcendentals`. Verifique workspace inteiro (crates/ + tools/ + shells/).
> - **Lens C (determinism/HR-5)**: rode `cargo test -p ph2d-ecs --test transform_determinism` e CONFIRME `cross_os_golden_hash_pinned` + `libm_exact_version_pin_enforced_in_workspace` passam.
> - **Lens A (scope/handoff fidelity)**: cross-check claims numéricos do commit body via re-grep (per memory `feedback-audit-commit-msg-claim-verification`). Specificamente: "23 sites swept" — re-grep e conte.
> - **(Opcional ganho de cobertura)** Lens D / E / meta se quiser maior diversidade — sessão anterior já cobriu D no `e3ad19f` e E + meta no `f9850bf`.
>
> **Claims a verificar empiricamente:**
> 1. `crates/ph2d-render/tests/sprite_versioned_postcard.rs` — 22 tests verde (incluindo `postcard_exact_version_pin_enforced_in_cargo_toml`).
> 2. `crates/ph2d-ecs/tests/transform_determinism.rs` — 4 tests verde (incluindo golden hash `d2a3ca34…cf07f` + libm pin gate).
> 3. `grep -rn '\.\(sin\|cos\|tan\)()' crates/ tools/ shells/` retorna apenas paths benignos (paint-overlay, debug oscillator, color animation, sprite_merge.rs foreign WIP). Zero hits em transform-write paths (gizmo/transform.rs, transform.rs, gizmo_drag.rs, input_dispatch.rs, snapshots.rs, sim_populate.rs, algorithm.rs em rasterize, gizmo/tests.rs).
> 4. ADR-0070-amendment-2 existe em `docs/architecture/decisions/0070-amendment-2.md` (NÃO `-1` — slot `-1` reservado pra dual-buffer perf de W1.T1.7b).
> 5. SESSION_ACTIVE Coord-A entry está atualizado refletindo T0.12+T0.13+T1.3.5 done.
> 6. Pre-existing failures (panel-hierarchy LOC + tool-painter compile) seguem documentados em §9.1, NÃO fixados (per `feedback-audit-scope-discipline`).
> 7. Working tree contamination listada em SESSION_ACTIVE.md (foreign WIP `M`/`??` files) NÃO foi staged em nenhum dos 4 commits — re-verifique via `git show <hash> --name-only`.
>
> **Saída da auditoria:** relatório de findings (severity-graded) + decisão GO/NO-GO pra T1.1.
> - **GO** se zero findings ≥ HIGH e os 4 commits batem com as claims.
> - **NO-GO** se algum HIGH/CRITICAL — ship fix-up commit antes de T1.1, OU pause e escala pro Enio.
>
> ### Após auditoria GREEN: próxima task = T1.1 (Sprite struct expansion v3→v4)
>
> Spec exata em [`Sprite_projeto/15_plano_de_implementacao.md`](Sprite_projeto/15_plano_de_implementacao.md) §T1.1 + [`01_anatomia_canonica.md §1.2`](Sprite_projeto/01_anatomia_canonica.md). Sumário:
> - **Editar `crates/ph2d-render/src/sprite.rs`** mudando `Sprite` de 5 → 20 fields (5 v3 + 14 v4 + 1 `version: u32` redundante-mas-Lens-C-M2-aceito).
> - **Novos fields v4** (todos com `#[serde(default = "fn")]` documentário — vide ADR-0070-amendment-2 §3, é DEAD sob postcard mas mantém-se como mirror docs): `self_tint`, `per_corner_tint`, `tint_fill`, `opacity`, `flip_x`, `flip_y`, `centered`, `offset`, `hframes`, `vframes`, `frame`, `region_enabled`, `region_rect`, `region_filter_clip`.
> - **`Sprite::VERSION` bump 3 → 4.**
> - **NÃO mexer no `RenderInstance` ainda** (esse é T1.7a/T1.7b — ABI 144B + 11 vertex attrs).
> - **NÃO escrever o migrator ainda** (W1.T1.6 separado).
> - **Atualizar `SpriteVersioned`** adicionando variant `V4(Sprite)` no fim da declaração (V3 fica em discriminant 0x00; V4 fica em 0x01 — verificado pelo arch-gate `v3_fixtures_start_with_zero_discriminant_byte` existente).
> - **Gates ativos pós-T1.1:**
>   - `architecture_sprite_inspector_surface` (criar em W1.T1.12 final): asserta `Sprite` field count == 20.
>   - `fixtures_match_canonical_serialization` (já existe): continua verde — fixtures v3 frozen.
>   - `spritev3_struct_wire_matches_live_sprite_v3` (já existe): **ESTE GATE VAI FALHAR APÓS T1.1** porque `SpriteV3` ↔ `Sprite` wire diverge pós-bump. Comportamento esperado (gate fala "drift gate during W0→W1 window"). Remover/retire o teste como parte de T1.1 OR convertê-lo em comment + retire o `cargo test`.
>
> ### Outras tasks pendentes do plano W1
>
> - **T1.2..T1.5** — Sprite v4 init defaults + region_filter_clip Atlas branch + Default impl + serde derives + tint_fill cap (24 PainterUiEdit não confundir com Sprite — vide `Sprite_projeto/11_arch_gates_e_caps.md`).
> - **T1.6** — `Sprite::migrate_v3_to_v4(SpriteV3) -> Sprite` + `crate::sprite_versioned::load_sprite(&[u8]) -> Result<Sprite, LoadError>`. Stub canonical já está em `crates/ph2d-render/tests/migrate_sprite_v3_to_v4.rs` (`#[ignore]`d com signature contract). Un-ignore + replace body com per-fixture assertions per spec §10.6.
> - **T1.7a** — ABI v4: `RenderInstance` field count 12, size 144 bytes, 11 vertex attrs (per `Sprite_projeto/10_schema_versionamento.md §10.5`). Gate `vertex_attr_offsets_match_struct` em `crates/ph2d-render/src/sprite.rs:343-375` existente — expandir.
> - **T1.7b** — Criterion bench `sprites_upload_144b_vs_72b` 10k sprites @ 60Hz tier M-series. Trigger condition pra dual-buffer mitigation ADR-0070 §2.5 (slot `ADR-0070-amendment-1` JÁ RESERVADO se o bench dispara — DO NOT TOUCH amendment-1 pra outra coisa).
> - **T1.8..T1.14** — gates restantes (NaN reject, scene cap, MemoryBudget, etc.). Detalhe completo em `Sprite_projeto/15_plano_de_implementacao.md`.
>
> ### NUNCA esqueça (carry-over do W0)
>
> - **`SpriteVersioned` wrapper enum** é caminho ÚNICO de back-compat ([ADR-0070-amendment-2](architecture/decisions/0070-amendment-2.md) ratifica). `#[serde(default)]` é documentário/aspiracional (dead sob postcard; vivo sob hypothetical self-describing format swap). NÃO depender do fallback.
> - **`SortedSmallVec` newtype** (ADR-0072 §2.1) enforce key-sorted invariant by construction; sem `push`/`insert_idx` permitidos.
> - **`SpriteAnimator.elapsed_ticks: u64`** fixed-point μs em SimWorld; `speed_scale_q16_16: i32`. Sem f32 accumulator (divergiria via FMA).
> - **NaN/Inf reject** em tint/opacity setters (gate `sprite_tint_finite_rejects_nan_and_inf`). Sem isso = cascade poisons hierarchy + GPU UB.
> - **MCP Destructive Operations Canonical Registry** ([`Sprite_projeto/README.md §7.1.2`](Sprite_projeto/README.md)): 7 destructive ops com HR-11 token obrigatório.
> - **`libm = "=0.2.16", default-features = false`** em 5 crates (ph2d-ecs/editor-core/tool-rasterize/desktop/asset-cooker). Bump = re-capture `EXPECTED_GLOBALS_HASH` em `transform_determinism.rs` + cross-OS CI re-verify + ADR amendment (slot AVAILABLE = `0070-amendment-3` ou mais; `-1` reservado dual-buffer; `-2` já existe T0.13).
> - **postcard `=1.1.3` exact-pin** em ph2d-render. Mesma disciplina.
> - **Working tree contamination** (vide [`SESSION_ACTIVE.md`](SESSION_ACTIVE.md) Coord-A entry): NÃO use `git add -A`; escope paths via `git add -- <path>`.

---

## 3. Mapa de pastas/crates afetados

| Pasta/crate | Papel | Status W1 |
|---|---|---|
| **`docs/Sprite_projeto/`** | spec normativa ratificada (17 arquivos) | READ-ONLY — fonte da verdade |
| **`docs/architecture/decisions/0069..0074-*.md` + `0025-amendment-1.md`** | 7 ADRs Accepted | READ-ONLY |
| **`crates/ph2d-render/src/sprite.rs`** | `Sprite` struct v3 → v4 bump | **MEXE em W1** |
| **`crates/ph2d-render/shaders/sprite.wgsl`** | shader atualizar v4 ABI | **MEXE em W1** (T1.11) |
| **`crates/ph2d-render/tests/`** | gates v4 + migrator + fixtures | **MEXE em W0.T0.12-T0.13 + W1** |
| **`crates/ph2d-ecs/src/transform.rs`** | `Transform::compose` libm sweep | **MEXE em W1.T1.3.5** (pre-amendment skew) |
| **`crates/ph2d-ecs/Cargo.toml`** | adicionar `libm = "0.2"` | **MEXE em W1.T1.3.5** |
| **`crates/ph2d-panel-inspector/`** | Inspector panel — expansão W2+ | NÃO mexer em W1 (só W2+) |
| **`tools/asset-cooker/`** | migrator v3→v4 entry | LEITURA W1; mexe em W1.T1.x (cooker bump) |
| **`crates/ph2d-asset/`** | schema serde | LEITURA W1; ⚠️ Coord-A pasta — verificar SESSION_ACTIVE |
| **`crates/ph2d-host/`** | `MemoryBudget { sprite_inspector_v2 }` | mexe em W1 ao final (declarar budget) — ⚠️ verificar SESSION_ACTIVE |

**Pastas NÃO tocar (Coord-A reserved se ativo):**
- `crates/ph2d-painter-stroke/`, `crates/ph2d-painter-contracts/` — Painter T1.8+
- `crates/ph2d-tool-painter/` — Painter ativo
- `shells/desktop/src/render_loop/painter_bridge.rs` — Painter ativo

Verifique [`docs/SESSION_ACTIVE.md`](SESSION_ACTIVE.md) **antes de cada burst de edição**.

---

## 4. Caminho canônico W0 → W1 → W8

```
W0 ✅ RATIFICADA 2026-05-28 (este handoff abre aqui)
   ↓ T0.12 fixtures v3 binárias (carry-over)
   ↓ T0.13 empirical postcard test (carry-over)
   ↓
W1 schema bump strategic-only
   ↓ T1.3.5 libm dep + sweep f32::sin_cos
   ↓ Sprite v3 → v4 (14 novos fields)
   ↓ RenderInstance v4 ABI (12 fields, 144 bytes, 11 vertex attrs)
   ↓ Migrator + 5 fixtures verde
   ↓ T1.7a ABI compile + T1.7b criterion bench bandwidth
   ↓ MemoryBudget declarado
   ↓
W2 Inspector seções 1-6 + OKLCH (estender BlenderColorPicker) + BulkSelect primitivo
   ↓
W3 Seções 7-9 + 7 Components ECS + ClipChildren regression + sorting fixture
   ↓
W4 Seções 10-11 + SpriteAnimator fixed-point
   ↓
W5 Seção 12 NamedAnchors + SortedSmallVec + validate + CameraFollowAnchor
   ↓
W6 Foundational widgets refinement (Rect2Editor + VariantEditor)
   ↓
W7 Polish + i18n (~155 keys) + a11y + bug bash
   ↓
W8 ⏳ Asset Cooker Integration (Aseprite full + Linked Cels + PSD) — wave separada
```

Detalhe completo em [`Sprite_projeto/15_plano_de_implementacao.md`](Sprite_projeto/15_plano_de_implementacao.md).

---

## 5. Pré-requisitos W1 (em ordem)

1. **Confira SESSION_ACTIVE** — se Coord-A tem `crates/ph2d-render/` ou `crates/ph2d-ecs/` reservado, pause e renegocie.
2. **T0.12 fixtures v3 binárias** (ANTES do bump):
   ```rust
   // crates/ph2d-render/tests/generate_v3_fixtures.rs (bin one-shot)
   let atlas = SpriteV3 { source: Atlas { key: 0 }, size: [10.0; 2], tint: [1.0; 4], anchor: [0.0; 2], premultiplied: false };
   let bytes = postcard::to_allocvec(&SpriteVersioned::V3(atlas)).unwrap();
   std::fs::write("fixtures/sprite_v3_atlas.postcard", bytes)?;
   // ... 4 fixtures restantes ...
   ```
3. **T0.13 empirical postcard test**:
   ```rust
   #[test] fn versioned_dispatch_loads_v3_via_wrapper_enum() { /* ... */ }
   #[test] fn serde_default_fallback_loads_v3_trailing_eof() { /* postcard semantics check */ }
   ```
4. **T1.3.5 libm sweep** em `Transform::compose` v1 ANTES de mexer no skew amendment.
5. **T1.1..T1.14** schema bump strategic-only (zero feature visível ainda).

Critério de fechamento W1:
- `cargo test -p ph2d-render` verde.
- `cargo test -p ph2d-ecs` verde (libm cross-OS hash bit-identical).
- 5 fixtures v3 → v4 carregam sem perda.
- `vertex_attr_offsets_match_struct` verde com 11 vertex attrs.
- Bench criterion T1.7b: < 8ms M-series tier (senão dispara ADR-0070-amendment-1 dual-buffer).
- Smoke do Enio: cenário visual atual renderiza IDÊNTICO (zero regression).

---

## 6. Anti-colisão Coord-A (Painter T1.8+ ativo)

Painter active no momento da ratificação (T1.8 Stroke Vector History). Disciplina:

1. **SESSION_ACTIVE primeiro.** Antes de qualquer burst em `crates/ph2d-{render,ecs,host,asset}/`, verifique se Coord-A liberou.
2. **`git status` antes de stage.** Outros agentes podem ter arquivos staged; coletar só seus paths via `git add -- <paths>`.
3. **`git commit -m "msg" -- <paths>`** scoped — não use `-A` ou `-a` (memory [`feedback-scoped-commit-shared-index`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_scoped_commit_shared_index.md)).
4. **Stage cedo** para fence contra `git reset --hard` de outros agentes (memory [`feedback-destructive-reset-collision`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_destructive_reset_collision_2026_05_28.md)).

---

## 7. Gates ativos pós-W0

Lista canônica em [`Sprite_projeto/11_arch_gates_e_caps.md`](Sprite_projeto/11_arch_gates_e_caps.md). Críticos para W1:

| Gate | Crate | Quando ativa |
|---|---|---|
| `architecture_sprite_inspector_surface` | `ph2d-render` | W1.T1.12 cria (`Sprite` fields == 20; `RenderInstance` == 12; size_of == 144) |
| `vertex_attr_offsets_match_struct` | `ph2d-render` | EXISTENTE; expandir para 11 vertex attrs em W1 |
| `migrate_sprite_v3_to_v4` | `ph2d-render` | W1.T1.6 (fixtures geradas em T0.12) |
| `sprite_tint_finite_rejects_nan_and_inf` | `ph2d-render` | W1 (NaN/Inf reject em setters) |
| `sprite_scene_load_size_cap_enforced` | `ph2d-render` | W1 (100MB postcard cap) |
| `transform_compose_with_skew_determinism` | `ph2d-ecs` | W2.T2.2 (após libm sweep T1.3.5) |
| `inspector_paint_no_alloc` | `ph2d-panel-inspector` | W2 (HR-3 zero-alloc) |
| `inspector_paint_budget_hr4_p95` | `ph2d-panel-inspector` | W2-W5 (criterion p95 per-wave) |
| `inspector_section_count_canonical == 12` | `ph2d-panel-inspector` | W2 ativa quando sections.rs refactor T2.1 fecha |
| `inspector_section_loc_cap` | `ph2d-panel-inspector` | **`#[ignore]` até W2.T2.1** (sections.rs atual = 574 LOC viola ≤500) |

---

## 8. Build / verificar / smoke

```bash
# Implementador — durante edição
CARGO_TARGET_DIR="$PWD/target/slot-coord-sprite" cargo check -p ph2d-render
CARGO_TARGET_DIR="$PWD/target/slot-coord-sprite" cargo test  -p ph2d-render
CARGO_TARGET_DIR="$PWD/target/slot-coord-sprite" cargo test  -p ph2d-ecs

# Fmt + clippy antes do commit
CARGO_TARGET_DIR="$PWD/target/slot-coord-sprite" cargo clippy -p ph2d-render --all-targets -- -D warnings
cargo fmt -p ph2d-render -- --check

# Criterion bench W1.T1.7b
CARGO_TARGET_DIR="$PWD/target/slot-coord-sprite" cargo bench -p ph2d-render --bench sprites_upload_144b_vs_72b

# Antes do PUSH (Enio decide)
./scripts/ship.sh    # paridade-CI completa
```

Smoke do Enio em cada wave: scene fixtures em [`Sprite_projeto/15_plano_de_implementacao.md §15.8.2`](Sprite_projeto/15_plano_de_implementacao.md) — `smoke_w2_color_tint.scene`, `smoke_w3_sorting.scene`, etc. Cada checklist tem **pixel-identifiable references** (ex: "Pixel (200, 100) deve ser #00FFFF8E").

---

## 9. Follow-ups diferidos (não-bloqueantes; das 5 auditorias)

- **W8 Asset Cooker Integration** — Aseprite full import + Linked Cels dedup-hash + PSD; código **NÃO existe ainda** em `tools/asset-cooker/src/`. Wave separada explícita (Lens D D5).
- **Dual-buffer mitigation** (ADR-0070 §2.5) — só dispara se T1.7b bench mostra 144B vira gargalo (> 8ms M-series tier).
- **`would_cycle` O(V²)** em propagate_transforms — otimização só se surgirem cenas grandes.
- **GlobalTransform skew propagation** — bump separado em W2.T2.2 sub-task; spec ADR-0025-amendment-1 §6 open question.
- **Mobile tier degradation** — ADR-0068 Vector Module pattern aplicável a Sprite Inspector v2 em wave futura (não v1.0).
- **Spec evolução pós-v1.0** — amendments policy [`Sprite_projeto/10_schema_versionamento.md §10.11.1`](Sprite_projeto/10_schema_versionamento.md) define caminho para bumps (`ADR-XXXX-amendment-N` vs nova ADR).

## 9.1 Cross-session pre-existing failures (T1.3.5 R2 audit discovery, NÃO fixadas)

Discovered durante audit de T1.3.5 via `cargo test -p ph2d-editor-core` + `cargo check -p ph2d-host-desktop`. Não tocadas per memory `feedback-audit-scope-discipline` (auditor adversarial acha bug em crate adjacent → handoff/issue pro owner). Listadas aqui para sobreviver git-log age-off:

1. **`crates/ph2d-panel-hierarchy/src/paint.rs::paint_hierarchy_body`** = 388 LOC > 200 cap. Falha `cargo test -p ph2d-editor-core --test architecture_panel_loc_cap`. Inflou via hierarchy commits `3fab958` (same-name duplicates) + `4fb822b` (icon hit-area / double-click focus / tree-line alignment). **Owner: hierarchy session.** Fix proposto: split body em per-section helpers (cada helper recebe `y: f32` in, retorna `y: f32` out) ou adicionar entry em `FN_OVERAGE_OK` com justificação.

2. **`crates/ph2d-tool-painter/src/tool.rs`** referencia `PanelEvent::Activated` variant que não existe em `crates/ph2d-editor-core/src/PanelEvent`. Falha `cargo check -p ph2d-host-desktop` (3 errors). Painter T1.9 WIP pós-`231d6cc` / `1485471` (tool-painter session). Bloqueia link do desktop binary até resolver. **Owner: Painter session.**

---

## 10. Commits desta sessão

**Não pushados (Enio orquestra commit + push):**

| Wave/Task | Commit | LOC | Audits |
|---|---|---|---|
| W0 ratification docs | (untracked) | 17 spec files + 7 ADRs + HANDOFF + SESSION_ACTIVE | 5-lens W0 ratification cascade |
| W0.T0.12 + T0.13 | `cef1959` | +1094 / -7 | R1 (Lens B + C), R2 (Lens E + A) |
| W0 R3 follow-up (ADR rename + DX panic msgs + postcard exact-pin gate) | `e3ad19f` | +123 / -39 | R3 (Lens D + meta) |
| W1.T1.3.5 libm workspace-wide sweep | `5974a84` | +196 / -19 | R1 (Lens B + C) |
| W1.T1.3.5 R2 follow-up (24 unmigrated split sin/cos + libm exact-pin arch-gate + doc propagation) | (pending fix-up) | +~90 | R2 (Lens E + A + meta) |

Untracked W0 ratification artifacts (`docs/Sprite_projeto/*` + `docs/architecture/decisions/0069..0074-*.md` + `0025-amendment-1.md`) carry R3 inline edits and ride-along on Enio's commit chain.

---

## 11. Referências canônicas

- **Spec normativa Sprite Inspector v2:** [`docs/Sprite_projeto/`](Sprite_projeto/) (17 files).
- **ADRs Accepted:** [0069](architecture/decisions/0069-sprite-inspector-v2.md), [0070](architecture/decisions/0070-sprite-schema-v4.md), [0071](architecture/decisions/0071-tint-channels-multiplicative.md), [0072](architecture/decisions/0072-named-anchor-unification.md), [0073](architecture/decisions/0073-sorting-canonical-order.md), [0074](architecture/decisions/0074-sprite-component-boundary.md), [0025-amendment-1](architecture/decisions/0025-amendment-1.md).
- **Plano executável:** [`Sprite_projeto/15_plano_de_implementacao.md`](Sprite_projeto/15_plano_de_implementacao.md).
- **i18n catalog:** [`Sprite_projeto/16_i18n_catalog.md`](Sprite_projeto/16_i18n_catalog.md).
- **DIRETRIZ multi-agente:** [`docs/IntegracaoMultiAgente/DIRETRIZ.md`](IntegracaoMultiAgente/DIRETRIZ.md).
- **SKILL Stack:** [`SKILL_Stack_PH2D_Definitiva.md`](../SKILL_Stack_PH2D_Definitiva.md) §HR-1..HR-18.
- **Memory:** [`project_sprite_inspector_v2_w0_ratified_2026_05_28`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/project_sprite_inspector_v2_w0_ratified_2026_05_28.md).

---

**Confiança:** W0 ratificada após 5 lentes adversariais rotacionadas com cobertura canônica esgotada. Spec entregue padrão-ouro absoluto, sem deferral. Próximo agente abre W1 (Coord-A foundational). O ship/push é do Enio.
