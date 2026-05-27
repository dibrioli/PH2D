# 🎨 HANDOFF — Painter PH2D · pós-T1.6 (brush mature) · 2026-05-27

**Para:** próxima LLM (agente novo) pegar T1.7 (ou o que vier depois) em Painter.

**T1.6 = brush mature**: shape variety (4 procedural kernels: round_hard
/ round_soft / square_hard / oval_hard) + multi-stamp (`shape_count` up
to 16) + rotation (`shape_rotation_follow` + `shape_scatter`) + color
jitter (4 axes em OKLab: lightness / darkness / hue / saturation) +
smoke env vars + auditoria adversarial multi-round (R3..R9 = 7 rounds
× ~35 lentes). Smoke do Enio passou.

---

## 0. LEIA ANTES DE CODAR — armadilhas que custaram retrabalho em T1.6

Estes são erros REAIS que cometi em T1.6 e que você NÃO pode repetir.
Cada item tem um link pra memória que documenta o incidente.

### 0.1. NÃO TRADUZA toasts / labels / UI strings pra pt-BR

🛑 **Regra absoluta:** UI strings do PH2D ficam SEMPRE em inglês,
mesmo que o Enio escreva o pedido em pt-BR. Sem exceção. Vide
[`feedback_app_ui_english_only`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_app_ui_english_only.md).

**O que cometi em T1.6 (NÃO repita):**
- R7 lens J1 marcou toasts EN do `painter.rs` drain como "HR-15
  violation" e recomendou pt-BR. **Aceitei sem conferir a memória.**
  Traduzi 5 strings em `painter.rs` drain.
- R9 lens V1 detectou "i18n split" (painter pt-BR vs outros tools EN)
  e recomendou padronizar. **Escalei o erro**, traduzindo bgremoval,
  padding, color_eq, upscale drains (4 arquivos, ~10+ strings).
- Enio: "me explique por que traduzir os toasts para PT-BR?" → tive
  que reverter tudo via commit `7fed63b`.

**Heurística defensiva:** quando um auditor adversarial citar HR-15
OU "i18n violation" OU "inconsistent language" em strings user-facing,
**CONFIRME `feedback_app_ui_english_only` ANTES** de aceitar. Default
do PH2D = English. HR-15 ≠ "translate to pt-BR".

Exemplo do que está CERTO:
```rust
toasts.push(Toast::info("Painter: no strokes to apply"));        // ✅ EN
toasts.push(Toast::error(format!("Painter failed: {err}")));     // ✅ EN
toasts.push(Toast::success("Painter applied · Cmd+Z to undo")); // ✅ EN
```

Exemplo do que ESTÁ ERRADO (não cometa):
```rust
toasts.push(Toast::info("Painter: nenhum traço para aplicar"));  // 🛑 NÃO
toasts.push(Toast::error(format!("Painter falhou: {err}")));     // 🛑 NÃO
toasts.push(Toast::success("Painter aplicado · Cmd+Z desfaz")); // 🛑 NÃO
```

**Comentários de código** podem ser misto pt-BR + EN (segue padrão do
arquivo). Restrição é só pra strings que viram pixels na tela do
usuário.

### 0.2. NÃO toque crates fora do escopo Painter

🛑 **Regra:** auditor adversarial acha bug em crate adjacent →
**handoff** pro owner, NÃO fixo eu mesmo. Vide
[`feedback_audit_scope_discipline`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_audit_scope_discipline.md).

**O que cometi em T1.6:**
- R7 auditores adversariais acharam achados em `bgremoval/algorithm`,
  `bgremoval/params`, `bgremoval_preview` (shells). Fixei tudo
  "porque já tinha tocado via CI recovery sweep". Errado.
- R8/R9 escalou pra `ph2d-color/OklchColor`, `bgremoval/scratch`,
  `bgremoval/tool.rs`, `padding/upscale/color_eq/equalize-sizes/painter
  params` non_exhaustive, 4 drain files. **Escopo creep flagrante.**
