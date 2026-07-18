---
name: feedback-token-rewrite-scopes-to-changed-files-not-whole-tree
description: "A token-scoped rewrite (ADR renumber) must sed only the LINE's changed files, never a whole-tree git grep — and verify at the SAME scope you mutate."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 88c383b7-922d-471c-9895-e2eee9929d56
---

Renumbering colliding ADRs during a 6-line integration (0122/0123 → 0126/0127 for gpu,
→ 0128/0129 for vector) I ran `git grep -lE '012[23]' | xargs sed -i 's/0122/.../'` over
the **whole worktree**. It rewrote `0122`/`0123` tokens that were NOT ADR references:
`InterVariable.ttf` (a **font binary** — same length, so no size change to notice), a
font-diagnostic test string (`"Inspector Hierarchy 0123"`), a **blake3 hash** in an audit
note, a UI mockup label, and a **session UUID** (`9af6224e-0122-…`). gpu's renumber landed
that damage into `main`; only the Vector rebase (which hit the same tokens) surfaced it.

The kill was that my **verification used a narrower scope than my mutation**: I checked
"all 012[23] are ADR refs" over `git diff --name-only main...HEAD` (the line's *changed*
files — all clean), but the sed ran over the whole tree, so inherited files were never
in the verification set. A clean check passed over a dirty mutation.

**Why:** a 4-digit token is not unique to your ADR. Fonts, UUIDs, hashes, and prose all
carry `0122`. A whole-tree grep+sed treats a coincidence as a reference; on a binary it
corrupts silently (no compile error — the .ttf is an asset, not code — and same-length
replacement keeps the file size identical). It nearly shipped.

**How to apply:** scope the rewrite to the files the line actually authored —
`git diff --name-only <merge-base>...HEAD -- ':!Cargo.lock' ':!*.ttf' <exclude binaries>` —
never `git grep` over the tree. Exclude binary/asset paths explicitly. Then run the SAME
scope in your "no false positives remain" check: if you verify a subset of what you mutate,
the check is theater. Related: [[feedback_sed_relative_path_hits_primary_cwd]] (mutate by
absolute path) · [[feedback_clean_text_merge_can_be_semantically_broken]] (the combined-tree
`check --workspace` is what actually catches the damage — it surfaced this on the Vector rebase).
