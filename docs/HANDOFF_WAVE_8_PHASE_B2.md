# HANDOFF — Wave 8 Phase B.2 (retomada após context refresh)

**Data:** 2026-05-18 noite
**Estado:** B.1 fechada (`f46a6c4`). B.2-5 pendente.
**Lê isto se você é uma LLM nova chegando para retomar.**

---

## 1. Leia primeiro (ordem)

1. [`CLAUDE.md`](../CLAUDE.md) — workflow operacional (auto-load).
2. [`docs/DIRETRIZ_CODIFICACAO_RAPIDA.md`](DIRETRIZ_CODIFICACAO_RAPIDA.md) v1.2
   — cadência: 1200 LOC threshold para cargo check, 1 commit por Phase,
   não duplicar pre-commit hook.
3. [`docs/architecture/decisions/0029-trait-driven-panel-host.md`](architecture/decisions/0029-trait-driven-panel-host.md)
   §5 Phase B — **o plano operacional do que fazer agora**. ADR completa em
   12 seções; §5 é o que você executa.
4. `git log --oneline -10` — vê os 9 commits empilhados em main.
5. Memória persistente: `~/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/MEMORY.md`
   especialmente `feedback_codificacao_rapida.md` v1.2.

---

## 2. Estado atual (verificar)

```bash
git log --oneline -1
# f46a6c4 feat(editor-core): ADR-0029 Phase B.1 — trait-driven panel host infrastructure

git status -sb
# ## main...origin/main [ahead 9]
# (clean — nenhuma mudança pendente)

cargo check -p ph2d-editor-core 2>&1 | tail -3
# Finished `dev` profile (verde, ~5-10s)
```

Se algo diverge, **pare e pergunte ao Enio** antes de mudar nada.

---

## 3. O que B.1 entregou (não toque)

`ph2d-editor-core::panel/` já tem TODA a trait infrastructure:

