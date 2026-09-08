//! **Whole-stream reductions** as side-metadata (ADR-0126) — the 6th resolver
//! channel, sibling of the kernel / grid / state-select / stream-op / algorithm
//! families, and the one the DEFORMERS needed.
//!
//! ## The shape no per-element kernel can express
//!
//! Every deformer in the library is built the same way, and it is not the
//! per-element shape the other 43 kernels are:
//!
//! ```text
//!     reduce (one number about the WHOLE layout) -> broadcast -> per-element map
//! ```
//!
//! `motion.bend` wraps the layout onto an arc scaled to its **X extent**;
//! `motion.twist` turns the rim by an angle scaled to the **max radius**;
//! `motion.spherize` bulges around the **centroid**. Element `i`'s answer
//! depends on a number that does not exist until every element has been looked
//! at — which is precisely why the whole family shipped CPU-only.
//!
//! A node declares the reduction it needs; the sequencer runs it BEFORE the
//! node's own kernel pass and hands the body the single result. The machinery
//! (the tree reduction, its block seam, its recursion) lives once in
//! `ph2d-gpu-cook`, exactly like the neighbourhood grid's build — the node
//! authors only *what* to fold and *over what*.
//!
//! ## Why the operator enum lives HERE and not in the GPU crate
//!
//! [`ReduceOp`] carries three things — a WGSL combine, a WGSL identity, and the
//! CPU fold that is the parity oracle. Those are three answers to ONE question
//! (*"what does this operator mean?"*), and splitting them across a pure-data
//! crate and a wgpu crate is how the device's `max` and the host's `max` come to
//! disagree about a NaN or an identity with nobody noticing. `ph2d-gpu-cook`
//! uses this type; it does not define a second one.

/// Which reduction to run over the stream.
///
/// The variants are ordered by how much they promise: the two exact ones first,
/// the order-dependent one last.
///
/// ## `Max`/`Min` are BIT-EXACT; `Sum` is not, and that is a fact about floats
///
/// A tree reduction visits the elements in a different order than a sequential
/// `fold`. For [`Self::Max`] and [`Self::Min`] that is **irrelevant**: they are
/// associative *and exact* over floats, so **every** evaluation order yields the
/// identical bit pattern — the primitive's parity gate asserts equality rather
/// than an ε, and does so by mathematics rather than by luck.
///
/// [`Self::Sum`] is a different animal: float addition is not associative, so
/// the tree's answer differs from the sequential one and its gate carries a
/// documented ε. (It is the same reason the Voronoi cook accumulates its
/// centroid in **integers**.) A caller that needs a bit-exact whole-stream sum
/// should quantise, not pretend.
///
/// ⚠️⚠️ **«Nos últimos ulps» era o que esta nota dizia, e a medição desmentiu-a
/// por quatro ordens de grandeza** (ciclo 3, W1 — doc 106 §4): num layout de
/// 409 600 elementos afastado da origem, a soma sequencial em `f32` do
/// `motion.spherize` divergia da árvore do dispositivo em **54 365 % da barra de
/// paridade**. As parciais chegam à magnitude do layout INTEIRO antes de se voltar
/// a dividir por `n`, e o erro sequencial cresce como `√n · ulp` contra o
/// `log n · ulp` da árvore — *das duas, a do dispositivo é a mais certa*. ⇒ quem
/// dobra esta coluna no hospedeiro usa [`centroid_of`], que acumula em `f64`.
///
/// ⚠️ **NaN is out of contract.** Rust's `f32::max` returns the non-NaN operand;
/// WGSL's `max` with a NaN operand is implementation-defined. The columns this
/// folds are positions and radii, and a NaN in one is already a bug upstream —
/// so this promises nothing about NaN rather than promising what it cannot keep
/// on both paths.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ReduceOp {
    /// Largest element. **Exact in any order** → bit-exact CPU↔GPU parity.
    Max,
    /// Smallest element. **Exact in any order** → bit-exact CPU↔GPU parity.
    Min,
    /// Sum. Float addition is not associative → the tree's answer differs from a
    /// sequential fold in the last ulps (documented ε, never a bit-exact claim).
    Sum,
}

