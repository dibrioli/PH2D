# Audit — W1.T9 KTX2 kvd preservation + bounds — Lens ξ (Bounds + DOS attack surface)

- **Commit under audit:** `9c31822` (W1.T9 — kvd preservation + PremulIntent + byte_size_estimate)
- **Scope:** ONLY the kvd / DOS-bounds / PremulIntent code added in `crates/ph2d-asset-ktx2/src/lib.rs`
- **Auditor lens:** ξ — order-of-checks, real enforcement, test coverage, semantic correctness of tri-state, UTF-8/key validation, integer overflow
- **Date:** 2026-05-28

---

## Summary verdict: **PASS_WITH_FINDINGS**

The core DOS defence is **sound**: the value-size bound is checked against a **zero-copy borrowed slice from the upstream `ktx2 0.5.0` iterator BEFORE any allocation**, and the entry-count bound is checked **before insertion**. There is **no unbounded-alloc-before-check** vulnerability. The tri-state `premul_intent()` is correct and panic-free. No integer overflow on the in-scope path; no transcendental math (HR-6 N/A).

The findings are: (1) **HIGH** — the two new DOS error variants (`TooManyKvdEntries`, `KvdValueTooLarge`) have **ZERO test coverage** on the real parse path; both are unreachable-by-test, and the rejection ordering relied upon by this PASS verdict is therefore unverified by CI; (2) **LOW** — duplicate-key transient-allocation churn is technically unbounded in *count* (though each ≤ 4 KiB and freed immediately); (3) **LOW** — `MAX_KVD_VALUE_BYTES` does not bound the *key* length nor the *aggregate* kvd memory; (4) **INFO** — count reporting cosmetic note.

No CRITICAL findings.

---

## Findings table

| # | Severity | Title | Location |
|---|----------|-------|----------|
| F1 | HIGH | New DOS bounds (`TooManyKvdEntries`, `KvdValueTooLarge`) have zero parse-path test coverage; ordering unverified by CI | `lib.rs:647-660`, tests `1305-1318` |
| F2 | LOW | Duplicate-key churn: unbounded *number* of transient `to_vec()` allocations (each ≤ 4 KiB, freed) | `lib.rs:646-661` |
| F3 | LOW | No bound on key length nor on aggregate kvd memory (64 × 4 KiB ≈ 256 KiB worst case, acceptable, but undocumented) | `lib.rs:647-660` |
| F4 | INFO | `count: kvd.len() + 1` reporting — correct but worth a one-line comment | `lib.rs:649` |
| F5 | PASS-evidence | Order-of-checks: value-size bound is on a zero-copy borrow, before `.to_vec()`; entry-count before `.insert()` | `lib.rs:653,660` + upstream `ktx2 0.5.0` |

---

## Per-finding detail

### F5 (PASS evidence) — Order-of-checks is CORRECT; no alloc-before-check

This is the central question of Lens ξ. **Proven by line ordering + upstream API contract.**

The parse loop (`crates/ph2d-asset-ktx2/src/lib.rs:646-661`):

```rust
for (key, value) in reader.key_value_data() {       // 646
    if kvd.len() >= MAX_KVD_ENTRIES {               // 647  — COUNT check
        return Err(Ktx2Error::TooManyKvdEntries {   // 648
            count: kvd.len() + 1, max: MAX_KVD_ENTRIES,
        });
    }
    if value.len() > MAX_KVD_VALUE_BYTES {           // 653  — SIZE check
        return Err(Ktx2Error::KvdValueTooLarge {     // 654
            key: key.to_string(), size: value.len(), max: MAX_KVD_VALUE_BYTES,
        });
    }
    kvd.insert(key.to_string(), value.to_vec());     // 660  — ALLOC happens HERE
}
```

The decisive fact is what `reader.key_value_data()` yields. From the upstream crate
(`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ktx2-0.5.0/src/lib.rs:321-322`):

```rust
impl<'data> Iterator for KeyValueDataIterator<'data> {
    type Item = (&'data str, &'data [u8]);   // ← value is a BORROW into the input buffer
```

and the value slice is produced by sub-slicing the original input with zero copy
(`ktx2-0.5.0/src/lib.rs:352-353`):

```rust
let key   = &key_and_value[..key_end_index];
let value = &key_and_value[key_end_index + 1..];   // zero-copy &[u8]
```

**Therefore the hostile-file scenario "100 entries each 10 MB" is handled correctly:**

