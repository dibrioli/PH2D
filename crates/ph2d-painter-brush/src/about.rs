//! `AboutParams` sub-struct. Brush Studio §1.3.12 (metadata do brush).
//! Cap ≤ 8 fields (v1 = 5).

use serde::{Deserialize, Serialize};

/// Cap máximo de reset save-points por brush. FIFO truncation via
/// [`AboutParams::push_reset_point`].
///
/// **Audit 2026-05-26 B-C2:** anteriormente o cap era documentation-only
/// (comment sem const + sem enforcement). Cap real declarado agora.
pub const MAX_RESET_POINTS: usize = 16;

/// Metadata do brush. §1.3.12.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AboutParams {
    pub name: String,
    pub author: String,
    /// Asset handle de signature image (PNG/preview opcional).
    pub signature_uri: Option<String>,
    /// Created at — ms since epoch.
    pub created_at_ms: u64,
    /// Reset save-points para revert no Brush Studio. Cap [`MAX_RESET_POINTS`]
    /// = 16 (FIFO via [`Self::push_reset_point`]). Cada entry é
    /// `BrushParamsHash` (32 bytes blake3 do snapshot).
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

impl AboutParams {
    /// Append a reset point, evicting the OLDEST (FIFO) if at cap.
    /// Cap = [`MAX_RESET_POINTS`].
    pub fn push_reset_point(&mut self, hash: [u8; 32]) {
        if self.reset_points.len() >= MAX_RESET_POINTS {
            self.reset_points.remove(0);
        }
        self.reset_points.push(hash);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_reset_point_respects_cap() {
        let mut a = AboutParams::default();
        for i in 0..(MAX_RESET_POINTS + 5) {
            let mut h = [0u8; 32];
            h[0] = i as u8;
            a.push_reset_point(h);
        }
        assert_eq!(a.reset_points.len(), MAX_RESET_POINTS);
        // FIFO: oldest evicted, newest at end. First remaining = entry #5.
        assert_eq!(a.reset_points[0][0], 5);
        // Last entry = #20 = MAX_RESET_POINTS + 5 - 1 = 20.
        assert_eq!(
            a.reset_points[MAX_RESET_POINTS - 1][0],
            (MAX_RESET_POINTS + 5 - 1) as u8
        );
    }
}
