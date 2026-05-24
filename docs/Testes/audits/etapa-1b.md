# Audit adversarial — Wave 10 / Etapa 1.B

**Data:** 2026-05-23
**Auditores:** 2 agentes paralelos `general-purpose` — lentes: (A) corretude/edge-cases técnicos · (B) consistência doc↔código + arquitetura

---

## Achados CRÍTICOS (corrigidos pré-commit)

### [A1] `set_source_snapshot` não invalidava `cached_canvas_preview` → stale frame cross-entity

**Onde:** `crates/ph2d-tool-bgremoval/src/tool.rs` — `pub fn set_source_snapshot`
**Cenário:** BgR ativo, sprite A selecionado, cache populated. Usuário muda para sprite B. Shell read de B falha (atlas miss). `set_source` nunca é chamado, mas o `params_dirty=true` foi setado por outra coisa. `current_preview` curto-circuita em `!has_source || canvas_src_rgba.is_empty()` mas o `cached_canvas_preview` ainda contém pixels de A.
**Severidade:** Crítico (data-loss user-visible — pinta sprite errado por cima do certo)
**Fix:** 1 linha — `self.cached_canvas_preview = None;` ao final de `set_source_snapshot`.
**Test novo:** `set_source_invalidates_cached_canvas_preview` em `tool::tests`.
**Status:** ✅ FIXED

### [A2] `Tool::on_deactivate` não limpava `cached_canvas_preview` → BgR→Brush→BgR mostra frame stale

**Onde:** `crates/ph2d-tool-bgremoval/src/tool.rs` — `fn on_deactivate` no `impl Tool`
**Cenário:** Usuário tem BgR ativo + preview no canvas. Troca para Brush via palette. `ToolRegistry::set_active` chama `Tool::on_deactivate` (não `RasterEditTool::deactivate`, porque Brush não é Raster — `drive_deactivate_cleanup` só roda na pré-condição "novo active é Raster"). O `on_deactivate` antigo limpava transient flags + dirty, mas NÃO o `cached_canvas_preview`. Re-ativando BgR (sem mexer na seleção), `current_preview` retorna None (dirty drained) e o overlay vê cache de sessão antiga.
**Severidade:** Crítico (user-visible artifact em workflow comum)
**Fix:** 1 linha — `self.cached_canvas_preview = None;` em `on_deactivate`.
**Test novo:** `on_deactivate_clears_cached_canvas_preview` em `tool::tests`.
**Status:** ✅ FIXED

### [3.1] `ph2d-tool-runtime` batia o pattern `ph2d-tool-*` do `tool-sync` → dep inerte

**Onde:** `tools/ph2d-tool-sync/src/lib.rs` — `scan_tool_crates` filter
**Cenário:** Ao criar `crates/ph2d-tool-runtime/`, o scan do sync incluiu o crate. `cargo_deps_in_sync_with_folder` (que NÃO filtra por `pub fn register/make` — usa scan cru) adicionou `ph2d-tool-runtime = { path = "..." }` em `crates/ph2d-tool-registry-init/Cargo.toml`. Dep inerte, mas ruído arquitetural + risco futuro de propagação.
**Severidade:** Alto (arquitetural; não user-visible mas vaza convenção)
**Fix:** Estender exclusion list incluindo `"ph2d-tool-runtime"` ao lado dos existentes `"ph2d-tool-registry"` e `"ph2d-tool-registry-init"`.
**Validation:** `cargo run -p ph2d-tool-sync` → 12 crates (era 13 com runtime). `cargo test -p ph2d-tool-registry-init --tests` → 3 staleness gates verdes.
**Status:** ✅ FIXED

---

## Achados MÉDIOS — doc stale (corrigidos pré-commit)

### [DOC-1] DIRETRIZ §3.8.3.1 estava STALE

**Texto antigo:** "RasterEditTool definido e congelado, mas **nenhum tool de produção implementa hoje** (`grep -rn "impl RasterEditTool" crates/ph2d-tool-*/` retorna zero)".
**Realidade pós-Etapa 1.B:** BgRemoval é o primeiro impl de produção (`crates/ph2d-tool-bgremoval/src/tool.rs` linha ~1273).
**Fix:** Reescrita completa de §3.8.3.1 documentando BgR como template + listando os 4 helpers de `ph2d-tool-runtime` + ressaltando que eyedropper/protect-brush continuam via downcast (exceção documentada ADR-0040 §3).
**Status:** ✅ FIXED

### [DOC-2] v4 plano §I prometia `Tool` cap = 11 + `RasterFrame/PixelLayout` typed wrappers

**Texto antigo:** `Tool` cap 10→11 (+ `as_raster_edit_mut`); trait com `current_preview(&mut self) → Option<RasterFrame>` + `pub struct RasterFrame { pixels: Arc<[u8]>, layout: PixelLayout }`.
**Realidade pós-Etapa 1.A:** Cap stays at 10 (rename do método não bumpa o counter); assinaturas crus `Vec<u8>` + `(&[u8], u32, u32)` — typed wrappers diferidos para Etapa 5 quando `ph2d-color` chegar. ADR-0041 §2.4 já documenta isso.
**Fix:** Adicionada nota de correção no v4 plan §II reconhecendo divergência + apontando ADR-0041 como verdade canônica.
**Status:** ✅ FIXED