- `value.len()` (line 653) reads the **length of a borrowed slice into the already-mmapped/owned input buffer**. No 10 MB allocation occurs to measure it. The first value > 4 KiB returns `KvdValueTooLarge` having allocated only the `key.to_string()` for the error (a small bounded string ≤ key length). The 10 MB is never copied into `kvd`.
- The entry-count check (line 647) runs **before** the `to_vec()` insert (line 660), so the `BTreeMap` never grows past `MAX_KVD_ENTRIES = 64` entries. On the 65th valid entry it returns `TooManyKvdEntries` before that entry's value is `to_vec()`-ed.

Order is: COUNT check → SIZE check → ALLOC. Both bounds gate the allocation. **No CRITICAL.**

Command run:
```
grep -rn "MAX_KVD_ENTRIES\|MAX_KVD_VALUE_BYTES" crates/ph2d-asset-ktx2/
# → constants referenced on the real path at lib.rs:647, 653 (not dead constants)
```

The `5..` (BTreeMap) insert never sees an oversize value because of the early `return`. This is **real, not theoretical**: the borrow-not-copy contract is enforced by the upstream type signature `&'data [u8]`, which the Rust borrow checker guarantees.

---

### F1 (HIGH) — New DOS bounds have ZERO parse-path test coverage

Both `Ktx2Error::TooManyKvdEntries` and `Ktx2Error::KvdValueTooLarge` are **unreachable by any test**. Confirmed:

```
ls crates/ph2d-asset-ktx2/tests/        → no tests dir
grep -rln "TooManyKvd\|KvdValue\|MAX_KVD" crates/ph2d-asset-ktx2/tests/  → none
```

The in-file tests cover the *helper* (`premul_intent`) via **struct-literal** construction
(`lib.rs:1320-1382`) and a smoke that the canonical fixture has **empty** kvd
(`lib.rs:1418-1425`), but **no test drives a KTX2 byte buffer through `decode_ktx2_bytes` with a populated kvd section.** The author acknowledges this explicitly at `lib.rs:1305-1318`:

> `Ktx2Error::TooManyKvdEntries` + `Ktx2Error::KvdValueTooLarge` require building synthetic KTX2 byte buffers com kvd section populated (FixtureSpec atual emite kvd_byte_length=0). Test coverage adicionado em W1.T2.3 …; bounds são `if-compare-return` pequenos, verified por inspection …

**Why HIGH (not LOW despite "small if-compares"):** the PASS verdict of this very audit rests on the *ordering* (count-before-insert, size-before-to_vec). That ordering is exactly the kind of invariant a future refactor can silently invert (e.g. someone moving the `to_vec()` above the size check for "clarity", or switching to `kvd.entry(...).or_insert_with(|| value.to_vec())` which allocates the closure capture early). With no test exercising the rejection path, **CI will not catch a regression that re-introduces alloc-before-check** — the precise DOS defeat this lens exists to prevent. A defence with no test is a defence one refactor away from silent removal.

**Concrete in-scope fix:** extend `FixtureSpec` + `build_fixture` to emit a kvd section
(populate `kvd_byte_offset`/`kvd_byte_length` and write `[u32 len][key\0value][pad to 4]`
entries — the wire format the upstream iterator already parses at `ktx2-0.5.0/src/lib.rs:324-364`).
Then add two tests:

```rust
#[test]
fn decode_rejects_too_many_kvd_entries() {
    // MAX_KVD_ENTRIES + 1 = 65 distinct keys, 1-byte values.
    let spec = FixtureSpec { kvd: (0..=MAX_KVD_ENTRIES)
        .map(|i| (format!("K{i}"), vec![0u8])).collect(), ..valid() };
    let err = decode_ktx2_bytes(&build_fixture(&spec)).expect_err("must reject");
    assert!(matches!(err, Ktx2Error::TooManyKvdEntries { count, max }
        if count == MAX_KVD_ENTRIES + 1 && max == MAX_KVD_ENTRIES), "got {err:?}");
}

#[test]
fn decode_rejects_kvd_value_too_large() {
    let spec = FixtureSpec { kvd: vec![("BIG".into(),
        vec![0u8; MAX_KVD_VALUE_BYTES + 1])], ..valid() };
    let err = decode_ktx2_bytes(&build_fixture(&spec)).expect_err("must reject");
    assert!(matches!(err, Ktx2Error::KvdValueTooLarge { size, max, .. }
        if size == MAX_KVD_VALUE_BYTES + 1 && max == MAX_KVD_VALUE_BYTES), "got {err:?}");
}
```