impl ReduceOp {
    /// The WGSL expression combining two accumulators `a` and `b`.
    #[must_use]
    pub const fn wgsl_combine(self) -> &'static str {
        match self {
            ReduceOp::Max => "max(a, b)",
            ReduceOp::Min => "min(a, b)",
            ReduceOp::Sum => "a + b",
        }
    }

    /// The identity element — what a lane past the end contributes. It has to be
    /// the operator's true identity, not merely "a big number": a `Max` seeded
    /// with `0.0` silently reports `0` for an all-negative column, which is the
    /// kind of wrong that looks plausible on every fixture anyone writes.
    #[must_use]
    pub const fn wgsl_identity(self) -> &'static str {
        match self {
            // Not `-1.0/0.0`: WGSL const-evaluates that to an error. The most
            // negative finite f32 is the identity for every finite input, and
            // NaN/inf are already out of contract (see the type docs).
            ReduceOp::Max => "-3.40282347e+38",
            ReduceOp::Min => "3.40282347e+38",
            ReduceOp::Sum => "0.0",
        }
    }

    /// The CPU oracle for this operator over `data` — the canonical answer the
    /// gates reconcile the device against. Sequential by construction: for
    /// `Max`/`Min` that is the same number the tree gets, and for `Sum` it is
    /// deliberately the *reference* order, not a second tree.
    #[must_use]
    pub fn cpu(self, data: &[f32]) -> f32 {
        match self {
            ReduceOp::Max => data.iter().copied().fold(f32::NEG_INFINITY, f32::max),
            ReduceOp::Min => data.iter().copied().fold(f32::INFINITY, f32::min),
            ReduceOp::Sum => data.iter().copied().fold(0.0, |a, b| a + b),
        }
    }

    /// The value an **empty** stream reduces to.
    ///
    /// A reduction over nothing has no measured answer, so this is the number the
    /// node would have used anyway — and every deformer already has one, because
    /// the CPU paths all guard the degenerate layout (`x_extent < MIN_ANGLE_RAD`
    /// → identity, `r_max.max(MIN_RADIUS)`). Publishing the operator's identity
    /// is the honest choice: it is the answer that makes the *map* fall into its
    /// own degenerate branch, rather than a sentinel the body would have to
    /// re-check.
    #[must_use]
    pub const fn empty(self) -> f32 {
        match self {
            ReduceOp::Max => f32::MIN,
            ReduceOp::Min => f32::MAX,
            ReduceOp::Sum => 0.0,
        }
    }
}

/// One whole-stream reduction a node's kernel needs, registered on the side like
/// its [`GridSpec`](crate::gpu::GridSpec).
///
/// Before the node's own pass the sequencer maps `column` on input `port`
/// through the [`Self::value`] expression to one `f32` per element, folds that
/// with [`Self::op`], and gives the generated module `reduce_<name>()` returning
/// the single result. Declaring a reduction is append-only side metadata; no
/// kernel that lacks one changes in any way.
///
/// **A slice, not a single spec** — [`crate::gpu::KernelResolver::reduces`]
/// returns `&[ReduceSpec]` — because the reduction has to be NAMED for the body
/// to read (a fixed `reduce_result()` would be unreadable the moment a node has
/// two), and once it is named, plurality costs nothing. `motion.spherize`'s
/// centroid is the second client: two `Sum`s, over `P.x` and `P.y`.
///
/// ⚠️ **The value expression is where parity is won or lost.** `Max` over given
/// data is bit-exact (see [`ReduceOp`]), but the number being folded is computed
/// by `value` on the device and by the node's own Rust on the host, and a
/// multiply-add there may be contracted into an FMA on one side and not the
/// other. An expression built only from subtraction and `abs` (`motion.bend`'s
/// `|x − pivot|`) has nothing to contract and stays exact; one with a product
/// (`motion.twist`'s radius) is an ε. Write the expression to mirror the CPU's
/// operations in the same order, and let the node's parity gate say which of the
/// two it got.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ReduceSpec {
    /// The symbol the kernel body calls: `reduce_<name>()`. Namespaced by
    /// convention when a node has several (`x_extent`, `cx`, `cy`).
    pub name: &'static str,
    /// The stream column the reduction reads.
    pub column: &'static str,
    /// The column's element type — `value` is written against a `v` of this type.
    pub dim: crate::port::Dim,
    /// Which input port that column is read from (`0` for a per-element node).
    pub port: usize,
    /// The value an **absent** column reads as, per element — the same identity
    /// its [`ColumnBinding`](crate::gpu::ColumnBinding) declares, and it must
    /// agree with it.
    ///
    /// ⚠️ **This is not defensive padding; without it the node answers WRONG.**
    /// The CPU deformers materialise an absent `P` as `vec![[0,0]; n]` and then
    /// reduce over *that* — so `motion.bend` with no `P` and a pivot at `x = 3`
    /// measures an extent of `3`, not "no extent". A reduction that skipped the
    /// absent column and published the operator's identity would take the
    /// degenerate branch while the CPU took the arc branch: the same graph,
    /// two different pictures, no error anywhere.
    pub identity: [f32; 4],
    /// How the per-element values are folded.
    pub op: ReduceOp,
    /// A WGSL **expression** yielding the `f32` to fold for one element, written
    /// against `v` (the column element, typed by [`Self::dim`]) and
    /// `params.<name>` for each declared param. No statements, no semicolon —
    /// the sequencer pastes it into a `return …;`.
    pub value: &'static str,
    /// The manifest params [`Self::value`] reads. Order defines the reduction
    /// pass's own uniform layout; names must be declared `ParamSpec`s of the
    /// node.
    ///
    /// **Declared separately from the kernel's `params`** even when they overlap
    /// (they always will): the reduce runs as its OWN dispatch with its own
    /// uniform, and inheriting the kernel's list would make a body's param edit
    /// silently re-lay-out a pass it does not belong to.
    pub params: &'static [&'static str],
}

