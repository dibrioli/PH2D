//! The Wet Paint **authored settings** — the checkbox (the persistent ARM), the FULL knob
//! table (doc 22: the W3 curated seven grew into the engine's whole registry, one storage),
//! the wet TOOL, the tilt dial, the canvas-action/overlay flags, and their panel routing.
//! Split from `wetpaint.rs` (workspace file-LOC cap), the exact sibling pattern of
//! `watercolor_settings.rs`: this file owns the AUTHORED state and its doors; `wetpaint.rs`
//! owns the SESSION (display-state) that reconciles against it.

use super::*;
use ph2d_wet_paint::tuning::{KNOB_COUNT, KNOB_DEFS, Knob, KnobGroup, knob_defaults};

/// The Wet Paint knob store — the engine's TWO slider knobs plus the whole
/// `KNOB_DEFS` registry, in ONE house (doc 22 §2.1: the basic section's
/// curated rows and the Tuning panel's table rows are two VIEWS of the same
/// value; a second store would be the two-doors disease). Authored,
/// persistent tool state — the session is display-state and dies on any
/// foreign mutation; knobs living in the engine would forget themselves on
/// every undo. The session **reconciles** the engine against this every
/// batch and tick, the same law as the paper.
///
/// `f64` on purpose — the engine's own precision, so [`WetKnobs::DEFAULT`]
/// can equal the engine's boot values EXACTLY (an `f32` copy of `0.4` is
/// `0.4000000059604645` in `f64`, and the reconcile would push that noise
/// into untouched knobs the first time any other knob moves). The panel's
/// `f32` rows convert at the paint boundary.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WetKnobs {
    /// Water per dab (engine `sliders.water`, `0..1`).
    pub water: f64,
    /// The wet eraser's lift per pass (engine `sliders.erase`, `0..1`).
    pub erase: f64,
    /// Every registry knob, indexed by `Knob as usize` (the table order).
    /// Values are pre-sanitized by [`WetKnobs::set`] (NaN falls back to the
    /// default, out-of-range clamps to the def's own bounds).
    pub knobs: [f64; KNOB_COUNT],
}

/// [`WetKnobs::DEFAULT`]'s knob array — the reference boot with Enio's five Wet Paint product tweaks
/// (2026-07-22). A `const fn` so `DEFAULT` stays a `const` (the panel's `FALLBACK_BRUSH` needs it).
/// Each override is inside its `KNOB_DEFS` range, so no clamp is skipped by writing the array directly.
const fn painter_default_knobs() -> [f64; KNOB_COUNT] {
    let mut k = knob_defaults();
    // O pincel sem preset é o Wet Round de fábrica (o pigmento e a orla calibrados contra o Rebelle, 10/10).
    k[Knob::PigmentPerDab as usize] = 78.0; // LITERAL-OK: wet-paint product default = Wet Round, engine boot 600
    k[Knob::EdgeDarkening as usize] = 12.0; // LITERAL-OK: wet-paint product default = Wet Round, engine boot 50
    k[Knob::PaperGate as usize] = 0.4; // LITERAL-OK: wet-paint product default (Enio), engine boot 0.6
    k[Knob::Felt as usize] = 0.03; // LITERAL-OK: wet-paint product default (Enio), engine boot 0.01
    k[Knob::BristleSize as usize] = 2.0; // LITERAL-OK: wet-paint product default (Enio), engine boot 1.0
    k[Knob::BristleCount as usize] = 2000.0; // LITERAL-OK: wet-paint product default (Enio), engine boot 950
    // O relevo da tinta nasce desligado (11/10): o render já o desenha sem papel, e o traço de hoje não muda.
    k[Knob::Emboss as usize] = 0.0; // LITERAL-OK: wet-paint product default (relief off), engine boot 1
    k
}

impl crate::PainterTool {
    /// ⭐ **A pressão do traço** (prompt 44): a do tool, e no Wet Paint a do PINCEL — os knobs
    /// [`Knob::PressureSize`] (o raio na pressão 0 é `1 − ela`; `0,9` = os 10 % de sempre, ao bit) e
    /// [`Knob::PressureOpacity`] (`0` = a lei de sempre; acima dela a tinta do dab segue a pressão, o Flat
    /// do Rebelle) com a curva [`Knob::PressureOpacityCurve`] (prompt 45). Lido UMA vez por traço, no
    /// `Stroke::new`.
    /// Nos outros meios a mão é a do pincel (`paint.dynamics`, L2). As duas pela MESMA lei
    /// ([`super::mao::escreve`]).
    #[must_use]
    pub(crate) fn dinamica_do_traco(&self) -> ph2d_painter_brush::Dynamics {
        self.mao_do_wet().map_or(self.paint.dynamics, |(d, _)| d)
    }

    /// ⭐ **A inclinação do PINCEL** (W6): no Wet Paint a dos knobs do pincel ([`Knob::DabRotation`] …
    /// [`Knob::TiltLag`]); nos outros meios a do pincel do tool (L2: o cartão Pen escreve-a).
    #[must_use]
    pub(crate) fn inclinacao_do_pincel(&self) -> ph2d_painter_brush::PenTilt {
        self.mao_do_wet()
            .map_or_else(|| self.inclinacao_dos_antigos(), |(_, t)| t)
    }

    /// A `PenTilt` do pincel como a lavagem a LÊ: ela não lê a cobertura do dab (`WASH_DEPOSIT_PEAK`, sem
    /// opacidade deitada) e, com a Shape «Automatic», o carimbo é o redondo dela (`wet_shape_stamp`: a pegada
    /// só com a Automatic desligada, doc 47 §6) — sem largura, comprimento nem rotação deitados.
    fn inclinacao_dos_antigos(&self) -> ph2d_painter_brush::PenTilt {
        let mut t = self.paint.brush.pen_tilt;
        if self.watercolor_render_active() {
            t.opacity = 0.0;
            if self.paint.brush.watercolor_shape_auto {
                t.width = 1.0;
                t.length = 1.0;
                t.rotation = ph2d_painter_brush::DabRotation::None;
            }
        }
        t
    }

