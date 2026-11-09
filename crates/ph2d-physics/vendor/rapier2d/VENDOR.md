# Vendored rapier2d 0.36.1 + shape change invalidates cached contact points

- **Upstream:** crates.io `rapier2d` 0.36.1 (https://github.com/dimforge/rapier), copied verbatim from the
  registry source (minus `Cargo.lock` and `Cargo.toml.orig`). **Licence:** Apache-2.0 (© Dimforge).
- **Wired by** `[patch.crates-io]` in the workspace `Cargo.toml`; excluded from the workspace (an empty
  `[workspace]` appended to `Cargo.toml`; `rustfmt.toml` disables formatting).

## What changed against 0.36.1

`src/geometry/narrow_phase/pair_update.rs`: when a collider of the pair has `ColliderChanges::SHAPE`, besides
`rigid.workspace = None` (upstream), every cached manifold gets `ContactManifold::mark_shapes_deformed()` —
the parry 0.31 API documented for exactly this («call it when a shape's geometry changed with its pose fixed»).

## Why (measured, `docs/PixelArt/P14_plano.md` «Smoke do dono 1 — pulinhos»)

`contact_manifold_cuboid_cuboid` first tries `manifold.try_update_contacts(pos12)`, which re-projects the
contact points stored in the collider's LOCAL frame. After `Collider::set_shape` + `set_position_wrt_parent`
on a resting body (a box 8×14 px → 8×10 px, offset 6 → 4 px, the bottom in the same place), the stale point
(local −7 px, the old bottom) sits 2 px inside the floor under the new collider frame: the solver
depenetrates, the body rises **2,000 px** and hovers (linvel 0 — bias only). Gate:
`ph2d-physics` `forma_encolhe::trocar_a_forma_de_um_corpo_pousado_nao_o_faz_pular` (red without this
change). Leaves when upstream invalidates the manifolds on a shape change.

`Cargo.toml`: an empty `[workspace]` and `[package.metadata.cargo-machete] ignored = ["glamx", "log"]`
(0.36.1 declares them; CI's `cargo machete` walks this folder).
