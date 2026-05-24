# Audit adversarial — Wave 10 / Etapa 3

**Data:** 2026-05-24
**Auditores:** 1 agente `general-purpose` (escopo isolado: 3 gates + 1 helper + 1 bridge refactor)

---

## Achados CRITICAL (corrigidos pré-commit)

### [C1] Allowlist gate #2 com 5 entries fantasma (pre-permissão)

`shells/desktop/tests/architecture_no_downcast_to_concrete_tool_in_shell.rs:62-68` listava 5 arquivos `hero_intents/image_edit/*.rs` que **NÃO contêm downcasts reais**. `grep -c "downcast_" ...` retornava 0 pra todos. Allowlist pré-concedendo permissão futura é exato oposto da disciplina que o gate enforça.

**Fix:** Removidos os 5 entries fantasma. Allowlist agora só tem entries com downcasts reais (6: eyedropper, protect_brush, bgremoval_preview, color_equalization_bridge, upscale_bridge, padding_bridge, equalize_sizes_bridge, image_edit.rs).

**Status:** ✅ FIXED

### [C2] Baseline gate #1 = 20 com 4 unidades de folga "achatada"

`shells/desktop/tests/architecture_no_per_tool_branch_in_render_loop.rs:70` definia `BASELINE_MENTIONS: usize = 20` mas count real era 16. Gate "warns if count GROWS" não cumpre o propósito se há folga > 0.

**Fix:** Snapped `BASELINE_MENTIONS = 16`. Gate agora falha em qualquer adição. Bumping down em Etapas 4-7 é encouraged.

**Status:** ✅ FIXED

### [C3] Drain duplicado de `take_pending_apply` regredia multi-select Apply

`shells/desktop/src/input_handlers.rs:316-328` drenava `bg.take_pending_apply()` e pushava `OneShotImageOp { entity_bits: hero.gizmo.selection }` — **single entity (primary)**.

`shells/desktop/src/render_loop/bgremoval_preview.rs:140` chamava `drive_pending_commit(bg, iter_selected())` — **multi entity**.

Como `take_*` é destrutivo, o input_handlers vencia (roda antes do render); bridge sempre via `false` → empurrava zero OneShotImageOps. **Multi-select + Apply Toggle no panel só bakava o primary sprite.**

**Fix:** Bloco DELETADO de `input_handlers.rs`. Bridge canônico em `bgremoval_preview.rs` cobre via `drive_pending_commit` (multi-sprite). Latência: 1 frame entre toggle e bake — visualmente equivalente.

**Status:** ✅ FIXED

---

## Achados ALTO (corrigidos pré-commit)

### [A1] `drive_multi_preview_cache` mantinha cache STALE em read_source miss

Quando `read_source(entity_2)` retornava `None` (atlas miss transient), helper fazia `continue` mas mantinha cache antigo de entity_2. Bridge continuava paintando frame computado com parâmetros antigos enquanto user movia sliders — **visualmente equivalente a uma mentira congelada**.

**Fix:** No path miss, `cache.remove(bits)` antes de `continue`. Honest signal "preview unavailable for this sprite right now". Novo test `drive_multi_preview_cache_drops_stale_entry_on_read_miss` documenta a invariante.

**Status:** ✅ FIXED

### [A2] Doc-comment `BgremovalPreview` (app_state.rs) stale

Dizia "CEQ + Upscale keep own structs until Etapa 2 migrates" — mas Etapa 2 já migrou Upscale, Etapa 3 migrou CEQ. **Fix:** atualizado para "Etapa 3 STATUS: all three are aliases of PreviewCache".

**Status:** ✅ FIXED

### [A3] Doc-comment módulo `color_equalization_bridge.rs` stale

Dizia "future drive_multi_preview_cache would let CEQ migrate" — mas o helper foi escrito nesta Etapa 3 e o bridge já usa. **Fix:** atualizado para "Etapa 3 STATUS: fully migrated".

**Status:** ✅ FIXED

---

## Achados MÉDIO/BAIXO (anotados, não-bloqueantes)

| # | Severidade | Descrição | Status |
|---|---|---|---|
| M1 | Baixa | Marker `ARCH-ALLOW: per-tool-branch` ainda não usado em produção (escape-hatch reservada para Etapas futuras). | 📝 ANOTADO |
| M2 | Baixa | Gate #1 não detecta literais multi-line (`concat!`, raw strings) — todos os 16 mentions atuais são single-line. Heurística aceitável pra Etapa 3. | 📝 ANOTADO |
| M3 | Baixa | Gate #2 só escaneia `shells/desktop/src/**` — outros shells futuros precisam gate similar. | 📝 ANOTADO |
| M4 | Baixa | Gate #1 não filtra `#[cfg(any())]` gated-out — não há ocorrência hoje. | 📝 ANOTADO |
| B1-5 | Baixa | Perf (`Vec` alloc), HashSet non-determinism, doc explicitness, sub-dir scan, `/* */` comments — todos casos hipotéticos sem ocorrência atual. | 📝 ANOTADO |

---

## Coverage de tests (Etapa 3 final)

| Test | Status |
|---|---|
| `architecture_no_per_tool_branch_in_render_loop` (gate #1) | ✅ |
| `architecture_no_downcast_to_concrete_tool_in_shell` (gate #2) | ✅ |
| `architecture_image_tool_kind_contract` (gate #3) | ✅ |
| `drive_multi_preview_cache_produces_one_frame_per_selection_entry` | ✅ |
| `drive_multi_preview_cache_drops_entries_outside_selection` | ✅ |
| `drive_multi_preview_cache_drops_stale_entry_on_read_miss` (audit [A1]) | ✅ |
| `drive_multi_preview_cache_skips_unreadable_entities` | ✅ |
| `drive_multi_preview_cache_empty_selection_clears_cache` | ✅ |
| `ph2d_tool_runtime_loc_under_cap` (cap 650) | ✅ |

**Total:** 3 arch-gates novos + 5 unit tests novos no helper + 1 cap test atualizado = **9 novos asserts permanentes**.

---

## Veredito final

**Pronto para commit como Etapa 3 padrão-ouro.** 3 críticos + 3 altos fixados pré-commit. Achados M/B anotados para revisão futura sem bloquear. Todos os arch-gates verdes; clippy workspace clean; cap LOC do runtime fixado em 650 (último bump permitido em Wave 10).

**Smoke do Enio** crucial em G1-G3 (cobre fix [C3] multi-select Apply via panel toggle).
