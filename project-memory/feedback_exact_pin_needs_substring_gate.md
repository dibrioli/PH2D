---
name: feedback-exact-pin-needs-substring-gate
description: "Exact-pin `=X.Y.Z` em Cargo.toml não sobrevive merge/rebase sem arch-gate substring test que assert o `=` prefix"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 2145cc4f-66b3-4eb1-b4ee-05d0486ac094
---

Toda vez que um workspace adota EXACT-PIN discipline (`postcard = "=1.1.3"`, `libm = "=0.2.16"`) por motivo HR-5 (cross-OS bit-identical wire/output), o `=` prefix é **frágil** — agente futuro rebasing merge pode "limpar" pra `"1.1.3"` (caret) achando que é overpinning. Cargo.lock floats; HR-5 contract quebra silenciosamente; só CI matrix cross-OS detecta (RED depois do push).

**Why:** O `=` é 1 caractere ASCII que parece estético; semverr docs raramente enfatizam (`=` vs caret = single-version vs caret-range). Sem gate, código segue compilando com caret + Cargo.lock pré-resolvido — a próxima `cargo update` floats. Auditor cego pra esse charset-level invariant. T1.3.5 R2 (Lens E-C3) flagaram o gap depois do cef1959 ter introduzido o padrão pro postcard mas NÃO ter sido replicado pro libm em 5974a84.

**How to apply:**
- **Toda vez que ship um EXACT-PIN dep**, ship junto um arch-gate test em `tests/<crate>/architecture_<dep>_exact_pin.rs` (ou inline em test file já existente) que:
  ```rust
  let toml = include_str!("../Cargo.toml");  // ou std::fs::read_to_string em workspace-scan
  assert!(toml.contains(r#"<dep> = { version = "=X.Y.Z""#) || matches_libm_pattern(toml),
          "<dep> exact-pin (`=X.Y.Z`) lost — HR-5 contract requires `=`");
  ```
- **Workspace scan variant:** se múltiplos crates dep do mesmo primitivo (libm em 5 crates p.ex.), gate único em UM crate canônico que itera `[(path, label), ...]` e asserta substring em CADA Cargo.toml. Pattern em `crates/ph2d-ecs/tests/transform_determinism.rs::libm_exact_version_pin_enforced_in_workspace`.
- **Tolerate formatting:** column-padding (`libm                  = { ... }`) pode quebrar `toml.contains(r#"libm = { ..."#)`; usar `.lines().find(|l| l.trim_start().starts_with("libm") && ...)` ou regex tolerante a whitespace.
- **Tolerate features:** assert `default-features = false` separadamente (substring presence) se feature flags são parte do contrato (libm `arch` feature defeats determinism). Cada feature flag relevante = uma assertion separada.
- **Recorda do que existe:** `postcard_exact_version_pin_enforced_in_cargo_toml` em `crates/ph2d-render/tests/sprite_versioned_postcard.rs` é template; replicate quando ship novo exact-pin.

**Reference:** sessão Sprite Inspector v2 2026-05-28; T0.13 estabeleceu padrão postcard, T1.3.5 R2 catch-up estabeleceu libm.
