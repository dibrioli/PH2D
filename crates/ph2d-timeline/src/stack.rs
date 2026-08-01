//! The clip stack: **lanes** of clip **strips** (ADR-0115).
//!
//! A strip is one *instance* of a clip placed on the timeline — where it plays,
//! which slice of the clip it uses, how fast, and what the source does when it
//! runs out. A lane is an ordered row of strips; lanes stack bottom to top.
//!
//! This module owns the two things a strip knows how to answer, and nothing
//! else — the blend across lanes lives in the evaluator:
//!
//! - **Where in the clip am I?** ([`ClipStrip::source_time`]) — the strip's time
//!   map. It maps *timeline* time to *clip* time. It does NOT touch the entity's
//!   Time Remap: that is the clip's own clock and composes **inside** this one
//!   (ADR-0115 R6, the AE precomp model). One map, one direction, no second
//!   clock invented alongside it.
//! - **How much do I count?** ([`ClipLane::weight_at`]) — the ease curve.
//!
//! **The gesture** (ADR-0115 R1, and the one thing Blender's NLA cannot do):
//! *overlapping two strips on a lane IS the crossfade.* The overlap's width is
//! the blend's duration — nobody types a number, and there are no two numbers to
//! keep in agreement. An authored `ease_in`/`ease_out` only applies where a strip
//! has no neighbour to blend against; where it has one, the overlap wins. This is
//! Unity's rule (the field is literally relabelled "Blend" and greyed out when an
//! overlap defines it), and it is what makes ease and blend the same curve rather
//! than two systems that must agree.
//!
//! The crossfade is **exactly complementary** — `w_a + w_b == 1` through the
//! whole overlap — because smoothstep satisfies `s(1 - u) == 1 - s(u)`. That is
//! not a nicety: complementary weights need **no base value**, so the crossfade
//! is immune to the "sag toward the default pose" that Unity ships a whole
//! `AnimationOutputWeightProcessor` to prevent. It is proved in the tests.

use ph2d_anim::Easing;
use serde::{Deserialize, Serialize};

/// What a strip's source does once it runs past its slice.
///
/// There is deliberately **no "Nothing"** variant (Blender's, which stops the
/// strip contributing while its span still covers the time). A strip that spans
/// time it cannot fill is a mis-trimmed strip, not a feature — and a strip whose
/// coverage silently drops to zero mid-span is exactly the hole that lets the
/// stack fall back to a default value and yank the sprite. Trim the strip.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum StripLoop {
    /// Play the slice once, then hold its last value for the rest of the span.
    #[default]
    Once,
    /// Wrap back to the slice's start.
    Loop,
    /// Reflect: play forward, then backward, then forward…
    PingPong,
}

/// How a lane's value enters the stack below it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LaneMode {
    /// Mix toward this lane's value (`lerp`). The lane *replaces* what is under
    /// it, by its coverage and weight.
    #[default]
    Override,
    /// Add this lane's **delta** — its value measured against the first frame of
    /// its own clip. A clip holding a constant pose therefore contributes
    /// nothing, which is the whole point: an additive lane carries *change*, not
    /// position. (Maya: "evaluates the clip relative to its first frame"; Unity's
    /// additive reference pose is frame 0 of the clip.)
    Additive,
}

/// A strip's stable identity, for as long as the document lives.
///
/// A strip cannot be addressed by its index: the lane keeps its strips **sorted
/// by start time**, so dragging one past its neighbour renumbers both. A drag
/// anchored on an index would silently grab the other strip at the exact moment
/// they crossed — which is the moment the animator is looking hardest. Selection
/// and undo have the same problem. Mirrors `KeyId`, for the same reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct StripId(pub u64);

/// **What a strip plays** — a clip, or a whole nested container (ADR-0133).
///
/// This is the ONE field the nesting work changes, and that is the point of the design: a
/// container instance is not a new mechanism, it is a strip whose source happens to be a
/// container. Everything else a strip knows — where it plays, which slice, how fast, how it
/// fades — is exactly the set every product in the research offers as per-instance override
/// (`speed` is Rive's `speed()`, `src_in` is Animate's `First`, `loop_mode` is its loop mode).
///
/// Both variants index their own list on the document (`clips()` / `containers()`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StripSource {
    /// A clip: keys that drive bound objects directly.
    Clip(u16),
    /// A container: an entire nested stack, with its own clock link (ADR-0133 §1).
    Container(u16),
}

