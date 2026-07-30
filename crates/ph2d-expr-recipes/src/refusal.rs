//! **The refusals** — the ten things the catalog does NOT do, and where they live.
//!
//! Each of these already has an owner in this product. Offering them here would be
//! the second door to a fact that already has an answer, which is the disease this
//! repo has paid for most. But refusing silently is worse than offering: the artist
//! types `loop`, finds nothing, and concludes the tool cannot loop.
//!
//! So the search answers for them. Typing `loop` returns a card that says the loop
//! of a track lives in its **Extrapolation**, with somewhere to go. The refusal
//! list turns the modal into the product's map of *where things live* — and no
//! competitor does this.

/// Where a refused idea actually lives. The UI turns this into a destination the
/// artist can be taken to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Destination {
    /// The track's out-of-range setting (`Track.extrap`, ADR-0143).
    TrackExtrapolation,
    /// The graph editor's tangents (`Interp::BezierW`).
    GraphEditor,
    /// The per-object clock (`PropKind::TimeRemap`).
    TimeRemapTrack,
    /// The motion path (`PropKind::Position`, ADR-0141).
    MotionPath,
    /// Motion Nodes — per-instance `i`/`n`.
    MotionNodes,
    /// The physics module (ADR-0131).
    PhysicsPanel,
    /// The `field.*` node family.
    FieldNodes,
    /// Shape interpolation (`ph2d-vec-blend` / `VecMorph`).
    VectorBlend,
    /// The Flip strip's exposure.
    FlipExposure,
    /// **A keyframe.** The answer to the whole Logic family (FASE A): *"acontece a partir
    /// de tal segundo"* is a key, and this app has a timeline.
    Keyframes,
    /// **Two rows in THIS card.** ⚠️ O único destino que aponta para dentro: quando a
    /// ideia é a COMPOSIÇÃO de duas receitas que já existem, um card próprio para ela é
    /// uma terceira resposta a uma pergunta que a PILHA já responde — e foi por isso que a
    /// família Field saiu (`fade-by-distance ~> distance-2d`, 6e-8).
    TwoRows,
}

impl Destination {
    /// Where to send the artist, in words (English — HR-15).
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Destination::TrackExtrapolation => "the track's Extrapolation",
            Destination::GraphEditor => "the graph editor",
            Destination::TimeRemapTrack => "a Time Remap track",
            Destination::MotionPath => "the motion path",
            Destination::MotionNodes => "Motion Nodes",
            Destination::PhysicsPanel => "the Physics panel",
            Destination::FieldNodes => "the field nodes",
            Destination::VectorBlend => "shape Blend",
            Destination::FlipExposure => "the Flip strip's exposure",
            Destination::Keyframes => "a keyframe on the timeline",
            Destination::TwoRows => "two rows, stacked here",
        }
    }
}

/// One thing the catalog refuses, and where it belongs instead.
#[derive(Clone, Copy, Debug)]
pub struct Refusal {
    pub key: &'static str,
    /// What the artist would call it.
    pub title: &'static str,
    /// The names the search must answer to.
    pub aliases: &'static [&'static str],
    /// One line: what it is, and why not here.
    pub body: &'static str,
    pub to: Destination,
}

