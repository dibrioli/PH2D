---
name: feedback-full-gate-periodically
description: "run the full workspace gate periodically during a wave, not only at ship — cargo-check inner loop hides full-gate violations"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 37f0a53e-9992-4212-a23b-2270710c72a8
---

Durante uma wave longa (W2 Sprite Inspector, 2026-05-30), o inner loop `cargo check -p <crate>` + auditoria por incremento deixaram **7 violações de gate full-workspace acumularem** silenciosas; só apareceram no `ship.sh` no fim → **5 rodadas de fix** antes do push verde.

As que o `cargo check`/clippy-por-crate NÃO pega mas o nextest --workspace / ship.sh pega:
clippy `items_after_test_module` (test mod no meio de fns) · `no_literal_color` (`from_rgba8` em panel code) · `hr15_no_hardcoded_ui_strings` (baseline path-keyed quebra ao mover arquivo num split) · `no_magic_numeric` (literais f32 sem `// LITERAL-PX-OK:`) · `architecture_panel_loc_cap` (fn>200/file>600 — o brace-counter do monolito subconta; split EXPÕE) · **`cooker_determinism::prefab_cook_hash_is_locked`** (mudança de serialização — ex: skew no Transform — muda os bytes cozidos).

**Why:** esses gates rodam só no nextest --workspace (crates de teste em ph2d-editor-core/asset-cooker varrem outros crates), nunca no inner loop por-crate.

**How to apply:** a cada feature fechada (ou ~3-5 commits), rode `cargo nextest run --workspace --no-fail-fast --cargo-profile ci-test` (acha TODAS de uma vez) + arch-gates do crate tocado — ~6min no slot warm, barato vs N rodadas no fim. Mudou serialização (component/struct cozido)? **re-lock o cook hash no MESMO commit.** Flakes caracterizados que NÃO bloqueiam: cooker ISPC macOS (CI retry/skip) · `painter_no_alloc` (passa standalone; só flaca sob testes paralelos nos 8GB). Vide [[feedback-smoke-at-end]] (smoke é no fim; o GATE não — é periódico).