    /// A mão dos knobs do Wet Paint (`None` fora dele): cada knob da caneta pela lei única do cartão Pen.
    fn mao_do_wet(&self) -> Option<(ph2d_painter_brush::Dynamics, ph2d_painter_brush::PenTilt)> {
        if !self.paint.wetpaint.armed {
            return None;
        }
        let k = &self.paint.wetpaint.knobs;
        let (mut d, mut t) = (self.paint.dynamics, ph2d_painter_brush::PenTilt::default());
        for l in super::mao::LINHAS_DA_MAO {
            if let super::mao::LinhaDaMao::Knob(kn) = l {
                super::mao::escreve(&mut d, &mut t, l, k.get(kn));
            }
        }
        Some((d, t))
    }

    /// ⭐ **Os fios do pincel** (W2, `ph2d_wet_paint::bristles_fios`): os knobs [`Knob::BristlePressure`] …
    /// [`Knob::BristleOscillation`]; o neutro = o tapete de antes ao bit.
    #[must_use]
    pub(crate) fn fios_do_pincel(&self) -> ph2d_wet_paint::bristles_fios::Fios {
        let k = &self.paint.wetpaint.knobs;
        ph2d_wet_paint::bristles_fios::Fios {
            pressao: k.get(Knob::BristlePressure),
            levanta: k.get(Knob::BristleLift),
            ruido_px: k.get(Knob::BristleNoiseLength),
            largura_px: k.get(Knob::BristleLaneWidth),
            onda: k.get(Knob::BristleWave),
            onda_px: k.get(Knob::BristleWaveLength),
            oscilacao_px: k.get(Knob::BristleOscillation),
            span_px: k.get(Knob::BristleSpan),
            span_comprimento_px: k.get(Knob::BristleSpanLength),
            cor: [
                k.get(Knob::BristleNoiseHue),
                k.get(Knob::BristleNoiseSat),
                k.get(Knob::BristleNoiseLight),
            ],
            veio: [k.get(Knob::BristleStreak), k.get(Knob::BristleStreakLight)],
            veio_px: k.get(Knob::BristleStreakLength),
            veio_corte: k.get(Knob::BristleStreakCut),
        }
    }

    /// O RELEVO de cada fio do pincel (W3): os knobs `bristleHeight`/`bristleRemove`.
    pub(crate) fn relevo_do_pincel(&self) -> ph2d_wet_paint::bristles_relevo::Relevo {
        let k = &self.paint.wetpaint.knobs;
        ph2d_wet_paint::bristles_relevo::Relevo {
            altura_px: k.get(Knob::BristleHeight),
            escava: k.get(Knob::BristleRemove),
        }
    }
}

impl WetKnobs {
    /// The reference model's boot — SPEC §16 (`Sliders::default` + `KNOB_DEFS`), the values the ENGINE
    /// actually boots with (`Engine::new`). This is the reconcile's **no-op baseline**: a session's
    /// `applied` field inits to this (via `WetEngineFacts::BOOT`), so the first reconcile measures the
    /// tool's authored delta against the engine's real starting state. It is NOT the app default — see
    /// [`Self::DEFAULT`] — and the two MUST stay distinct, or the reconcile's early-return would skip
    /// the product values (panel showing 800 over an engine still at 600).
    pub const ENGINE_BOOT: Self = Self {
        water: 1.0,
        erase: 0.4,
        knobs: knob_defaults(),
    };

    /// The app's Wet Paint **default** — [`Self::ENGINE_BOOT`] with Enio's five product tweaks
    /// (2026-07-22: Pigment 800, Paper Gate 0.4, Felt 0.03, Bristle Size 2.0, Bristle Count 2000).
    /// Distinct from the reference boot ON PURPOSE: the engine stays a faithful 1:1 port of the JS
    /// reference (its fingerprint untouched, its `KNOB_DEFS` intact), and the painter opens the fluid on
    /// values chosen for the tool. The session's reconcile pushes these into the engine at birth, because
    /// they differ from the boot baseline it inits `applied` to. A `const` so the panel's
    /// `FALLBACK_BRUSH` (a `const` item) can carry it.
    pub const DEFAULT: Self = Self {
        water: 1.0,
        erase: 0.4,
        knobs: painter_default_knobs(),
    };

    /// Read one registry knob.
    #[must_use]
    pub fn get(&self, knob: Knob) -> f64 {
        self.knobs[knob as usize]
    }

    /// ⭐ **As cerdas destes knobs como a ponta do Shape** (plano 27 W2): `forma` com o tipo Bristles e os
    /// quatro sliders exactos destes valores (`BristleKnobs::to_unit`, ida-e-volta ao bit) — o resto da
    /// forma (Angle, Size, Follow…) fica como estava.
    #[must_use]
    pub fn cerdas_no_shape(
        &self,
        forma: ph2d_painter_brush::TextureSettings,
    ) -> ph2d_painter_brush::TextureSettings {
        let u = ph2d_wet_paint::brush::BristleKnobs {
            felt: self.get(Knob::Felt),
            bristle_count: self.get(Knob::BristleCount),
            bristle_strength: self.get(Knob::BristleStrength),
            bristle_size: self.get(Knob::BristleSize),
        }
        .to_unit();
        let mut params = forma.params;
        params[..4].copy_from_slice(&u);
        ph2d_painter_brush::TextureSettings {
            kind: ph2d_painter_brush::TextureKind::Bristles,
            params,
            ..forma
        }
    }

    /// Write one registry knob: NaN (a garbled numeric input) falls back to
    /// the def's default — the model's own law — and everything else clamps
    /// to the def's bounds (const literals, table-order gated in the engine).
    pub fn set(&mut self, knob: Knob, v: f64) {
        let def = &KNOB_DEFS[knob as usize];
        self.knobs[knob as usize] = if v.is_nan() {
            // NaN (garbled input) falls back to the app DEFAULT, not the engine's SPEC boot — "default"
            // means the product value the artist opened with (== SPEC for every knob but Enio's five).
            Self::DEFAULT.knobs[knob as usize]
        } else {
            v.clamp(def.min, def.max)
        };
    }