/// Every refusal, with its destination.
pub const REFUSALS: &[Refusal] = &[
    Refusal {
        key: "condition",
        title: "Conditions",
        aliases: &[
            // Os RÓTULOS das seis receitas de Logic + o Switch, mais o vocabulário delas.
            // Cortar sem herdar é esconder capacidade; aqui não há sobrevivente, então o
            // que se herda é o CAMINHO (ver `retired.rs`).
            "if",
            "if greater",
            "if less",
            "if near",
            "gate",
            "gate both",
            "gate either",
            "after time",
            "switch",
            "condition",
            "threshold",
            "compare",
            "greater than",
            "less than",
            "when",
            "trigger",
            "and",
            "or",
        ],
        body: "A condition on a number is programming. What an animator wants — \"this \
               happens from here on\" — is a keyframe, and the timeline has one.",
        to: Destination::Keyframes,
    },
    Refusal {
        key: "compose",
        title: "Fields by Distance",
        aliases: &[
            "fade by distance",
            "scale by proximity",
            "driven by another",
            "falloff",
            "proximity fade",
            "near far",
            "distance opacity",
            "dim",
            "react to distance",
            "magnet scale",
            "closeness",
            "grow near",
            "remap link",
            "driver",
            "controlled by",
            "map from",
        ],
        body: "Reacting to a distance is Distance and then Remap - two rows, stacked. A \
               card of its own would be a third answer to what the stack already does.",
        to: Destination::TwoRows,
    },
    Refusal {
        key: "loop",
        title: "Loop",
        aliases: &[
            "loop",
            "loopout",
            "loopin",
            "cycle",
            "repeat track",
            "pingpong track",
        ],
        body: "Repeating a track past its last key is the track's own setting, not a formula.",
        to: Destination::TrackExtrapolation,
    },
    Refusal {
        key: "ease",
        title: "Ease",
        aliases: &[
            "ease",
            "easing",
            "ease in",
            "ease out",
            "smooth keys",
            "bezier",
            "linear ease",
        ],
        body: "How a key eases into the next one is the curve's tangents.",
        to: Destination::GraphEditor,
    },
    Refusal {
        key: "time-remap",
        title: "Retime",
        aliases: &[
            "time remap",
            "retime",
            "slow down",
            "speed up object",
            "freeze frame",
            "reverse object",
        ],
        body: "Retiming everything an object does is its own clock, not a per-property formula.",
        to: Destination::TimeRemapTrack,
    },
    Refusal {
        key: "path",
        title: "Follow a Path",
        aliases: &[
            "motion path",
            "along path",
            "trajectory",
            "follow curve",
            "spline",
        ],
        body: "Moving along a drawn path is what a motion path is.",
        to: Destination::MotionPath,
    },
    Refusal {
        key: "stagger",
        title: "Stagger Copies",
        aliases: &[
            "stagger",
            "offset copies",
            "index",
            "per instance",
            "duplicates",
            "delay each",
        ],
        body: "Offsetting many copies needs a per-copy index, which lives where the copies do.",
        to: Destination::MotionNodes,
    },
    Refusal {
        key: "physics",
        title: "Real Physics",
        aliases: &[
            "collide",
            "collision",
            "rigid body",
            "spring physics",
            "bounce off",
            "gravity real",
        ],
        body: "Bodies that collide and rest on each other need the solver, not an expression.",
        to: Destination::PhysicsPanel,
    },
    Refusal {
        key: "forces",
        title: "Attract / Repel",
        aliases: &[
            "attract",
            "repel",
            "vortex",
            "wind",
            "drag force",
            "magnet",
            "swirl",
        ],
        body: "Forces acting on many particles are nodes, not a property formula.",
        to: Destination::MotionNodes,
    },
    Refusal {
        key: "field",
        title: "Falloff Field",
        aliases: &[
            "falloff field",
            "influence region",
            "spatial field",
            "radial field",
            "box field",
        ],
        body: "A field of influence over space is its own node family.",
        to: Destination::FieldNodes,
    },
    Refusal {
        key: "morph",
        title: "Morph Between Shapes",
        aliases: &[
            "morph shapes",
            "shape blend",
            "interpolate shapes",
            "tween shapes",
        ],
        body: "Turning one drawn shape into another is shape blending, not a number.",
        to: Destination::VectorBlend,
    },
    Refusal {
        key: "hold-frame",
        title: "Hold a Drawing",
        aliases: &[
            "hold frame",
            "exposure",
            "on twos drawing",
            "strobe drawing",
            "shoot on",
        ],
        body: "How long a drawing stays on screen is its exposure on the strip.",
        to: Destination::FlipExposure,
    },
];