impl StripSource {
    /// The clip index, or `None` when this strip plays a container.
    #[must_use]
    pub fn clip_index(self) -> Option<u16> {
        match self {
            Self::Clip(i) => Some(i),
            Self::Container(_) => None,
        }
    }

    /// The container index, or `None` when this strip plays a clip.
    #[must_use]
    pub fn container_index(self) -> Option<u16> {
        match self {
            Self::Container(i) => Some(i),
            Self::Clip(_) => None,
        }
    }
}

/// One placement of a clip on the timeline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClipStrip {
    /// Stable identity (see [`StripId`]). Allocated by the document.
    pub id: StripId,
    /// **What this strip plays** — a clip, or a nested container ([`StripSource`]).
    ///
    /// Was `clip: u16` through `DOC_VERSION` 7. This is a *replacement*, not an append, so
    /// v7 blobs are **rejected** by the version gate rather than misread field-for-field —
    /// the same policy every bump in this document has followed (`DOC_VERSION` 7 -> 8).
    pub source: StripSource,
    /// Timeline seconds: where the strip starts.
    pub t_start: f64,
    /// Timeline seconds: where it ends (exclusive).
    pub t_end: f64,
    /// Clip seconds: the first frame of the slice used.
    pub src_in: f64,
    /// Clip seconds: the end of the slice used.
    pub src_out: f64,
    /// Source playback rate. 1.0 = real time; 2.0 = twice as fast.
    pub speed: f64,
    /// What the source does past its slice.
    pub loop_mode: StripLoop,
    /// Authored fade-in, in seconds. **Ignored where an overlap defines the
    /// blend** — the overlap is the blend (see the module docs).
    pub ease_in: f64,
    /// Authored fade-out, in seconds. Same rule.
    pub ease_out: f64,
    /// **Outward fade-in ("lead-in"), in seconds** — the fade that lives in the GAP
    /// *before* the strip, not inside it (Enio, 2026-07-16). Where `ease_in` blends
    /// against this clip while it PLAYS (its opening is spent in the crossfade), the
    /// lead-in blends against the clip's FROZEN first frame: the object travels from
    /// the previous strip's held pose to this clip's start pose during the gap, and
    /// then the clip plays from frame 0 untouched. It reaches back from `t_start` and
    /// is mutually exclusive with `ease_in` (the fade-in grip is on one side of the
    /// edge or the other). Appended (`DOC_VERSION` 5 -> 6); `0.0` is the old
    /// behaviour byte-for-byte.
    pub lead_in: f64,
    /// **What each corner's last edit DID, in seconds — the change bar** (Enio,
    /// 2026-07-16). Index it with [`mark_index`]; the value is signed, and it is
    /// `edge_before_the_gesture - edge_now`.
    ///
    /// The panel draws the interval between the edge and `edge + mark` in the
    /// corner's own colour, so **which way it points falls out of the sign** rather
    /// than being a second rule to keep in step: pull a start edge outward and the
    /// span it gained is now INSIDE the strip; push it inward and the span it lost
    /// is OUTSIDE. Same formula at both edges, both operations.
    ///
    /// It is a **delta, not a time**, which is what lets it survive a slide: move
    /// the whole strip and the mark travels with the edge it describes, because it
    /// never referred to an absolute moment in the first place.
    ///
    /// It lives in the document (rather than in the panel's drag state) because
    /// "permanently visible" has to mean *after you let go, after an undo, and
    /// after a reload* — a mark that evaporates on load would be a mark that lies
    /// about being permanent. Appended (`DOC_VERSION` 6 -> 7); all-zero is the old
    /// behaviour, and zero draws nothing.
    pub marks: [f64; 4],
    /// **Outward fade-out ("lead-out"), in seconds** — the mirror of [`Self::lead_in`], in
    /// the GAP *after* the strip (Enio, 2026-07-19). Where `ease_out` blends this clip out
    /// while it still PLAYS (its last frames are spent in the crossfade), the lead-out lets
    /// the clip play to its END untouched and THEN fades, in the gap, from the clip's FROZEN
    /// LAST frame toward the next strip's start. It reaches forward from `t_end` and is
    /// mutually exclusive with `ease_out` (the fade-out grip is on one side of the end edge
    /// or the other). Appended (`DOC_VERSION` 8 -> 9); `0.0` is the old behaviour
    /// byte-for-byte.
    pub lead_out: f64,
    /// **A curva do fade de ENTRADA** (Enio, 2026-07-31: *"no menu do botão direito sobre o
    /// fade de uma strip vamos colocar as mesmas opções de easing que temos nos clips"*) —
    /// `None` = a de fábrica.
    ///
    /// ⚠️ **`Option`, e não uma `Easing` com um default, porque a curva de fábrica NÃO ESTÁ
    /// no catálogo:** o fade sempre foi um `smoothstep` (`u²(3−2u)`), e o `Quad InOut` mais
    /// próximo dá 0,125 onde ele dá 0,15625. Guardar um preset como default reescreveria a
    /// forma de todo fade já autorado — e o `fade_fingerprint` diria isso na hora. Com
    /// `None` a ausência de escolha continua sendo o `smoothstep`, byte a byte.
    ///
    /// Uma curva por BORDA (decisão do Enio): a saída pode acelerar enquanto a entrada
    /// desacelera, na mesma strip. Apendado (`DOC_VERSION` 17 -> 18).
    #[serde(default)]
    pub curve_in: Option<Easing>,
    /// **A curva do fade de SAÍDA** — o espelho de [`Self::curve_in`].
    ///
    /// ⚠️ **Na costura de um LOOP esta é a que PREVALECE** (decisão do Enio): as duas
    /// metades da volta são UMA travessia, e duas curvas a moldariam com um joelho no meio.
    /// Fora de um loop não há costura, e aí cada fade usa a sua ([`ClipLane::seam_curve`]).
    #[serde(default)]
    pub curve_out: Option<Easing>,
}