    /// Restore one GROUP to the app DEFAULTS (the Tuning panel's header reset). Reads [`Self::DEFAULT`],
    /// not `KNOB_DEFS`, so a reset lands on the same values the section opened with — the product tweaks
    /// included (for every knob but Enio's five, `DEFAULT` and `KNOB_DEFS` agree, so this is a no-op change).
    pub fn reset_group(&mut self, group: KnobGroup) {
        for def in KNOB_DEFS.iter().filter(|d| d.group == group) {
            self.knobs[def.knob as usize] = Self::DEFAULT.knobs[def.knob as usize];
        }
    }

    // The W3 curated accessors — the basic section's five knob rows read the
    // SAME array the Tuning panel edits.
    /// Pigment per dab (`Knob::PigmentPerDab`).
    #[must_use]
    pub fn pigment(&self) -> f64 {
        self.get(Knob::PigmentPerDab)
    }
    /// Settled-paint pickup — the dirty brush (`Knob::Pickup`).
    #[must_use]
    pub fn pickup(&self) -> f64 {
        self.get(Knob::Pickup)
    }
    /// Evaporation multiplier — how fast the water dries (`Knob::Evaporation`).
    #[must_use]
    pub fn dry_speed(&self) -> f64 {
        self.get(Knob::Evaporation)
    }
    /// Edge darkening — the watercolor rim (`Knob::EdgeDarkening`).
    #[must_use]
    pub fn edge_darkening(&self) -> f64 {
        self.get(Knob::EdgeDarkening)
    }
    /// Gravity magnitude — the run/drip pull (`Knob::Gravity`).
    #[must_use]
    pub fn gravity(&self) -> f64 {
        self.get(Knob::Gravity)
    }

    /// Os knobs da FÍSICA da água que moram no cartão Wet Paint (saíram do Tuning a 13/10): `(id do
    /// cartão, valor, mín, máx, passo)` — a faixa e o passo da `KNOB_DEFS` do motor, a mesma fonte do Tuning.
    #[must_use]
    pub fn knobs_no_cartao(&self) -> [(ph2d_a11y::NodeId, f64, f64, f64, f64); 10] {
        KNOBS_NO_CARTAO.map(|(id, knob)| {
            let d = &KNOB_DEFS[knob as usize];
            (id, self.get(knob), d.min, d.max, d.step)
        })
    }

    /// ⭐ **A linha da ferramenta na mão** (dono 25/10, o painel do Rebelle): `(id, chave do rótulo, valor, mín,
    /// máx, passo)` dos knobs que o leitor de `tool` lê — vazia para o Pincel e o Wet (o Water deles é o do topo).
    #[must_use]
    pub fn knobs_da_ferramenta(
        &self,
        tool: WetTool,
    ) -> Vec<(ph2d_a11y::NodeId, &'static str, f64, f64, f64, f64)> {
        KNOBS_DA_FERRAMENTA
            .iter()
            .filter(|(_, _, t, _)| *t == tool)
            .map(|&(id, knob, _, rotulo)| {
                let d = &KNOB_DEFS[knob as usize];
                (id, rotulo, self.get(knob), d.min, d.max, d.step)
            })
            .collect()
    }

    /// Os knobs que só o leitor de `tool` lê — o que um preset dela escreve (os outros são do Pincel da água).
    pub(crate) fn so_da_ferramenta(tool: WetTool) -> impl Iterator<Item = Knob> {
        KNOBS_DA_FERRAMENTA
            .iter()
            .filter(move |(_, _, t, _)| *t == tool)
            .map(|&(_, knob, ..)| knob)
    }

    /// O CORPO de `tool`: os knobs dela e os do pincel que o leitor dela lê ([`CORPO_DA_AGUA`]) — o que um preset
    /// dela escreve e o slot dela guarda.
    pub(crate) fn do_corpo(tool: WetTool) -> impl Iterator<Item = Knob> {
        let corpo = CORPO_DA_AGUA
            .iter()
            .find(|(t, _)| *t == tool)
            .map_or(&[][..], |(_, k)| *k);
        Self::so_da_ferramenta(tool).chain(corpo.iter().copied())
    }

    /// Os knobs que trocam com o slot de uma ferramenta da água: o corpo de TODAS (cada slot guarda o seu).
    pub(crate) fn dos_corpos() -> impl Iterator<Item = Knob> {
        let mut todos: Vec<Knob> = WetTool::DO_TRILHO
            .into_iter()
            .flat_map(Self::do_corpo)
            .collect();
        todos.sort_by_key(|k| *k as usize);
        todos.dedup();
        todos.into_iter()
    }
}

/// O corpo de cada ferramenta da água fora os knobs dela: os do pincel que mudam o traço dela e não a tela parada
/// (medido, `corpo_da_agua_tests::o_corpo_de_cada_ferramenta_da_agua_e_o_que_ela_le`). A física da tela (Leveling,
/// Capillary, Brake, Level Clamp, os efeitos da água) muda o traço dela também, mas muda a tela sem ela: é partilhada.
pub(crate) const CORPO_DA_AGUA: [(WetTool, &[Knob]); 5] = [
    (
        WetTool::Smear,
        &[
            CERDAS[0], CERDAS[1], CERDAS[2], CERDAS[3], CERDAS[4], CERDAS[5],
        ],
    ),
    (
        WetTool::Blend,
        &[
            CERDAS[0],
            CERDAS[1],
            CERDAS[2],
            CERDAS[3],
            CERDAS[4],
            CERDAS[5],
            Knob::PaperGate,
            Knob::Spacing,
            Knob::GateSaturation,
        ],
    ),
    (
        WetTool::Wet,
        &[
            CERDAS[0],
            CERDAS[1],
            CERDAS[2],
            CERDAS[3],
            CERDAS[4],
            CERDAS[5],
            Knob::WaterPush,
        ],
    ),
    (
        WetTool::Dry,
        &[
            CERDAS[0],
            CERDAS[1],
            CERDAS[2],
            CERDAS[3],
            CERDAS[4],
            CERDAS[5],
            Knob::DryerEdge,
        ],
    ),
    (
        WetTool::Blow,
        &[
            CERDAS[0], CERDAS[1], CERDAS[2], CERDAS[3], CERDAS[4], CERDAS[5],
        ],
    ),
];

