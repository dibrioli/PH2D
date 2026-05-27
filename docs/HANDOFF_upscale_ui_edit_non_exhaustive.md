# HANDOFF — `ph2d-tool-upscale` `UpscaleUiEdit` `#[non_exhaustive]`

**Origem:** Painter T1.6 R9 audit, lens V1 (cross-tool consistency), finding V1-H2.
**Severidade:** HIGH (semver-additive boundary).
**Owner sugerido:** dono(a) do crate `ph2d-tool-upscale`.
**Status:** NÃO FIXADO — fora do escopo Painter.
**Data:** 2026-05-27.

---

## Resumo

Igual ao caso `PaddingUiEdit` — `UpscaleUiEdit` em [`crates/ph2d-tool-upscale/src/params.rs:149`](../crates/ph2d-tool-upscale/src/params.rs#L149) está sem `#[non_exhaustive]`. Adicionar variant é breaking change pra downstream com match exhaustive. `BgRemovalUiEdit` recebeu o tratamento em R7 I1-1 (commit `5f7680c`); padronizar.

## Reprodução

```bash
grep -B1 "pub enum UpscaleUiEdit" crates/ph2d-tool-upscale/src/params.rs
# 148: #[derive(Copy, Clone, Debug, PartialEq)]
# 149: pub enum UpscaleUiEdit {  ← sem #[non_exhaustive]
```

## Fix sugerido

```rust
/// **Audit T1.6 R9 V1-H2:** `#[non_exhaustive]` mirrors the
/// `BgRemovalUiEdit` precedent (R7 I1-1).
#[derive(Copy, Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum UpscaleUiEdit { ... }
```

Verificar callers: `crates/ph2d-panel-upscale/`, `crates/ph2d-tool-upscale/src/tool.rs::apply_ui_edit`, `shells/desktop/`.

## Por que não fixei

Scope creep — vide HANDOFF companion. Painter T1.6 implementor não toca tool-upscale.

## Cross-ref

- Audit transcript: `/private/tmp/.../tasks/af31041832a38e98d.output` (R9 V1).
- Memory `feedback_audit_scope_discipline`.
- Companion HANDOFFs: `HANDOFF_padding_ui_edit_non_exhaustive.md`, `HANDOFF_color_equalization_ui_edit_non_exhaustive.md`, `HANDOFF_equalize_sizes_ui_edit_non_exhaustive.md`.
