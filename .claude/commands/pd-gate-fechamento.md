---
description: Batched, 1× sobre o diff. Sem consertar antes de mostrar a lista.
argument-hint: [Crate ou paths]
---
Rode o gate batched de fechamento sobre o diff acumulado, 1× só:

1. `CARGO_INCREMENTAL=0 bash scripts/nextest-impacted.sh` (o perfil `ci-test` roda em
   BATCH; incremental não colhe nada ali e paga 11 GB)
2. clippy `--all-targets` + features
3. `shells/desktop/tests/file_loc_caps.rs` + `arch_safe_clamp_only`
4. auditoria ≥2 lentes

Para reconferir UM crate durante o conserto, use `bash scripts/cargo-test-narrow.sh
<crate>` — não o `cargo test` cru com filtro à mão.

Escopo: $1

Reporte cada ✗ com o comando exato que o reproduz. NÃO conserte nada antes de me
mostrar a lista inteira.
