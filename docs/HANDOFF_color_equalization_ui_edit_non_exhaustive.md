# HANDOFF — `ph2d-tool-color-equalization` `ColorEqualizationUiEdit` `#[non_exhaustive]`

**Origem:** Painter T1.6 R9 audit, lens V1 (cross-tool consistency), finding V1-H2.
**Severidade:** HIGH (semver-additive boundary).
**Owner sugerido:** dono(a) do crate `ph2d-tool-color-equalization`.
**Status:** NÃO FIXADO — fora do escopo Painter.
**Data:** 2026-05-27.

---

## Resumo

`ColorEqualizationUiEdit` em [`crates/ph2d-tool-color-equalization/src/params.rs:521`](../crates/ph2d-tool-color-equalization/src/params.rs#L521) está sem `#[non_exhaustive]`. Esse enum é especialmente sensível porque o crate está em desenvolvimento ativo (recentes commits adicionaram `DenoiseMethod` Domain Transform, A-Trous Wavelet, Wavelet Shrinkage, Total Variation, Anisotropic Diffusion, Guided Filter — vide `git log --oneline -- crates/ph2d-tool-color-equalization/`). Cada novo método tipicamente adiciona variants ao UI edit set; cada um é breaking change pra downstream sem o `non_exhaustive`.

## Reprodução

```bash
grep -B1 "pub enum ColorEqualizationUiEdit" crates/ph2d-tool-color-equalization/src/params.rs
# 520: #[derive(Copy, Clone, Debug, PartialEq)]
# 521: pub enum ColorEqualizationUiEdit {  ← sem #[non_exhaustive]
```

## Fix sugerido

```rust
/// **Audit T1.6 R9 V1-H2:** `#[non_exhaustive]` mirrors the
/// `BgRemovalUiEdit` precedent (R7 I1-1). Especially relevant for
/// this enum given the ongoing denoise-method expansion (Domain
/// Transform, A-Trous Wavelet, Wavelet Shrinkage, ...).
#[derive(Copy, Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum ColorEqualizationUiEdit { ... }
```

Callers a checar: `crates/ph2d-panel-color-equalization/` (atualmente em WIP — vide nota abaixo), `crates/ph2d-tool-color-equalization/src/tool.rs::apply_ui_edit`, `shells/desktop/`.

**Nota:** `ph2d-panel-color-equalization` está com build quebrado em main hoje (campo `denoise_method` referenciado mas removido do snapshot). Coordenar o fix do non_exhaustive com a estabilização da panel.

## Por que não fixei

Scope creep. Painter T1.6 não toca color-equalization.

## Cross-ref

- Audit transcript: `/private/tmp/.../tasks/af31041832a38e98d.output` (R9 V1).
- Memory `feedback_audit_scope_discipline`.
- Companion HANDOFFs: `HANDOFF_padding_ui_edit_non_exhaustive.md`, `HANDOFF_upscale_ui_edit_non_exhaustive.md`, `HANDOFF_equalize_sizes_ui_edit_non_exhaustive.md`.
