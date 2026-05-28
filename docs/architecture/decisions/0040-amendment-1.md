# ADR-0040 Amendment 1 — `EditorAction` cap-bump 4 → 5 (Vector Module `VectorOp` variant)

**Status:** Accepted (2026-05-29)
**Amenda:** [ADR-0040 — Tool-as-isolated-feature-crate](0040-tool-as-isolated-feature-crate.md).
**Decisor(es):** Enio + Claude (Coord-A, sessão Vector W0).
**Triggered by:** [ADR-0057 — Vector edit dispatch + CRDT](0057-vector-edit-dispatch-crdt.md) §2.1.
**Tags:** amendment, vector, contract, cap-bump

---

## 1. Contexto

ADR-0040 §7 congelou `EditorAction` enum com **cap = 4 variants** (`ActivateTool`, `OneShotImageOp`, `ToolPanelEvent`, `CancelActiveTool`). Vector Module W0 requires variant novo `VectorOp(VectorOp)` para dispatchar mutations do data model VectorNetwork (vide ADR-0057 §2.1).

Antigravity 2ª iter rejected `ToolPanelEvent` reuse para vector ops (payload size de `VectorOp::AddRegion { segments: SmallVec<[(SegmentId, bool); 16]> }` não cabe em PanelEvent key-value simple model).

---

## 2. Amendment

### 2.1 Cap-bump `EditorAction` 4 → 5

```rust
// ph2d-editor-core::action_bus
#[non_exhaustive]
pub enum EditorAction {
    ActivateTool(ToolId),
    OneShotImageOp(ImageOpRequest),
    ToolPanelEvent(PanelEvent),
    CancelActiveTool,
    VectorOp(VectorOp),         // NEW — Vector Module 2026-05-29
}
```

Arch-gate `architecture_tool_contract_surface` em `crates/ph2d-editor-core/tests/architecture_tool_contract_surface.rs` cap bumped: `EditorAction = 5` (era 4).

### 2.2 Caps preservados (sem amendment)

- `Tool = 10` ✓ preserved.
- `RasterEditTool = 5` ✓ preserved.
- `PanelEvent = 4` ✓ preserved.

Apenas `EditorAction` recebeu +1 variant.

### 2.3 Justificativa do amendment

Reuse de `ToolPanelEvent::SetValue|Click(id, value)` era considerado mas payload de `VectorOp` (segments + tangents + style refs) excede key-value model. Variant novo é solução canônica para multi-payload action types — padrão estabelecido em outros enum bumps PH2D.

---

## 3. Consequências

### 3.1 Positivas

- **Vector Module dispatch consistente** com pattern PH2D (variant per major action category).
- **Type safety preserved** — `VectorOp` é typed enum (ADR-0056 §2.8), pas variant inline.

### 3.2 Negativas

- **Recompilation tooling existing** (ph2d-tool-sync, etc.) — `EditorAction` `#[non_exhaustive]` mitiga via match arms.
- **CI gate update** obrigatório — `architecture_tool_contract_surface` cap valor.

---

## 4. Implementação (Wave 1)

- **T1.2** (W1): `ph2d-vector-doc::edit_log::VectorOp` enum definição (ADR-0056 §2.8).
- **T1.X**: Bump cap em `crates/ph2d-editor-core/tests/architecture_tool_contract_surface.rs`.
- **T1.Y**: Update dispatch handlers em chrome handlers + `apply_event` consumers.

---

## 5. Referências

- ADR-0040 §7 cap original.
- ADR-0057 §2.1 (triggering decision).
- Vector Module README §11.B + §11.C (3 iterações Antigravity).