/// Index into [`ClipStrip::marks`] for one corner: `stretch` picks the GREEN top
/// pair over the RED bottom pair, `edge` is the document's usual `0` = start,
/// `1` = end.
///
/// One door, because three places ask this question — the trim apply, the stretch
/// apply, and the painter — and a corner whose mark is written at one index and
/// read at another is a mark that silently describes the wrong edit.
#[must_use]
pub const fn mark_index(stretch: bool, edge: u8) -> usize {
    (if stretch { 2 } else { 0 }) + (edge != 0) as usize
}

impl ClipStrip {
    /// A strip playing all of `clip` over `[t_start, t_end)`, at speed 1, no ease.
    ///
    /// The id is left at zero: authoring goes through [`crate::TimelineDoc`], which
    /// allocates one. (Two strips sharing an id would confuse a drag, not corrupt
    /// the document — the evaluator never reads the id.)
    #[must_use]
    pub fn new(source: StripSource, t_start: f64, t_end: f64, src_len: f64) -> Self {
        Self {
            id: StripId(0),
            source,
            t_start,
            t_end,
            src_in: 0.0,
            src_out: src_len,
            speed: 1.0,
            loop_mode: StripLoop::Once,
            ease_in: 0.0,
            ease_out: 0.0,
            lead_in: 0.0,
            marks: [0.0; 4],
            lead_out: 0.0,
            curve_in: None,
            curve_out: None,
        }
    }

    /// Builder: stamp the identity the document allocated.
    #[must_use]
    pub fn with_id(mut self, id: StripId) -> Self {
        self.id = id;
        self
    }

    /// How long the strip occupies the timeline.
    #[must_use]
    pub fn span(&self) -> f64 {
        (self.t_end - self.t_start).max(0.0)
    }

    /// How much of the clip it uses.
    #[must_use]
    pub fn slice(&self) -> f64 {
        (self.src_out - self.src_in).max(0.0)
    }

