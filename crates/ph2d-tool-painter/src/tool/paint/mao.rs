//! **A MÃO do pincel nos quatro meios** (L2, `docs/Painter/Pinceis/03_MOTORES.md` §4): o que a pressão e a
//! inclinação da caneta fazem ao carimbo. UMA lei — cada linha do cartão Pen vira um campo da
//! [`Dynamics`]/[`PenTilt`] em [`escreve`] — e dois armazéns: no Wet Paint os knobs do pincel (o registo
//! `KNOB_DEFS`, W6), nos três meios antigos os campos do pincel (`paint.dynamics`, `brush.pen_tilt`). A chave de
//! cada linha é a do knob do Wet, e por isso o preset de qualquer meio grava a mão no mesmo mapa `engine`.
//!
//! ⭐ O valor de fábrica de cada linha é o neutro, e o neutro é o traço de sempre ao bit (`pressureSize 0,9` =
//! `size_min 0,1`; gate `o_neutro_da_mao_e_o_traco_de_hoje`). ⛔ Sem piso de pressão novo
//! (`feedback_pen_pressure_floor_must_keep_the_owners_hand_range`). Pressão→FLUXO não é uma linha: o motor
//! multiplica a Strength e o Flow no MESMO produto por dab (`ph2d_painter_brush::dab`, BUGS #41), e a linha
//! `pressureOpacity` já é esse produto — uma segunda seria um controlo que mente.

use super::PaintMedia;
use ph2d_painter_brush::{DabRotation, Dynamics, PenTilt};
use ph2d_wet_paint::tuning::{KNOB_DEFS, Knob};

/// Uma linha do cartão Pen: um knob da caneta do Wet (W6) ou a inclinação→opacidade (L2, sem knob no Wet).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinhaDaMao {
    Knob(Knob),
    TiltOpacity,
    /// A velocidade estreita a marca (L12, [`Dynamics::speed_size`]).
    SpeedSize,
    /// A velocidade apaga a marca (L12, [`Dynamics::speed_opacity`]).
    SpeedOpacity,
    /// A curva em S entre o piso e o tecto ([`ph2d_painter_brush::Pen::s_curve`], J3 canetas 2.ª onda).
    PressureSCurve,
    /// A pressão leve espalha ([`Dynamics::jitter_pressure`], J3 canetas 2.ª onda).
    PressureJitter,
    /// A velocidade com atraso ([`Dynamics::speed_lag_s`], J4 efeitos).
    SpeedLag,
}

/// As linhas do cartão, pela ordem do painel.
pub const LINHAS_DA_MAO: [LinhaDaMao; 21] = [
    LinhaDaMao::Knob(Knob::PressureSize),
    LinhaDaMao::Knob(Knob::PressureSizeCurve),
    LinhaDaMao::Knob(Knob::PressureOpacity),
    LinhaDaMao::Knob(Knob::PressureOpacityCurve),
    LinhaDaMao::Knob(Knob::PressureFloor),
    LinhaDaMao::Knob(Knob::PressureCeiling),
    LinhaDaMao::Knob(Knob::PressureSmoothing),
    LinhaDaMao::Knob(Knob::PulledString),
    LinhaDaMao::Knob(Knob::DabRotation),
    LinhaDaMao::Knob(Knob::TiltStart),
    LinhaDaMao::Knob(Knob::TiltFull),
    LinhaDaMao::Knob(Knob::TiltWidth),
    LinhaDaMao::Knob(Knob::TiltLength),
    LinhaDaMao::Knob(Knob::TiltSmooth),
    LinhaDaMao::Knob(Knob::TiltLag),
    LinhaDaMao::TiltOpacity,
    LinhaDaMao::SpeedSize,
    LinhaDaMao::SpeedOpacity,
    LinhaDaMao::PressureSCurve,
    LinhaDaMao::PressureJitter,
    LinhaDaMao::SpeedLag,
];

/// Quantas linhas o cartão tem.
pub const N_LINHAS_DA_MAO: usize = LINHAS_DA_MAO.len();

/// O valor de fábrica de cada linha — a mão do pincel novo (o instantâneo de recurso do painel).
pub const MAO_DE_FABRICA: [f64; N_LINHAS_DA_MAO] = {
    let mut v = [0.0; N_LINHAS_DA_MAO];
    let mut i = 0;
    while i < N_LINHAS_DA_MAO {
        v[i] = match LINHAS_DA_MAO[i] {
            LinhaDaMao::Knob(k) => KNOB_DEFS[k as usize].default,
            LinhaDaMao::TiltOpacity => TILT_OPACITY.4,
            LinhaDaMao::SpeedSize => SPEED_SIZE.4,
            LinhaDaMao::SpeedOpacity => SPEED_OPACITY.4,
            LinhaDaMao::PressureSCurve => PRESSURE_S_CURVE.4,
            LinhaDaMao::PressureJitter => PRESSURE_JITTER.4,
            LinhaDaMao::SpeedLag => SPEED_LAG.4,
        };
        i += 1;
    }
    v
};