/// O carimbo de cerdas e a tinta que ele leva: o que as cinco lêem.
const CERDAS: [Knob; 6] = [
    Knob::Felt,
    Knob::BristleCount,
    Knob::BristleStrength,
    Knob::BristleSize,
    Knob::Intensity,
    Knob::CapsuleSpacing,
];

/// Os knobs de cada ferramenta da água: o id do cartão, o knob, a ferramenta que o lê e o rótulo.
const KNOBS_DA_FERRAMENTA: [(ph2d_a11y::NodeId, Knob, WetTool, &str); 5] = [
    (
        crate::ids::PAINTER_WETPAINT_SMEAR_STRENGTH,
        Knob::Smear,
        WetTool::Smear,
        "panel.painter_wet_cartao.strength",
    ),
    (
        crate::ids::PAINTER_WETPAINT_SMEAR_WATER,
        Knob::SmearWater,
        WetTool::Smear,
        "panel.painter_wet_cartao.smear_water",
    ),
    (
        crate::ids::PAINTER_WETPAINT_BLEND_STRENGTH,
        Knob::BlendForce,
        WetTool::Blend,
        "panel.painter_wet_cartao.strength",
    ),
    (
        crate::ids::PAINTER_WETPAINT_DRY_ABSORBENCY,
        Knob::Dryer,
        WetTool::Dry,
        "panel.painter_wet_cartao.absorbency",
    ),
    (
        crate::ids::PAINTER_WETPAINT_BLOW_STRENGTH,
        Knob::Blow,
        WetTool::Blow,
        "panel.painter_wet_cartao.strength",
    ),
];

/// Os knobs do motor que saíram do Tuning para o cartão: Leveling · Level Clamp · Capillary · Brake (dono 13/10) e
/// o Brush dust (dono 19/10) — o id do cartão e o knob. Os cinco de baixo (dono 25/10: os que mais decidem o pincel e
/// o preset; o Pigment diffusion pedido no smoke) saíram também do Tuning: o que está no cartão não se repete lá
/// (R3). O `ExtDryBrush` ficou de fora: o `dry_gate` é 0 nos 29 presets de fábrica (medido 25/10); só a cobertura
/// que cai (a velocidade → opacidade) o abre (`o_extdrybrush_so_mexe_com_dry_gate_positivo`).
const KNOBS_NO_CARTAO: [(ph2d_a11y::NodeId, Knob); 10] = [
    (crate::ids::PAINTER_WETPAINT_LEVELING, Knob::Leveling),
    (crate::ids::PAINTER_WETPAINT_LEVEL_CLAMP, Knob::LevelClamp),
    (crate::ids::PAINTER_WETPAINT_CAPILLARY, Knob::Capillary),
    (crate::ids::PAINTER_WETPAINT_BRAKE, Knob::Brake),
    (crate::ids::PAINTER_WETPAINT_DUST, Knob::Dust),
    (crate::ids::PAINTER_WETPAINT_BACKRUN, Knob::ExtBackrun),
    (
        crate::ids::PAINTER_WETPAINT_GRANULATION,
        Knob::ExtGranulation,
    ),
    (crate::ids::PAINTER_WETPAINT_DIFFUSION, Knob::ExtDiffusion),
    (crate::ids::PAINTER_WETPAINT_WET_BLOOM, Knob::WetOpen),
    (crate::ids::PAINTER_WETPAINT_TIP_PICKUP, Knob::TipReservoir),
];

impl Default for WetKnobs {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// The wet TOOL (doc 22 §2.5 — the model's 7-button radio, minus Erase,
/// which is the rail eraser's other view). Selecting one USES it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WetTool {
    #[default]
    Paint,
    Smear,
    Blend,
    Wet,
    Dry,
    Blow,
}

impl WetTool {
    /// The engine-side tool this maps to.
    pub(crate) fn engine(self) -> ph2d_wet_paint::painter::Tool {
        use ph2d_wet_paint::painter::Tool;
        match self {
            WetTool::Paint => Tool::Paint,
            WetTool::Smear => Tool::Smear,
            WetTool::Blend => Tool::Blend,
            WetTool::Wet => Tool::Wet,
            WetTool::Dry => Tool::Dry,
            WetTool::Blow => Tool::Blow,
        }
    }

    /// Paint and Blend deposit through the LANE trails (one per symmetry
    /// copy); the rest are per-dab grid ops through the direct-tool door.
    pub(crate) fn uses_lanes(self) -> bool {
        matches!(self, WetTool::Paint | WetTool::Blend)
    }

    /// As ferramentas da água que o trilho oferece (dono 25/10), na ordem do Rebelle.
    pub const DO_TRILHO: [WetTool; 5] = [
        WetTool::Blend,
        WetTool::Smear,
        WetTool::Wet,
        WetTool::Dry,
        WetTool::Blow,
    ];

