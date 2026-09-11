---
description: Batched, 1× sobre o diff. Sem consertar antes de mostrar a lista.
argument-hint: [Crate ou paths]
---
Rode o gate batched de fechamento sobre o diff acumulado, 1× só:

1. `bash scripts/nextest-impacted.sh` (o perfil `ci-test` tem `incremental = false` e o
   `.config/nextest.toml` tem tecto global de 180 s e `fail-fast = false` desde 10/09 —
   os prefixos `CARGO_INCREMENTAL=0` e `--no-fail-fast` deixaram de ser necessários)
2. clippy `--all-targets` + features
3. `shells/desktop/tests/file_loc_caps.rs` + `arch_safe_clamp_only`
4. auditoria ≥2 lentes

Para reconferir UM crate durante o conserto, use `bash scripts/cargo-test-narrow.sh
<crate>` — não o `cargo test` cru com filtro à mão.

Escopo: $1

Reporte cada ✗ com o comando exato que o reproduz. NÃO conserte nada antes de me
mostrar a lista inteira.
