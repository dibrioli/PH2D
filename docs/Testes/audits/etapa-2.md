# Audit adversarial — Wave 10 / Etapa 2

**Data:** 2026-05-24
**Auditores:** 2 agentes paralelos `general-purpose` — lentes: (A) corretude/regressões técnicas · (B) consistência doc↔código + arquitetura

---

## Achado CRITICAL (corrigido pré-commit)

### [C1] Bridges chamando `drive_deactivate_cleanup` em `tools.active_mut()` zeram state de outras RasterEditTools ativas

**Onde:**
- `shells/desktop/src/render_loop/upscale_bridge.rs:60-78` (introduzido nesta Etapa 2)
- `shells/desktop/src/render_loop/bgremoval_preview.rs:179-193` (introduzido na Etapa 1.B)

**Cenário:**

1. Usuário ativa **BgRemoval** + mexe sliders + Tolerance/Brush.
2. A cada frame, **todos** os bridges rodam (`bgremoval_preview::dispatch`, `upscale_bridge::dispatch`, `color_equalization_bridge::dispatch`).
3. `upscale_bridge` (que está no path inativo porque Upscale NÃO é o tool ativo) entra no `if !active { drive_deactivate_cleanup(...) }`.
4. `tools.active_mut()` retorna a **BgR ativa** (não a Upscale que o bridge representa).
5. `as_raster_edit_mut()` em BgR retorna `Some(self)` (pós-Etapa 1.B).
6. `drive_deactivate_cleanup(BgR_raster, upscale_preview, last_upscale_pushed)` → chama `BgR.deactivate()` → **zera `BgR.pending_apply`, `BgR.params_dirty`, `BgR.cached_canvas_preview`, etc.**
7. Resultado: usuário move slider → BgR não responde (params_dirty drained); clica Apply → nada acontece (pending_apply drained); preview congela.

**Repetir para CEQ, BgR, Upscale em qualquer ordem.** Cada bridge inativo destrói o tool ativo de outro tipo. A tool **inteira** quebra silenciosamente sem panic, só não responde.

**Severidade:** CRITICAL (data-loss user-visible — Apply silenciosamente não dispara; produto inutilizável quando mais de 1 raster tool existe).

**Por que passou em testes unit?** Cada tool é exercitada em isolamento; nenhum test stress-exercita o cenário "BgR ativo + Upscale bridge rodando inativo".

**Fix:**

- **Removido** `drive_deactivate_cleanup` do path inativo em `upscale_bridge.rs` e `bgremoval_preview.rs`.
- Substituído por limpeza local pura: `*cache = None; *last_pushed_entity = None;` + reset do snapshot do panel.
- A própria tool já fica corretamente desativada quando `ToolRegistry::set_active` é chamado (chama `Tool::on_deactivate`, que pós-fix A2 da Etapa 1.B já limpa `cached_canvas_preview` + todos os transient flags).
- `drive_deactivate_cleanup` permanece no `ph2d-tool-runtime` mas com **warning explícito** no doc-comment: "Never call with tools.active_mut() from a bridge whose own tool is NOT the currently-active one".
- Novo test `drive_deactivate_cleanup_unconditionally_deactivates_passed_tool` documenta a invariante (helper não tem awareness de ownership).

**Status:** ✅ FIXED

---

## Achados DOC (corrigidos pré-commit)

### [DOC-1] v4 plan §II Etapa 2 prometia 4 tools, entregou 2

Plan original: Padding + CEQ + Upscale + EqualizeSizes. Realidade: CEQ + Upscale + Padding/EqualizeSizes como documented exception. **Fix:** nota pós-execução adicionada em v4 plan §II Etapa 2.

### [DOC-2] DIRETRIZ §3.8.3.1 stale

Texto antigo dizia "BgR é o primeiro" mas agora 3 tools implementam. **Fix:** §3.8.3.1 reescrita listando os 3 + Padding/EqSizes como exception + mencionando `bgremoval_preview.rs` como template canônico.

### [DOC-3] `docs/Testes/audits/etapa-2.md` referenciado mas inexistente