    /// ⭐ **O fio do trilho** — o vocabulário de [`PainterTool::set_paint_tool_mode`]. O Pincel da água tem o
    /// seu (`wet_paint`) para o botão Pincel voltar a ele; o que [`PainterTool::active_paint_mode_id`] devolve
    /// para ele é `"brush"`.
    #[must_use]
    pub fn fio(self) -> &'static str {
        match self {
            WetTool::Paint => "wet_paint",
            WetTool::Smear => "wet_smear",
            WetTool::Blend => "wet_blend",
            WetTool::Wet => "wet_water",
            WetTool::Dry => "wet_dry",
            WetTool::Blow => "wet_blow",
        }
    }

    /// O inverso de [`Self::fio`].
    #[must_use]
    pub fn do_fio(fio: &str) -> Option<WetTool> {
        [
            WetTool::Paint,
            WetTool::Smear,
            WetTool::Blend,
            WetTool::Wet,
            WetTool::Dry,
            WetTool::Blow,
        ]
        .into_iter()
        .find(|t| t.fio() == fio)
    }

    /// O índice nos slots de pincel da água (o Pincel não tem: usa o do modo `WetPaint`).
    pub(crate) fn slot_da_agua(self) -> Option<usize> {
        match self {
            WetTool::Paint => None,
            WetTool::Smear => Some(0),
            WetTool::Blend => Some(1),
            WetTool::Wet => Some(2),
            WetTool::Dry => Some(3),
            WetTool::Blow => Some(4),
        }
    }
}

impl PainterTool {
    /// The Wet Paint checkbox's setter (the panel's Enable + the smoke's
    /// arm). Arming while holding the plain Brush enters the mode on the
    /// spot; disarming while wet exits to the plain Brush — and the exit's
    /// teardown ends the session, which IS the bake. From any other tool
    /// the flag just flips: the next `"brush"` honours it.
    pub fn set_wetpaint_armed(&mut self, on: bool) {
        if self.paint.wetpaint.armed == on {
            return;
        }
        self.paint.wetpaint.armed = on;
        match self.paint.paint_mode {
            // A Borracha entra e sai da água como Borracha (dono 24/10: ela tem o Wet Paint como meio).
            PaintMode::Paint | PaintMode::WetPaint
                if on == (self.paint.paint_mode == PaintMode::Paint) =>
            {
                self.set_paint_tool_mode(if self.paint.eraser { "eraser" } else { "brush" });
            }
            _ => {}
        }
    }

    /// Flip the arm. The panel reaches it through `set_paint_media` (the Paint Mode dropdown);
    /// the section reset and the smoke call it directly.
    pub fn toggle_wetpaint_armed(&mut self) {
        self.set_wetpaint_armed(!self.paint.wetpaint.armed);
    }

    /// The section reset — the Watercolor reset's exact semantics: restore
    /// the knob table, the tilt and the overlay flags (the reconcile
    /// carries the values into any live session's engine). ⚠️ **The arm
    /// STAYS** (decisão do dono, 2026-10-04: o Reset mantém o meio escolhido —
    /// desarmar levava o pincel ao slot do Digital, doc 45 §2.1).
    pub fn reset_brush_wetpaint(&mut self) {
        let w = &mut self.paint.wetpaint;
        // A ferramenta fica: ela é a escolha do trilho, e o pincel na mão é o slot dela.
        w.knobs = WetKnobs::default();
        w.tilt_on = true;
        w.tilt_ring = 4;
        w.tilt_spoke = 3;
        w.km_mixing = false;
        w.agua_viva = false;
        w.borda_que_pesa = 0.0;
        let need_recomposite = w.show_wet
            || w.paper_visual
            || w.km_glaze
            || w.smooth_edges > 0.0
            || w.oleo_real.is_some();
        w.show_wet = false;
        w.paper_visual = false;
        w.smooth_edges = 0.0;
        w.oleo_real = None;
        w.oleo_real_valores = ph2d_wet_paint::oleo_real::OleoReal::default();
        w.km_glaze = false;
        w.tuning_open = false;
        if need_recomposite {
            self.wet_recomposite_full();
        }
    }

    /// A intensidade do **Smooth Edges** do Wet Paint (`0..1`; NaN = `0`): mudá-la recompõe a tela inteira.
    pub fn set_wet_smooth_edges(&mut self, forca: f32) {
        let forca = if forca.is_nan() {
            0.0
        } else {
            forca.clamp(0.0, 1.0)
        };
        if self.paint.wetpaint.smooth_edges != forca {
            self.paint.wetpaint.smooth_edges = forca;
            self.wet_recomposite_full();
        }
    }

    /// A intensidade do **Smooth Edges** do Wet Paint (`0` = desligado).
    #[must_use]
    pub fn wet_smooth_edges(&self) -> f32 {
        self.paint.wetpaint.smooth_edges
    }

    /// O **Óleo Real** (doc 49): a óptica do relevo — a fenda na cor da tinta, a sombra que cai também no
    /// papel, o brilho. `None` = o composite de hoje, ao byte. Mudá-lo recompõe a tela inteira.
    pub fn set_wet_oleo_real(&mut self, oleo: Option<ph2d_wet_paint::oleo_real::OleoReal>) {
        if let Some(o) = oleo {
            self.paint.wetpaint.oleo_real_valores = o;
        }
        if self.paint.wetpaint.oleo_real != oleo {
            self.paint.wetpaint.oleo_real = oleo;
            self.wet_recomposite_full();
        }
    }

    /// O **Óleo Real** do Wet Paint (`None` = desligado).
    #[must_use]
    pub fn wet_oleo_real(&self) -> Option<ph2d_wet_paint::oleo_real::OleoReal> {
        self.paint.wetpaint.oleo_real
    }

    /// A caixa **Real Oil**: ligada = `Some` dos últimos valores dos knobs (os de fábrica na primeira vez),
    /// desligada = `None` — os valores ficam guardados para a próxima vez.
    pub fn set_wet_oleo_real_ligado(&mut self, ligado: bool) {
        let o = ligado.then_some(self.paint.wetpaint.oleo_real_valores);
        self.set_wet_oleo_real(o);
    }

    /// Os últimos valores dos knobs do **Real Oil** (os do painel, ligada ou não a caixa).
    #[must_use]
    pub fn wet_oleo_real_valores(&self) -> ph2d_wet_paint::oleo_real::OleoReal {
        self.paint.wetpaint.oleo_real_valores
    }