- `host.rs` — `PanelHost` (público stable) + `PanelHostInternal: PanelHost` (#[doc(hidden)])
- `panel_trait.rs` — `pub trait Panel { type State; fn paint/apply_event/populate }`
- `paint_ctx.rs` — `PaintCtx { host: &mut dyn PanelHostInternal, layout, scene, … }`
- `erased.rs` — `ErasedPanel` wrapper com `Box<dyn Any + Send>` state
- `manifest.rs` — `PanelManifest` + `for_panel::<P>()` ctor
- `registry.rs` — `PANEL_REGISTRY: OnceLock<Mutex<PanelRegistry>>` com `Vec<ErasedPanel>`
- `event_outcome.rs` — `EventOutcome::{Consumed, Ignored, Observed}`

`ph2d-editor-core::screen/layout.rs` — `HeroLayout` + 11 chrome consts.

**Nada está wired ainda.** É só foundation.

---

## 4. O que B.2-5 precisa fazer (do ADR-0029 §5)

### B.2 — move HeroScreen + screens/hero/* + grid_snap/* para editor-core

Files a mover de `crates/ph2d-editor/src/` → `crates/ph2d-editor-core/src/`:

- `action_bus.rs` → `editor-core/src/action_bus.rs` (top-level)
- `image_edit/` → `editor-core/src/image_edit/`
- `tool.rs` → `editor-core/src/tool.rs`
- `tools/` → `editor-core/src/tools/`
- `test_support.rs` → `editor-core/src/test_support.rs`
- `grid_snap/` → `editor-core/src/grid_snap/`
- `screens/hero.rs` + `screens/hero/*` → `editor-core/src/screen/hero.rs` + `editor-core/src/screen/hero/*`
- `screens/mod.rs` → mover/merge em `editor-core/src/screen/mod.rs`

**DELETAR:** `ph2d-editor/src/panel_registry.rs` (substituído pelo novo
`editor-core/src/panel/`).

### B.3 — HeroScreen impl PanelHostInternal

Após a move, add `impl PanelHostInternal for HeroScreen` no novo
`editor-core/src/screen/hero.rs`. Surface atual mínima (4 métodos do trait):
`store()`, `store_mut()`, `hit_index_mut()`, `theme()`, `project()`.

### B.4 — orchestrator atualiza pra ErasedPanel

`paint_hero_screen` itera `with_registry(|r| for p in r.panels_mut() { p.paint(&mut ctx) })`
em vez do `for manifest in panels()` antigo. Manifests antigos (em
`screens/hero/{inspector,hierarchy,widget_gallery,grid_snap}/mod.rs::PANEL_MANIFEST`)
**continuam existindo** durante a transição — são da forma fn-pointer antiga e o
orchestrator preserva compat ATÉ Phase C migrar painéis pra `Panel<State>`.

⚠️ **Tensão técnica:** B.4 quer ErasedPanel iteration, mas os painéis ainda
estão na forma fn-pointer antiga. Solução em B.4: manter o panel_registry
antigo de ph2d-editor PARALELO ao novo de editor-core durante a transição.
Ou: ajustar B.4 pra manter iteration sobre OLD manifests até Phase C.
**Sugestão:** ajustar B.4 — orchestrator preserva iteration old-style; Phase C
faz o swap. Mais conservador.

### B.5 — ph2d-editor vira shim

`ph2d-editor/src/lib.rs` vira basicamente:

```rust
//! Wave 8 Phase B.5 — shim re-exporting from ph2d-editor-core.
//! Pre-1.0 deprecation alvo: deletar em ADR-0030 ou após 1 release de uso.
pub use ph2d_editor_core::*;
```

Mantém o nome `ph2d-editor` resolvendo durante transição. Painéis crates
(`ph2d-panel-*`) continuam com `use ph2d_editor::*;` transparentemente.

### B.6 — checkpoint smoke

```
cargo test --workspace --exclude ph2d-asset  → 1315 verde
cargo clippy --workspace --all-targets -- -D warnings  → clean
./play.command  → editor abre, painéis renderizam, input funciona
```

**Smoke do Enio é gate** antes de Phase C.

---

## 5. Gotchas observados em Phase A/B.1

- `cargo fmt --all` AUTO-RUN antes do commit. Pre-commit hook às vezes
  falha em fmt mesmo após `cargo check` verde. Solução: `cargo fmt --all`
  staged junto.
- Clippy `doc_lazy_continuation` é exigente — todo doc comment com lista
  continuada precisa indentação consistente. Falha o hook se errado.
- `ph2d-editor` consumers (panel crates, shell) NÃO precisam mudar nada
  durante B.5 graças ao shim. Só EM Phase C eles trocam pro caminho
  `ph2d_editor_core::*` direto.
- HR-12 a11y test (`tests/hr12_widgets_a11y.rs`) tem A11Y_OPT_OUT list.
  Quando mover files para `widget/*` ou `panel/*` (subdirs de widget),
  pode precisar de novo opt-out. Atualmente já cobre `panel_chrome.rs` +
  `showcase/*` da Phase 2.A.

---

## 6. Cadência v1.2 (mantenha)

- Edit burst até ~1200 LOC sem cargo check.
- `cargo check -p <crate>` opcional entre 400-1200.
- `cargo check -p <crate>` obrigatório acima de 1200 ou módulo inteiro.
- Pre-commit hook é a matriz oficial — não duplicar `cargo test --workspace`
  antes do commit.
- 1 commit por Phase. Phase B = 1 commit (ou natural break em sub-phases).
- Smoke do Enio ao fim do Phase B.

---

## 7. Após B.6 pass

- Phase C (migrate 4 painéis para `Panel<State>` trait) é nova sessão.
- Phase D (cleanup + arch tests + push) é nova sessão.

---

## 8. Quando este doc é obsoleto

Quando Phase B fecha e B.6 smoke passou. Aí Enio decide se deleta ou se
arquiva em `docs/Migracao/`.
