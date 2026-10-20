//! **Taper** — the stroke thins at its START because it is near the beginning, not because a pen
//! pressed lightly (Procreate's *Touch Taper*; Enio 2026-08-08: *"como Procreate faz com sua ferramenta
//! de Taper (sem pressão da caneta)"*).
//!
//! ## What it is
//!
//! Procreate's Brush Studio splits the idea in two: **Pressure Taper** (lengthen the taper a pen's
//! pressure already produces) and **Touch Taper** (the same shaping for a finger, which reports no
//! pressure). On a mouse the first does not exist — every sample arrives at pressure `1.0` — so the
//! second *is* the taper, and it is the only way a mouse draws a calligraphic line at all.
//!
//! What this ports:
//!
//! * one length, authored on a stroke-shaped widget with a handle at the head;
//! * a **Tip** size, "sharp" at the low end and "blunt" at the high end — how thin the very tip gets;
//! * an **Opacity** amount, so the taper can fade as well as narrow.
//!
//! ## ⛔ THE FAR END IS GONE — do not rebuild it (Enio 2026-08-10)
//!
//! The reference app tapers both ends and this crate once did too: a second length, a second tip, a
//! *Link tip sizes* toggle, a `min` of the two windows, and — because a freehand drag does not know
//! where it will end — a pen-up replay that put the last window's ink back and laid it again, tapered.
//!
//! The order that removed it is *"quanto à cauda do taper vamos desativar para todos os modos de
//! pintura; deixe o ajuste apenas para o início do traço, como já funciona perfeitamente"*. It is a
//! product decision, not a defect: the head taper is exact, live and free on every method, and the tail
//! never was — it cost a canvas restore + a full re-stamp at pen-up, it was **structurally out of
//! impasto** (the restore took the artist's paint and the replay did not put it back — measured: `0`
//! inked rows with the restore against `20` without), and it was out of watercolor and Wet Paint too
//! (the wash rebuilds from its own accumulators and the fluid cannot be rewound). What it left behind
//! was a control that was live in a corner and mute everywhere else.
//!
//! ⚠️ **What was lost with it, so nobody discovers it as a bug:** the shape editors (Line / Arc / Curve
//! / Free Hand) knew their whole path before the first dab, so *their* end taper was exact and cost
//! nothing. It went with the rest, because a length that only shapes some methods is the half-live
//! control this codebase refuses. Reviving it is the law here plus a span the fills declare — one wave,
//! and it needs the order that this one had.
//!
//! ## A cauda volta — L5 (20/10), no DIGITAL, com a ordem que pedia
//!
//! A missão do dono de 19/10 (*«os melhores pincéis do mundo… faça tudo que for necessário»*, Pinceis/00) e
//! o catálogo pedem o afilar do FIM nas canetas (Procreate Studio/Technical Pen, Monoline, Script —
//! C-4, C-6, D-1, D-2, todas Digital). Ela volta sem nenhum dos três preços que a tiraram: (1) no Digital o
//! traço pinta num PLANO próprio (`traco_plano` no tool) — re-pintar a cauda é limpar o plano NA JANELA da
//! cauda e voltar a carimbar só os dabs que lhe tocam, sem restaurar a tela nem re-carimbar o traço; (2) é
//! UMA transacção (antes do fecho do traço); (3) nos outros três meios a linha ESMAECE (o meio não a lê) —
//! não é um controlo vivo num canto e mudo no resto. A cabeça continua exacta e ao vivo como sempre.
//!
//! ## The law
//!
//! A dab knows `s`, the arc travelled from the pen-down. The width multiplier is `tip` at the very tip,
//! `1` once past the taper length, and a **smoothstep** in between — C¹ at both ends, so the taper joins
//! the full-width body with no crease and the tip lands with no corner. No transcendental (HR-5).
//!
//! ## Why the length is in DIAMETERS
//!
//! A taper measured in pixels reads completely differently at two brush sizes — 60 px of taper is a
//! flourish on a 6 px liner and invisible on a 120 px wash. Brush-relative is what makes one number
//! mean one thing at every size, and it is what this crate already does for
//! [`crate::spec::BrushSpec::spacing`], for [`crate::spec::BrushSpec::jitter`] and for the heading's
//! smoothing length.
//!
//! ⚠️ **A fraction of the STROKE would have been the other obvious choice, and it cannot work here:**
//! a percentage needs the total length, and a freehand stroke does not have one until the pen lifts —
//! it would make even the start taper unknowable while the artist draws.

/// Taper lengths are authored in **diameters**; this is the top of the slider.
///
/// Eight is where a 40 px brush tapers over 320 px — already longer than anything that reads as "the
/// brush", and far past any taper in the reference app. It costs no latency at any value: nothing is
/// ever held back, because the head is known from the first dab.
pub const MAX_TAPER_DIAMETERS: f32 = 8.0;

