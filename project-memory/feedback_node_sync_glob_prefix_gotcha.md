---
name: feedback_node_sync_glob_prefix_gotcha
description: A helper crate in the node area must NOT start with ph2d-node- or node-sync auto-registers it and breaks registry-init
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 9a06dd51-73b4-4fc3-abe4-4ed101c3ae24
---

`ph2d-node-sync` (tools/ph2d-node-sync) scans **`crates/ph2d-node-*`** and emits a
`<crate>::register(reg)?;` line into `ph2d-node-registry-init` for each — excluding
ONLY `ph2d-node-registry` and `ph2d-node-registry-init`. So any crate whose name
starts with `ph2d-node-` is assumed to be a node with a `pub fn register`.

**Why:** a W4 audit/harness crate named `ph2d-node-vector-audit` would match the
glob; node-sync would generate `ph2d_node_vector_audit::register(reg)?;` to a
function that doesn't exist → `registry-init` fails to compile.
**How to apply:** name non-node helper/audit/bench crates **outside** the
`ph2d-node-` prefix (e.g. `ph2d-vector-fanout-audit`). Only real nodes (a
`pub fn register` + a `MANIFEST`) get the `ph2d-node-vector-<slug>` name. Same
spirit as the tool-sync friction note [[feedback_fanout_registry_init_friction]].
Run `cargo run -p ph2d-node-sync` after adding a node and check the wired count.