A third test should assert a *valid* kvd round-trips (`PH2D_PREMUL` → `[1]` → `premul_intent() == Premultiplied`) so the happy path through the real parser — not just struct literals — is also covered. The `KvdValueTooLarge` test in particular pins the size-before-to_vec ordering: with a `MAX_KVD_VALUE_BYTES + 1`-byte value, a failing impl that allocates first would still return the right error but a memory-instrumented variant would catch the copy; at minimum the test locks the boundary value `> max` vs `>=`.

Note the author defers this to "W1.T2.3 snapshot tests"; per the project's
`feedback-perfection-no-deferrals` rule, a known-gap on a DOS-defence path is in-scope work, not a deferral. The fixture extension is ~30 LOC.

---

### F2 (LOW) — Duplicate-key transient allocation churn

`MAX_KVD_ENTRIES` bounds the BTreeMap *cardinality*, not the *iteration count*. A hostile
file can contain thousands of entries **all using the same key**. `kvd.insert` on a
duplicate key replaces in place, so `kvd.len()` stays at 1 and `TooManyKvdEntries` never
fires. Each iteration still executes `key.to_string()` + `value.to_vec()` (line 660),
allocating then immediately dropping on the next replace.

**Real impact: bounded.** Each transient value is ≤ `MAX_KVD_VALUE_BYTES` (4 KiB, checked at
line 653 before the to_vec), and the total number of entries is bounded by the kvd section
size, which upstream caps at `kvd_byte_offset + kvd_byte_length < input.len()`
(`ktx2-0.5.0/src/lib.rs:101-110`) — i.e. by the file size itself. So this is **CPU/allocator
churn proportional to file size, not memory exhaustion** (peak resident kvd memory stays
≤ 64 × 4 KiB). Classified LOW: a 100 MB file of duplicate-key 4-KiB values would do ~25k
alloc/free cycles at load time — annoying, not fatal, and load-time (HR-3 two-world: not
hot path).

**Concrete in-scope fix (optional):** count *iterations* not *map size* for the entry cap:

```rust
let mut seen = 0usize;
for (key, value) in reader.key_value_data() {
    seen += 1;
    if seen > MAX_KVD_ENTRIES { return Err(Ktx2Error::TooManyKvdEntries { count: seen, max: MAX_KVD_ENTRIES }); }
    if value.len() > MAX_KVD_VALUE_BYTES { return Err(...); }
    kvd.insert(key.to_string(), value.to_vec());
}
```

This caps duplicate-key files at 64 iterations too. Trade-off: it would reject a (weird but
spec-legal) file that lists > 64 entries where duplicates collapse to ≤ 64 unique keys.
Given kvd is metadata, rejecting > 64 raw entries is the more defensive choice. Document
whichever semantic is chosen.

---

### F3 (LOW) — Key length and aggregate kvd memory unbounded by name

`MAX_KVD_VALUE_BYTES` bounds the *value*. There is no explicit bound on the *key* length —
`key.to_string()` (line 660) and `key.to_string()` in the error (line 655) copy a key whose
length is bounded only by the (file-size-bounded) kvd section. In practice the upstream
iterator requires a NUL terminator within the entry and the entry length comes from a u32,
so a single key is ≤ ~4 GiB theoretically but ≤ file size practically. Combined with
F2's per-iteration churn this is the same file-size-proportional bound, hence LOW.

Aggregate worst case for the *stored* map: 64 entries × (4 KiB value + key). With long keys
this exceeds the naive "256 KiB" mental model but is still small. **Concrete fix:** add a
doc line on `MAX_KVD_ENTRIES` stating the aggregate worst case, and optionally fold key
length into the per-entry size check (`key.len() + value.len() > MAX_KVD_VALUE_BYTES`).

---

### F4 (INFO) — `count: kvd.len() + 1` reporting

At `lib.rs:649`, when the map holds 64 and a 65th valid entry arrives, the error reports
`count: 65`. This is **correct** (it counts the offending entry). With the F2 fix this
naturally becomes `count: seen`. No action required beyond an optional clarifying comment.

---

## Other lens checks (all PASS)

