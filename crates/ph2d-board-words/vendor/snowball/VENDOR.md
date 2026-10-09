# Vendored Snowball 3.1.1 — English and Portuguese stemmers (Rust)

- **Upstream:** https://github.com/snowballstem/snowball, tag `v3.1.1`
  (`snowball-3.1.1.tar.gz`, sha256 `d8714aa91ed4333654708472a7a98b529c867a8f99b05c5e66febf4ca72c44c7`).
- **Licence:** BSD-3-Clause (`COPYING`, © Dr Martin Porter, Richard Boulton, Yoshiki Shibukawa, Olly Betts).

## What is here and how it was made

- `src/snowball/among.rs`, `src/snowball/snowball_env.rs`, `src/snowball/mod.rs` — copied verbatim from
  `rust/src/snowball/` of the tarball (the Rust runtime of the generated code).
- `src/snowball/algorithms/english_stemmer.rs`, `portuguese_stemmer.rs` — the compiler's own output,
  untouched: `make snowball && ./snowball algorithms/<lang>.sbl -rust -o <lang>_stemmer`.
- `src/snowball/algorithms/mod.rs` and `src/lib.rs` — hand-written (upstream builds the module list in a
  `build.rs`; here it is two lines, plus the `stem()` entry point).
- `edition = "2015"`: the generated code says `use snowball::SnowballEnv;` from the crate root.
- `[workspace]` (empty) in `Cargo.toml`: it is its own workspace root, like the vendored `deep_filter`.
- `rustfmt.toml` with `disable_all_formatting = true`: `cargo fmt --all` reaches path dependencies, and the
  generated code must stay exactly what the compiler emitted.

## Why vendored (and why this version)

Measured 2026-10-09 against the installed `libstemmer` 3.1.1 (Arch `libstemmer`, BSD-3) over the official
Snowball vocabularies (`snowball-data`: 42 649 English, 32 016 Portuguese words): `rust-stemmers` 1.2.0
differs in 57 English words, `waken_snowball` 0.1.0 in 17 (all `inter*`, the 3.1 rule); this code differs
in **0** in both languages. The gate is `crates/ph2d-board-words/tests/it/stems_match_libstemmer.rs`.
Study: `docs/MiroClone/05_facilitacao.md` §1.3.
