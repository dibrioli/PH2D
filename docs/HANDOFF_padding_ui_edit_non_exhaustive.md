# HANDOFF — `ph2d-tool-padding` `PaddingUiEdit` `#[non_exhaustive]`

**Origem:** Painter T1.6 R9 audit, lens V1 (cross-tool consistency), finding V1-H2.
**Severidade:** HIGH (semver-additive boundary).
**Owner sugerido:** dono(a) do crate `ph2d-tool-padding`.
**Status:** NÃO FIXADO — fora do escopo Painter.
**Data:** 2026-05-27.

---

## Resumo

`BgRemovalUiEdit` recebeu `#[non_exhaustive]` em R7 (commit `5f7680c`, audit I1-1) pra permitir variants adicionais sem semver-break em downstream `match`. **`PaddingUiEdit` não recebeu o mesmo tratamento.** Inconsistência: adicionar variant em bgremoval é safe, em padding é breaking change pra downstream que faz match exhaustive.

## Reprodução

```bash
grep -B1 "pub enum PaddingUiEdit" crates/ph2d-tool-padding/src/params.rs
# 73: #[derive(Copy, Clone, Debug, PartialEq, Eq)]
# 74: pub enum PaddingUiEdit {  ← sem #[non_exhaustive]

grep -B1 "pub enum BgRemovalUiEdit" crates/ph2d-tool-bgremoval/src/params.rs
# (mostra #[non_exhaustive] já presente)
```

## Fix sugerido

Adicionar `#[non_exhaustive]` ao enum + comentário citando o precedente R7 I1-1:

```rust
/// **Audit T1.6 R9 V1-H2:** `#[non_exhaustive]` mirrors the
/// `BgRemovalUiEdit` precedent (R7 I1-1). Adding new variants in
/// future waves is no longer a semver-breaking change for downstream
/// crates that exhaustively `match`. Tool-author-side `match` arms
/// inside this crate still see all variants.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum PaddingUiEdit { ... }
```

Verificar callers em `crates/ph2d-panel-padding/`, `shells/desktop/`, `crates/ph2d-tool-padding/src/tool.rs::apply_ui_edit` — se algum match exaustivo existir, adicionar arm `_ => {}` ou deixar como erro pendente (clippy `non_exhaustive_omitted_patterns` lint, opt-in).

## Por que não fixei

R9 escopo creep — toquei 5 UiEdit enums (`PaddingUiEdit`, `UpscaleUiEdit`, `EqualizeSizesUiEdit`, `ColorEqualizationUiEdit`, `PainterUiEdit`) e Enio apontou. Mantive só o `PainterUiEdit` (escopo Painter); reverti os outros 4 via `git checkout HEAD --`.

## Cross-ref

- Audit transcript: `/private/tmp/.../tasks/af31041832a38e98d.output` (R9 V1).
- Memory `feedback_audit_scope_discipline`.
- Companion HANDOFFs: `HANDOFF_upscale_ui_edit_non_exhaustive.md`, `HANDOFF_color_equalization_ui_edit_non_exhaustive.md`, `HANDOFF_equalize_sizes_ui_edit_non_exhaustive.md`.