/// O topo do slider do FALL-OFF e do seu patamar, em diâmetros. Não é recurso: é a faixa do controlo. O oráculo
/// mais longo medido acaba aos ~15·D (ArtRage Oil ThinAndDry, 600 px a 40 px — Pinceis/06 §5); 64·D é uma caneta
/// de 5 px a esgotar-se aos 320 px e um pincel de 40 px aos 2560 px (a tela inteira).
pub const MAX_FALL_OFF_DIAMETERS: f32 = 64.0;

/// The stroke-head shaping of a brush (Procreate *Touch Taper*). All-zero is off, and an off taper is
/// byte-identical to no taper at all — the multiplier is exactly `1.0` and the dab is untouched.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Taper {
    /// Taper length at the **start** of the stroke, in diameters (`0` = off).
    pub start: f32,
    /// Width at the very **start** tip as a fraction of full width: `0` = a sharp point, `1` = blunt
    /// (no narrowing at all, so the taper is only whatever [`Self::opacity`] fades). Procreate's
    /// "Tip", whose own slider runs sharp → blunt.
    pub tip_start: f32,
    /// How much the taper also **fades**, `0..1`. `0` = the taper is purely a width (the tip is thin
    /// but as opaque as the body); `1` = coverage narrows exactly like the width, so the tip
    /// dissolves. Procreate's taper Opacity.
    pub opacity: f32,
    /// ⭐ **L5 — a CAUDA** (Pinceis/03 §4): o comprimento do afilar no FIM, em diâmetros (`0` = desligado).
    /// O fim não se conhece em tempo real: a cauda é RE-PINTADA ao levantar, dentro da transacção do traço
    /// (a lei do Procreate), no plano do traço do Digital (`traco_plano` no tool). Ver § «A cauda volta».
    pub end: f32,
    /// A largura na ponta do FIM, `0..1` (o «Tip» da cauda).
    pub tip_end: f32,
    /// ⭐ **L5 — o FALL-OFF**: a carga que se ESGOTA pelo comprimento, em diâmetros (`0` = desligado): a
    /// cobertura de cada dab cai de `1` a `0` ao longo deste comprimento (smoothstep), depois do patamar
    /// [`Self::fall_off_start`]. Vale nos quatro meios (a cobertura do dab; no Wet, a carga do pigmento).
    pub fall_off: f32,
    /// O patamar antes de a carga começar a acabar, em diâmetros (ArtRage Oil ThinAndDry: ~7·D).
    pub fall_off_start: f32,
}

impl Taper {
    /// The **neutral** taper: off, and byte-identical to a brush that never heard of one.
    ///
    /// A `const` because the panel's pre-publish fallback brush is a `const` too, and a second neutral
    /// spelled out there would be the version that quietly stops matching this one.
    pub const NEUTRAL: Self = Self {
        start: 0.0,
        // A tapered end is a POINT unless the artist asks otherwise: this is the sharp end of
        // Procreate's own Tip slider, and the reason the feature is worth having on a mouse.
        tip_start: 0.0,
        opacity: 0.0,
        end: 0.0,
        tip_end: 0.0,
        fall_off: 0.0,
        fall_off_start: 0.0,
    };
}

impl Default for Taper {
    fn default() -> Self {
        Self::NEUTRAL
    }
}