---

## Achados BAIXOS — cosméticos (corrigidos pré-commit)

### [LOW-1] Doc-comment de `drive_pending_commit` mencionava `tool_id` que não está na signature

**Onde:** `crates/ph2d-tool-runtime/src/lib.rs` — doc-comment de `drive_pending_commit`
**Issue:** comment dizia "`tool_id` is borrowed only for the trait method signature" — confuso, porque o helper não recebe `tool_id`.
**Fix:** Reescrita pra explicar intenção: "intentionally `tool_id`-agnostic: new tools don't need a runtime patch".
**Status:** ✅ FIXED

---

## Achados BAIXOS — opt-out anotados (não bloqueiam Etapa 1.B)

### [LOW-2] Nome `ph2d-tool-runtime` colide com pattern (mesmo fixado via exclusion)

**Achado:** Padrão-ouro seria nome que NÃO matche o glob `ph2d-tool-*`. Alternativas: `ph2d-raster-runtime`, `ph2d-shell-tool-driver`.
**Decisão:** Mantido nome atual nesta etapa. Rename agora forçaria churn em todos bridges Etapa 2 que vão importar. Reabrir como issue se mais um `ph2d-tool-*-infra` aparecer.
**Status:** 📝 ANOTADO (não bloqueia)

### [LOW-3] DIRETRIZ §3.8.3 tabela "sabor (3)" + §3.8.5 checklist ainda apontam downcast como padrão atual

**Achado:** Pós-Etapa 1.B, padrão canônico para tool raster nova é "impl RasterEditTool". Tabela + checklist ainda implicam downcast como default.
**Decisão:** Update virá com **Etapa 5** (quando ph2d-color introduz typed shapes + reescreve DIRETRIZ §3.8 inteira). Mexer agora prematuramente fragmenta o doc. §3.8.3.1 já tem o ponto canônico.
**Status:** 📝 ANOTADO (deferido para Etapa 5)

### [LOW-4] v4 §III tabela métrica "downcasts ≤2 pós Etapa 3"

**Achado:** Pós-Etapa 1.B BgR ainda tem ~4 downcasts no `bgremoval_preview.rs` (panel snapshot + protect mask + brush ring + outros). Métrica do plano é aspiracional para pós-Etapa 3.
**Decisão:** Métrica do plano permanece — é meta. Realidade pós-Etapa 1.B vai pra docs/Testes/README.md métricas.
**Status:** 📝 ANOTADO

---

## Coverage de tests (após fixes)

| Cenário | Test name | Status |
|---|---|---|
| Upcast funciona | `as_raster_edit_mut_returns_some_for_bgremoval` | ✅ |
| set_source delega | `raster_edit_set_source_delegates_to_set_source_snapshot` | ✅ |
| current_preview drains dirty + cacheia | `raster_edit_current_preview_drains_dirty_and_caches` | ✅ |
| current_preview retorna None sem source | `raster_edit_current_preview_returns_none_without_source` | ✅ |
| take_pending_commit drena Apply | `raster_edit_take_pending_commit_drains_apply_flag` | ✅ |
| run_full retorna owned buffer | `raster_edit_run_full_returns_owned_buffer` | ✅ |
| deactivate limpa todos transientes | `raster_edit_deactivate_clears_all_transient_state` | ✅ |
| **[A1]** set_source invalida cache cross-selection | `set_source_invalidates_cached_canvas_preview` | ✅ |
| **[A2]** on_deactivate limpa cache (path Brush) | `on_deactivate_clears_cached_canvas_preview` | ✅ |
| Runtime helpers (9 cenários) | em `crates/ph2d-tool-runtime/src/lib.rs::tests` | ✅ |

**Total novos tests:** 9 BgR + 9 runtime = **18 testes determinísticos** cobrindo o contrato + os 2 audit fixes.

**Gaps reconhecidos (não bloqueiam — deferidos):**
- `raster_edit_run_full_panics_without_source` — `run_full` panics via `assert!(self.has_source())` se chamado sem source. Não tem teste `#[should_panic]` mas a invariante é cumprida pelo wrap.

---

## Veredito final

**Pronto para commit como Etapa 1.B padrão-ouro.** Os 3 achados Crítico/Alto foram fixados + 2 doc-stale corrigidos + 1 cosmético. Tests workspace verdes. Clippy clean. Staleness gates verdes.

**Smoke do Enio AINDA é requisito** antes do push da Wave 10 final — 6 cenários listados em `docs/Testes/README.md` §E1B. Eles cobrem os caminhos que mudaram estruturalmente, especialmente os dois cenários dos fixes [A1] e [A2] (selection drift e BgR→Brush→BgR).