/// A chave, a faixa (`min`, `max`, `passo`) e o valor de fábrica da inclinação→opacidade.
const TILT_OPACITY: (&str, f64, f64, f64, f64) = ("tiltOpacity", 0.0, 1.0, 0.01, 0.0);
/// A velocidade→tamanho (L12): o expoente da potência `(v/v₀)^−p` (`06` §10.4: Fineliner 0,06 · pen 0,14 ·
/// Marker 0,45); o tecto é o do motor.
const SPEED_SIZE: (&str, f64, f64, f64, f64) = (
    "speedSize",
    0.0,
    ph2d_painter_brush::dynamics::VELOCIDADE_EXPOENTE_MAX as f64,
    0.01,
    0.0,
);
/// A curva em S da pressão: `0` a recta de sempre, `1` o smoothstep (o G-pen do Krita).
const PRESSURE_S_CURVE: (&str, f64, f64, f64, f64) = ("pressureSCurve", 0.0, 1.0, 0.01, 0.0);
/// A pressão leve espalha: o disco de Jitter na pressão 0, em diâmetros (a esferográfica do MyPaint: 0,354).
const PRESSURE_JITTER: (&str, f64, f64, f64, f64) = ("pressureJitter", 0.0, 1.0, 0.01, 0.0);
/// O atraso da velocidade (s): o `speed1_slowness` do MyPaint (0,04 em 164 dos 196 pincéis dele; o maior 0,2).
const SPEED_LAG: (&str, f64, f64, f64, f64) = ("speedLag", 0.0, 0.5, 0.01, 0.0);
/// A velocidade→opacidade (L12), a mesma lei na cobertura do dab (Thin_Pen ≈ 0,2).
const SPEED_OPACITY: (&str, f64, f64, f64, f64) = (
    "speedOpacity",
    0.0,
    ph2d_painter_brush::dynamics::VELOCIDADE_EXPOENTE_MAX as f64,
    0.01,
    0.0,
);

impl LinhaDaMao {
    /// A chave da linha — a do knob no registo do Wet, e a do mapa `engine` de todo preset.
    #[must_use]
    pub fn chave(self) -> &'static str {
        match self {
            Self::Knob(k) => KNOB_DEFS[k as usize].key,
            Self::TiltOpacity => TILT_OPACITY.0,
            Self::SpeedSize => SPEED_SIZE.0,
            Self::SpeedOpacity => SPEED_OPACITY.0,
            Self::PressureSCurve => PRESSURE_S_CURVE.0,
            Self::PressureJitter => PRESSURE_JITTER.0,
            Self::SpeedLag => SPEED_LAG.0,
        }
    }

    /// `(min, max, passo)`.
    #[must_use]
    pub fn faixa(self) -> (f64, f64, f64) {
        match self {
            Self::Knob(k) => {
                let d = &KNOB_DEFS[k as usize];
                (d.min, d.max, d.step)
            }
            Self::TiltOpacity => (TILT_OPACITY.1, TILT_OPACITY.2, TILT_OPACITY.3),
            Self::SpeedSize => (SPEED_SIZE.1, SPEED_SIZE.2, SPEED_SIZE.3),
            Self::SpeedOpacity => (SPEED_OPACITY.1, SPEED_OPACITY.2, SPEED_OPACITY.3),
            Self::PressureSCurve => (PRESSURE_S_CURVE.1, PRESSURE_S_CURVE.2, PRESSURE_S_CURVE.3),
            Self::PressureJitter => (PRESSURE_JITTER.1, PRESSURE_JITTER.2, PRESSURE_JITTER.3),
            Self::SpeedLag => (SPEED_LAG.1, SPEED_LAG.2, SPEED_LAG.3),
        }
    }

    /// O preset grava a linha mesmo no neutro? As linhas que nasceram DEPOIS da fábrica gravada (a curva em S,
    /// J3) só vão para o ficheiro quando mexidas: o neutro é o valor de quem a não tem, e assim todo preset de
    /// antes volta ao byte (`cada_preset_de_fabrica_e_a_sua_receita`).
    #[must_use]
    pub fn grava_no_neutro(self) -> bool {
        !matches!(self, Self::PressureSCurve | Self::PressureJitter | Self::SpeedLag)
    }

    /// O valor de fábrica (o neutro).
    #[must_use]
    pub fn omissao(self) -> f64 {
        match self {
            Self::Knob(k) => KNOB_DEFS[k as usize].default,
            Self::TiltOpacity => TILT_OPACITY.4,
            Self::SpeedSize => SPEED_SIZE.4,
            Self::SpeedOpacity => SPEED_OPACITY.4,
            Self::PressureSCurve => PRESSURE_S_CURVE.4,
            Self::PressureJitter => PRESSURE_JITTER.4,
            Self::SpeedLag => SPEED_LAG.4,
        }
    }

    /// `v` dentro da faixa; NaN = o valor de fábrica.
    #[must_use]
    pub fn limpa(self, v: f64) -> f64 {
        let (lo, hi, _) = self.faixa();
        if v.is_nan() {
            self.omissao()
        } else {
            v.clamp(lo, hi)
        }
    }

    /// **O pincel de agora lê esta linha?** — [`Self::lida_por`] e, na Watercolor com a Shape «Automatic», a
    /// rotação NÃO (o carimbo redondo dela não tem para onde rodar; a largura e o comprimento deitados esmaecem,
    /// `Dependente::PegadaDaAutomatica`). ⚠️ A rotação sai em vez de esmaecer: ela também é inerte noutros
    /// estados de todos os meios (nada a rodar), e o gate do esmaecido exige a lei EXACTA.
    #[must_use]
    pub fn lida(self, m: PaintMedia, automatica: bool) -> bool {
        self.lida_por(m)
            && !(m == PaintMedia::Watercolor
                && automatica
                && self == Self::Knob(Knob::DabRotation))
    }

    /// **O meio LÊ esta linha?** A Watercolor não lê a cobertura do dab (`WASH_DEPOSIT_PEAK`: a Strength e
    /// a pressão nela não são da lavagem, dono 12/08), e o Wet Paint não tem knob da inclinação→opacidade.
    #[must_use]
    pub fn lida_por(self, m: PaintMedia) -> bool {
        !matches!(
            (self, m),
            (Self::TiltOpacity, PaintMedia::WetPaint | PaintMedia::Watercolor)
                | (
                    Self::Knob(Knob::PressureOpacity | Knob::PressureOpacityCurve)
                        | Self::SpeedOpacity,
                    PaintMedia::Watercolor
                )
        )
    }
}