    /// `true` while the strip covers `t`.
    #[must_use]
    pub fn covers(&self, t: f64) -> bool {
        t >= self.t_start && t < self.t_end
    }

    /// The **clip** time this strip reads at timeline time `t`, or `None` when it
    /// does not cover `t`.
    ///
    /// This is the strip's whole contract with time. What it hands back is a time
    /// in the clip's own frame — the entity's Time Remap track (which lives in
    /// that clip) then maps it to the entity's source time. Outer map, inner map,
    /// one direction: never a second clock running beside the first.
    #[must_use]
    pub fn source_time(&self, t: f64) -> Option<f64> {
        self.covers(t).then(|| self.fold(t - self.t_start))
    }

    /// Where the outward fade-in reaches back to — `t_start - lead_in`, clamped so a
    /// lead never starts before time zero.
    #[must_use]
    pub fn lead_start(&self) -> f64 {
        (self.t_start - self.lead_in.max(0.0)).max(0.0)
    }

    /// Where the outward fade-OUT reaches forward to — `t_end + lead_out`. The mirror of
    /// [`Self::lead_start`]; equals `t_end` when there is no lead-out.
    #[must_use]
    pub fn lead_end(&self) -> f64 {
        self.t_end + self.lead_out.max(0.0)
    }

    /// The clip time this strip reads INCLUDING its outward lead-in.
    ///
    /// In the lead-in window `[lead_start, t_start)` it returns `src_in` — the clip's
    /// FROZEN first frame, the pose the object travels TO across the gap — regardless
    /// of loop mode (the travel is to the first frame, not to a wrapped one). In the
    /// **lead-OUT** window `[t_end, lead_end)` it returns [`Self::hold_source_time`] — the
    /// clip's FROZEN LAST frame, the pose the object travels FROM as it fades in the gap
    /// after. Inside the span it is [`Self::source_time`]; outside all three, `None`. This
    /// is the door the evaluator samples through, so both lead windows contribute a still
    /// pose, not a negative/extrapolated time.
    #[must_use]
    pub fn source_time_with_lead(&self, t: f64) -> Option<f64> {
        if self.lead_in > 0.0 && t >= self.lead_start() && t < self.t_start {
            Some(self.src_in) // the frozen first frame — the travel target
        } else if self.lead_out > 0.0 && t >= self.t_end && t < self.lead_end() {
            Some(self.hold_source_time()) // the frozen last frame — travel FROM it
        } else {
            self.source_time(t)
        }
    }

    /// **The clip time this strip HOLDS once it is over** — its reading at the very
    /// end of its span.
    ///
    /// A strip's pose does not evaporate at its edge: it persists until something
    /// else takes over, which is what makes a lone fade-in *cross* from the previous
    /// strip instead of from the rest pose (Blender's `Hold` extrapolation, Unity's
    /// clip extrapolation). See [`ClipLane::hold_at`].
    ///
    /// The limit of [`Self::source_time`] as `t → t_end`, not a second opinion about
    /// it: both fold through [`Self::fold`], so a looping strip holds exactly the
    /// frame it was on and cannot disagree with the frame it was showing a moment
    /// before.
    #[must_use]
    pub fn hold_source_time(&self) -> f64 {
        self.fold(self.span())
    }

    /// How far into the clip a strip that has been running for `elapsed` seconds of
    /// TIMELINE time is reading — rate, then whatever the source does past its slice.
    ///
    /// The one place the folding lives. [`Self::source_time`] asks it for an instant
    /// inside the span and [`Self::hold_source_time`] for the span's end.
    pub(crate) fn fold(&self, elapsed: f64) -> f64 {
        let slice = self.slice();
        if slice <= 0.0 {
            return self.src_in; // a zero-length slice is a pose, not a clip
        }
        let advanced = elapsed * self.speed;
        let folded = match self.loop_mode {
            StripLoop::Once => advanced.clamp(0.0, slice),
            StripLoop::Loop => advanced.rem_euclid(slice),
            StripLoop::PingPong => {
                // Reflect over a period of two slices: forward, then backward.
                let u = advanced.rem_euclid(slice * 2.0);
                if u <= slice { u } else { slice * 2.0 - u }
            }
        };
        self.src_in + folded
    }
}

