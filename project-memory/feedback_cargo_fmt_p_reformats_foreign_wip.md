---
name: feedback_cargo_fmt_p_reformats_foreign_wip
description: "cargo fmt -p <crate> reformats ALL files in the crate, incl. other agents' uncommitted WIP — touches foreign files"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 08f6a613-4a63-4a4e-8305-1b658212543e
---

`rustup run <pin> cargo fmt -p <crate>` reformats **every file in that crate**, not just the ones you edited — including another agent's **uncommitted WIP** in the same crate. In the shared multi-agent tree this silently modifies foreign files (e.g. fmt'ing the shell crate touched the Vector impl's live `vector_scene.rs` / `vector_inspector_bridge.rs`).

**Why:** rustfmt operates per-crate (per the crate's module tree), not per-file-list. There's no built-in "format only my staged files."

**How to apply:**
- When a crate has foreign uncommitted WIP (check `git status` first), format ONLY your own files: `rustfmt <path/to/my_file.rs> ...` (rustfmt takes explicit file paths), NOT `cargo fmt -p`.
- ⚠️ **`rustfmt <files>` only limits scope if NONE of them is a crate root.** rustfmt follows the `mod` tree from every file it is given, so passing `src/lib.rs` or `src/main.rs` formats the **whole crate** — exactly what you were avoiding. Measured 2026-07-28 (`line/anim`): `rustfmt … lib.rs main.rs …` swept 5 unrelated files into the working tree, 3 of which reached a feature commit before `git show --stat` caught them. Pass leaf files only; if you need a crate root formatted, expect the cascade and check `git status` right after.
- After ANY fmt, `git status -s` before `git add`, and never `git add -- <dir>/` (a directory pathspec re-collects whatever the cascade touched).
- Damage control after an accidental `cargo fmt -p`:
  - A file that was **clean at HEAD** before fmt → `git checkout -- <file>` is safe (restores HEAD).
  - A file that was **already `M`** (foreign WIP) → do NOT `git checkout` (destroys their WIP). rustfmt changes are non-semantic whitespace, so leaving it is the least-harmful option; flag it. You can't restore their exact pre-fmt WIP (no snapshot).
- Never stage the foreign files — keep your commit pathspec'd to your own files ([[feedback_scoped_commit_shared_index]]).

Related: [[feedback_destructive_git_outside_pasta]], [[feedback_parallel_agent_collision]], [[feedback_ci_direct_lint_gates_and_fmt_skew]] (the `rustup run <pin>` part is still needed to avoid toolchain skew).