    /// Uma linha do **Real Oil** (`PAINTER_WETPAINT_OLEO_FIELDS`), nas unidades do painel — os ângulos em
    /// GRAUS. Guarda o valor e, com a caixa ligada, recompõe. `false` = o id não é dela; NaN = sem efeito.
    ///
    /// As faixas são as de `PAINTER_WETPAINT_OLEO_FAIXAS` (o porquê do piso de 5° da luz está lá); a
    /// direcção da luz dá a volta em vez de se prender.
    fn set_wet_oleo_knob(&mut self, id: ph2d_a11y::NodeId, v: f64) -> bool {
        let ids = crate::ids::PAINTER_WETPAINT_OLEO_FIELDS;
        let Some(k) = ids.iter().position(|i| *i == id) else {
            return false;
        };
        if v.is_nan() {
            return true;
        }
        let (lo, hi) = crate::ids::PAINTER_WETPAINT_OLEO_FAIXAS[k];
        let mut o = self.paint.wetpaint.oleo_real_valores;
        match k {
            0 => o.profundidade = v.clamp(lo, hi),
            1 => o.sombra = v.clamp(lo, hi),
            2 => o.brilho = v.clamp(lo, hi),
            3 => o.altitude = v.clamp(lo, hi).to_radians(),
            4 => o.azimute = v.rem_euclid(hi).to_radians(),
            5 => o.fenda = v.clamp(lo, hi),
            6 => o.alcance_fenda = v.clamp(lo, hi),
            7 => o.forca_ambiente = v.clamp(lo, hi),
            _ => o.material.poe(
                k - crate::ids::PAINTER_WETPAINT_OLEO_MATERIAL_DE,
                v.clamp(lo, hi),
            ),
        }
        self.guarda_oleo_valores(o);
        true
    }

    /// O **Environment** do Real Oil pelo índice em `Ambiente::TODOS` (o wire do chip). Fora da faixa =
    /// sem efeito (`false`). Guarda-o e, com a caixa ligada, recompõe.
    fn set_wet_oleo_ambiente_indice(&mut self, i: u8) -> bool {
        let Some(a) = ph2d_wet_paint::oleo_real::Ambiente::de_indice(i) else {
            return false;
        };
        let mut o = self.paint.wetpaint.oleo_real_valores;
        o.ambiente = a;
        self.guarda_oleo_valores(o);
        true
    }

    /// O **Reflection Map** do Real Oil pelo wire `MaterialCartao::mapa_indice` (`0` = o Environment). Fora
    /// da faixa = sem efeito (`false`).
    fn set_wet_oleo_mapa_reflexo_indice(&mut self, i: u8) -> bool {
        let Some(m) = ph2d_wet_paint::oleo_real::MaterialCartao::mapa_de_indice(i) else {
            return false;
        };
        let mut o = self.paint.wetpaint.oleo_real_valores;
        o.material.mapa_reflexo = m;
        self.guarda_oleo_valores(o);
        true
    }

    /// Os valores novos dos knobs do Real Oil: ficam guardados e, com a caixa ligada, vão à tela.
    fn guarda_oleo_valores(&mut self, o: ph2d_wet_paint::oleo_real::OleoReal) {
        self.paint.wetpaint.oleo_real_valores = o;
        if self.paint.wetpaint.oleo_real.is_some() {
            self.set_wet_oleo_real(Some(o));
        }
    }

    /// A **Live Water** (W1g): a água corre com o traço aberto.
    pub fn set_wet_agua_viva(&mut self, ligada: bool) {
        self.paint.wetpaint.agua_viva = ligada;
    }

    /// A **Live Water** está ligada?
    #[must_use]
    pub fn wet_agua_viva(&self) -> bool {
        self.paint.wetpaint.agua_viva
    }

    /// A **Heavy Edge** (W1g, `0..1`; NaN = o neutro).
    pub fn set_wet_borda_que_pesa(&mut self, v: f64) {
        self.paint.wetpaint.borda_que_pesa = if v.is_nan() { 0.0 } else { v.clamp(0.0, 1.0) };
    }

    /// A **Heavy Edge** de agora.
    #[must_use]
    pub fn wet_borda_que_pesa(&self) -> f64 {
        self.paint.wetpaint.borda_que_pesa
    }

    /// The authored knob values (the panel snapshot's source).
    #[must_use]
    pub fn wet_knobs(&self) -> WetKnobs {
        self.paint.wetpaint.knobs
    }

    /// Pick a wet tool by its index (Paint · Erase · Smear · Blend · Wet · Dry · Blow) — the door the macros
    /// and the tests keep; the rail sends the same wires ([`WetTool::fio`]). Erase is the rail eraser.
    pub fn pick_wet_tool(&mut self, index: usize) {
        let tool = match index {
            0 => WetTool::Paint,
            1 => {
                self.set_paint_tool_mode("eraser");
                return;
            }
            2 => WetTool::Smear,
            3 => WetTool::Blend,
            4 => WetTool::Wet,
            5 => WetTool::Dry,
            _ => WetTool::Blow,
        };
        self.set_paint_tool_mode(tool.fio());
    }

    /// **A razão da grade do fluido** (1..=30 px por célula) — a porta ÚNICA.
    ///
    /// ⚠️ **Trocar a razão encerra a sessão de água viva**, e isso é o desenho,
    /// não uma limitação: a grade tem dimensão, e uma sessão viva já a tem
    /// congelada em `WetSession::ratio`. Encerrar É o bake (a tinta que se vê
    /// está no `canvas_rgba`), então o artista não perde pintura — perde a água
    /// AINDA MOLHADA, que é exactamente o que "mudar a resolução do fluido"
    /// significa. A alternativa seria reamostrar catorze planos de `f32` para a
    /// resolução nova, inventando água que o solver nunca produziu.
    ///
    /// Sem mudança = **sem encerramento** (o guard de igualdade é o que torna
    /// seguro o chip numérico re-emitir o mesmo valor a cada frame de arrasto).
    pub fn set_wet_grid_ratio(&mut self, v: f64) {
        use super::wetpaint::grid_map;
        let want = grid_map::clamp_ratio(v.round().clamp(0.0, 255.0) as u8);
        if self.paint.wetpaint.grid_ratio == want {
            return;
        }
        self.paint.wetpaint.grid_ratio = want;
        // A sessão viva nasceu com a razão antiga; a próxima nasce com esta.
        self.wetpaint_end_session();
    }