/// A row of strips. Lanes stack bottom to top; the strips inside one are ordered
/// by start time and **may overlap** — that is how a crossfade is authored.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClipLane {
    /// Author-visible name.
    pub name: String,
    /// A muted lane contributes nothing at all — which is **not** the same as a
    /// weight of zero. Zero weight still asserts the lane's coverage and mixes
    /// toward it; muting removes the lane from the stack. (Blender's own layered
    /// design draws this distinction explicitly, having learned it the hard way.)
    pub muted: bool,
    /// The lane's influence over the stack below it, `[0, 1]`.
    pub weight: f64,
    /// How it enters the stack.
    pub mode: LaneMode,
    /// Ordered by `t_start` (see [`ClipLane::insert`]).
    pub strips: Vec<ClipStrip>,
}

impl ClipLane {
    /// A fresh, empty lane at full weight.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            muted: false,
            weight: 1.0,
            mode: LaneMode::Override,
            strips: Vec::new(),
        }
    }

    /// Add a strip, keeping the lane ordered by start time. The order is an
    /// invariant: [`Self::weight_at`] reads a strip's neighbours to find the
    /// overlap that defines its blend, and neighbours only mean something in a
    /// sorted row.
    pub fn insert(&mut self, strip: ClipStrip) -> usize {
        let at = self.strips.partition_point(|s| s.t_start <= strip.t_start);
        self.strips.insert(at, strip);
        at
    }

    /// Where the strip with this identity currently sits, if it is here.
    ///
    /// An index is a *position*, not a name: a drag holds the [`StripId`] and asks
    /// this each time, because moving a strip past its neighbour renumbers both.
    #[must_use]
    pub fn index_of(&self, id: StripId) -> Option<usize> {
        self.strips.iter().position(|s| s.id == id)
    }

    /// Restore the sort after a strip's start time changed — the invariant that
    /// [`Self::weight_at`] rests on (a neighbour only means something in order).
    pub fn resort(&mut self) {
        self.strips
            .sort_by(|a, b| a.t_start.total_cmp(&b.t_start).then(a.id.cmp(&b.id)));
    }

    /// **A COSTURA de um loop, resolvida** — quem são as duas pontas, que curva as molda, e
    /// onde a volta cai dentro dessa curva.
    ///
    /// `None` sem loop (não há costura: cada fade usa a própria curva — decisão do Enio),
    /// quando a última strip não fadeia para fora, ou quando ela é a própria cabeça.
    ///
    /// Resolvida **uma vez por lane por frame** pelo chamador, nunca por strip: ela varre a
    /// lane duas vezes, e `weight_at` roda por strip.
    #[must_use]
    pub fn seam(&self, loop_range: Option<(f64, f64)>) -> Option<Seam> {
        let (a, b) = loop_range?;
        let (ti, tail) = self
            .strips
            .iter()
            .enumerate()
            .max_by(|(_, x), (_, y)| x.t_end.total_cmp(&y.t_end))?;
        let l_out = tail.lead_out.max(0.0) + self.blend_out(ti);
        if tail.t_end > b || l_out <= 0.0 {
            return None;
        }
        let (hi, head) = self
            .strips
            .iter()
            .enumerate()
            .filter(|(_, s)| s.lead_start() <= a && s.lead_end() > a)
            .min_by(|(_, x), (_, y)| x.t_start.total_cmp(&y.t_start))?;
        // A cabeça É a cauda: uma strip só, e não há duas curvas para discordar.
        if hi == ti {
            return None;
        }
        let l_in = head.lead_in.max(0.0) + self.blend_in(hi);
        let total = l_out + l_in;
        (total > 0.0).then(|| Seam {
            curve: tail.curve_out,
            tail: ti,
            head: hi,
            f: l_out / total,
        })
    }

    /// The empty span before strip `i` — from the end of the nearest strip that ends
    /// at or before it (or time 0 if none) up to its start. This is how far a lead-in
    /// may reach without overrunning a neighbour's live span: the outward fade lives in
    /// the GAP, and the gap ends where the previous strip does.
    #[must_use]
    pub fn gap_before(&self, i: usize) -> f64 {
        let s = &self.strips[i];
        let prev_end = self
            .strips
            .iter()
            .enumerate()
            .filter(|(j, o)| *j != i && o.t_end <= s.t_start)
            .map(|(_, o)| o.t_end)
            .fold(0.0_f64, f64::max);
        (s.t_start - prev_end).max(0.0)
    }

    /// The empty span AFTER strip `i` — from its end up to the start of the nearest strip
    /// that starts at or after it. Mirror of [`Self::gap_before`]: how far a lead-OUT may
    /// reach without overrunning the next strip's live span. `f64::INFINITY` when nothing
    /// follows (the last strip has no neighbour to overrun).
    #[must_use]
    pub fn gap_after(&self, i: usize) -> f64 {
        let s = &self.strips[i];
        let next_start = self
            .strips
            .iter()
            .enumerate()
            .filter(|(j, o)| *j != i && o.t_start >= s.t_end)
            .map(|(_, o)| o.t_start)
            .fold(f64::INFINITY, f64::min);
        (next_start - s.t_end).max(0.0)
    }
}

