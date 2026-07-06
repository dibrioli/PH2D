---
name: feedback_ship_parity_gaps_ci_only
description: ship.sh não é 100% paridade com CI — 3 blind spots que passam local e vermelham no CI
metadata:
  type: feedback
---

`./scripts/ship.sh` verde NÃO garante CI verde. Três blind spots reais, todos batidos no ship do cutover Vector (ADR-0108, 2026-07-06):

1. **`ph2d-bindgen --check` (HR-10 parity) NÃO está no ship.sh.** O job `lint` do CI roda `cargo run -p ph2d-bindgen --locked -- --check`, que compara `runtime/luau/ph2d.d.luau` + `runtime/mcp/schema.json` com o gerado. Deletar/alterar tools MCP (ex.: cutover removeu `vector.*`) sem regenerar esses artefatos = drift → CI vermelho. **Fix:** `cargo run -p ph2d-bindgen -- --write` e commitar os 2 arquivos.

2. **`~/.cargo/advisory-db` local envelhece → cargo-audit/cargo-deny locais mentem verde.** O CI faz fetch fresco e pega RUSTSEC novos (ex.: RUSTSEC-2026-0204 crossbeam-epoch <0.9.20, invalid pointer deref). Antes de confiar no audit/deny local, `git -C ~/.cargo/advisory-db pull --ff-only`; reproduza o deny do CI com `cargo deny --all-features check` (o `--all-features` é o que o CI usa). Advisory novo unrelated ao diff = fix por bump (`cargo update -p <crate>`) ou ignore em `deny.toml`.

3. **`scripts/nextest-impacted.sh` quebra em cutover que DELETA crates.** Ele deriva nomes de pacote do diff `main...HEAD` e passa `rdeps(<crate-deletada>)` pro nextest → "operator didn't match any packages" (exit 94). O `foundational-integrate.sh` usa ele no passo 5. **Contorno:** rode `cargo nextest run --workspace` direto (check mais forte) e faça o `git merge --ff-only` manual. **Follow-up aberto:** filtrar os nomes contra `cargo metadata` vivo antes do rdeps.

**Why:** os 3 escaparam do gate local e só apareceram no CI (~30min/ciclo), custando pushes extras.

**How to apply:** num diff que mexe em MCP/tools → rode bindgen --write. Antes de ship de fim-de-jornada → refresh advisory-db + `cargo deny --all-features check`. Em teardown grande → nextest --workspace + ff manual. Ver [[project_vector_cutover_adr0108]], [[feedback_ci_direct_lint_gates_and_fmt_skew]], [[feedback_full_gate_periodically]].
