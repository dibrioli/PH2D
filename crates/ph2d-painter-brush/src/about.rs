//! `AboutParams` sub-struct. Brush Studio §1.3.12 (metadata do brush).
//! Cap ≤ 8 fields (v1 = 5).

use serde::{Deserialize, Serialize};

/// Metadata do brush. §1.3.12.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AboutParams {
    pub name: String,
    pub author: String,
    /// Asset handle de signature image (PNG/preview opcional).
    pub signature_uri: Option<String>,
    /// Created at — ms since epoch.
    pub created_at_ms: u64,
    /// Reset save-points para revert no Brush Studio. Cap interno via
    /// `MAX_RESET_POINTS = 16` (sem nested struct; é só `Vec<BrushHash>`).
    /// Cada entry é `BrushParamsHash` (32 bytes blake3 do snapshot).
    pub reset_points: Vec<[u8; 32]>,
    // 5 fields v1, 3 slots de headroom (cap 8).
}

impl Default for AboutParams {
    fn default() -> Self {
        Self {
            name: "Untitled Brush".to_string(),
            author: String::new(),
            signature_uri: None,
            created_at_ms: 0,
            reset_points: Vec::new(),
        }
    }
}
