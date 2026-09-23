//! **Multi-pass GPU algorithms** as side-metadata (ADR-0139) — the 5th resolver
//! channel, sibling of the kernel / grid / state-select / stream-op families.
//!
//! A source node whose cook is an ALGORITHM — several dependent passes with
//! internal scratch, producing a stream — is neither a per-element map
//! ([`crate::gpu::GpuKernel`]) nor a structural stream operation
//! ([`crate::gpu::StreamOp`]). The node opts in by naming which of the engine's
//! algorithms it is and how its params map; the MACHINERY lives once in the
//! sequencer (`ph2d-gpu-cook`), exactly like the neighbourhood grid's build.

/// Which engine algorithm a node's GPU cook runs, and how its manifest params
/// feed it. Pure `'static` data (ADR-0126).
#[derive(Copy, Clone, Debug)]
pub enum GpuAlgorithm {
    /// **Lloyd relaxation toward a CVT via Jump Flooding** (ADR-0139) — the
    /// device form of `motion.voronoi`: seed `count` hashed points, then per
    /// iteration JFA-assign a `res²` grid and move each point to its cell's
    /// centroid (accumulated in INTEGER texel indices — exact and
    /// order-independent), finally lerp raw→relaxed by the `relax` port's
    /// row 0. `res` follows the node's own law (`√(count·samples)`, clamped),
    /// so the two paths discretise identically.
    LloydVoronoi {
        count_param: &'static str,
        width_param: &'static str,
        height_param: &'static str,
        seed_param: &'static str,
        iterations_param: &'static str,
        /// The VALUE input carrying `relax` (row 0; absent → 1.0).
        relax_port: usize,
        /// The node's own caps and sampling law, passed so the two paths share
        /// ONE set of numbers (the node crate is the source of truth).
        max_points: usize,
        min_res: usize,
        max_res: usize,
        samples_per_point: usize,
        /// The iteration clamp (the CPU's `clamp(0, 64)`).
        max_iterations: i64,
        /// **A MÉTRICA de distância** (doc 89, folha 01) — o param cujo valor escolhe a
        /// distância com que um texel é de um dono: [`GpuAlgorithm::LLOYD_METRIC_MANHATTAN`],
        /// [`GpuAlgorithm::LLOYD_METRIC_CHEBYSHEV`], e qualquer outro valor a Euclidiana.
        ///
        /// ⛔⛔ Ela chegou à CPU e não à placa: o device inundava SEMPRE com a Euclidiana, e um
        /// Voronoi em Chebyshev saía redondo **pela placa e quadrado pela CPU**, com o controlo
        /// do painel a não fazer nada ali (doc 119 §7, achado pela varredura das cenas de várias
        /// saídas — a `=93` põe as duas métricas lado a lado).
        metric_param: &'static str,
    },
}

impl GpuAlgorithm {
    /// O valor do param de métrica que pede a Manhattan (`L¹`) — ver `metric_param`.
    pub const LLOYD_METRIC_MANHATTAN: i32 = 1;
    /// O valor do param de métrica que pede a Chebyshev (`L∞`) — ver `metric_param`.
    pub const LLOYD_METRIC_CHEBYSHEV: i32 = 2;

    /// The Lloyd sampling-grid side for `count` points: `√(count·samples)`,
    /// clamped — **THE law**, called by the CPU node's `resolution` and the
    /// device service so the two paths discretise identically (a one-texel
    /// difference in `res` is a different assignment, which would read as
    /// parity noise no ε could explain).
    pub fn lloyd_resolution(
        count: usize,
        samples_per_point: usize,
        min_res: usize,
        max_res: usize,
    ) -> usize {
        let ideal = ((count * samples_per_point) as f32).sqrt().ceil() as usize;
        ideal.clamp(min_res, max_res)
    }
}