/// O `f32` de um campo como o valor da linha: pelo menor texto que volta ao mesmo `f32` (`0.9`, não
/// `0.8999999761581421`), para o preset e o painel lerem o número que o artista escreveu.
fn curto(v: f32) -> f64 {
    format!("{v}").parse().unwrap_or(f64::from(v))
}

/// ⭐ **A lei**: a linha `l` com o valor `v` no campo do pincel que ela governa. Os dois armazéns passam
/// por aqui (o Wet em [`super::PainterTool::dinamica_do_traco`], os antigos no setter do cartão).
pub(crate) fn escreve(d: &mut Dynamics, t: &mut PenTilt, l: LinhaDaMao, v: f64) {
    let f = v as f32;
    match l {
        // O raio na pressão 0 é `1 − ela` (`0,9` = os 10 % de sempre, ao bit).
        // `0` desliga o canal (o raio na pressão 0 seria o de 1 — o MESMO `1,0` ao bit, sem rampa).
        LinhaDaMao::Knob(Knob::PressureSize) => {
            d.size_pressure = v > 0.0;
            d.size_min = (1.0 - v) as f32;
        }
        LinhaDaMao::Knob(Knob::PressureSizeCurve) => d.size_curve = f,
        // Acima de 0 a tinta do dab segue a pressão (o Flat do Rebelle); 0 = a lei de sempre.
        LinhaDaMao::Knob(Knob::PressureOpacity) => {
            d.strength_pressure = v > 0.0;
            if v > 0.0 {
                d.strength_min = (1.0 - v) as f32;
            }
        }
        LinhaDaMao::Knob(Knob::PressureOpacityCurve) => d.strength_curve = f,
        LinhaDaMao::Knob(Knob::PressureFloor) => d.pen.floor = f,
        LinhaDaMao::Knob(Knob::PressureCeiling) => d.pen.ceiling = f,
        LinhaDaMao::Knob(Knob::PressureSmoothing) => d.pen.smoothing = f,
        LinhaDaMao::Knob(Knob::PulledString) => d.pen.pulled_string_px = f,
        LinhaDaMao::Knob(Knob::DabRotation) => t.rotation = DabRotation::from_index(v),
        LinhaDaMao::Knob(Knob::TiltStart) => t.start_deg = f,
        LinhaDaMao::Knob(Knob::TiltFull) => t.full_deg = f,
        LinhaDaMao::Knob(Knob::TiltWidth) => t.width = f,
        LinhaDaMao::Knob(Knob::TiltLength) => t.length = f,
        LinhaDaMao::Knob(Knob::TiltSmooth) => t.smooth = f,
        LinhaDaMao::Knob(Knob::TiltLag) => t.lag_px = f,
        LinhaDaMao::TiltOpacity => t.opacity = f,
        LinhaDaMao::SpeedSize => d.speed_size = f,
        LinhaDaMao::SpeedOpacity => d.speed_opacity = f,
        LinhaDaMao::PressureSCurve => d.pen.s_curve = f,
        LinhaDaMao::PressureJitter => d.jitter_pressure = f,
        LinhaDaMao::SpeedLag => d.speed_lag_s = f,
        LinhaDaMao::Knob(k) => unreachable!("knob fora da mão: {k:?}"),
    }
}

