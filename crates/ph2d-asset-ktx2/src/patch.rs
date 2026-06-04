use thiserror::Error;

use crate::{PH2D_PREMUL_KEY, PremulIntent};

// ── post-hoc KVD patcher (W1.T8.1) ──────────────────────────────────

/// Failures while patching a `PH2D_PREMUL` key into an already-cooked
/// KTX2 container ([`patch_premul_intent`]).
///
/// `#[non_exhaustive]` for the same forward-compat reason as
/// [`Ktx2Error`](crate::Ktx2Error): a future patcher may grow new structural-rejection
/// arms, and downstream `match` sites match a wildcard today.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Ktx2PatchError {
    /// The input is not a parseable KTX2 container (bad magic, truncated
    /// header / level index, declared section bounds outside the file).
    /// The wrapped string is the upstream `ktx2` diagnostic; treat as
    /// opaque.
    #[error("input is not a valid KTX2 container: {0}")]
    InvalidContainer(String),

    /// The container already carries a `PH2D_PREMUL` key. The patcher is
    /// insert-only by design (W1.T8.1): re-tagging would silently mask a
    /// double-cook bug. Callers that legitimately need to overwrite must
    /// strip the key first (not supported in Fase 1) or re-cook the
    /// source. The carried [`PremulIntent`] is what the file already
    /// declares, so a caller can short-circuit when it matches.
    #[error("KTX2 already has a {PH2D_PREMUL_KEY} key (declares {existing:?})")]
    KeyAlreadyPresent { existing: PremulIntent },

    /// [`PremulIntent::Unspecified`] is the *absence* of a tag — there is
    /// no on-disk byte encoding for it (see [`encode_premul_value`]).
    /// Patching it in would be a no-op that still rewrites every offset,
    /// so it is rejected as a programming error instead.
    #[error("cannot patch PremulIntent::Unspecified (it has no on-disk encoding)")]
    UnspecifiedIntent,

    /// A recomputed section offset/length did not fit in the KTX2
    /// header's `u32`/`u64` fields, or the new KVD section would push the
    /// file past addressable bounds. Practically unreachable for real
    /// textures (capped at [`MAX_TOTAL_BYTES`](crate::MAX_TOTAL_BYTES)); surfaced rather than
    /// silently wrapping.
    #[error("patched KTX2 layout overflows a header offset field")]
    OffsetOverflow,
}

/// On-disk value byte for a [`PremulIntent`] in the `PH2D_PREMUL` kvd
/// entry. Mirrors the read path in [`Ktx2Image::premul_intent`](crate::Ktx2Image::premul_intent):
/// `Straight → 0`, `Premultiplied → 1`. [`PremulIntent::Unspecified`]
/// has **no** encoding — it is the absence of the key — so this returns
/// `None` for it (the patcher rejects that case explicitly).
#[must_use]
pub fn encode_premul_value(intent: PremulIntent) -> Option<u8> {
    match intent {
        PremulIntent::Straight => Some(0),
        PremulIntent::Premultiplied => Some(1),
        PremulIntent::Unspecified => None,
        // `#[non_exhaustive]` enum within its own crate: a future intent
        // must add an explicit byte here, so no wildcard arm.
    }
}

/// Round a `u32` up to the next multiple of `align` (a power of two).
/// `align` is always a small spec constant (4 or 8), never zero.
fn align_up_u32(value: u32, align: u32) -> Option<u32> {
    debug_assert!(align.is_power_of_two());
    value.checked_add(align - 1).map(|v| v & !(align - 1))
}

/// Same as [`align_up_u32`] for `u64` level offsets.
fn align_up_u64(value: u64, align: u64) -> Option<u64> {
    debug_assert!(align.is_power_of_two());
    value.checked_add(align - 1).map(|v| v & !(align - 1))
}

/// Serialise a sorted set of `(key, value)` KVD pairs into the on-disk
/// KTX2 `keyValueData` layout: per entry a little-endian `u32`
/// `keyAndValueByteLength` (= key bytes + 1 NUL + value bytes), the
/// NUL-terminated UTF-8 key, the value, then `valuePadding` zero bytes
/// to the next 4-byte boundary (KTX2 spec §3.10.8 + the alignment loop
/// in `ktx2::KeyValueDataIterator::next`). The returned buffer length is
/// always a multiple of 4.
pub(crate) fn serialize_kvd(entries: &[(String, Vec<u8>)]) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    for (key, value) in entries {
        let key_bytes = key.as_bytes();
        let kv_len = key_bytes.len() + 1 + value.len();
        out.extend_from_slice(&(kv_len as u32).to_le_bytes());
        out.extend_from_slice(key_bytes);
        out.push(0u8);
        out.extend_from_slice(value);
        while !out.len().is_multiple_of(4) {
            out.push(0u8);
        }
    }
    out
}

