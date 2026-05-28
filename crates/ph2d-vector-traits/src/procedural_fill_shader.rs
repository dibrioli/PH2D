//! `ProceduralFillShader` trait — compile a fill DAG into WGSL source.
//!
//! W1 mocks return a stub WGSL string. The real DAG → WGSL compiler arrives
//! in W6 via `ph2d-vector-fill` (per [ADR-0060](../../../../docs/architecture/decisions/0060-vector-procedural-fill.md)).

/// Compiled WGSL source string.
///
/// Newtype around `String` for documentation; opaque otherwise. Producers
/// guarantee the source is syntactically valid WGSL; consumers feed it to
/// `naga` / `wgpu::ShaderModule`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WgslSource(String);

impl WgslSource {
    /// Wrap an owned WGSL source string.
    #[must_use]
    pub fn new(src: impl Into<String>) -> Self {
        Self(src.into())
    }

    /// Borrow the source as `&str`.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume and return the owned source string.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

/// Procedural fill shader compiler.
///
/// `compile` is NOT called on the render hot path — it runs once when the
/// shader DAG topology changes (per [ADR-0060 §2 topology vs UBO split](../../../../docs/architecture/decisions/0060-vector-procedural-fill.md)),
/// so allocation here is permitted.
pub trait ProceduralFillShader {
    /// Compile the DAG into a WGSL source string.
    fn compile(&self) -> WgslSource;
}
