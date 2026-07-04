---
name: feedback-ci-direct-lint-gates-and-fmt-skew
description: "Push-direto-sem-ship.sh → rode os gates de lint local antes; e `cargo fmt` plain usa toolchain default, não a pinada do CI (skew)"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 08f6a613-4a63-4a4e-8305-1b658212543e
---

Quando o Enio pede **"CI no git direto (sem script ship)"** (ship de fim de sessão pulando o ship.sh de ~10min), o CI pega os gates de lint **em série** (1 vermelho → fix → re-push → ~18min/ciclo). Na sessão 2026-06-01 foram 3 ciclos vermelhos seguidos (machete → typos → fmt-skew) antes do verde.

**How to apply:** antes de um push-direto, rode os gates de lint LOCAL de uma vez (são segundos, não os ~10min do ship.sh completo): `cargo machete` + `typos` + `rustup run <pin> cargo fmt --check --all` (+ clippy --all-targets nas crates tocadas). Pega os 3 de uma vez em vez de 3 ciclos de CI.

**clippy `-D warnings` (armadilha confirmada no ship 2026-06-06):** `cargo clippy -p <crate>` PLAIN trata `type_complexity` (e outros warn-by-default) como WARNING → passa. O CI/ship.sh roda `clippy ... -- -D warnings` → vira ERRO. Shipei código `clippy -p`-limpo que reprovou no ship por 2 `type_complexity` (tupla `(Arc<Vec<u8>>,u32,u32)` num thread_local; `fn([f32;3])->[f32;3]` num `&[(...)]`) — fix = `type Alias = ...`. **Sempre rode clippy local com `-- -D warnings`** (igual o CI). Outra armadilha do mesmo `-D warnings` (2026-06-06): `doc_lazy_continuation` — uma linha de doc-comment (`///` ou `//!`) começando com `+`/`-`/`*` é lida como bullet markdown → as linhas seguintes reprovam como "doc list item without indentation"; reescreva sem o char inicial (ex.: `+ an above layer` → `plus an above layer`). E pro batch grande, `nextest --no-fail-fast` enumera TODAS as falhas de teste de uma vez (o ship.sh/nextest é fail-fast: 1 por ciclo) — vide [[feedback_ship_prep_no_fail_fast]].

**Why (a armadilha do fmt-skew — não-óbvia):** `cargo fmt` plain usa a toolchain **default** (`stable`), NÃO a `channel` do `rust-toolchain.toml` que o CI usa — mesmo quando o rustup mostra a pinada "active". As duas versões de rustfmt discordam em casos de borda (ex.: quebrar um `assert_eq!` de ~100 chars). Resultado: `cargo fmt --all` local fica "limpo" mas o `cargo fmt --check` do CI (toolchain pinada) reprova. **Sempre formate com `rustup run <channel> cargo fmt --all`** (a versão exata do CI), não `cargo fmt`. Confirme com `rustup run <channel> cargo fmt --check --all` antes do push.

**Armadilha correlata (confirmada 2026-07-02, custou 1 ciclo de CI):** rodar `rustfmt <arquivo>` DIRETO (ex.: `rustup run 1.95 rustfmt --edition 2024 foo.rs`) IGNORA o `rustfmt.toml` do projeto (`style_edition = "2024"`, `max_width = 100`) — `--edition` não é `style_edition`. Resultado: uma linha de re-export de ~101 chars ficou numa linha só (rustfmt-direto aceita), mas o `cargo fmt --check` do CI (lê o config → style_edition 2024) quebra a linha e reprova. Fix: **NUNCA `rustfmt <arquivo>` avulso; sempre `cargo fmt` (com o pin)** — só o `cargo fmt` lê o `rustfmt.toml`.

`typos` (gate de CI) flagra falsos-positivos em comentários pt-BR (ex.: `continuem`→`continuum`); reescreva a palavra (evite o stem com near-match em inglês) ou adicione ao allowlist `.typos.toml`. machete flagra dep direto que na verdade chega via re-export de outra crate (ex.: `BlendMode` via `ph2d_tool_painter` em vez de `ph2d_painter_brush` direto) — remova o dep direto. Relacionado: [[feedback_fast_mode_ship]] [[feedback_ci_handling]] [[feedback_parallel_agent_collision]].