/// W1.T8.1 — insert a `PH2D_PREMUL` key/value entry into an
/// already-cooked KTX2 container, returning a fresh byte buffer.
///
/// **Why this exists:** the cooker's `ctt` / `ktx2 0.5` stack is
/// read-only (no KTX2 *writer*), so a freshly cooked texture never
/// carries PH2D's alpha-intent tag and [`Ktx2Image::premul_intent`](crate::Ktx2Image::premul_intent)
/// always reports [`PremulIntent::Unspecified`]. This patches the bytes
/// post-hoc so the round-trip
/// `patch_premul_intent(bytes, Premultiplied) → decode_ktx2_bytes →
/// premul_intent() == Premultiplied` holds.
///
/// ## What it rewrites
///
/// Inserting a KVD entry changes the size of the `keyValueData` section,
/// which shifts every section that follows it. The patcher rebuilds the
/// tail of the file and rewrites all affected header fields, preserving
/// the KTX2 alignment contract:
///
/// - the **DFD** and the 80-byte header + level index are copied
///   verbatim;
/// - the new **KVD** section is the original entries merged with
///   `PH2D_PREMUL`, **re-sorted by key** (spec requires codepoint order),
///   each entry padded to a 4-byte boundary, starting at the original
///   `kvdByteOffset` (or `align(4)` past the DFD when the original had no
///   KVD);
/// - the **SGD** section (if any) is copied verbatim to the next
///   `align(8)` boundary after the KVD (spec `sgdPadding`);
/// - each **mip level** is copied byte-for-byte, preserving the exact
///   inter-level gaps of the original so every level keeps its original
///   `lcm(texel_block_size, 4)` alignment; only the common base offset
///   moves.
///
/// `kvdByteOffset` / `kvdByteLength`, `sgdByteOffset`, and every level
/// `byteOffset` are rewritten in the header to match.
///
/// ## Idempotency
///
/// **Insert-only.** If the container already carries a `PH2D_PREMUL`
/// key, returns [`Ktx2PatchError::KeyAlreadyPresent`] (carrying the
/// existing intent) rather than silently overwriting — a second tag
/// almost always signals a double-cook bug, and masking it would lose
/// data. Callers that already declare the wanted intent can treat that
/// error as success.
///
/// [`PremulIntent::Unspecified`] has no on-disk encoding and is rejected
/// with [`Ktx2PatchError::UnspecifiedIntent`].
pub fn patch_premul_intent(bytes: &[u8], intent: PremulIntent) -> Result<Vec<u8>, Ktx2PatchError> {
    let value_byte = encode_premul_value(intent).ok_or(Ktx2PatchError::UnspecifiedIntent)?;

    // Parse + validate the whole container up-front: this rejects bad
    // magic, truncated headers, out-of-bounds section declarations, and
    // a malformed level index BEFORE we touch any offsets. We then read
    // raw sections directly from `bytes` (not via the typed Format
    // accessors) so the patcher stays agnostic to the Fase-1 format
    // subset — it must round-trip ANY valid KTX2, not just the ones the
    // decoder can classify.
    let reader =
        ktx2::Reader::new(bytes).map_err(|e| Ktx2PatchError::InvalidContainer(format!("{e:?}")))?;
    let header = reader.header();
    let index = header.index;

    // Reject an existing PH2D_PREMUL key (insert-only contract).
    for (key, value) in reader.key_value_data() {
        if key == PH2D_PREMUL_KEY {
            let existing = match value {
                [0] => PremulIntent::Straight,
                [1] => PremulIntent::Premultiplied,
                _ => PremulIntent::Unspecified,
            };
            return Err(Ktx2PatchError::KeyAlreadyPresent { existing });
        }
    }

    // Collect the original KVD entries (owned), then merge in our key and
    // re-sort by key bytes — KTX2 spec mandates codepoint ordering.
    let mut entries: Vec<(String, Vec<u8>)> = reader
        .key_value_data()
        .map(|(k, v)| (k.to_string(), v.to_vec()))
        .collect();
    entries.push((PH2D_PREMUL_KEY.to_string(), vec![value_byte]));
    entries.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));

    let new_kvd = serialize_kvd(&entries);
    let new_kvd_len = u32::try_from(new_kvd.len()).map_err(|_| Ktx2PatchError::OffsetOverflow)?;

    // KVD starts where the original KVD did, or — if the original had no
    // KVD — at the first align(4) past the DFD (spec layout order is
    // header · levelIndex · DFD · KVD · SGD · levelData).
    let new_kvd_off: u32 = if index.kvd_byte_offset != 0 {
        index.kvd_byte_offset
    } else {
        let dfd_end = index
            .dfd_byte_offset
            .checked_add(index.dfd_byte_length)
            .ok_or(Ktx2PatchError::OffsetOverflow)?;
        align_up_u32(dfd_end, 4).ok_or(Ktx2PatchError::OffsetOverflow)?
    };
    let kvd_end = new_kvd_off
        .checked_add(new_kvd_len)
        .ok_or(Ktx2PatchError::OffsetOverflow)?;

    // SGD (verbatim) on the next align(8) boundary, only when present.
    let sgd = reader.supercompression_global_data();
    let (new_sgd_off, sgd_end): (u32, u32) = if sgd.is_empty() {
        (0, kvd_end)
    } else {
        let off = align_up_u32(kvd_end, 8).ok_or(Ktx2PatchError::OffsetOverflow)?;
        let len = u32::try_from(sgd.len()).map_err(|_| Ktx2PatchError::OffsetOverflow)?;
        let end = off.checked_add(len).ok_or(Ktx2PatchError::OffsetOverflow)?;
        (off, end)
    };

    // Level index (largest→smallest). Reproduce the original inter-level
    // gaps so every level keeps its original alignment; only the common
    // base offset (level 0) moves. Level 0's required alignment is at
    // least its original power-of-two divisor, so the rebuilt base offset
    // is rounded up to that — never weaker than the spec requirement.
    let levels: Vec<ktx2::Level<'_>> = reader.levels().collect();
    if levels.is_empty() {
        // `Reader::new` guarantees ≥1 level (level_count.max(1)), so this
        // is defensive only.
        return Err(Ktx2PatchError::InvalidContainer(
            "KTX2 container declares zero mip levels".to_string(),
        ));
    }
    // Original level 0 offset & alignment (smallest mip is LAST in the
    // index; level 0 is the largest and sits at the lowest file offset).
    let raw_level_index = parse_level_index(bytes, header.level_count.max(1))
        .ok_or_else(|| Ktx2PatchError::InvalidContainer("level index truncated".to_string()))?;
    let level0_off = raw_level_index[0].0;
    // Smallest power-of-two the original offset is a multiple of; floor at
    // 8 (spec's weakest mip alignment) so an empty / degenerate file still
    // gets a conservative boundary.
    let level0_align: u64 = if level0_off == 0 {
        8
    } else {
        1u64 << level0_off.trailing_zeros()
    };

    // Never shift the level region BACKWARD: if the larger KVD still fits
    // within the original padding between metadata and level 0, keep the
    // original offset (zero shift). Otherwise push forward to the next
    // aligned boundary past the new SGD end. `max` guarantees
    // `new_level0_off >= level0_off`, so the shift is always non-negative.
    let aligned_sgd_end =
        align_up_u64(u64::from(sgd_end), level0_align).ok_or(Ktx2PatchError::OffsetOverflow)?;
    let new_level0_off = aligned_sgd_end.max(level0_off);
    // Infallible by construction (`max` ensures lhs >= rhs), but use
    // `checked_sub` to keep the arithmetic total / panic-free (HR).
    let level_shift = new_level0_off
        .checked_sub(level0_off)
        .ok_or(Ktx2PatchError::OffsetOverflow)?;

    // New per-level offsets = original + common shift (gaps preserved).
    let mut new_level_offsets: Vec<u64> = Vec::with_capacity(raw_level_index.len());
    for &(off, _len) in &raw_level_index {
        new_level_offsets.push(
            off.checked_add(level_shift)
                .ok_or(Ktx2PatchError::OffsetOverflow)?,
        );
    }
    // Total output length = end of the last (smallest) level's data.
    let last = raw_level_index
        .last()
        .ok_or_else(|| Ktx2PatchError::InvalidContainer("empty level index".to_string()))?;
    let out_len = usize::try_from(
        new_level_offsets
            .last()
            .copied()
            .unwrap()
            .checked_add(last.1)
            .ok_or(Ktx2PatchError::OffsetOverflow)?,
    )
    .map_err(|_| Ktx2PatchError::OffsetOverflow)?;

    // ── assemble the output buffer ──────────────────────────────────
    let mut out = vec![0u8; out_len];

    // 1. Header + level index + DFD copied verbatim from the original,
    //    up to the original DFD end (these never move). Defensive bounds:
    //    KTX2 layout puts DFD before KVD/SGD/levelData, so `dfd_end_orig`
    //    is always within both buffers — but guard rather than panic if a
    //    pathological file violates the ordering.
    let dfd_end_orig = usize::try_from(
        index
            .dfd_byte_offset
            .checked_add(index.dfd_byte_length)
            .ok_or(Ktx2PatchError::OffsetOverflow)?,
    )
    .map_err(|_| Ktx2PatchError::OffsetOverflow)?;
    if dfd_end_orig > out.len()
        || dfd_end_orig > bytes.len()
        || (new_kvd_off as usize) < dfd_end_orig
    {
        return Err(Ktx2PatchError::InvalidContainer(
            "KTX2 section ordering is not header·levelIndex·DFD·KVD·SGD·levelData".to_string(),
        ));
    }
    out[..dfd_end_orig].copy_from_slice(&bytes[..dfd_end_orig]);

    // 2. Rewrite the KVD + SGD index fields in the header (offsets 56..72).
    out[56..60].copy_from_slice(&new_kvd_off.to_le_bytes());
    out[60..64].copy_from_slice(&new_kvd_len.to_le_bytes());
    out[64..72].copy_from_slice(&u64::from(new_sgd_off).to_le_bytes());
    // sgdByteLength (72..80) is unchanged in size — copy original explicitly
    // in case the verbatim header copy above didn't cover a moved field.
    out[72..80].copy_from_slice(&index.sgd_byte_length.to_le_bytes());

    // 3. Rewrite each level's byteOffset in the level index (entry i at
    //    80 + i*24, byteOffset is the first u64). byteLength /
    //    uncompressedByteLength are untouched (copied verbatim in step 1).
    for (i, &new_off) in new_level_offsets.iter().enumerate() {
        let start = 80 + i * 24;
        out[start..start + 8].copy_from_slice(&new_off.to_le_bytes());
    }

    // 4. New KVD section.
    let kvd_start = new_kvd_off as usize;
    out[kvd_start..kvd_start + new_kvd.len()].copy_from_slice(&new_kvd);

    // 5. SGD section, verbatim, if present.
    if !sgd.is_empty() {
        let s = new_sgd_off as usize;
        out[s..s + sgd.len()].copy_from_slice(sgd);
    }

    // 6. Each mip level's data, byte-for-byte at its shifted offset.
    for (level, &new_off) in levels.iter().zip(&new_level_offsets) {
        let start = usize::try_from(new_off).map_err(|_| Ktx2PatchError::OffsetOverflow)?;
        out[start..start + level.data.len()].copy_from_slice(level.data);
    }

    Ok(out)
}

/// Read the raw `(byte_offset, byte_length)` of every level directly from
/// the on-disk level index (the 24-byte entries immediately after the
/// 80-byte header). `Reader::new` has already validated these bounds, so
/// this only returns `None` on a truncated buffer (defensive).
pub(crate) fn parse_level_index(bytes: &[u8], level_count: u32) -> Option<Vec<(u64, u64)>> {
    let count = level_count as usize;
    let end = 80usize.checked_add(count.checked_mul(24)?)?;
    let table = bytes.get(80..end)?;
    let mut out = Vec::with_capacity(count);
    for chunk in table.chunks_exact(24) {
        let byte_offset = u64::from_le_bytes(chunk[0..8].try_into().ok()?);
        let byte_length = u64::from_le_bytes(chunk[8..16].try_into().ok()?);
        out.push((byte_offset, byte_length));
    }
    Some(out)
}