/// O valor da linha `l` lido dos campos (o inverso de [`escreve`]: escrever o que se leu não muda um bit).
#[must_use]
pub(crate) fn le(d: &Dynamics, t: &PenTilt, l: LinhaDaMao) -> f64 {
    match l {
        LinhaDaMao::Knob(Knob::PressureSize) => {
            if d.size_pressure {
                curto(1.0 - d.size_min)
            } else {
                0.0
            }
        }
        LinhaDaMao::Knob(Knob::PressureSizeCurve) => curto(d.size_curve),
        LinhaDaMao::Knob(Knob::PressureOpacity) => {
            if d.strength_pressure {
                curto(1.0 - d.strength_min)
            } else {
                0.0
            }
        }
        LinhaDaMao::Knob(Knob::PressureOpacityCurve) => curto(d.strength_curve),
        LinhaDaMao::Knob(Knob::PressureFloor) => curto(d.pen.floor),
        LinhaDaMao::Knob(Knob::PressureCeiling) => curto(d.pen.ceiling),
        LinhaDaMao::Knob(Knob::PressureSmoothing) => curto(d.pen.smoothing),
        LinhaDaMao::Knob(Knob::PulledString) => curto(d.pen.pulled_string_px),
        LinhaDaMao::Knob(Knob::DabRotation) => match t.rotation {
            DabRotation::None => 0.0,
            DabRotation::FollowTrajectory => 1.0,
            DabRotation::PenTilt => 2.0,
        },
        LinhaDaMao::Knob(Knob::TiltStart) => curto(t.start_deg),
        LinhaDaMao::Knob(Knob::TiltFull) => curto(t.full_deg),
        LinhaDaMao::Knob(Knob::TiltWidth) => curto(t.width),
        LinhaDaMao::Knob(Knob::TiltLength) => curto(t.length),
        LinhaDaMao::Knob(Knob::TiltSmooth) => curto(t.smooth),
        LinhaDaMao::Knob(Knob::TiltLag) => curto(t.lag_px),
        LinhaDaMao::TiltOpacity => curto(t.opacity),
        LinhaDaMao::SpeedSize => curto(d.speed_size),
        LinhaDaMao::SpeedOpacity => curto(d.speed_opacity),
        LinhaDaMao::PressureSCurve => curto(d.pen.s_curve),
        LinhaDaMao::PressureJitter => curto(d.jitter_pressure),
        LinhaDaMao::SpeedLag => curto(d.speed_lag_s),
        LinhaDaMao::Knob(k) => unreachable!("knob fora da mão: {k:?}"),
    }
}

/// A mão de FÁBRICA de um pincel dos meios antigos: a de cada linha no seu valor de fábrica, a partir da
/// `Dynamics`/`PenTilt` de sempre — é a de `Dynamics::default()` e `PenTilt::default()` ao bit
/// (`a_mao_de_fabrica_e_a_de_sempre`).
#[must_use]
pub(crate) fn de_fabrica() -> (Dynamics, PenTilt) {
    let (mut d, mut t) = (Dynamics::default(), PenTilt::default());
    for l in LINHAS_DA_MAO {
        escreve(&mut d, &mut t, l, l.omissao());
    }
    (d, t)
}

/// As linhas que governam a `Dynamics` (a pressão e a corda); as outras são da `PenTilt`, um campo do pincel.
fn da_pressao(l: LinhaDaMao) -> bool {
    matches!(
        l,
        LinhaDaMao::SpeedSize
            | LinhaDaMao::SpeedOpacity
            | LinhaDaMao::PressureSCurve
            | LinhaDaMao::PressureJitter
            | LinhaDaMao::SpeedLag
    ) || matches!(
        l,
        LinhaDaMao::Knob(
            Knob::PressureSize
                | Knob::PressureSizeCurve
                | Knob::PressureOpacity
                | Knob::PressureOpacityCurve
                | Knob::PressureFloor
                | Knob::PressureCeiling
                | Knob::PressureSmoothing
                | Knob::PulledString
        )
    )
}

/// A `Dynamics` dos meios antigos pelas chaves das linhas — o que o estado inicial de uma MACRO leva dela (a
/// `PenTilt` já vai no pincel, `campos_da_macro`).
pub(crate) fn dinamica_para_json(d: &Dynamics) -> serde_json::Value {
    let t = PenTilt::default();
    LINHAS_DA_MAO
        .into_iter()
        .filter(|l| da_pressao(*l) && (l.grava_no_neutro() || le(d, &t, *l) != l.omissao()))
        .map(|l| (l.chave().to_owned(), serde_json::json!(le(d, &t, l))))
        .collect::<serde_json::Map<_, _>>()
        .into()
}

/// O inverso de [`dinamica_para_json`]. Uma chave ausente (uma macro de antes da L2) é a de FÁBRICA, nunca a mão
/// de agora: a macro toca o que gravou.
pub(crate) fn dinamica_de_json(d: &mut Dynamics, v: &serde_json::Value) {
    let fab = de_fabrica().0;
    *d = Dynamics {
        pen: ph2d_painter_brush::Pen {
            tilt: d.pen.tilt,
            ..fab.pen
        },
        ..fab
    };
    let mut t = PenTilt::default();
    for l in LINHAS_DA_MAO.into_iter().filter(|l| da_pressao(*l)) {
        let x = v
            .get(l.chave())
            .and_then(serde_json::Value::as_f64)
            .unwrap_or_else(|| l.omissao());
        escreve(d, &mut t, l, l.limpa(x));
    }
}