Bridge CEQ + `app_state.rs` referenciam esse arquivo. **Fix:** este arquivo criado.

---

## Achados MÉDIOS — anotados (não bloqueiam Etapa 2)

### [M1] `drive_multi_preview_cache` helper faltando

CEQ bridge mantém BTreeMap multi-cache via downcast porque os helpers single-cache do runtime não batem. **Investigação adversarial:** runtime tem 101 LOC de folga (399/500); helper trivial (~40 LOC) generalizaria. **Decisão padrão-ouro:** adicionado em Etapa 3 junto com `arch_no_per_tool_branch_in_render_loop` (mesma sessão Coord-A, ROI alto pra consolidar).

### [M2] EqualizeSizes doc-comment poderia ser mais explícito

Atualmente diz "RasterEditTool seria wrong shape" — adversarial sugeriu explicitar que é o `MaxOfSelection` mode que força multi-buffer (não engenharia preguiçosa). **Decisão:** anotado mas não corrigido (já tem doc-comment OK; reforço fica para próximo refactor desse tool).

### [M3] DIRETRIZ §3.8.5 checklist ainda menciona "as_image_edit_mut"

Verificação: já corrigido em Etapa 1.A (rename). Adversarial reportou false-positive. ✓

---

## Coverage de tests (após fixes)

| Cenário | Test name | Status |
|---|---|---|
| CEQ upcast | `as_raster_edit_mut_returns_some_for_ceq` | ✅ |
| CEQ set_source delega | `raster_edit_set_source_delegates` | ✅ |
| CEQ current_preview drena dirty | `raster_edit_current_preview_drains_dirty` | ✅ |
| CEQ current_preview None sem source | `raster_edit_current_preview_none_without_source` | ✅ |
| CEQ take_pending_commit | `raster_edit_take_pending_commit_drains` | ✅ |
| CEQ run_full owned buffer | `raster_edit_run_full_returns_owned_buffer` | ✅ |
| CEQ deactivate limpa transientes | `raster_edit_deactivate_clears_transient_flags` | ✅ |
| Upscale upcast | `as_raster_edit_mut_returns_some_for_upscale` | ✅ |
| Upscale set_source delega | `raster_edit_set_source_delegates` | ✅ |
| Upscale current_preview drena | `raster_edit_current_preview_drains_dirty` | ✅ |
| Upscale current_preview None sem source | `raster_edit_current_preview_none_without_source` | ✅ |
| Upscale take_pending_commit | `raster_edit_take_pending_commit_drains` | ✅ |
| Upscale run_full owned buffer | `raster_edit_run_full_returns_owned_buffer` | ✅ |
| Upscale deactivate limpa transientes | `raster_edit_deactivate_clears_transient_flags` | ✅ |
| **[C1]** runtime helper warning regression | `drive_deactivate_cleanup_unconditionally_deactivates_passed_tool` | ✅ |

**Total novos tests Etapa 2:** 14 (7 CEQ + 7 Upscale) + 1 runtime regression = **15 testes determinísticos**.

**Gaps reconhecidos (não bloqueiam — Etapa 3):**

- Cross-bridge integration test "BgR ativo + Upscale bridge inativo não destrói BgR state" — exigiria smoke harness no shell (não trivial em test unit). Smoke manual U4 cenários 2+3 cobrem.
- `set_source invalida cached_canvas_preview` para Upscale (mirror do test BgR A1) — invariante já é testada implicitamente via `current_preview drains dirty` (que rebuild cache do source novo). Deferred.

---

## Veredito final

**Pronto para commit como Etapa 2 padrão-ouro.** C1 (CRITICAL) corrigido nos 2 bridges + warning explícito no helper + test regression no runtime. 3 docs corrigidos. 1 follow-up M1 deferido para Etapa 3 com justificativa explícita (`drive_multi_preview_cache` helper). Tests workspace verdes. Clippy clean.

**Smoke do Enio AINDA é requisito** antes do push da Wave 10 final — 9 cenários listados em `docs/Testes/README.md` §E2, especialmente U4 cenários 1+2+3 que verificam o fix C1 (cross-bridge não destrói state).
