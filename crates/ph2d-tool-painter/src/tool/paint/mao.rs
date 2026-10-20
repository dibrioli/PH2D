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
}

/// As linhas do cartão, pela ordem do painel.
pub const LINHAS_DA_MAO: [LinhaDaMao; 16] = [
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
        };
        i += 1;
    }
    v
};

/// A chave, a faixa (`min`, `max`, `passo`) e o valor de fábrica da inclinação→opacidade.
const TILT_OPACITY: (&str, f64, f64, f64, f64) = ("tiltOpacity", 0.0, 1.0, 0.01, 0.0);

impl LinhaDaMao {
    /// A chave da linha — a do knob no registo do Wet, e a do mapa `engine` de todo preset.
    #[must_use]
    pub fn chave(self) -> &'static str {
        match self {
            Self::Knob(k) => KNOB_DEFS[k as usize].key,
            Self::TiltOpacity => TILT_OPACITY.0,
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
        }
    }

    /// O valor de fábrica (o neutro).
    #[must_use]
    pub fn omissao(self) -> f64 {
        match self {
            Self::Knob(k) => KNOB_DEFS[k as usize].default,
            Self::TiltOpacity => TILT_OPACITY.4,
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

    /// **O meio LÊ esta linha?** A Watercolor não lê a cobertura do dab (`WASH_DEPOSIT_PEAK`: a Strength e
    /// a pressão nela não são da lavagem, dono 12/08), e o Wet Paint não tem knob da inclinação→opacidade.
    #[must_use]
    pub fn lida_por(self, m: PaintMedia) -> bool {
        !matches!(
            (self, m),
            (Self::TiltOpacity, PaintMedia::WetPaint | PaintMedia::Watercolor)
                | (
                    Self::Knob(Knob::PressureOpacity | Knob::PressureOpacityCurve),
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
        LinhaDaMao::Knob(Knob::PressureSize) => d.size_min = (1.0 - v) as f32,
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
        .filter(|l| da_pressao(*l))
        .map(|l| (l.chave().to_owned(), serde_json::json!(le(d, &t, l))))
        .collect::<serde_json::Map<_, _>>()
        .into()
}

/// O inverso de [`dinamica_para_json`]; uma chave ausente (uma macro de antes da L2) não mexe.
pub(crate) fn dinamica_de_json(d: &mut Dynamics, v: &serde_json::Value) {
    let mut t = PenTilt::default();
    for l in LINHAS_DA_MAO.into_iter().filter(|l| da_pressao(*l)) {
        if let Some(x) = v.get(l.chave()).and_then(serde_json::Value::as_f64) {
            escreve(d, &mut t, l, l.limpa(x));
        }
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
            (false, _) => le(&self.paint.dynamics, &self.paint.brush.pen_tilt, l),
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
            (false, _) => escreve(
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
            assert_eq!(t.inclinacao_do_pincel(), wet.inclinacao_do_pincel(), "{m:?}");
            assert_eq!((d.size_min, d.size_curve, d.pen), (w.size_min, w.size_curve, w.pen), "{m:?}");
            if m == PaintMedia::Watercolor {
                assert!(!d.strength_pressure, "a Watercolor não lê a opacidade do dab");
            } else {
                assert_eq!(d, w, "{m:?}");
            }
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