- Enio: "Por que vc que está implementando o painter está vendo
  outros módulos como BGRemoval, CEQ dentre outros?" → revert via
  `7fed63b`. Os achados foram movidos pra HANDOFFs nos respectivos
  donos:
  - [`HANDOFF_ph2d_color_oklch_serde`](HANDOFF_ph2d_color_oklch_serde.md)
  - [`HANDOFF_bgremoval_audit_carryovers`](HANDOFF_bgremoval_audit_carryovers.md)
  - [`HANDOFF_padding_ui_edit_non_exhaustive`](HANDOFF_padding_ui_edit_non_exhaustive.md)
  - [`HANDOFF_upscale_ui_edit_non_exhaustive`](HANDOFF_upscale_ui_edit_non_exhaustive.md)
  - [`HANDOFF_color_equalization_ui_edit_non_exhaustive`](HANDOFF_color_equalization_ui_edit_non_exhaustive.md)
  - [`HANDOFF_equalize_sizes_ui_edit_non_exhaustive`](HANDOFF_equalize_sizes_ui_edit_non_exhaustive.md)

**Painter scope explícito (toque APENAS aqui):**
- `crates/ph2d-painter-brush/*` ✓
- `crates/ph2d-painter-contracts/*` ✓
- `crates/ph2d-tool-painter/*` ✓
- `docs/Painter_projeto/*` ✓
- `docs/HANDOFF_painter*.md` (este, T1.5 close, master)
- **Painter-específicos em shells/desktop:**
  - `shells/desktop/src/render_loop/painter_bridge.rs`
  - `shells/desktop/src/hero_intents/image_edit/painter.rs`
  - `shells/desktop/src/app_state.rs` (campos Painter)
  - `shells/desktop/src/input_dispatch/painter_input.rs`
  - `shells/desktop/src/render_loop/mod.rs` — APENAS lines que tocam
    PainterTool downcast (L1-1 destructive-deactivate warn é o
    precedente).

Se achado de auditor for fora dessa lista: HANDOFF, não fix.

### 0.3. `git add` + validação longa + `git commit` = janela de colisão

🛑 **Regra:** quando commitar painter, use `git commit -- <paths>`
(com `--`) que é **stage+commit atômico**. NÃO faça `git add` + `cargo
check --workspace` (5min) + `git commit` — outro agente paralelo pode
pegar seus arquivos staged durante a janela. Vide
[`feedback-parallel-agent-collision`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_parallel_agent_collision.md).

**Aconteceu em T1.6 R8:** stagiei 7 arquivos via `git add`, rodei
`cargo check --workspace` (4m07s), depois `git commit` retornou "no
changes added to commit". Outro agente havia absorvido meus arquivos
em `90abf85` (color-eq commit). Trabalho preservado em HEAD mas
atribuição perdida — tive que criar empty commit `61a1428` documentando
o R8.

Sequência correta:
```bash
# Validate FIRST (sem add ainda)
cargo check -p ph2d-painter-brush -p ph2d-tool-painter
cargo test  -p ph2d-painter-brush -p ph2d-tool-painter
# Atomic stage+commit (janela ms)
git commit -- crates/ph2d-painter-brush/src/file1.rs \
              crates/ph2d-tool-painter/src/file2.rs ...
```

### 0.4. Auditoria adversarial — diversifique lentes, mas controle scope

A T1.6 fez 7 rounds × ~35 lentes (P-Z, A1-W1). Padrão útil:
- **2-5 lentes paralelas por round** (não 1) — diversidade pega bugs
  que uma lente sozinha não vê.
- **Rotacione lentes entre rounds** — não repete a mesma lente.
- **Brief explícito**: "auditor lens X deve apenas relatar achados
  em crates {painter-brush, tool-painter, painter-contracts, …};
  achados em outros crates devem sair como bullet handoff, NÃO como
  remediation target." (Isso vai te evitar o scope creep que sofri.)
- **Trate findings com severity rubric clara**: CRITICAL (panic/UB/
  data loss), HIGH (silent wrong behavior), MEDIUM (foot-gun),
  LOW (nit). Pareto: fixe Crit+High; doc-and-defer Med+Low.