/// The fade curve: `smoothstep(elapsed / window)` by default, and 1 where there is no
/// window — com a escolha do artista quando ela existe.
///
/// `smoothstep(1 - u) == 1 - smoothstep(u)`, which is why two strips sharing an
/// overlap sum to exactly 1 through it (proved in the tests). Complementary
/// weights need no base value to blend against — that property is what keeps the
/// crossfade immune to sagging toward a default pose. ⚠️ É por isso que uma curva autorada
/// **não alcança um crossfade de sobreposição** ([`ClipLane::weight_at_with`]): ali a
/// simetria é load-bearing, e uma curva qualquer não a tem.
///
/// ⚠️ **O `None` NÃO é um preset do catálogo, e é por isso que ele existe**: a curva de
/// fábrica é o `smoothstep`, que o `EasingFamily` não tem (o `Quad InOut` mais próximo dá
/// 0,125 onde ele dá 0,15625). Guardar um preset como default reescreveria a forma de todo
/// fade já autorado; com `None`, um documento que nunca escolheu curva é byte-idêntico.
pub(crate) fn ramp_with(elapsed: f64, window: f64, curve: Option<Easing>) -> f64 {
    if window <= 0.0 {
        return 1.0;
    }
    fade_ramp(elapsed / window, curve)
}

/// **A forma de um fade, dado o quanto dele já passou** (`u`, clampado a `[0, 1]`).
///
/// A metade aritmética do [`ramp_with`], pública porque o PAINEL desenha esta curva dentro
/// da cunha do fade: se ele a re-derivasse, o desenho e o blend seriam duas respostas à
/// pergunta *"que forma tem este fade?"*, e a divergência apareceria só numa screenshot —
/// onde ninguém lê número.
#[must_use]
pub fn fade_ramp(u: f64, curve: Option<Easing>) -> f64 {
    let u = u.clamp(0.0, 1.0); // CLAMP-OK: uma fração de janela
    match curve {
        None => u * u * (3.0 - 2.0 * u),
        Some(e) => e.eval(u),
    }
}

/// **A travessia da costura de um loop** — UMA, com UMA curva, partida pela volta.
///
/// ⚠️ A regra *"a curva da última prevalece sobre a da primeira"* existia porque as duas
/// metades da volta são uma travessia só, e duas curvas a moldariam com um joelho no meio.
/// Ela dava a mesma CURVA aos dois lados e deixava cada um correr um S INTEIRO na própria
/// janela — o que põe o joelho de volta pelo outro caminho: medido, a velocidade subia a
/// −0,299, caía a **0,000 exatamente na volta** e recomeçava. O objeto PARAVA na costura
/// (Enio, 2026-08-01: *"a curva desenhada não expressa essa continuidade, mas deve"*).
///
/// Agora a curva é parametrizada pela travessia INTEIRA e cada ponta desenha e toca a sua
/// FATIA: a cauda `[0, f]`, a cabeça `[f, 1]`, com `f` = a fração do percurso que acontece
/// antes da volta. É isso que faz a curva *começar na fade final e terminar na inicial*.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Seam {
    /// A curva da travessia — a de SAÍDA da cauda (`None` = o `smoothstep` de fábrica).
    pub curve: Option<Easing>,
    /// A strip que fadeia para FORA no fim do alcance.
    pub tail: usize,
    /// A que fadeia para DENTRO no começo dele.
    pub head: usize,
    /// A fração da travessia que cabe ANTES da volta — `janela_de_saída / (saída + entrada)`.
    pub f: f64,
}

