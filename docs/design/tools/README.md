# Design canonical tool TOMLs

Wave 2 PR 11.5. Source-of-truth for **tool functionality** — the
designer (Claude Design / Enio) edits the `.toml` here; the matching
`ToolManifest` const in `crates/ph2d-tool-<slug>/` mirrors the
declaration in Rust.

CI (`tests/tool_manifest_design_sync.rs` in
`ph2d-tool-registry-init`) compares each `.toml` against the
registered `MANIFEST` field-by-field. Divergence fails the workspace
test with a clear diff.

## Authoring a new tool

1. Drop `docs/design/tools/<slug>.toml` here (use any of the four
   committed files as a template).
2. Create `crates/ph2d-tool-<slug>/` per ADR-0027 / Appendix A of the
   migration plan; populate `MANIFEST` to match the TOML.
3. Add one line to `ph2d-tool-registry-init::register_all`.
4. CI runs `tool_manifest_design_sync` and confirms parity.

## Schema (frozen as of Wave 2 PR 11.5)

```toml
[tool]
id          = "<snake_case slug — manifest.id>"
cluster     = "<image_tools | settings | ...>"
zone        = "top_right" | "sidebar" | "center" | "top_left"
order       = <u32 — sort key inside (cluster, order, id)>
a11y_role   = "Button" | "ToggleButton" | "MenuItem" | ...
icon_slug   = "<docs/design/icons/<icon_slug>.svg basename>"
touches_sim = true | false

[label]
pt_br_inline = "Display label (pt-BR fallback until Fluent lands)"
en_us_inline = "Display label (en-US fallback)"
fluent_key   = "tool.<id>.label"

[memory_budget]
vram_mb         = <u32>
ram_mb          = <u32>
heap_script_mb  = <u32>
```

The cross-validation test ignores `[label].*_inline` (those exist for
the eventual Fluent bundle activation per HR-15 deferred) — only
`fluent_key` participates in the manifest comparison.
