# ADR-0070-amendment-1 — Wrapper enum is the SOLE back-compat path (empirical T0.13 finding)

**Status:** Accepted (W0 carry-over, 2026-05-28)
**Amends:** [ADR-0070 — Sprite schema v4 (`SpriteVersioned` + `RenderInstance` ABI)](0070-sprite-schema-v4.md)
**Spec sections superseded:** `docs/Sprite_projeto/10_schema_versionamento.md` §10.4 (DECISÃO HÍBRIDA PÓS-AUDIT).
**Reference:** [`crates/ph2d-render/tests/sprite_versioned_postcard.rs`](../../../crates/ph2d-render/tests/sprite_versioned_postcard.rs) — empirical pin via `postcard_rejects_trailing_serde_default_on_short_payload`.

---

## 1. Context

ADR-0070 §10.4 ("Decisão híbrida pós-audit") declared a two-tier back-compat strategy for the v3→v4 `Sprite` schema bump:

1. **PRIMARY** — wrapper enum `SpriteVersioned` (§10.3): postcard dispatches V3 / V4 via a varint discriminant byte.
2. **DEFESA-EM-PROFUNDIDADE** — `#[serde(default = "fn")]` on every new v4 field, so a v3 postcard blob deserializes into the v4 struct shape with benign defaults filling the trailing missing fields.

The §10.4 prose hedged the second tier: *"`#[serde(default)]` LEVE — depende de comportamento postcard **não documentado como garantido**"*. T0.13 was specifically designed to **empirically validate** the claim before W1 depends on it.

## 2. Empirical finding (T0.13, 2026-05-28)

Test `postcard_rejects_trailing_serde_default_on_short_payload` in `crates/ph2d-render/tests/sprite_versioned_postcard.rs`:

```rust
#[derive(serde::Serialize)]
struct ProbeV1 { a: u32, b: u32 }

#[derive(serde::Deserialize)]
struct ProbeV2 {
    a: u32,
    b: u32,
    #[serde(default = "probe_default_c")] c: u32,
    #[serde(default)] d: u32,
}

let bytes = postcard::to_allocvec(&ProbeV1 { a: 1, b: 2 })?;
let result: Result<ProbeV2, _> = postcard::from_bytes(&bytes);
// EMPIRICAL: result is Err(postcard::Error::DeserializeUnexpectedEnd).
```

**Postcard (1.1.3) REJECTS trailing-missing fields with `Error::DeserializeUnexpectedEnd`, even when those fields carry `#[serde(default)]`.**

Root cause is structural: postcard is a non-self-describing positional format. Serde's `#[serde(default)]` fires only when the underlying data format reports the field as ABSENT (JSON `{}`, CBOR map key missing, …); postcard reports EOF, which serde treats as a deserialization error, not as "field missing → apply default".

This is consistent with postcard's documented attribute support list at <github.com/jamesmunns/postcard> (which does NOT list `serde(default)` on trailing-missing fields).

## 3. Decision

The §10.4 hybrid is reduced to a **single tier**:

- **SOLE BACK-COMPAT PATH:** wrapper enum `SpriteVersioned` (§10.3). The variant discriminant byte IS the version signal. `load_sprite` (W1.T1.6) dispatches V3 → `migrate_v3_to_v4` → `Sprite` v4; V4 → identity. No other deserialize path is supported.
- **`#[serde(default = ...)]` ON V4 NEW FIELDS** stays as a documentary/aspirational attribute (so the codebase remains correct under a hypothetical future swap to a self-describing format — JSON debug dumps, CBOR network protocols). It does **NOT** serve as a fallback for postcard v3→v4 dispatch and must not be relied on as one.
- **`SpriteV3.anchor` retains `#[serde(default)]`** as a faithful mirror of the live v3 `Sprite::anchor` attribute set, on the same documentary basis. Lens E E-H1 (Round 2 audit) reconciled the asymmetry.

## 4. Migrator contract (W1.T1.6)

```rust
pub fn load_sprite(bytes: &[u8]) -> Result<Sprite, LoadError> {
    match postcard::from_bytes::<SpriteVersioned>(bytes)? {
        SpriteVersioned::V3(v3) => Ok(migrate_v3_to_v4(v3)),
        SpriteVersioned::V4(v4) => Ok(v4),
    }
}
```

There is no fallback path. A v2-or-older blob without the wrapper discriminant byte deserialises as garbage (likely `Err`), which is the correct behavior — no silent corruption.

## 5. Consequence for W1+

- The W1 migrator MUST land — `#[serde(default)]` alone cannot carry back-compat.
- `Sprite_projeto/10_schema_versionamento.md §10.4` "Decisão híbrida pós-audit" is **SUPERSEDED** by this amendment; treat the spec line as historical context.
- The "DEFESA-EM-PROFUNDIDADE" framing must not be re-introduced as a load-bearing tier in any future ADR/amendment without first re-running T0.13's empirical pin against the candidate format. If the format ever supports it (e.g., a JSON debug bridge), update this amendment.

## 6. Forward-compat note (postcard rename gate)

The empirical pin asserts `Error::DeserializeUnexpectedEnd` by explicit `match` arm. A postcard 1.2 release renaming the variant to (hypothetically) `EofShort` would land in the `other => panic!(...)` arm — the test would fail loudly, prompting a manual audit of semantic equivalence before this amendment is updated.

## 7. Provenance

- Empirical evidence: [`crates/ph2d-render/tests/sprite_versioned_postcard.rs`](../../../crates/ph2d-render/tests/sprite_versioned_postcard.rs)
- Postcard version pinned: `=1.1.3` in [`crates/ph2d-render/Cargo.toml`](../../../crates/ph2d-render/Cargo.toml)
- Round 1 audit findings (Lens B + Lens C): consolidated into the test suite changes preceding this amendment.
- Round 2 audit findings (Lens A + Lens E): A-C1 (this CRITICAL — write the amendment) + E-H1 (restore `#[serde(default)]` on `SpriteV3.anchor` as documentary mirror).