/// **Uma fatia da curva única da costura** — o que UMA borda desenha dela (ADR-0115).
///
/// A travessia da volta é uma só: a cauda mostra `[0, f]` e a cabeça `[f, 1]` da MESMA
/// curva, e é assim que ela *começa a ser desenhada na fade final e termina na inicial*
/// (Enio, 2026-08-01). As duas fatias desenham o PROGRESSO da travessia — por isso as duas
/// sobem, e por isso se encontram na volta.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SeamSlice {
    /// Qual borda desta strip carrega a fatia: `0` = entrada (a cabeça), `1` = saída.
    pub edge: u8,
    /// O começo da fatia dentro da curva inteira.
    pub u0: f64,
    /// E o fim dela.
    pub u1: f64,
    /// A curva da travessia — a de SAÍDA da cauda.
    pub curve: Option<ph2d_anim::Easing>,
}

impl Seam {
    /// **A fatia que a strip `i` desenha** — `None` se ela não é nenhuma das duas pontas.
    ///
    /// A cauda leva `[0, f]` na borda de SAÍDA e a cabeça `[f, 1]` na de ENTRADA: uma curva,
    /// duas fatias, e é UMA porta que as reparte (o painel não re-deriva a divisão).
    #[must_use]
    pub fn slice_for(self, i: usize) -> Option<SeamSlice> {
        let (edge, u0, u1) = if self.tail == i {
            (1_u8, 0.0, self.f)
        } else if self.head == i {
            (0_u8, self.f, 1.0)
        } else {
            return None;
        };
        Some(SeamSlice {
            edge,
            u0,
            u1,
            curve: self.curve,
        })
    }
}

impl Seam {
    /// A travessia é de fato PARTIDA pelas duas pontas?
    ///
    /// ⚠️ Com uma ponta só (`f` em 0 ou 1) não há continuidade a resolver — há um fade, um S,
    /// uma travessia — e o caminho fica **byte-idêntico ao que o Enio aprovou**. A
    /// reparametrização só existe onde havia o joelho.
    #[must_use]
    pub fn split(self) -> bool {
        self.f > 0.0 && self.f < 1.0
    }

    /// O progresso da travessia NO instante da volta — a fração em que a pose da costura
    /// mistura as duas pontas ([`ClipLane::seam_split`] a usa no lugar do `f` linear, senão
    /// as duas metades discordariam sobre onde a volta caiu).
    #[must_use]
    pub fn at_wrap(self) -> f64 {
        fade_ramp(self.f, self.curve)
    }

    /// O peso de quem SAI, no ponto `u` da janela dela (`0` no começo do fade, `1` no fim).
    #[must_use]
    pub fn tail_weight(self, u: f64) -> f64 {
        let w = self.at_wrap();
        if w <= 0.0 {
            return 1.0 - u;
        }
        (1.0 - fade_ramp(self.f * u, self.curve) / w).clamp(0.0, 1.0) // CLAMP-OK: um peso
    }

    /// O peso de quem CHEGA, no ponto `u` da janela dela.
    #[must_use]
    pub fn head_weight(self, u: f64) -> f64 {
        let w = self.at_wrap();
        if w >= 1.0 {
            return u;
        }
        ((fade_ramp(self.f + (1.0 - self.f) * u, self.curve) - w) / (1.0 - w)).clamp(0.0, 1.0) // CLAMP-OK: um peso
    }
}
