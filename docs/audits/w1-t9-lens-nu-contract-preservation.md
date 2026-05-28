# Audit — Lens ν (nu): Fase 1 contract preservation — W1.T9 (commit 9c31822)

**Crate:** `crates/ph2d-asset-ktx2/`
**Scope:** ONLY the `9c31822` diff (`feat(asset-ktx2): W1.T9 — kvd preservation + PremulIntent + byte_size_estimate`).
**Date:** 2026-05-28
**Auditor lens:** ν — adversarial verification of the "strategic-only, purely additive, zero breaking changes" claim.

---

## Summary verdict: **PASS**

The W1.T9 claim holds. Every change in `9c31822` is purely additive at the
source level, and — critically — `ph2d-asset-ktx2` currently has **zero
real consumers** in the workspace (no crate declares it as a dependency, no
`.rs` file imports it outside the crate's own doctests). The new `kvd`
field therefore cannot break any external struct-literal construction, and
the type is **not** `Serialize`/`Deserialize`, so there is no postcard /
serde wire format to break. `cargo test -p ph2d-asset-ktx2` is green
(32 lib tests + 2 doctests). No CRITICAL or HIGH findings.

Findings below are all informational / LOW (drift and latent-trap notes that
become relevant only when Fase 2 wires a real consumer in).

---

## Findings table

| ID | Severity | Area | One-line |
|----|----------|------|----------|
| ν-1 | INFO (PASS) | (a) Cargo.toml | Cargo.toml unchanged by 9c31822; no new deps; consumers unaffected. |
| ν-2 | INFO (PASS) | (b) API additivity | All 4 pre-existing public fields & methods byte-identical; only additions. |
| ν-3 | INFO (PASS) | (c) wire format | `Ktx2Image` derives only `Debug, Clone` — no serde/postcard; new field cannot break any blob. |
| ν-4 | INFO (PASS) | (c) struct literals | All 3 `Ktx2Image { … }` literals are inside the crate (parser + tests), all updated. Zero external sites. |
| ν-5 | INFO (PASS) | (d) accounting | `byte_size_estimate()` is new & unused by any consumer; `Asset::byte_size()` still uses `blob.len()` independently. |
| ν-6 | LOW | doc drift | Doc-comment at `asset.rs:38` references non-existent `ph2d_asset_ktx2::parse`; pre-existing, not introduced by 9c31822. Adjacent — owner `ph2d-asset`. |
| ν-7 | LOW | latent trap | No `#[serde(default)]` / no `#[non_exhaustive]` on `Ktx2Image`; the additive-safety guarantee silently expires the moment Fase 2 adds serde or an external struct-literal consumer. |

---

## Per-finding detail

### ν-1 — (a) Cargo.toml: zero breaking changes — **PASS**

```
$ git show 9c31822 -- crates/ph2d-asset-ktx2/Cargo.toml
(no output)
```

The commit touched a single file (`git show 9c31822 --stat`):
`crates/ph2d-asset-ktx2/src/lib.rs | 220 +++…` — Cargo.toml is absent from
the diff. Current manifest (`crates/ph2d-asset-ktx2/Cargo.toml`) lists only
the two pre-existing deps:

```
[dependencies]
ktx2 = "2"        # actually "0.5"
thiserror = "2"
```

(`ktx2 = "0.5"`, `thiserror = "2"`.) The new field uses
`std::collections::BTreeMap`, which is `std` — no dependency added,
no feature toggled, no version bump. Consumers' lockfiles / feature graphs
are unaffected. **PASS.**

### ν-2 — (b) Public API additions are purely additive — **PASS**

Diff of the struct, pre vs post (`git show 9c31822^:…/lib.rs` vs current):

Pre-9c31822 `Ktx2Image`:
```rust
pub struct Ktx2Image {
    pub format: Ktx2Format,
    pub width: u32,
    pub height: u32,
    pub mip_levels: Vec<MipLevel>,
}
```
Post: identical four fields, plus an appended `pub kvd: BTreeMap<String, Vec<u8>>`.
No field renamed, reordered in a way that changes its type, retyped, or
removed. `base_level(&self) -> &MipLevel` is unchanged.

Additions only:
- consts `MAX_KVD_ENTRIES`, `MAX_KVD_VALUE_BYTES`, `PH2D_PREMUL_KEY` (+ 2 const asserts);
- 2 new `Ktx2Error` variants (`TooManyKvdEntries`, `KvdValueTooLarge`);
- new `pub enum PremulIntent`;
- new methods `byte_size_estimate()`, `premul_intent()`;
- parser loop that fills `kvd`.

One nuance worth recording: `Ktx2Error` is **not** `#[non_exhaustive]`, so
adding variants is technically a breaking change for any *external* `match`
on `Ktx2Error` that lacks a wildcard arm. Verified there are **none**:

```
$ grep -rn "Ktx2Error" --include="*.rs" . | grep -v crates/ph2d-asset-ktx2/
(no output)
```

No external code matches on `Ktx2Error`, so this is harmless today. **PASS.**

### ν-3 — (c) serde / postcard wire-format compatibility — **PASS (concern is moot)**

The audit brief flags a deserialize-of-old-blob risk. That risk does not
exist, because `Ktx2Image` is not a serde type:

```
$ grep -n -B3 "pub struct Ktx2Image" crates/ph2d-asset-ktx2/src/lib.rs
#[derive(Debug, Clone)]
pub struct Ktx2Image {
```
```
$ grep -rn "serde\|postcard\|Serialize\|Deserialize" crates/ph2d-asset-ktx2/
(no matches; exit 1)
```

The crate has no serde/postcard dependency and the type derives only
`Debug, Clone`. `Ktx2Image` is a transient decode product, never persisted —
the persisted form is `Asset::TextureKtx2 { blob, tier }` in `ph2d-asset`,
which stores the *raw KTX2 bytes*, not a serialized `Ktx2Image`. Adding a
field to a non-serde struct cannot break any wire format. **PASS.**

### ν-4 — (c) struct-literal construction break — **PASS (no external site exists)**

```
$ grep -rn "Ktx2Image\s*{" --include="*.rs" .   # + "Ktx2Image{" variant
crates/ph2d-asset-ktx2/src/lib.rs:401:pub struct Ktx2Image {   (definition)
crates/ph2d-asset-ktx2/src/lib.rs:663:    Ok(Ktx2Image {        (parser — updated, sets kvd)
crates/ph2d-asset-ktx2/src/lib.rs:1323:        Ktx2Image {        (test helper — sets kvd)
crates/ph2d-asset-ktx2/src/lib.rs:1386:        let img = Ktx2Image {  (test — sets kvd)
```

Every literal lives inside the crate's own `lib.rs` and all three real
constructors were updated to pass `kvd` in the same commit. There are **no**
struct-literal sites elsewhere in the workspace, because nothing else even
references the type:

```
$ grep -rn "Ktx2Image" --include="*.rs" . | grep -v crates/ph2d-asset-ktx2/
crates/ph2d-asset/src/asset.rs:40:   // ... doc-comment prose only
crates/ph2d-asset/src/asset.rs:78:   // ... doc-comment prose only
crates/ph2d-asset/src/db.rs:273:     // ... doc-comment prose only
```
All three matches are inside `///` / `//` comments — no code references the
type. And no crate depends on it:
```
$ grep -rn "ph2d-asset-ktx2" --include="Cargo.toml" . | grep -v crates/ph2d-asset-ktx2/Cargo.toml
(no output)
```

So the theoretical struct-literal break has **no real call site**. Because
`Ktx2Image` has all-public fields and is not `#[non_exhaustive]`, the break
*would* be real the instant an external crate constructs it via a literal —
but that day has not arrived (see ν-7). **PASS today.**

### ν-5 — (d) `byte_size_estimate()` accounting semantics — **PASS**

`Ktx2Image::byte_size_estimate()` (new) sums **decoded mip payload bytes**:
```rust
self.mip_levels.iter().map(|m| m.data.len()).sum()
```
The only existing byte-accounting consumer, `Asset::byte_size()` in
`ph2d-asset/src/asset.rs:80-82`, accounts the **raw serialized blob**:
```rust
Self::TextureKtx2 { blob, tier } =>
    blob.len() + size_of_val(tier) + size_of_val(&**blob)
```
These are two different quantities (compressed/serialized container bytes vs.
decoded mip pyramid bytes), they live in different crates, and `byte_size()`
does **not** call `byte_size_estimate()` (it can't — no dependency edge
exists). The new helper changes the meaning of nothing currently in use; it
is forward-looking scaffolding the commit message itself flags as "future
migration para Arc<Ktx2Image>". The method also correctly excludes kvd and
Arc/Vec overhead per its doc-comment. No existing accounting is altered.
**PASS.**

### ν-6 — LOW — pre-existing doc drift in adjacent crate

`crates/ph2d-asset/src/asset.rs:38` reads:
`/// via ph2d_asset_ktx2::parse(&blob) no upload path …`
There is no `parse` function in the crate — the public entry point is
`decode_ktx2_bytes`. This is stale doc prose, **pre-existing** (not in the
9c31822 diff), and harmless (it's a comment, not code). **Adjacent —
owner = `ph2d-asset`.** Out of scope to fix here; flag to that owner.

### ν-7 — LOW — additive-safety guarantee is unguarded (latent trap)

The "purely additive" property currently holds only because there are zero
external consumers. Nothing structurally preserves it:
- `Ktx2Image` is **not** `#[non_exhaustive]` and has all-`pub` fields → any
  future external struct literal breaks on the next added field.
- `Ktx2Error` is **not** `#[non_exhaustive]` → any future external
  wildcard-less `match` breaks on the next added variant.
- There is no arch-gate / contract-surface test in the crate
  (`ls crates/ph2d-asset-ktx2/tests/` → none; `grep contract_surface` →
  none), unlike the Node/Tool/Painter/Vector contracts which freeze their
  caps with a gate.

**Recommended fix (in-scope, optional, low effort):** before Fase 2 wires a
real consumer, add `#[non_exhaustive]` to both `Ktx2Image` and `Ktx2Error`.
This converts the current "happens to be safe" into "guaranteed additive":
external crates would then be forced to use `..` / wildcard arms, so future
field/variant additions stay non-breaking by construction. (Note: making
`Ktx2Image` `#[non_exhaustive]` also forbids external struct-literal
construction, which is the desired posture for a parser-output type — callers
should only ever read it.) This is a hardening suggestion, not a defect in
the 9c31822 changes themselves.

---

## Commands run (reproducibility)

```
git show 9c31822 --stat -- crates/ph2d-asset-ktx2/
git show 9c31822 -- crates/ph2d-asset-ktx2/Cargo.toml          # empty
git show 9c31822 -- crates/ph2d-asset-ktx2/src/lib.rs
git show 9c31822^:crates/ph2d-asset-ktx2/src/lib.rs | grep -A12 "pub struct Ktx2Image"
grep -n -B3 "pub struct Ktx2Image" crates/ph2d-asset-ktx2/src/lib.rs
grep -rn "Ktx2Image\s*{"  --include="*.rs" .
grep -rn "Ktx2Image{"     --include="*.rs" .
grep -rn "Ktx2Image"      --include="*.rs" . | grep -v crates/ph2d-asset-ktx2/
grep -rn "Ktx2Error"      --include="*.rs" . | grep -v crates/ph2d-asset-ktx2/
grep -rn "ph2d-asset-ktx2" --include="Cargo.toml" . | grep -v crates/ph2d-asset-ktx2/Cargo.toml
grep -rn "ph2d_asset_ktx2" --include="*.rs" .
grep -rn "serde\|postcard\|Serialize\|Deserialize" crates/ph2d-asset-ktx2/   # none
cargo test -p ph2d-asset-ktx2                                  # 32 + 2 green
```