impl Taper {
    /// Whether anything about this taper can change a dab. A brush whose taper answers `false` never
    /// enters the taper path at all, which is what keeps a default brush byte-identical.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.start > 0.0
    }

    /// Taper length at the start in **pixels**, for a brush of `diameter_px`.
    #[must_use]
    pub fn start_px(&self, diameter_px: f32) -> f32 {
        self.start.clamp(0.0, MAX_TAPER_DIAMETERS) * diameter_px.max(0.0)
    }

    /// The **width multiplier** for a dab that is `from_start` px along the stroke, on a brush of
    /// `diameter_px`.
    ///
    /// Every stroke method can answer this from its first dab — [`crate::Dab::arc_len`] carries it
    /// exactly — so there is no case where a dab has to be held back to find out, and no method where
    /// the taper is inert.
    #[must_use]
    pub fn width(&self, from_start: f32, diameter_px: f32) -> f32 {
        ramp(from_start, self.start_px(diameter_px), self.tip_start)
    }

    /// Se a CAUDA está armada (L5): o fim re-pinta-se ao levantar.
    #[must_use]
    pub fn end_active(&self) -> bool {
        self.end > 0.0
    }

    /// Comprimento da cauda em **px**, para um pincel de `diameter_px`.
    #[must_use]
    pub fn end_px(&self, diameter_px: f32) -> f32 {
        self.end.clamp(0.0, MAX_TAPER_DIAMETERS) * diameter_px.max(0.0)
    }

    /// O multiplicador de LARGURA de um dab a `to_end` px do FIM do traço — a MESMA rampa da cabeça.
    #[must_use]
    pub fn width_end(&self, to_end: f32, diameter_px: f32) -> f32 {
        ramp(to_end, self.end_px(diameter_px), self.tip_end)
    }

    /// Se o FALL-OFF está armado (L5).
    #[must_use]
    pub fn fall_active(&self) -> bool {
        self.fall_off > 0.0
    }

    /// ⭐ **O fall-off**: o multiplicador de COBERTURA de um dab a `arc` px do início — `1` no patamar,
    /// a rampa da cabeça espelhada até `0` ao fim de [`Self::fall_off`] diâmetros. Desligado = `1.0` exacto.
    #[must_use]
    pub fn fall(&self, arc: f32, diameter_px: f32) -> f32 {
        if !self.fall_active() {
            return 1.0;
        }
        let d = diameter_px.max(0.0);
        let s0 = self.fall_off_start.clamp(0.0, MAX_FALL_OFF_DIAMETERS) * d;
        let len = self.fall_off.clamp(0.0, MAX_FALL_OFF_DIAMETERS) * d;
        ramp(s0 + len - arc, len, 0.0)
    }

    /// ⭐ **A ponta ROMBA** (`tip_start ≥ 1`): o taper não estreita, só ESMAECE pela [`Self::opacity`] — a
    /// cobertura vai de `1 − opacity` na ponta a `1` ao fim do comprimento (a promessa do [`Self::tip_start`]).
    /// Fora dela `1.0` exacto: a ponta que estreita já esmaece pela [`Self::coverage_mult`].
    #[must_use]
    pub fn fade_blunt(&self, from_start: f32, diameter_px: f32) -> f32 {
        if self.tip_start < 1.0 || self.opacity <= 0.0 {
            return 1.0;
        }
        ramp(from_start, self.start_px(diameter_px), 1.0 - self.opacity.clamp(0.0, 1.0))
    }

    /// The **coverage multiplier** that goes with a width multiplier `w`. `opacity = 0` leaves the dab
    /// as opaque as the body; `opacity = 1` fades it exactly as fast as it narrows.
    #[must_use]
    pub fn coverage_mult(&self, w: f32) -> f32 {
        let k = self.opacity.clamp(0.0, 1.0);
        (1.0 + k * (w - 1.0)).clamp(0.0, 1.0)
    }
}

/// The head's width multiplier: `tip` at distance `0`, `1.0` from `len` on, smoothstep between.
///
/// `len <= 0` (no taper) short-circuits to exactly `1.0`, so an off taper costs one compare and cannot
/// perturb a single bit.
fn ramp(d: f32, len: f32, tip: f32) -> f32 {
    // Written as a POSITIVE pair of comparisons on purpose: everything that is not definitely inside a
    // real taper window — no window (`len <= 0`), past it, a NaN — falls through to full width.
    // Negating the comparisons instead would send NaN down the ramp, and the one thing worse than a
    // taper that ignores a bad number is a taper that trusts it.
    if len > 0.0 && d < len {
        let t = (d.max(0.0) / len).clamp(0.0, 1.0);
        let s = t * t * (3.0 - 2.0 * t); // smoothstep: C¹ at both ends, no transcendental (HR-5)
        let tip = tip.clamp(0.0, 1.0);
        tip + (1.0 - tip) * s
    } else {
        1.0
    }
}

/// Scale a dab in place by a taper width `w`, keeping the radius inside the engine's own bounds.
///
/// The single place a taper ever touches a [`crate::Dab`], and it is applied at EMISSION — the instant
/// the dab is made, at the position the pointer put it. Nothing is buffered and nothing is replayed.
///
/// ⚠️ [`crate::Dab::stroke_radius_px`] is deliberately NOT scaled. It is the stroke's *nominal* radius
/// and it is stroke-constant by contract — the Shape **Flow** mapping divides its along-the-stroke
/// coordinate by it, so a per-dab divisor re-phases the whole accumulated pattern (the bug that cost
/// `0.42` of a tile when it rode the live radius). A taper is exactly the kind of per-dab breathing
/// that contract exists to keep out.
pub fn scale_dab(dab: &mut crate::Dab, taper: &Taper, w: f32) {
    if w >= 1.0 {
        return;
    }
    dab.radius_px = (dab.radius_px * w.max(0.0)).clamp(0.5, crate::spec::MAX_BRUSH_RADIUS_PX);
    dab.coverage = (dab.coverage * taper.coverage_mult(w)).clamp(0.0, 1.0);
}