    /// **A razão da grade de FLUXO** — quantas células de FLUIDO medem uma
    /// célula de velocidade/pressão (plano 30).
    ///
    /// Irmã exata do [`Self::set_wet_grid_ratio`], e pelo mesmo motivo: a grade
    /// tem dimensão, uma sessão viva já a tem congelada, e reamostrar quatro
    /// planos de `f32` inventaria velocidade que o solver nunca produziu.
    /// Encerrar É o bake — a tinta que se vê está no `canvas_rgba`.
    ///
    /// Sem mudança = **sem encerramento** (o guard que torna seguro o chip
    /// numérico re-emitir o mesmo valor a cada frame de arrasto).
    pub fn set_wet_flow_ratio(&mut self, v: f64) {
        let want = ph2d_wet_paint::flow::clamp_ratio(v.round().clamp(0.0, 255.0) as usize) as u8;
        if self.paint.wetpaint.flow_ratio == want {
            return;
        }
        self.paint.wetpaint.flow_ratio = want;
        self.wetpaint_end_session();
    }

    /// Route one W3/tilt `SetValue` to its clamped field. Knob ranges are
    /// the engine's own (`KNOB_DEFS`); the two engine sliders are `0..1`.
    fn set_wet_knob_value(&mut self, id: ph2d_a11y::NodeId, v: f64) -> bool {
        // ⚠️ A razão da grade vem ANTES do empréstimo dos knobs: ela não é um
        // knob do motor (o motor nem sabe que ela existe — ver
        // `wetpaint::grid_map`), e o setter dela ENCERRA a sessão, o que precisa
        // de `self` inteiro.
        if id == crate::ids::PAINTER_WETPAINT_GRID {
            self.set_wet_grid_ratio(v);
            return true;
        }
        if id == crate::ids::PAINTER_WETPAINT_FLOW {
            self.set_wet_flow_ratio(v);
            return true;
        }
        if id == crate::ids::PAINTER_WETPAINT_SMOOTH_EDGES {
            self.set_wet_smooth_edges(v as f32);
            return true;
        }
        if id == crate::ids::PAINTER_WETPAINT_HEAVY_EDGE {
            self.set_wet_borda_que_pesa(v);
            return true;
        }
        if self.set_wet_oleo_knob(id, v) {
            return true;
        }
        let w = &mut self.paint.wetpaint;
        let k = &mut w.knobs;
        match id {
            x if x == crate::ids::PAINTER_WETPAINT_WATER => k.water = v.clamp(0.0, 1.0),
            x if x == crate::ids::PAINTER_WETPAINT_PIGMENT => k.set(Knob::PigmentPerDab, v),
            x if x == crate::ids::PAINTER_WETPAINT_PICKUP => k.set(Knob::Pickup, v),
            x if x == crate::ids::PAINTER_WETPAINT_DRY_SPEED => k.set(Knob::Evaporation, v),
            x if x == crate::ids::PAINTER_WETPAINT_EDGE => k.set(Knob::EdgeDarkening, v),
            x if x == crate::ids::PAINTER_WETPAINT_GRAVITY => k.set(Knob::Gravity, v),
            x if let Some(&(_, knob)) = KNOBS_NO_CARTAO.iter().find(|(i, _)| *i == x) => {
                k.set(knob, v);
            }
            x if let Some(&(_, knob, ..)) = KNOBS_DA_FERRAMENTA.iter().find(|(i, ..)| *i == x) => {
                k.set(knob, v);
            }
            x if x == crate::ids::PAINTER_WETPAINT_ERASE => k.erase = v.clamp(0.0, 1.0),
            // The tilt dial's two carriers (the pad converts its 2D drag to
            // ring/spoke and forwards them here). Touching the dial turns
            // the tilt ON — the model's "dragging the knob implies on".
            x if x == crate::ids::PAINTER_WETPAINT_TILT_RING => {
                w.tilt_ring = (v.round().clamp(0.0, 8.0)) as u8;
                w.tilt_on = true;
            }
            x if x == crate::ids::PAINTER_WETPAINT_TILT_SPOKE => {
                w.tilt_spoke = (v.round().rem_euclid(12.0)) as u8;
                w.tilt_on = true;
            }
            _ => return false,
        }
        true
    }