Lentes que renderam bem em T1.6 (use como inspiração):
- math correctness / edge cases (P, Q, T) — pega NaN/clamp drift
- HR-5 determinism (O1) — pega libm transcendental cross-OS
- WGSL/GPU parity (S1, K1) — pega CPU↔shader divergence
- API stability (I1, P1) — pega breaking changes futuras
- threading (R1) — pega Send+Sync auto-derive holes
- savefile ABI (W1) — pega serde derive missing
- spec/doc drift (Q1) — pega doc claims que não batem com código
- cross-tool consistency (V1) — **mas use como SINAL pra handoff,
  não como invitation pra escalar fora do escopo**

---

## 1. Estado da entrega (T1.6 close)

### 1.1. Commits locais (Painter scope)

```
0d8f0b2 docs(handoffs): R7+R8+R9 audit carryovers para crates adjacentes ao Painter
7fed63b fix(painter): T1.6 R9 audit remediations (Painter scope) + revert R7 pt-BR toasts
61a1428 chore(painter): T1.6 R8 audit attribution note (post-collision)
90abf85 [SWALLOWED] color-eq commit que absorveu T1.6 R8 painter-brush + tool-painter
5f7680c fix(painter): T1.6 R7 audit remediations — 5 lenses, padrão-ouro
2dac48b docs(painter): handoff T1.5→T1.6 — smoke Enio confirmado + escopo T1.6
7cb95d4 fix(bgremoval): gamma-correct premultiply for overlay — kill light halo (T1.5)
...
14e416f feat(painter): T1.5 — CPU stamp render Day-7 marker (the brush mature foundation)
```

**Branch:** `main` local. **NÃO push autônomo** — Enio decide quando.

### 1.2. Test surface verde

```bash
cargo test -p ph2d-painter-brush --lib --tests       # 154/154 ✓
cargo test -p ph2d-tool-painter   --lib --tests      # 28/28 ✓
cargo test -p ph2d-tool-painter   --test smoke_env_contract  # 11/11 ✓
cargo test -p ph2d-painter-contracts --tests         # 75/75 arch gates ✓
```

**Workspace check NÃO está verde em main hoje** —
`ph2d-panel-color-equalization` tem WIP alheia (campo `denoise_method`
referenciado mas removido do snapshot). Não é Painter, mas convém
mencionar pro Enio antes de qualquer `cargo check --workspace` rodar.

### 1.3. Auditoria realizada

7 rounds × ~35 lentes. Resumo evolutivo:
- **R3+R4+R5** (commit `932316e`): zero Crit/High padrão-ouro.
- **R6** (commit `728d5df`): close final padrão-ouro pós-6 audit
  cycles.
- **R7** (commit `5f7680c`): 28 findings (5 lenses H1/I1/J1/K1/L1) →
  remediados (mas pulou escopo, vide §0.2 lessons learned).
- **R8** (commit `90abf85` swallow + `61a1428` attribution): 36
  findings (5 lenses M1/N1/O1/P1/Q1) → remediados (Painter scope
  parcial).
- **R9** (commit `7fed63b` + `0d8f0b2` handoffs): 32 findings (6
  lenses R1/S1/T1/U1/V1/W1) → 7 remediados em Painter scope, 9
  movidos para HANDOFF nos respectivos donos.

**Conclusão R9:** todos os achados Painter-scope estão fechados ou
documentados como W2+ follow-ups no plano (vide
`Painter_projeto/15_plano_de_implementacao.md`). Achados cross-crate
estão em HANDOFFs (vide §0.2).

---

## 2. Próximo passo: T1.7 (e além)

Vide [`15_plano_de_implementacao.md`](Painter_projeto/15_plano_de_implementacao.md)
para a sequência canônica. T1.7 historicamente trazia:
- **Opacity per-stroke** (vs `flow` per-stamp já wired em T1.6) — o
  scheduler `push_one_stamp` tem TODO marcando `opacity` hardcoded a
  1.0. Vide [`stamp_scheduler.rs`](../crates/ph2d-painter-brush/src/stamp_scheduler.rs).
- **Taper opacity** (transição lápis-de-grafite em strokes começando
  ou terminando). Vide [`taper.rs`](../crates/ph2d-painter-brush/src/taper.rs).

