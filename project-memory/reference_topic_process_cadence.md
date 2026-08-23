---
name: reference-topic-process-cadence
description: Cadência de processo e armadilhas de CI/ship (commit/CI/smoke/fase/toolchain/caps) — o gist já mora em CLAUDE.md §2-§3 (17)
metadata: 
  node_type: memory
  type: reference
  originSessionId: d2f2dbec-7784-4b38-bcf8-424045e2fd3c
  modified: 2026-08-23T00:59:14.815Z
---

- [[feedback_ci_handling]] — forneça o link da run; não fique em polling
- [[feedback_ci_batching]] — acumule commits; push único no fim
- [[feedback_commit_cadence]] — não comite a cada fix; acumule em blocos
- [[feedback_smoke_at_end]] — 1× no fim de TODA a implementação
- [[feedback_refactor_workflow]] — commits locais; Enio testa antes de push
- [[feedback_phase_cascade_2026_05_19]] — cada fase fecha + handoff + spawna a próxima (Modo C)
- [[feedback_codificacao_rapida]] — `cargo check -p <crate>`, nunca `--workspace`
- [[feedback_precommit_arch_gates]] — arch-gate antes de commit estrutural
- [[feedback_full_gate_periodically]] — re-lock cook hash ao mudar serialização
- [[feedback_ship_prep_no_fail_fast]] — `nextest --no-fail-fast` enumera TODAS
- [[feedback_ci_direct_lint_gates_and_fmt_skew]] — CI direto + fmt-skew: `rustup run <pin> cargo fmt`
- [[feedback_ship_committed_vs_worktree_wip]] — ship mede o COMMITTED: `git worktree --detach HEAD`
- [[project_ci_rustcache_stable_drift_pin]] — CI cold-build drift; pin `@1.95`
- [[feedback_ship_parity_gaps_ci_only]] — ship.sh ≠ paridade CI: bindgen/advisory-db escapam
- [[feedback_rustup_default_loses_to_the_toolchain_file]] — `rustup default` PERDE para o `rust-toolchain.toml`; meça com `RUSTUP_TOOLCHAIN`
- [[feedback_duplicate_allowlist_key_kills_the_gate_at_parse]] — allowlist duplicada mata o gate: o TOML morre no parse
- [[feedback_an_impacted_test_selector_that_maps_paths_by_prefix_is_blind_outside_it]] — seletor de impacto por prefixo é CEGO fora de `crates/` (curado 22/08: deriva de `cargo metadata`)
- [[project_diretriz_v68_2026_05_22]] — HISTÓRICO: o modelo de 2 papéis/Coordenador (DIRETRIZ v68) foi superseded no workstation pelo Modo L (ADR-0106/0107)