    /// Route the Wet Paint section controls (Enable + reset + tools + tilt +
    /// canvas actions + overlay checkboxes + the Tuning side panel's table)
    /// from the layers panel's generic channel — sibling of
    /// `route_brush_watercolor_event`.
    pub(crate) fn route_brush_wetpaint_event(
        &mut self,
        event: &ph2d_editor_core::tool::PanelEvent,
    ) -> bool {
        use ph2d_editor_core::tool::PanelEvent;
        if self.route_wet_preset_event(event) || self.route_mao_event(event) {
            return true;
        }
        match event {
            PanelEvent::Click(id) if *id == crate::ids::PAINTER_WETPAINT_RESET => {
                self.reset_brush_wetpaint();
                true
            }
            PanelEvent::Click(id) if *id == crate::ids::PAINTER_WETPAINT_TILT_TOGGLE => {
                // Flips WITHOUT losing the dial's direction (the model).
                self.paint.wetpaint.tilt_on = !self.paint.wetpaint.tilt_on;
                true
            }
            PanelEvent::Click(id) if *id == crate::ids::PAINTER_WETPAINT_WETCANVAS => {
                self.wetpaint_wet_canvas();
                true
            }
            PanelEvent::Click(id) if *id == crate::ids::PAINTER_WETPAINT_DRYCANVAS => {
                self.wetpaint_dry_canvas();
                true
            }
            PanelEvent::Click(id) if *id == crate::ids::PAINTER_WETPAINT_FASTDRY => {
                self.wetpaint_fast_dry();
                true
            }
            PanelEvent::Click(id) if *id == crate::ids::PAINTER_WETPAINT_SHOWWET => {
                let on = !self.paint.wetpaint.show_wet;
                self.paint.wetpaint.show_wet = on;
                self.wet_recomposite_full();
                true
            }
            PanelEvent::Click(id) if *id == crate::ids::PAINTER_WETPAINT_PAPER_VISUAL => {
                // A Texture on Paint da secção Paper (W1e, F2).
                self.paint.wetpaint.paper_visual = !self.paint.wetpaint.paper_visual;
                self.wet_recomposite_full();
                true
            }
            PanelEvent::Click(id) if *id == crate::ids::PAINTER_WETPAINT_OLEO_REAL => {
                self.set_wet_oleo_real_ligado(self.paint.wetpaint.oleo_real.is_none());
                true
            }
            PanelEvent::SelectOption(id, v)
                if *id == crate::ids::PAINTER_WETPAINT_OLEO_AMBIENTE =>
            {
                v.parse::<u8>()
                    .is_ok_and(|i| self.set_wet_oleo_ambiente_indice(i))
            }
            PanelEvent::SelectOption(id, v)
                if *id == crate::ids::PAINTER_WETPAINT_OLEO_MAPA_REFLEXO =>
            {
                v.parse::<u8>()
                    .is_ok_and(|i| self.set_wet_oleo_mapa_reflexo_indice(i))
            }
            PanelEvent::Click(id) if *id == crate::ids::PAINTER_WETPAINT_LIVE_WATER => {
                self.set_wet_agua_viva(!self.paint.wetpaint.agua_viva);
                true
            }
            PanelEvent::Click(id) if *id == crate::ids::PAINTER_WETPAINT_TUNING => {
                self.paint.wetpaint.tuning_open = !self.paint.wetpaint.tuning_open;
                true
            }
            PanelEvent::Click(id) if *id == crate::ids::WET_TUNING_KM_MIXING => {
                // Sim-side: changes how FUTURE mixing happens; nothing on
                // screen moves until paint does.
                self.paint.wetpaint.km_mixing = !self.paint.wetpaint.km_mixing;
                true
            }
            PanelEvent::Click(id) if *id == crate::ids::WET_TUNING_KM_GLAZE => {
                self.paint.wetpaint.km_glaze = !self.paint.wetpaint.km_glaze;
                self.wet_recomposite_full();
                true
            }
            PanelEvent::Click(id) => self.route_wet_tuning_click(*id),
            PanelEvent::SetValue(id, v) => {
                self.set_wet_knob_value(*id, *v) || self.route_wet_tuning_set(*id, *v)
            }
            _ => false,
        }
    }

    /// The Tuning side panel's per-knob RESET / per-group RESET clicks
    /// (dynamic id family — resolved through [`wet_tuning_id_map`]).
    fn route_wet_tuning_click(&mut self, id: ph2d_a11y::NodeId) -> bool {
        if let Some(gi) = crate::ids::WET_TUNING_GROUP_RESETS
            .iter()
            .position(|g| *g == id)
        {
            self.paint
                .wetpaint
                .knobs
                .reset_group(crate::ids::WET_TUNING_RESET_GROUPS[gi]);
            return true;
        }
        if let Some(&(idx, kind)) = wet_tuning_id_map().get(&id)
            && kind == TuneWidget::Reset
        {
            let def = &KNOB_DEFS[idx];
            self.paint.wetpaint.knobs.set(def.knob, def.default);
            return true;
        }
        false
    }

    /// The Tuning side panel's knob `SetValue`s (slider or chip commit).
    fn route_wet_tuning_set(&mut self, id: ph2d_a11y::NodeId, v: f64) -> bool {
        if let Some(&(idx, kind)) = wet_tuning_id_map().get(&id)
            && kind != TuneWidget::Reset
        {
            self.paint.wetpaint.knobs.set(KNOB_DEFS[idx].knob, v);
            return true;
        }
        false
    }
}

/// Which face of a Tuning-panel knob row an id belongs to.
#[derive(Clone, Copy, PartialEq, Eq)]
enum TuneWidget {
    Slider,
    Chip,
    Reset,
}

/// The dynamic id family, resolved ONCE: every VISIBLE knob's slider / chip /
/// reset id (the hidden §17 group has no UI, exactly like the model). Built
/// from the same `KNOB_DEFS` + id-derive functions the panel paints from —
/// no table to drift.
fn wet_tuning_id_map() -> &'static std::collections::BTreeMap<ph2d_a11y::NodeId, (usize, TuneWidget)>
{
    static MAP: std::sync::OnceLock<
        std::collections::BTreeMap<ph2d_a11y::NodeId, (usize, TuneWidget)>,
    > = std::sync::OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = std::collections::BTreeMap::new();
        for (i, def) in KNOB_DEFS.iter().enumerate() {
            // O PAPER não tem linhas no Tuning (W1e, F2): o papel é o da secção Paper do painel — só o
            // RELEVO da tinta (11/10).
            if def.group == KnobGroup::Hidden
                || (def.group == KnobGroup::Paper
                    && !crate::ids::WET_TUNING_RELEVO.contains(&def.key))
            {
                continue;
            }
            m.insert(
                crate::ids::wet_tuning_slider_id(def.key),
                (i, TuneWidget::Slider),
            );
            m.insert(
                crate::ids::wet_tuning_chip_id(def.key),
                (i, TuneWidget::Chip),
            );
            m.insert(
                crate::ids::wet_tuning_reset_id(def.key),
                (i, TuneWidget::Reset),
            );
        }
        m
    })
}