**Q4 — `premul_intent()` tri-state correctness (`lib.rs:444-450`).** Correct and panic-free.
The byte-pattern match is:
```rust
match self.kvd.get(PH2D_PREMUL_KEY).map(|v| v.as_slice()) {
    Some([0]) => Straight,
    Some([1]) => Premultiplied,
    _ => Unspecified,            // key absent, empty value, [2], [255], multi-byte → all graceful
}
```
Key absent → `None` → `Unspecified` (wildcard). Malformed/garbage value (any slice that is
not exactly `[0]` or `[1]`) → falls through to `Unspecified`. **No panic** — slice patterns
`[0]`/`[1]` simply don't match other lengths/values; there is no indexing. This is the
intended graceful tri-state and it is **well covered** by `premul_intent_unspecified_for_invalid_value`
(`lib.rs:1358-1372`, exercising `[2]`, `[255]`, `[0,1]`, `[]`). Good.

**Q5 — UTF-8 / key validation.** The upstream iterator returns keys as **already-validated
`&'data str`** (`ktx2-0.5.0/src/lib.rs:355-358`: `core::str::from_utf8(key)` — on `Err` the
entry is `continue`-skipped, not surfaced). So PH2D's `key.to_string()` operates on a valid
`&str`; **no `from_utf8`/lossy conversion in PH2D code, no panic path on invalid UTF-8.**
Values are intentionally raw `&[u8]` (KTX2 values are not required to be UTF-8) and PH2D
stores them as `Vec<u8>` — correct. Note as a *property of the dependency, not a defect*:
the upstream silently drops malformed kvd entries (bad UTF-8 key, missing NUL, truncated
length). PH2D therefore never sees them — which is why F2's count is "valid entries". This
is acceptable for metadata but means a hostile file's malformed-entry padding is invisible
to the entry cap; the F2 iteration-count fix would still bound *valid* iterations only
(malformed ones are skipped inside `next()` before yielding). Adjacent to upstream;
owner = `gfx-rs/ktx2`. Do not fix.

**Q6 — Integer overflow / underflow in size arithmetic.** None on the in-scope path.
PH2D's kvd loop does no offset arithmetic — it consumes pre-sliced `(key, value)` pairs.
All kvd offset math lives upstream and uses `checked_add`
(`ktx2-0.5.0/src/lib.rs:105` for `kvd_byte_offset + kvd_byte_length`, and `:332`
`offset.checked_add(length)` in the iterator). The PH2D mip-bytes accumulator uses
`saturating_add` (`lib.rs:606`) — out of kvd scope but noted as correct. **No overflow.**

**HR-6 determinism.** No transcendental math anywhere in the crate:
```
grep -nE '\.(sin|cos|tan|atan2|exp|sqrt|pow)\b' crates/ph2d-asset-ktx2/src/lib.rs  → none
```
kvd stored in `BTreeMap` (deterministic iteration order — `lib.rs:413` doc explicitly cites
HR-6). PASS.

---

## Commands run (reproducibility)

```
grep -rn "MAX_KVD_ENTRIES\|MAX_KVD_VALUE_BYTES" crates/ph2d-asset-ktx2/
grep -n "ktx2" crates/ph2d-asset-ktx2/Cargo.toml ; grep -A3 'name = "ktx2"' Cargo.lock     # ktx2 0.5.0
find ~/.cargo -type d -name "ktx2-0.5.0"
grep -rn "key_value_data\|fn next\|KeyValueDataIterator" ktx2-0.5.0/src/lib.rs
ls crates/ph2d-asset-ktx2/tests/ ; grep -rln "TooManyKvd\|KvdValue\|MAX_KVD" crates/ph2d-asset-ktx2/tests/
grep -nE '\.(sin|cos|tan|atan2|exp|sqrt|pow)\b' crates/ph2d-asset-ktx2/src/lib.rs
```

## Recommended priority

1. **F1 (HIGH)** — add the 3 parse-path tests (reject-too-many, reject-too-large, valid-round-trip)
   by extending `FixtureSpec`/`build_fixture` with a kvd writer. This is the one finding that
   converts the verdict's "PASS by inspection" into "PASS verified by CI", protecting the
   ordering invariant against future refactors. ~30-40 LOC, in-scope.
2. F2 (LOW) — switch entry cap from map-cardinality to iteration-count to also bound
   duplicate-key churn; decide and document the semantic.
3. F3 (LOW) — fold key length into the size check or document aggregate worst case.
