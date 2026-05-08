//! Governance layer (HR-11): single-use confirmation tokens with TTL.

use std::collections::HashMap;
use std::time::{Duration, Instant};

const TTL: Duration = Duration::from_secs(5 * 60);

/// Token issued by the server to a UI/human, consumed once by a destructive
/// MCP call within `TTL`.
#[derive(Clone, Debug)]
pub struct ConfirmationToken {
    value: String,
}

impl ConfirmationToken {
    pub fn value(&self) -> &str {
        &self.value
    }
}

pub struct ConfirmationStore {
    issued: HashMap<String, Instant>,
    counter: u64,
}

impl ConfirmationStore {
    pub fn new() -> Self {
        Self { issued: HashMap::new(), counter: 0 }
    }

    pub fn issue(&mut self) -> ConfirmationToken {
        self.counter += 1;
        // For the spike skeleton: deterministic-ish value derived from counter +
        // process-relative time. Real impl: cryptographically random + signed.
        let value = format!("cnf-{:016x}", self.counter ^ Self::process_relative_nanos());
        self.issued.insert(value.clone(), Instant::now());
        ConfirmationToken { value }
    }

    /// Returns true if the token was valid and unused; consumes it.
    pub fn consume(&mut self, value: &str) -> bool {
        match self.issued.remove(value) {
            Some(issued_at) => issued_at.elapsed() <= TTL,
            None => false,
        }
    }

    fn process_relative_nanos() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos() as u64)
            .unwrap_or(0)
    }
}

impl Default for ConfirmationStore {
    fn default() -> Self {
        Self::new()
    }
}