/// ⭐⭐⭐ **O CENTROIDE de uma coluna `P`, na CPU** — a metade de hospedeiro do par
/// `Sum(v.x)` / `Sum(v.y)` que os nós declaram como [`ReduceSpec`], e a ÚNICA
/// porta que a responde (ciclo 3, W1 — doc 106 §4).
///
/// `None` para uma lista vazia: uma média de nada não é zero, é ausente, e quem
/// chama fica então sem nada em torno de que pivotar.
///
/// ## ⚠️⚠️ O acumulador é `f64`, e isso NÃO é zelo — é a barra de paridade
///
/// A nota do [`ReduceOp::Sum`] acima diz que a árvore e o fold sequencial *«diferem
/// nos últimos ulps»*. **Medido, é muito mais que isso**, porque as parciais
/// chegam à magnitude do layout INTEIRO antes de se voltar a dividir por `n`, e a
/// soma sequencial erra como um passeio aleatório de `√n · ulp` enquanto a árvore
/// erra `log n · ulp` — *das duas, a do dispositivo é a mais certa*.
///
/// | nó | fixtura | antes (fold `f32`) | depois (`f64`) |
/// |---|---|---:|---:|
/// | `motion.spherize` | 409 600 elementos, layout a `4` da origem | **54 365 % da barra** (`2e-4`) | **7,6 %** |
/// | `motion.spherize` | 16 384 — *o tamanho do próprio gate* | **535 %** | 1,9 % |
/// | `motion.transform` | 409 600, layout a `4,3` | **917 % da barra** (`2e-3`) | 0,4 % |
///
/// ⛔ **E os gates de paridade dos dois estavam VERDES** — cada um mede um tamanho
/// (e o do spherize uma lente de raio `6`, pequena de mais para o centroide
/// morder). *Uma folga medida num tamanho é uma afirmação sobre esse tamanho.*
/// O gate que fixa esta função é [`tests::the_centroid_is_the_mean_and_not_a_random_walk`],
/// corre na CPU e portanto **no CI** — ao contrário dos de paridade, que precisam
/// de adapter e são `#[ignore]`.
#[must_use]
pub fn centroid_of(p: &[[f32; 2]]) -> Option<[f32; 2]> {
    if p.is_empty() {
        return None;
    }
    let sum = p.iter().fold([0.0f64, 0.0], |a, q| {
        [a[0] + f64::from(q[0]), a[1] + f64::from(q[1])]
    });
    #[expect(
        clippy::cast_precision_loss,
        reason = "uma contagem de elementos — exacta ate' 2^53"
    )]
    let n = p.len() as f64;
    #[expect(clippy::cast_possible_truncation, reason = "o consumidor e' f32")]
    Some([(sum[0] / n) as f32, (sum[1] / n) as f32])
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐⭐ **O centroide é a MÉDIA, não um passeio aleatório** — o gate de
    /// [`centroid_of`], e a prova de mutação dele está no doc daquela função.
    ///
    /// ⚠️ **A fixtura tem de estar LONGE da origem e ter MANTISSA CHEIA.** `n`
    /// cópias de `4.0` somam-se **exactamente** em `f32` (`65 536 × 4 = 2^18`) e
    /// deixariam este gate verde sobre o fold que ele existe para reprovar; e a
    /// `40` em vez de `400` a mutação morre por `2,1×` só, contra os `159×` de
    /// hoje — *o erro é proporcional à magnitude das parciais, então a distância
    /// à origem é metade da fixtura*.
    #[test]
    fn the_centroid_is_the_mean_and_not_a_random_walk() {
        const N: usize = 65_536;
        let pts: Vec<[f32; 2]> = (0..N)
            .map(|i| {
                #[expect(clippy::cast_precision_loss, reason = "uma fixtura")]
                let t = i as f32;
                [400.0 + t * 0.000_173, -250.0 + t * 0.000_291]
            })
            .collect();
        let exact = {
            let s = pts.iter().fold([0.0f64, 0.0], |a, q| {
                [a[0] + f64::from(q[0]), a[1] + f64::from(q[1])]
            });
            #[expect(clippy::cast_precision_loss, reason = "uma contagem")]
            let n = N as f64;
            [s[0] / n, s[1] / n]
        };
        let c = centroid_of(&pts).expect("ha' posicoes");
        for k in 0..2 {
            let d = (f64::from(c[k]) - exact[k]).abs();
            assert!(
                d <= 1e-4,
                "eixo {k}: o centroide deu {} e a media exacta e' {} (|Δ| {d:e} > 1e-4) — \
                 uma soma sequencial em f32 sobre parciais da magnitude do layout",
                c[k],
                exact[k]
            );
        }
        // Uma média de nada é ausente, não zero.
        assert_eq!(centroid_of(&[]), None);
    }

    /// The identity is the OPERATOR's, on both sides of the fence. A `Max`
    /// seeded with `0.0` answers `0` for an all-negative column — plausible,
    /// wrong, and agreed with by every fixture that happens to hold a positive
    /// number. Pinned here rather than only in the device gate because this is a
    /// fact about the enum, and the device gate needs an adapter to run.
    #[test]
    fn the_identity_is_the_operators_own_not_merely_a_big_number() {
        assert_eq!(ReduceOp::Max.cpu(&[]), f32::NEG_INFINITY);
        assert_eq!(ReduceOp::Min.cpu(&[]), f32::INFINITY);
        assert_eq!(ReduceOp::Sum.cpu(&[]), 0.0);
        // An all-negative column reports its own maximum; an all-positive one its
        // own minimum. These are the cases a zero-seeded fold gets wrong while
        // agreeing with every fixture that happens to straddle zero.
        assert_eq!(ReduceOp::Max.cpu(&[-3.0, -1.0, -7.0]), -1.0);
        assert_eq!(ReduceOp::Min.cpu(&[3.0, 1.0, 7.0]), 1.0);

        // ⚠️ **The WGSL literal and the Rust identity must be the SAME NUMBER**,
        // which is stronger and more useful than pinning the literal's text: the
        // device seeds its padding lanes from the string and the host reasons
        // about `empty()`, so a typo in one digit is a device that disagrees with
        // its own oracle by an amount no fixture would reveal. Parsed, not
        // compared as text — a reformat of the literal must not fail this, and a
        // changed VALUE must.
        for op in [ReduceOp::Max, ReduceOp::Min, ReduceOp::Sum] {
            let parsed: f32 = op
                .wgsl_identity()
                .parse()
                .expect("the WGSL identity must be a valid f32 literal");
            assert_eq!(
                parsed.to_bits(),
                op.empty().to_bits(),
                "{op:?}: the device's seed and the host's empty answer must agree"
            );
        }
    }

    /// `Max`/`Min` are associative AND exact, so the tree order the device uses
    /// and the sequential order the host uses provably agree — this is what
    /// licenses the bit-exact parity claim, and it is asserted rather than
    /// assumed. `Sum` deliberately gets no such assertion.
    #[test]
    fn max_and_min_are_exactly_associative_so_any_tree_order_agrees() {
        let data: Vec<f32> = (0..1000)
            .map(|i| ((i * 7919) % 1000) as f32 * 0.031 - 15.5)
            .collect();
        for op in [ReduceOp::Max, ReduceOp::Min] {
            // Fold as a balanced tree, the way a workgroup does.
            let mut level: Vec<f32> = data.clone();
            while level.len() > 1 {
                level = level
                    .chunks(2)
                    .map(|c| match c {
                        [a, b] => op.cpu(&[*a, *b]),
                        [a] => *a,
                        _ => unreachable!(),
                    })
                    .collect();
            }
            assert_eq!(
                op.cpu(&data).to_bits(),
                level[0].to_bits(),
                "{op:?}: a tree fold and a sequential fold must agree BIT for bit"
            );
        }
    }

    /// The empty-stream answer is the operator's identity, so a degenerate
    /// layout lands in the map's own degenerate branch instead of needing a
    /// sentinel the body would have to re-test.
    #[test]
    fn an_empty_reduction_publishes_the_operators_identity() {
        assert_eq!(ReduceOp::Max.empty(), f32::MIN);
        assert_eq!(ReduceOp::Min.empty(), f32::MAX);
        assert_eq!(ReduceOp::Sum.empty(), 0.0);
        // Finite, so it can be written into a buffer and read back as a number.
        assert!(ReduceOp::Max.empty().is_finite());
        assert!(ReduceOp::Min.empty().is_finite());
    }
}