impl super::PainterTool {
    /// O valor de cada linha do cartão Pen no meio de agora (o que o painel mostra).
    #[must_use]
    pub fn mao(&self) -> [f64; N_LINHAS_DA_MAO] {
        let k = &self.paint.wetpaint.knobs;
        LINHAS_DA_MAO.map(|l| match (self.paint.wetpaint.armed, l) {
            (true, LinhaDaMao::Knob(kn)) => k.get(kn),
            (true, LinhaDaMao::TiltOpacity) => l.omissao(),
            // A velocidade e a curva em S não têm knob no Wet: moram na `Dynamics` do pincel, que a mão do Wet herda.
            (_, _) => le(&self.paint.dynamics, &self.paint.brush.pen_tilt, l),
        })
    }

    /// **Uma linha do cartão Pen**, no armazém do meio de agora: os knobs do pincel no Wet Paint, os campos do
    /// pincel nos outros três. Uma linha que o meio não lê não muda nada (o painel nem a pinta).
    pub fn set_mao(&mut self, l: LinhaDaMao, v: f64) {
        let v = l.limpa(v);
        if !l.lida_por(self.paint_media()) {
            return;
        }
        match (self.paint.wetpaint.armed, l) {
            (true, LinhaDaMao::Knob(kn)) => self.paint.wetpaint.knobs.set(kn, v),
            (true, LinhaDaMao::TiltOpacity) => {}
            (_, _) => escreve(
                &mut self.paint.dynamics,
                &mut self.paint.brush.pen_tilt,
                l,
                v,
            ),
        }
    }

    /// A mão dos meios antigos de volta à de fábrica (o pincel novo de um preset que não a fixa).
    pub(crate) fn repor_mao(&mut self) {
        (self.paint.dynamics, self.paint.brush.pen_tilt) = de_fabrica();
    }

    /// O Reset do cartão Pen: a mão de fábrica do meio de agora (no Wet Paint, os knobs da caneta).
    fn repor_mao_do_meio(&mut self) {
        if self.paint.wetpaint.armed {
            for l in LINHAS_DA_MAO {
                if let LinhaDaMao::Knob(kn) = l {
                    let v = super::WetKnobs::DEFAULT.get(kn);
                    self.paint.wetpaint.knobs.set(kn, v);
                }
            }
            (self.paint.dynamics.speed_size, self.paint.dynamics.speed_opacity) = (0.0, 0.0);
            self.paint.dynamics.pen.s_curve = 0.0;
            self.paint.dynamics.jitter_pressure = 0.0;
            self.paint.dynamics.speed_lag_s = 0.0;
        } else {
            self.repor_mao();
        }
    }

