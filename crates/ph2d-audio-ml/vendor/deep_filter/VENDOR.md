# Vendored `deep_filter` (`libDF`) — DeepFilterNet v0.5.6

This directory is a **vendored copy** of the `libDF` crate from
[DeepFilterNet](https://github.com/Rikorose/DeepFilterNet), tag **`v0.5.6`**
(commit `978576aa8400552a4ce9730838c635aa30db5e61`).

- **Upstream:** https://github.com/Rikorose/DeepFilterNet, `libDF/`
- **Licence:** MIT OR Apache-2.0, © 2021 Hendrik Schröter (see `LICENSE-MIT`, `LICENSE-APACHE`).
- **Model:** `models/DeepFilterNet3_onnx.tar.gz` (7.6 MB) is the upstream default DFN3
  weights, which upstream itself embeds via `include_bytes!` in its own MIT/Apache
  crate and ships in its releases and distro packages — redistribution verified
  (ADR-0123 §3.4).

## Why vendored (not a git dependency)

The workspace is hermetic on purpose: `deny.toml` sets `unknown-git = "deny"` with
an empty `allow-git`, and the ADR-0123 decision to reject `ort` was largely about
not letting the build reach the network. A git dependency on a `-pre`-adjacent tag
would put CI at the mercy of GitHub. Vendoring is the same pattern
`ph2d-audio-opus` uses for a sensitive dependency, and it is what ADR-0123 §3.6 /
§5 prescribe for closing W7.

## What was changed from upstream

Only the manifest and one path — **no source logic was touched**:

- `Cargo.toml` trimmed to the `tract` + `default-model` runner path (plus the
  `transforms`/`logging` they pull). Dropped: the `dataset`/`bin`/`vorbis`/`flac`/
  `capi`/`wav-utils` features and their dependencies — including the crate's **one
  git dependency, `hdf5`** (which alone would violate the hermetic policy).
  `[[bin]]` targets and packaging metadata removed. `unexpected_cfgs = "allow"`
  added so the trimmed feature set does not warn on the dead `cfg` blocks.
- `src/tract.rs`: the model `include_bytes!` path `../../models/…` → `../models/…`,
  because the model now lives inside this crate directory rather than at the
  DeepFilterNet repo root.

The `src/*.rs` files for the dropped features (`dataset.rs`, `dataloader.rs`,
`augmentations.rs`, `hdf5_key_cache.rs`, `capi.rs`, `util.rs`, `wav_utils.rs`) are
kept for completeness but are `cfg`'d out and never compiled with our feature set.

## Updating

Re-vendor from a new tag by copying `libDF/src/*.rs` + `models/DeepFilterNet3_onnx.tar.gz`
+ the two `LICENSE-*` files, then re-applying the two changes above. Confirm the
parity gate (`crates/ph2d-audio-ml/tests/parity_with_reference_cli.rs`) still holds.
