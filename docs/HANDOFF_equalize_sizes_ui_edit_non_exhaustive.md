# HANDOFF — `ph2d-tool-equalize-sizes` `EqualizeSizesUiEdit` `#[non_exhaustive]`

**Origem:** Painter T1.6 R9 audit, lens V1 (cross-tool consistency), finding V1-H2.
**Severidade:** HIGH (semver-additive boundary).
**Owner sugerido:** dono(a) do crate `ph2d-tool-equalize-sizes`.
**Status:** NÃO FIXADO — fora do escopo Painter.
**Data:** 2026-05-27.

---

## Resumo

`EqualizeSizesUiEdit` em [`crates/ph2d-tool-equalize-sizes/src/params.rs:139`](../crates/ph2d-tool-equalize-sizes/src/params.rs#L139) está sem `#[non_exhaustive]`. Mesma família de finding que padding/upscale/color-equalization (R9 V1-H2). Adicionar variant é breaking change pra downstream com match exhaustive.

## Reprodução

```bash
grep -B1 "pub enum EqualizeSizesUiEdit" crates/ph2d-tool-equalize-sizes/src/params.rs
# 138: #[derive(Copy, Clone, Debug, PartialEq, Eq)]
# 139: pub enum EqualizeSizesUiEdit {  ← sem #[non_exhaustive]
```

## Fix sugerido

```rust
/// **Audit T1.6 R9 V1-H2:** `#[non_exhaustive]` mirrors the
/// `BgRemovalUiEdit` precedent (R7 I1-1).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum EqualizeSizesUiEdit { ... }
```

Callers a checar: `crates/ph2d-panel-equalize-sizes/`, `crates/ph2d-tool-equalize-sizes/src/tool.rs::apply_ui_edit`, `shells/desktop/`.

## Por que não fixei

Scope creep. Painter T1.6 não toca equalize-sizes.

## Cross-ref

- Audit transcript: `/private/tmp/.../tasks/af31041832a38e98d.output` (R9 V1).
- Memory `feedback_audit_scope_discipline`.
- Companion HANDOFFs: `HANDOFF_padding_ui_edit_non_exhaustive.md`, `HANDOFF_upscale_ui_edit_non_exhaustive.md`, `HANDOFF_color_equalization_ui_edit_non_exhaustive.md`.
