#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! `ph2d-anim` — keyframe store + editable curves (Camada 1 of the Motion
//! Nodes study, `docs/Motion Nodes/00_estudo_estado_da_arte.md`).
//!
//! This crate gives **real** implementations to the two animation traits that
//! ship only as mocks today in [`ph2d_vector_traits`]:
//!
//! - [`Track`] implements [`AttributeEvaluator`] — a keyframe store that
//!   resolves a parameter at an absolute time `t` (seconds).
//! - [`AnimCurve`] implements [`AnimationCurveSampler`] — a first-class,
//!   editable curve (the object of the future graph editor).
//!
//! # Design decisions (see the study for rationale — do not re-litigate)
//!
//! 1. **Time storage = [`RationalTime`]** (OpenTimelineIO pattern):
//!    integer `num`/`den` so long sessions never drift. `f64` appears **only**
//!    at the boundary of [`AttributeEvaluator::sample`] / [`AnimationCurveSampler::at`].
//! 2. **Value interpolation reuses [`LinearInterp`]** from `ph2d-vector-traits`
//!    (OKLCH hue takes the short arc) — this crate never re-implements value lerp.
//! 3. **Easing = internal MIT copy** of the classic Penner / `simple_easing`
//!    equations (no crates.io dependency). Polynomial families are
//!    transcendental-free ([`Easing::is_deterministic`] `== true`).
//! 4. **Hybrid model:** a keyframe is *sampleable data*, not a subgraph. This
//!    crate delivers only the data + sampler; the wiring into the graph
//!    (`motion.clip`) and the app timeline lives in future lines.
//!
//! # Target agnosticism
//!
//! A [`Track`] is target-agnostic; a [`Clip`] binds tracks to opaque
//! [`AnimTarget`]s. `ph2d-anim` never knows what an [`AnimTarget`] *is* — the
//! consumer (the general timeline / motion bridge) maps the opaque key to a
//! setter. This is what keeps the crate app-general and isolated.
//!
//! # Thread-safety
//!
//! [`Track`] and [`Clip`] are plain data (+ a relaxed atomic playback cursor),
//! so they are `Send + Sync` and satisfy
//! `Box<dyn AttributeEvaluator + Send + Sync>` at the future bridge site.
//!
//! ```
//! use ph2d_anim::{AnimValue, AttributeEvaluator, Interp, Key, RationalTime, Track};
//!
//! // A 0 → 10 linear ramp over one second.
//! let track = Track::new(vec![
//!     Key { t: RationalTime::from_seconds(0.0), value: AnimValue::Float(0.0), interp: Interp::Linear },
//!     Key { t: RationalTime::from_seconds(1.0), value: AnimValue::Float(10.0), interp: Interp::Hold },
//! ]);
//!
//! // Sampled at the midpoint, a linear segment is the exact average.
//! assert_eq!(track.sample(0.5), AnimValue::Float(5.0));
//! ```

pub(crate) mod anim_value_serde;
pub mod clip;
pub mod curve;
pub mod curve_fit;
pub(crate) mod curve_weighted;
pub mod easing;
pub mod time;
pub mod track;

/// Schema version of the persistable animation data (`Track`/`Clip`/curves).
///
/// The timeline document embeds this so future format changes can migrate on
/// load (HR-14). Bump it whenever a serialized shape changes incompatibly.
pub const SCHEMA_VERSION: u32 = 1;

pub use clip::{AnimTarget, Clip};
pub use curve::{AnimCurve, CurveKey, Interp};
pub use curve_fit::{FitKey, fit_fcurve};
pub use easing::{Easing, EasingFamily, EasingMode};
pub use time::RationalTime;
pub use track::{Key, KeyId, Track};

// Ergonomic re-export of the contracts this crate implements, so consumers can
// import the trait + value type from one place. The traits themselves stay
// owned (and frozen) by `ph2d-vector-traits`.
pub use ph2d_vector_traits::{AnimValue, AnimationCurveSampler, AttributeEvaluator, LinearInterp};
