---
name: feedback-worktree-agent-stale-base
description: "Agent worktree isolation branches from session-start HEAD, not current HEAD — unusable for work building on same-session commits"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: f0521d79-636c-4f46-a1fa-5c161bcf6d2e
---

`Agent(isolation: "worktree")` creates the worktree branch from the
**session-start HEAD**, NOT the current HEAD at dispatch time. In a long
session where the Coordenador commits mid-session (e.g. `e344b82`) and
then dispatches worktree agents, those agents branch from the OLD base
(the session's initial commit, e.g. `b59381d`) and never see the
mid-session commit's changes.

**Why it bit us:** dispatched per-panel editing agents AFTER committing a
big UI-consolidation base. Their worktrees branched from the pre-commit
HEAD, so their files LACKED all the committed work. Copying their files
back into main silently REVERTED the committed changes (lost
`paint_panel_title` migration, segmented delegation, etc.). Caught only
by grepping for an expected committed change after the copy.

**How to apply:**
- Before copying a worktree agent's file back, verify the worktree's base
  with `git -C <worktree> log --oneline -1` / `merge-base` — confirm it
  includes your latest commit.
- For work that builds on same-session commits, EITHER push/share the
  commit so worktrees branch from it, OR don't use `isolation: "worktree"`
  (run agents on the current tree, sequentially to avoid `target/` lock +
  git-index collision), OR just do the work in the main session.
- The parallel agent group is still great for **read-only audits** (no
  base sensitivity) — that's where it shone here.

Related: [[feedback_parallel_agent_collision]].