Antes de começar: **confirme o que o Enio quer pra próxima task.** O
plano §4 lista candidatos; nem todos são T1.7 — outras
"T-color-full / T-tier / T-input / T-durability" podem estar
preferidas dependendo do estado do projeto.

---

## 3. Documentação canônica (leia antes de codar)

- [`docs/HANDOFF_painter.md`](HANDOFF_painter.md) — handoff master (mandato §0 + loop §1)
- [`docs/HANDOFF_painter_t15_close.md`](HANDOFF_painter_t15_close.md) — handoff T1.5→T1.6 (predecessor)
- [`docs/Painter_projeto/15_plano_de_implementacao.md`](Painter_projeto/15_plano_de_implementacao.md) — plano de implementação Painter (cascata W0..W∞)
- [`docs/Painter_projeto/01_brush_engine.md`](Painter_projeto/01_brush_engine.md) — spec do brush engine (§1.3.8.1 tem o que R7-R9 ratificou)
- [`docs/architecture/decisions/0043-painter-contract.md`](architecture/decisions/0043-painter-contract.md) … 0053 — 11 ADRs Painter ratificados
- [`docs/IntegracaoMultiAgente/DIRETRIZ.md`](IntegracaoMultiAgente/DIRETRIZ.md) v7.0 — protocolo multi-agente (sanity check §0, slot-env §1.2, gate cadence §5)
- [`SKILL_Stack_PH2D_Definitiva.md`](../SKILL_Stack_PH2D_Definitiva.md) — Hard Rules HR-1..HR-18 (HR-3 zero-alloc hot path, HR-5 cross-OS det, HR-15 i18n, …)

---

## 4. Memórias acionadas em T1.6 (leia o índice antes de começar)

[`MEMORY.md`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/MEMORY.md)
é carregado automaticamente em toda sessão; varra antes de codar.
Especialmente relevantes pra T1.7+:

- `feedback_app_ui_english_only` — vide §0.1 deste handoff
- `feedback_audit_scope_discipline` — vide §0.2 (NOVO em T1.6)
- `feedback_parallel_agent_collision` — vide §0.3
- `feedback_audit_lens_diversity` — rotacione lentes, ≥2 paralelas
- `feedback_perfection_no_deferrals` — padrão-ouro absoluto, sem cortar invariante
- `feedback_pipeline_inject_dont_cap` — feature nova injeta no buffer do pipeline, não capeia final
- `feedback_codificacao_rapida` — `cargo check -p <crate>` em vez de --workspace; pre-commit hook é o safety-net final
- `project_painter_t14_complete_2026_05_26` + `project_painter_t15_complete_2026_05_26` — estado do projeto chegando em T1.6

---

## 5. Sanity check antes da primeira linha de código

```bash
# 1. Confirma HEAD
git log --oneline -5
# Esperado: 0d8f0b2 (handoffs R9) → 7fed63b (R9 Painter scope) → ...

# 2. Working tree limpo (alheia ok, painter scope deve estar clean)
git status -sb | grep -E "painter|tool-painter"
# Esperado: nenhum diff em crates painter (você começou clean)

# 3. Painter test surface verde
cargo test -p ph2d-painter-brush -p ph2d-tool-painter \
  -p ph2d-painter-contracts --lib --tests 2>&1 | grep "test result"
# Esperado: 4× "test result: ok. ... 0 failed"

# 4. Slot env (se DIRETRIZ v7.0 §1.2 aplicar)
source scripts/slot-env.sh impl-1   # ou outro slot conforme Enio
```

Qualquer divergência → **pare e reporte ao Enio.** Não tente
auto-recover.

---

## 6. Fechamento

Padrão-ouro mantido em T1.6. Auditoria adversarial intensiva (7
rounds × ~35 lentes) deixou Painter no melhor estado possível dentro
do escopo legítimo. Lições aprendidas (§0) preservadas em memória
+ neste handoff pra evitar regressão de processo na próxima sessão.

**Boa sorte com T1.7.** A barra é "melhor que Procreate em 2D, com
superioridade técnica genuína em pelo menos 5 dimensões". Não corte
invariante.

— Painter T1.6 implementor, 2026-05-27
