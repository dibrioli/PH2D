# Vendored wgpu-core 30.0.1 + backport of gfx-rs/wgpu#10160

- **Upstream:** crates.io `wgpu-core` 30.0.1 (https://github.com/gfx-rs/wgpu, tag `v30.0.1`), copied
  verbatim from the registry source (minus `Cargo.lock` and `Cargo.toml.orig`).
- **Licence:** MIT OR Apache-2.0 (`LICENSE.MIT`, `LICENSE.APACHE`, © the wgpu developers).
- **Wired by** `[patch.crates-io]` in the workspace `Cargo.toml`; excluded from the workspace (our
  clippy/fmt/deny don't police it — `rustfmt.toml` disables formatting, `[workspace]` makes it its own root).

## What changed against 30.0.1

The fix of gfx-rs/wgpu#10160 (*«fix(core): Clean up resources after failed initialization»*, commit
`818586c4d1a8e3529e3db016e68260f0a76f3971`, merged 2026-09-01 — not in any release up to 30.0.1):

- `src/indirect_validation/{mod,dispatch,draw}.rs` — the upstream diff, applied as is.
- `src/device/resource.rs` — upstream's `Device::new` had diverged (it went through an intermediate
  `DeviceResources`), so the same idea is written by hand on 30.0.1: the fence, the zero buffer, the
  empty BGL, the external-texture params buffer and the indirect validation are held in
  `scopeguard::guard`s and destroyed if `Device::new` returns early; defused just before `Ok(Self { … })`.
- `Cargo.toml` — `scopeguard = "1.2"` (as upstream), an empty `[workspace]`, and
  `unfulfilled_lint_expectations = "allow"`: as a path dependency `CARGO_BUILD_WARNINGS=deny` (CI) sees
  the `expect(unused)` of `lock/ranked.rs:83` that `--cap-lints allow` hides in the registry build.

## Why

wgpu-hal 30 (Vulkan) replaced `gpu-descriptor` with its own `DescriptorAllocator`, whose `Drop` asserts
that every bind-group layout was unregistered. When `Device::new` fails half-way (out of memory — e.g.
another program holds the VRAM, or a `MemoryBudgetThresholds` refusal), 30.0.1 drops the raw device
with the empty BGL still registered: **a panic instead of the `RequestDeviceError`** that
`ph2d_gpu::memoria::tenta_enquanto_falta_memoria` waits on and retries. In wgpu 29 the same leak was
silent. Seen red by `ph2d-app-light2d` `os_tetos_da_placa::a_luz_recua_quando_a_placa_recusa_a_memoria`
(backtrace: `DescriptorAllocator::drop` ← `drop_glue::<wgpu_hal::vulkan::Device>` ← `wgpu_core::Device::new`).

## When to delete

When crates.io has a `wgpu-core` that contains #10160 (30.0.2 or 31): bump, delete this folder, the
`exclude` line and the `[patch.crates-io]` entry, and re-run that test.
