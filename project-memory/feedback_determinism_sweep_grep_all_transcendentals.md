---
name: feedback-determinism-sweep-grep-all-transcendentals
description: "Sweep workspace-wide com `grep \\.sin_cos\\(\\)` apenas é INCOMPLETO; split-form `.sin()` / `.cos()` precisa do mesmo libm route — pegar AMBAS as formas"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 2145cc4f-66b3-4eb1-b4ee-05d0486ac094
---

Quando swept um primitivo de determinismo (libm, postcard, blake3 etc.) que substitui um `f32::*` method, **GREP DUAS FORMAS**: a compound form (`\.sin_cos()`) E a split form (`\.sin()|\.cos()`). T1.3.5 R1 audit (Lens B) pegou `sin_cos()` mas missed 24 split sites em gizmo/transform.rs + sim_populate.rs + gizmo/tests.rs; descoberto em R2 (Lens E + meta-review CRITICAL) — depois de já ter committed o sweep "completo".

**Why:** Rust API tem dois entrypoints semanticamente equivalentes: `x.sin_cos()` (returns tuple) e `x.sin()` + `x.cos()` (separate calls). Codebase mistura ambos por hábito (split é mais idiomático quando você só precisa de um dos valores; compound quando precisa dos dois). Audit que só procura UM dos padrões deixa o outro como leak silencioso. Lens B em R1 escreveu literal "`\.sin_cos()`" no grep — perdeu 24 sites.

**How to apply:**
- **Pre-sweep grep:** `grep -rn '\.sin()\|\.cos()\|\.sin_cos()\|\.tan()'` (alternation, not único pattern). Para outros primitivos (`atan2`, `sqrt`, `exp`, `pow`), enumerar TODAS as APIs equivalentes ou usar regex broader como `\.\(sin\|cos\|tan\|atan2\|exp\|sqrt\|pow\)\b`.
- **Pre-commit verify:** ANTES de claim "9 sites swept" no commit msg, re-rodar o grep ampliado. Audit em commit msg que mente custa duas rodadas (R1 → R2 → re-commit) que poderiam ter sido uma.
- **Arch-gate complement:** após o sweep, considerar `grep_arch_gate` test que assert ZERO occurrences do padrão amplo em paths sensíveis — vide `libm_exact_version_pin_enforced_in_workspace` em `crates/ph2d-ecs/tests/transform_determinism.rs` (pin de pin) + `architecture_no_f32_trig_in_transform_layer` (recomendado, não implementado ainda).
- **Lens B grep pattern review:** ao escrever briefing de auditor Lens B, listar VARIANTES (compound + split + variants like `(x).sin()` parenthesized). Sem isso, auditor itera o mesmo grep estreito que o implementer fez.

**Reference:** sessão Sprite Inspector v2 2026-05-28; T1.3.5 commit `5974a84` + R2 fix-up. R2 Lens E-C1 + meta-review META-C1 ambos flagaram convergente.
