//! HR-11 destructive-tool governance (ADR-0061 §2.7): the destructive MCP tools
//! (`vector_delete_path`, `vector_clear_scene`) require a **single-use
//! confirmation token** (5-min TTL) or an explicit `--unsafe-mcp` dev/CI bypass.
//! A destructive call with no/invalid/expired/reused token is **rejected**
//! (gate `vector_mcp_governance_bypass_rejected`).
//!
//! This is the pure validation logic; issuing the prompt + wiring the MCP server
//! is the host (Coord). The clock is passed in (`now`, unix seconds) — no
//! ambient time, so it is deterministic and testable.

use std::collections::BTreeMap;

/// Confirmation-token lifetime (ADR-0061 §2.8: 5 minutes).
pub const TOKEN_TTL_SECS: u64 = 5 * 60;

/// The governance class of an MCP tool (ADR-0061 §2.2 — 6 canonical tools).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ToolKind {
    /// Reads only (`vector_query_shape`, `vector_inspect_shape`).
    ReadOnly,
    /// Adds / edits geometry (`vector_paint_shape`, `vector_modify_shape`).
    Mutative,
    /// Irreversible (`vector_delete_path`, `vector_clear_scene`) — HR-11 gated.
    Destructive,
}

/// The governance class of each canonical tool by name (the 6-tool frozen set).
pub fn tool_kind(tool: &str) -> Option<ToolKind> {
    Some(match tool {
        "vector_query_shape" | "vector_inspect_shape" => ToolKind::ReadOnly,
        "vector_paint_shape" | "vector_modify_shape" => ToolKind::Mutative,
        "vector_delete_path" | "vector_clear_scene" => ToolKind::Destructive,
        _ => return None,
    })
}

/// Why a destructive call was rejected.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GovernanceError {
    /// No confirmation token supplied for a destructive op.
    ConfirmationRequired,
    /// The token is unknown (never issued, or already consumed).
    InvalidToken,
    /// The token is past its [`TOKEN_TTL_SECS`] lifetime.
    ExpiredToken,
}

impl core::fmt::Display for GovernanceError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            GovernanceError::ConfirmationRequired => {
                f.write_str("destructive tool requires a confirmation token")
            }
            GovernanceError::InvalidToken => f.write_str("confirmation token unknown or consumed"),
            GovernanceError::ExpiredToken => f.write_str("confirmation token expired"),
        }
    }
}

impl std::error::Error for GovernanceError {}

/// Tracks issued single-use confirmation tokens (HR-11).
#[derive(Debug, Default)]
pub struct Governance {
    /// `token → issued_at` (unix seconds). [`BTreeMap`] for determinism (HR-5).
    issued: BTreeMap<String, u64>,
    /// `--unsafe-mcp`: dev/CI bypass of confirmation (ADR-0061 §2.7).
    unsafe_mode: bool,
}

impl Governance {
    pub fn new() -> Self {
        Self::default()
    }

    /// A governance that bypasses confirmation (`--unsafe-mcp`).
    pub fn unsafe_mode() -> Self {
        Self {
            issued: BTreeMap::new(),
            unsafe_mode: true,
        }
    }

    /// Issue a confirmation token for a destructive op. Derived deterministically
    /// from `nonce + now` (no ambient RNG); the host shows it to the user and
    /// echoes it back on the confirmed call.
    pub fn issue(&mut self, nonce: &str, now: u64) -> String {
        let mut h = blake3::Hasher::new();
        h.update(nonce.as_bytes());
        h.update(&now.to_le_bytes());
        let token = h.finalize().to_hex().to_string();
        self.issued.insert(token.clone(), now);
        token
    }

    /// Authorize a tool call. Non-destructive tools always pass; destructive
    /// tools require a valid, unexpired, **single-use** token (consumed here) or
    /// `--unsafe-mcp`.
    pub fn authorize(
        &mut self,
        kind: ToolKind,
        token: Option<&str>,
        now: u64,
    ) -> Result<(), GovernanceError> {
        if kind != ToolKind::Destructive || self.unsafe_mode {
            return Ok(());
        }
        let token = token.ok_or(GovernanceError::ConfirmationRequired)?;
        let issued_at = *self
            .issued
            .get(token)
            .ok_or(GovernanceError::InvalidToken)?;
        if now.saturating_sub(issued_at) > TOKEN_TTL_SECS {
            self.issued.remove(token); // expired tokens never become valid again
            return Err(GovernanceError::ExpiredToken);
        }
        self.issued.remove(token); // single-use: consume on success
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_destructive_needs_no_token() {
        let mut g = Governance::new();
        assert!(g.authorize(ToolKind::ReadOnly, None, 0).is_ok());
        assert!(g.authorize(ToolKind::Mutative, None, 0).is_ok());
    }

    #[test]
    fn destructive_without_token_is_rejected() {
        // Gate `vector_mcp_governance_bypass_rejected`: the core guarantee.
        let mut g = Governance::new();
        assert_eq!(
            g.authorize(ToolKind::Destructive, None, 0),
            Err(GovernanceError::ConfirmationRequired)
        );
    }

    #[test]
    fn invalid_token_rejected() {
        let mut g = Governance::new();
        assert_eq!(
            g.authorize(ToolKind::Destructive, Some("deadbeef"), 0),
            Err(GovernanceError::InvalidToken)
        );
    }

    #[test]
    fn valid_token_authorizes_once() {
        let mut g = Governance::new();
        let tok = g.issue("delete path 7", 100);
        assert!(g.authorize(ToolKind::Destructive, Some(&tok), 150).is_ok());
        // single-use: the second attempt fails.
        assert_eq!(
            g.authorize(ToolKind::Destructive, Some(&tok), 160),
            Err(GovernanceError::InvalidToken)
        );
    }

    #[test]
    fn expired_token_rejected() {
        let mut g = Governance::new();
        let tok = g.issue("clear scene", 0);
        assert_eq!(
            g.authorize(ToolKind::Destructive, Some(&tok), TOKEN_TTL_SECS + 1),
            Err(GovernanceError::ExpiredToken)
        );
    }

    #[test]
    fn unsafe_mode_bypasses() {
        let mut g = Governance::unsafe_mode();
        assert!(g.authorize(ToolKind::Destructive, None, 0).is_ok());
    }

    #[test]
    fn tool_classification() {
        assert_eq!(tool_kind("vector_paint_shape"), Some(ToolKind::Mutative));
        assert_eq!(tool_kind("vector_delete_path"), Some(ToolKind::Destructive));
        assert_eq!(tool_kind("vector_query_shape"), Some(ToolKind::ReadOnly));
        assert_eq!(tool_kind("nonexistent"), None);
    }
}