    /// As linhas do cartão Pen (`SetValue`) e o Reset dele (`Click`), do canal do painel.
    pub(crate) fn route_mao_event(&mut self, event: &ph2d_editor_core::tool::PanelEvent) -> bool {
        use ph2d_editor_core::tool::PanelEvent;
        match event {
            PanelEvent::SetValue(id, v) => {
                let Some(i) = crate::ids::PAINTER_PEN_FIELDS.iter().position(|f| f == id) else {
                    return false;
                };
                self.set_mao(LINHAS_DA_MAO[i], *v);
                true
            }
            PanelEvent::SelectOption(id, v) if *id == crate::ids::PAINTER_PEN_ROTATION => {
                let Ok(i) = v.parse::<u8>() else {
                    return false;
                };
                self.set_mao(LinhaDaMao::Knob(Knob::DabRotation), f64::from(i));
                true
            }
            PanelEvent::Click(id) if *id == crate::ids::PAINTER_PEN_RESET => {
                self.repor_mao_do_meio();
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐ A mão de fábrica é a `Dynamics`/`PenTilt` de sempre ao bit (o neutro da L2).
    #[test]
    fn a_mao_de_fabrica_e_a_de_sempre() {
        let (d, t) = de_fabrica();
        assert_eq!(d, Dynamics::default());
        assert_eq!(t, PenTilt::default());
    }

    /// Os ids do cartão Pen são os das linhas, pela ordem e pela chave.
    #[test]
    fn cada_linha_tem_o_seu_id_pela_chave() {
        for (l, id) in LINHAS_DA_MAO.into_iter().zip(crate::ids::PAINTER_PEN_FIELDS) {
            let pela_chave =
                ph2d_tool_registry::hash_node_id_runtime(&format!("painter_brush.pen.{}", l.chave()));
            assert_eq!(id, pela_chave, "{}", l.chave());
        }
        assert_eq!(MAO_DE_FABRICA, LINHAS_DA_MAO.map(LinhaDaMao::omissao));
    }

    /// ⭐ **A pressão do Digital afina como a do Wet** (L2): as mesmas linhas no cartão de cada meio dão a
    /// MESMA `Dynamics` e a MESMA `PenTilt` ao traço — a lei é uma, os armazéns é que são dois. Na Watercolor
    /// as linhas da opacidade não se escrevem (o meio não as lê) e o resto é igual.
    #[test]
    fn a_pressao_digital_afina_como_a_do_wet() {
        let valores = [
            (Knob::PressureSize, 0.5),
            (Knob::PressureSizeCurve, 0.7),
            (Knob::PressureOpacity, 0.4),
            (Knob::PressureOpacityCurve, 1.5),
            (Knob::PressureFloor, 0.1),
            (Knob::PressureCeiling, 0.9),
            (Knob::PressureSmoothing, 0.3),
            (Knob::PulledString, 12.0),
            (Knob::DabRotation, 2.0),
            (Knob::TiltStart, 20.0),
            (Knob::TiltFull, 50.0),
            (Knob::TiltWidth, 1.5),
            (Knob::TiltLength, 2.0),
            (Knob::TiltSmooth, 0.4),
            (Knob::TiltLag, 30.0),
        ];
        let mut wet = super::super::super::PainterTool::default();
        wet.set_paint_media(PaintMedia::WetPaint);
        for (k, v) in valores {
            wet.set_mao(LinhaDaMao::Knob(k), v);
        }
        for m in [PaintMedia::Digital, PaintMedia::Watercolor, PaintMedia::Impasto] {
            let mut t = super::super::super::PainterTool::default();
            t.set_paint_media(m);
            for (k, v) in valores {
                t.set_mao(LinhaDaMao::Knob(k), v);
            }
            let (d, w) = (t.dinamica_do_traco(), wet.dinamica_do_traco());
            let (mut ti, wi) = (t.inclinacao_do_pincel(), wet.inclinacao_do_pincel());
            if m == PaintMedia::Watercolor {
                // A Shape «Automatic» (a de fábrica da Watercolor) não lê a pegada deitada.
                assert_eq!((ti.width, ti.length, ti.rotation), (1.0, 1.0, DabRotation::None));
                (ti.width, ti.length, ti.rotation) = (wi.width, wi.length, wi.rotation);
            }
            assert_eq!(ti, wi, "{m:?}");
            assert_eq!((d.size_min, d.size_curve, d.pen), (w.size_min, w.size_curve, w.pen), "{m:?}");
            if m == PaintMedia::Watercolor {
                assert!(!d.strength_pressure, "a Watercolor não lê a opacidade do dab");
            } else {
                assert_eq!(d, w, "{m:?}");
            }
        }
    }

    /// Uma MACRO gravada antes da L2 (sem a chave `mao`, e a `pen_tilt` sem `opacity`) toca com a mão de FÁBRICA,
    /// nunca com a de agora.
    #[test]
    fn uma_macro_sem_a_mao_volta_a_mao_de_fabrica() {
        let mut d = Dynamics::default();
        let mut t = PenTilt::default();
        for l in LINHAS_DA_MAO {
            let (lo, hi, _) = l.faixa();
            escreve(&mut d, &mut t, l, lo + (hi - lo) * 0.3);
        }
        dinamica_de_json(&mut d, &serde_json::Value::Null);
        assert_eq!(d, Dynamics::default());
        let mut b = ph2d_painter_brush::BrushSpec::default();
        b.pen_tilt.opacity = 0.7;
        let mut m = super::super::wetpaint_preset::json::pincel_para_json(&b, &["pen_tilt"]);
        m.get_mut("pen_tilt")
            .and_then(serde_json::Value::as_object_mut)
            .unwrap()
            .remove("opacity");
        let lida = super::super::wetpaint_preset::json::pincel_de_json(&m, &["pen_tilt"], b);
        assert_eq!(lida.pen_tilt.opacity, 0.0, "a pen_tilt antiga não traz a opacidade de agora");
    }

    /// ⭐ **As linhas que nasceram depois da fábrica só vão para o preset MEXIDAS** (a curva em S, a pressão leve —
    /// J3 canetas 2.ª onda): o pincel de fábrica não as grava (todo preset de antes fica ao byte), e mexida, grava-a.
    #[test]
    fn as_linhas_novas_so_vao_para_o_preset_mexidas() {
        use super::super::wetpaint_preset::nome_do_meio;
        use ph2d_brush_presets::Parametros;
        let novas = [LinhaDaMao::PressureSCurve, LinhaDaMao::PressureJitter, LinhaDaMao::SpeedLag];
        let mut t = super::super::super::PainterTool::default();
        t.set_paint_media(PaintMedia::Digital);
        let base = Parametros::novo(nome_do_meio(PaintMedia::Digital), "g", "n");
        let (p, _) = t.preset_capturar(&base);
        for l in novas {
            assert!(!p.engine.contains_key(l.chave()), "{} no preset de fábrica", l.chave());
            t.set_mao(l, 0.5);
        }
        let (p, _) = t.preset_capturar(&base);
        for l in novas {
            assert_eq!(p.engine.get(l.chave()).copied(), Some(0.5), "{} mexida e não gravada", l.chave());
        }
    }

    /// Ler e voltar a escrever cada linha não muda um bit dos campos (o painel e o preset leem por [`le`]).
    #[test]
    fn ler_e_escrever_a_mao_e_a_identidade() {
        let (mut d, mut t) = de_fabrica();
        for (i, l) in LINHAS_DA_MAO.into_iter().enumerate() {
            let (lo, hi, _) = l.faixa();
            escreve(&mut d, &mut t, l, lo + (hi - lo) * (i as f64 + 1.0) / 20.0);
        }
        let (d0, t0) = (d, t);
        for l in LINHAS_DA_MAO {
            let v = le(&d0, &t0, l);
            escreve(&mut d, &mut t, l, v);
        }
        assert_eq!((d, t), (d0, t0));
    }
}

/// L12 — a velocidade pela lei da mão, pela porta do ponteiro (`06_LEIS_DAS_LACUNAS.md` §10.4).
#[cfg(test)]
mod velocidade {
    use super::*;
    use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

    /// Um recto horizontal a `passo` px por evento, um quadro de 8 ms por evento (o `sonda_gestos::gesto`);
    /// devolve a altura da faixa pintada na coluna do meio.
    fn altura(m: PaintMedia, passo: f32, linha: Option<(LinhaDaMao, f64)>) -> usize {
        alturas(m, passo, linha, &[400])[0]
    }

    /// [`altura`] nas colunas `xs`.
    fn alturas(
        m: PaintMedia,
        passo: f32,
        linha: Option<(LinhaDaMao, f64)>,
        xs: &[usize],
    ) -> Vec<usize> {
        alturas_com(m, passo, linha.as_slice(), xs)
    }

    /// [`alturas`] com várias linhas da mão.
    fn alturas_com(m: PaintMedia, passo: f32, linhas: &[(LinhaDaMao, f64)], xs: &[usize]) -> Vec<usize> {
        let mut t = super::super::super::PainterTool::default();
        t.set_source(vec![255u8; 600 * 120 * 4], 600, 120);
        t.set_paint_media(m);
        t.set_brush_size_px(16.0);
        for &(l, v) in linhas {
            t.set_mao(l, v);
        }
        let cp = |x: f32, phase| CanvasPointer {
            pos: [x, 60.0],
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        };
        t.on_canvas_pointer(cp(20.0, PointerPhase::Down));
        let mut x = 20.0;
        while x < 580.0 {
            x += passo;
            t.on_canvas_pointer(cp(x.min(580.0), PointerPhase::Move));
            t.paint_tick(0.008);
        }
        t.on_canvas_pointer(cp(580.0, PointerPhase::Up));
        xs.iter()
            .map(|&x| {
                (0..120)
                    .filter(|y| t.canvas_rgba[(y * 600 + x) * 4] < 200)
                    .count()
            })
            .collect()
    }

    /// ⭐ **A velocidade vale desde o 1.º dab** (J3, `docs/Painter/Pinceis/exames/canetas.md` 2.ª onda): o traço só
    /// mede a velocidade no TIQUE, e os dabs do 1.º quadro saíam a `v = 0` — a largura cheia — e a rampa ainda
    /// subia de 0 no 2.º: uma BOLHA no início do traço rápido (o Technical Pen contra o Fineliner do Krita, que
    /// começa fino). Com o Marker (`speedSize 0,45`) a 50 px por evento, o início (a 2 px do pen-down e a 30 px)
    /// tem a largura do meio (± 1 px); sem a lei, idem (o neutro não muda).
    #[test]
    fn a_velocidade_vale_desde_o_primeiro_dab() {
        let xs = [22, 50, 75, 400];
        for m in [PaintMedia::Digital, PaintMedia::Impasto] {
            let h = alturas(m, 50.0, Some((LinhaDaMao::SpeedSize, 0.45)), &xs);
            for (x, a) in xs.iter().zip(&h) {
                assert!(a.abs_diff(h[3]) <= 1, "{m:?}: a {x} px a largura é {a}, a do meio {} — {h:?}", h[3]);
            }
        }
    }

    /// ⭐ **A velocidade com ATRASO** (J4 efeitos, `docs/Painter/Pinceis/exames/efeitos.md`; o `speed1_slowness` do
    /// MyPaint): com `speedLag` 0,04 s o traço rápido NASCE largo (o filtro ainda não viu a mão: `v = 0`) e afina à
    /// medida que o filtro a apanha — a cabeça do traço rápido do `experimental/glow`; sem o atraso, o início tem a
    /// largura do meio (a lei da J3, intacta). O 1.º dab sai à largura de SEM velocidade, e um atraso maior afina
    /// mais tarde (o filtro, não só o arranque a zero).
    #[test]
    fn a_velocidade_com_atraso_nasce_larga() {
        let xs = [22, 200, 560];
        let marker = (LinhaDaMao::SpeedSize, 0.45);
        let lag = |s: f64| (LinhaDaMao::SpeedLag, s);
        let plena = alturas_com(PaintMedia::Digital, 50.0, &[], &xs);
        let curto = alturas_com(PaintMedia::Digital, 50.0, &[marker, lag(0.04)], &xs);
        let longo = alturas_com(PaintMedia::Digital, 50.0, &[marker, lag(0.2)], &xs);
        let sem = alturas_com(PaintMedia::Digital, 50.0, &[marker], &xs);
        assert!(sem[0].abs_diff(sem[2]) <= 1, "sem atraso o início é o meio: {sem:?}");
        assert!(curto[0] + 1 >= plena[0], "com atraso o 1.º dab sai à largura cheia: {curto:?} × {plena:?}");
        assert!(curto[2] + 3 <= curto[0], "com atraso o traço afina: {curto:?}");
        assert!(longo[1] >= curto[1] + 2, "o atraso maior afina mais tarde: {longo:?} × {curto:?}");
    }

    /// A velocidade→opacidade chega ao Wet Paint (a mão dele herda a `Dynamics` do pincel): no recto rápido a
    /// tinta muda (no lento do censo a tinta molhada enche até cheia — `o_censo_so_encolhe`).
    #[test]
    fn a_velocidade_chega_ao_wet() {
        let tela = |v: f64| {
            let mut t = super::super::super::PainterTool::default();
            t.set_source(vec![255u8; 600 * 120 * 4], 600, 120);
            t.set_paint_media(PaintMedia::WetPaint);
            t.set_mao(LinhaDaMao::SpeedOpacity, v);
            let cp = |x: f32, phase| CanvasPointer {
                pos: [x, 60.0],
                pressure: 1.0,
                tilt: [0.0, 0.0],
                phase,
            };
            t.on_canvas_pointer(cp(20.0, PointerPhase::Down));
            let mut x: f32 = 20.0;
            while x < 580.0 {
                x += 40.0;
                t.on_canvas_pointer(cp(x.min(580.0), PointerPhase::Move));
                t.paint_tick(0.008);
            }
            t.on_canvas_pointer(cp(580.0, PointerPhase::Up));
            for _ in 0..30 {
                t.paint_tick(1.0 / 60.0);
            }
            t.canvas_rgba.to_vec()
        };
        let (a, b) = (tela(0.0), tela(1.0));
        let n = a.iter().zip(&b).filter(|(x, y)| x != y).count();
        assert!(n > 1000, "a velocidade não chegou ao Wet: {n} bytes");
    }

    /// ⭐ **`recto lento × rápido`**: com a lei do Marker do MyPaint (`speedSize 0,45`) o recto a 50 px por
    /// evento (6 250 px/s) sai mais FINO que o de 3 px (375 px/s) — perto da potência medida, `(6250/375)^−0,45`
    /// = 0,28 da lenta ± 2 px —, e sem a lei (o Rebelle: 221 presets neutros) os dois têm a mesma largura.
    #[test]
    fn a_velocidade_estreita_o_traco() {
        for m in [PaintMedia::Digital, PaintMedia::Impasto] {
            let (lento, rapido) = (altura(m, 3.0, None), altura(m, 50.0, None));
            assert!(lento > 8 && lento.abs_diff(rapido) <= 1, "{m:?} neutro: {lento} × {rapido}");
            let marker = Some((LinhaDaMao::SpeedSize, 0.45));
            let (l, r) = (altura(m, 3.0, marker), altura(m, 50.0, marker));
            let esperado = l as f32 * (6250.0f32 / 375.0).powf(-0.45);
            assert!(
                (r as f32 - esperado).abs() <= 2.0,
                "{m:?}: rápido {r} px × lento {l} px (a lei: {esperado:.1})"
            );
        }
    }
}

/// A pressão leve espalha (J3 canetas 2.ª onda), pela porta do ponteiro.
#[cfg(test)]
mod pressao_leve {
    use super::*;
    use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

    /// Um recto à pressão `p` (3 px por evento) com a linha `pressureJitter` a `j`: a tela inteira.
    fn recto(p: f32, j: f64) -> Vec<u8> {
        let mut t = super::super::super::PainterTool::default();
        t.set_source(vec![255u8; 600 * 120 * 4], 600, 120);
        t.set_paint_media(PaintMedia::Digital);
        t.set_brush_size_px(16.0);
        t.set_mao(LinhaDaMao::PressureJitter, j);
        let cp = |x: f32, phase| CanvasPointer { pos: [x, 60.0], pressure: p, tilt: [0.0, 0.0], phase };
        t.on_canvas_pointer(cp(20.0, PointerPhase::Down));
        let mut x: f32 = 20.0;
        while x < 580.0 {
            x += 3.0;
            t.on_canvas_pointer(cp(x.min(580.0), PointerPhase::Move));
            t.paint_tick(0.008);
        }
        t.on_canvas_pointer(cp(580.0, PointerPhase::Up));
        t.canvas_rgba.to_vec()
    }

    /// As linhas da tela com tinta (algum texel abaixo de 250 no vermelho).
    fn linhas_com_tinta(px: &[u8]) -> usize {
        (0..120).filter(|y| (100..500).any(|x| px[(y * 600 + x) * 4] < 250)).count()
    }

    /// ⭐ **A pressão leve espalha** (`Dynamics::jitter_pressure`; a esferográfica do MyPaint, `offset_by_random`
    /// 0,354 → 0 com a pressão): à pressão 0,2 a tinta espalha-se por mais linhas do que sem a lei; à pressão 1 a
    /// lei não move um dab (a tela é a de sem ela, ao byte).
    #[test]
    fn a_pressao_leve_espalha_e_a_cheia_nao() {
        let (sem, com) = (linhas_com_tinta(&recto(0.2, 0.0)), linhas_com_tinta(&recto(0.2, 0.5)));
        assert!(com >= sem + 4, "à pressão 0,2 a lei não espalhou: {sem} → {com} linhas");
        assert!(recto(1.0, 0.0) == recto(1.0, 0.5), "à pressão 1 a lei mexeu na tinta");
    }
}
