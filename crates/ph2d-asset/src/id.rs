//! [`AssetId`] — blake3 hash of the raw asset bytes.
//!
//! Per HR-6: identity is content, not path. Two files with identical
//! bytes share an id; renaming a file leaves the id intact. The hash
//! is computed once at insert/load and stored — we never re-hash on
//! lookup.

/// 32-byte blake3 digest. Used as the cache key in [`crate::AssetDb`].
///
/// `Serialize`/`Deserialize` derives let `AssetId` cross the postcard
/// pipeline that `PrefabDoc`/`SceneDoc` consume (M14.3) and the
/// `LuauScript` component carries (M14.2). The 32-byte tuple is
/// encoded as a fixed-length byte array — no length prefix overhead.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub struct AssetId([u8; 32]);

impl AssetId {
    /// Hash `bytes` with blake3 and wrap as an `AssetId`.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self(*blake3::hash(bytes).as_bytes())
    }

    /// Construct from an already-computed digest. Mostly for tests
    /// and deserialization paths.
    pub fn from_digest(digest: [u8; 32]) -> Self {
        Self(digest)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Lowercase hex (64 chars). The canonical form for logs and IDs
    /// in any user-facing output.
    pub fn to_hex(&self) -> String {
        let mut out = String::with_capacity(64);
        for b in self.0 {
            use std::fmt::Write as _;
            let _ = write!(out, "{b:02x}");
        }
        out
    }
}

impl std::fmt::Debug for AssetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Short prefix is plenty for debug — full 64 chars is noise.
        let hex = self.to_hex();
        write!(f, "AssetId({}…)", &hex[..12])
    }
}

impl std::fmt::Display for AssetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_hex())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_bytes_hash_to_same_id() {
        let a = AssetId::from_bytes(b"hello world");
        let b = AssetId::from_bytes(b"hello world");
        assert_eq!(a, b);
    }

    #[test]
    fn different_bytes_hash_to_different_ids() {
        let a = AssetId::from_bytes(b"hello world");
        let b = AssetId::from_bytes(b"hello world!");
        assert_ne!(a, b);
    }

    #[test]
    fn hex_round_trip_is_64_chars() {
        let id = AssetId::from_bytes(b"x");
        let hex = id.to_hex();
        assert_eq!(hex.len(), 64);
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn id_is_total_ordering() {
        // Spot-check: BTreeMap key requires Ord. Sorting two ids by
        // their bytes should compare lexicographically.
        let a = AssetId::from_digest([0u8; 32]);
        let b = AssetId::from_digest([1u8; 32]);
        assert!(a < b);
    }
}
