//! **As linhas da MARCA no painel** (L4, `docs/Painter/Pinceis/03_MOTORES.md` §4): cada campo do
//! [`ph2d_painter_brush::DabJitter`] é uma linha numérica do cartão Jitter, nos quatro meios (o carimbo é um
//! só). UMA tabela — a chave (a do preset e a do id), a faixa, o valor de fábrica — e duas portas: [`escreve`]
//! e [`le`]. O preset grava o campo inteiro no `BrushSpec` (`dab_jitter`), não estas chaves.

use ph2d_painter_brush::{BrushSpec, CorDaMao, DabJitter};

/// Uma linha numérica da marca.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinhaDaMarca {
    Opacity,
    Count,
    Roundness,
    FlipX,
    FlipY,
    AnglePerStroke,
    Across,
    Particle,
    // A cor secundária e a cor pela MÃO (L11, `ph2d_painter_brush::CorDaMao`), na secção Randomize Color.
    SecondaryJitter,
    PressureSecondary,
    PressureHue,
    PressureSat,
    PressureVal,
}

/// Onde a linha se pinta.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CartaoDaMarca {
    /// O cartão Jitter da secção Stroke (L4).
    Jitter,
    /// A secção Randomize Color (L11).
    Cor,
}

/// As linhas, pela ordem do painel.
pub const LINHAS_DA_MARCA: [LinhaDaMarca; 13] = [
    LinhaDaMarca::Opacity,
    LinhaDaMarca::Count,
    LinhaDaMarca::Roundness,
    LinhaDaMarca::FlipX,
    LinhaDaMarca::FlipY,
    LinhaDaMarca::AnglePerStroke,
    LinhaDaMarca::Across,
    LinhaDaMarca::Particle,
    LinhaDaMarca::SecondaryJitter,
    LinhaDaMarca::PressureSecondary,
    LinhaDaMarca::PressureHue,
    LinhaDaMarca::PressureSat,
    LinhaDaMarca::PressureVal,
];

/// Quantas linhas a marca tem.
pub const N_LINHAS_DA_MARCA: usize = LINHAS_DA_MARCA.len();

/// O valor de fábrica de cada linha (o neutro: o traço de sempre).
pub const MARCA_DE_FABRICA: [f64; N_LINHAS_DA_MARCA] = {
    let mut v = [0.0; N_LINHAS_DA_MARCA];
    let mut i = 0;
    while i < N_LINHAS_DA_MARCA {
        v[i] = LINHAS_DA_MARCA[i].tabela().4;
        i += 1;
    }
    v
};

impl LinhaDaMarca {
    /// `(chave, min, max, passo, fábrica)`.
    const fn tabela(self) -> (&'static str, f64, f64, f64, f64) {
        match self {
            Self::Opacity => ("opacity", 0.0, 1.0, 0.01, 0.0),
            Self::Count => ("count", 0.0, 1.0, 0.01, 0.0),
            Self::Roundness => ("roundness", 0.0, 1.0, 0.01, 0.0),
            Self::FlipX => ("flipX", 0.0, 1.0, 0.01, 0.0),
            Self::FlipY => ("flipY", 0.0, 1.0, 0.01, 0.0),
            Self::AnglePerStroke => ("anglePerStroke", 0.0, 1.0, 0.01, 0.0),
            Self::Across => ("across", 0.0, DabJitter::ACROSS_MAX as f64, 0.01, 0.0),
            Self::Particle => ("particle", DabJitter::PARTICULA_MIN as f64, 1.0, 0.01, 1.0),
            Self::SecondaryJitter => ("secondaryJitter", 0.0, 1.0, 0.01, 0.0),
            Self::PressureSecondary => ("pressureSecondary", 0.0, 1.0, 0.01, 0.0),
            Self::PressureHue => (
                "pressureHue",
                -(CorDaMao::HUE_MAX as f64),
                CorDaMao::HUE_MAX as f64,
                0.01,
                0.0,
            ),
            Self::PressureSat => ("pressureSat", 0.0, 1.0, 0.01, 0.0),
            Self::PressureVal => ("pressureVal", 0.0, 1.0, 0.01, 0.0),
        }
    }

    /// Onde a linha se pinta.
    #[must_use]
    pub fn cartao(self) -> CartaoDaMarca {
        match self {
            Self::SecondaryJitter
            | Self::PressureSecondary
            | Self::PressureHue
            | Self::PressureSat
            | Self::PressureVal => CartaoDaMarca::Cor,
            _ => CartaoDaMarca::Jitter,
        }
    }

    /// A chave da linha (o id do painel é `painter_brush.marca.<chave>`).
    #[must_use]
    pub fn chave(self) -> &'static str {
        self.tabela().0
    }

    /// `(min, max, passo)`.
    #[must_use]
    pub fn faixa(self) -> (f64, f64, f64) {
        let t = self.tabela();
        (t.1, t.2, t.3)
    }

    /// O valor de fábrica (o neutro).
    #[must_use]
    pub fn omissao(self) -> f64 {
        self.tabela().4
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
}

/// ⭐ **A lei**: a linha `l` com o valor `v` no campo do pincel que ela governa.
pub(crate) fn escreve(b: &mut BrushSpec, l: LinhaDaMarca, v: f64) {
    let v = l.limpa(v) as f32;
    let c = &mut b.cor_mao;
    match l {
        LinhaDaMarca::SecondaryJitter => return c.secondary_jitter = v,
        LinhaDaMarca::PressureSecondary => return c.pressure_secondary = v,
        LinhaDaMarca::PressureHue => return c.pressure_hue = v,
        LinhaDaMarca::PressureSat => return c.pressure_sat = v,
        LinhaDaMarca::PressureVal => return c.pressure_val = v,
        _ => {}
    }
    let j = &mut b.dab_jitter;
    match l {
        LinhaDaMarca::Opacity => j.opacity = v,
        LinhaDaMarca::Count => j.count = v,
        LinhaDaMarca::Roundness => j.roundness = v,
        LinhaDaMarca::FlipX => j.flip_x = v,
        LinhaDaMarca::FlipY => j.flip_y = v,
        LinhaDaMarca::AnglePerStroke => j.angle_per_stroke = v,
        LinhaDaMarca::Across => j.across = v,
        LinhaDaMarca::Particle => j.particle = v,
        _ => {}
    }
}

/// O valor da linha `l` lido do pincel (o inverso de [`escreve`]), pelo menor texto que volta ao mesmo `f32`.
#[must_use]
pub(crate) fn le(b: &BrushSpec, l: LinhaDaMarca) -> f64 {
    let j = &b.dab_jitter;
    let c = &b.cor_mao;
    let v = match l {
        LinhaDaMarca::Opacity => j.opacity,
        LinhaDaMarca::Count => j.count,
        LinhaDaMarca::Roundness => j.roundness,
        LinhaDaMarca::FlipX => j.flip_x,
        LinhaDaMarca::FlipY => j.flip_y,
        LinhaDaMarca::AnglePerStroke => j.angle_per_stroke,
        LinhaDaMarca::Across => j.across,
        LinhaDaMarca::Particle => j.particle,
        LinhaDaMarca::SecondaryJitter => c.secondary_jitter,
        LinhaDaMarca::PressureSecondary => c.pressure_secondary,
        LinhaDaMarca::PressureHue => c.pressure_hue,
        LinhaDaMarca::PressureSat => c.pressure_sat,
        LinhaDaMarca::PressureVal => c.pressure_val,
    };
    format!("{v}").parse().unwrap_or(f64::from(v))
}

impl super::PainterTool {
    /// O valor de cada linha da marca (o que o painel mostra).
    #[must_use]
    pub fn marca(&self) -> [f64; N_LINHAS_DA_MARCA] {
        LINHAS_DA_MARCA.map(|l| le(&self.paint.brush, l))
    }

    /// **Uma linha da marca** no pincel de agora (os quatro meios leem o mesmo carimbo).
    pub fn set_marca(&mut self, l: LinhaDaMarca, v: f64) {
        escreve(&mut self.paint.brush, l, v);
    }

    /// A cor secundária do pincel (L11; sRGB `0..1`).
    pub fn set_marca_secundaria(&mut self, c: [f32; 3]) {
        self.paint.brush.cor_mao.secondary = c.map(|x| x.clamp(0.0, 1.0));
    }

    /// As linhas da marca (`SetValue`) e a cor secundária (`SelectOption` «r,g,b» do seletor), do canal do
    /// painel.
    pub(crate) fn route_marca_event(&mut self, event: &ph2d_editor_core::tool::PanelEvent) -> bool {
        use ph2d_editor_core::tool::PanelEvent;
        if let PanelEvent::SelectOption(id, v) = event
            && *id == crate::ids::PAINTER_MARCA_SECONDARY
        {
            let mut it = v.split(',').filter_map(|p| p.trim().parse::<u8>().ok());
            if let (Some(r), Some(g), Some(b)) = (it.next(), it.next(), it.next()) {
                self.set_marca_secundaria([r, g, b].map(|x| f32::from(x) / 255.0));
            }
            return true;
        }
        let PanelEvent::SetValue(id, v) = event else {
            return false;
        };
        let Some(i) = crate::ids::PAINTER_MARCA_FIELDS
            .iter()
            .position(|f| f == id)
        else {
            return false;
        };
        self.set_marca(LINHAS_DA_MARCA[i], *v);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// O valor de fábrica de cada linha é o do `DabJitter::NEUTRO` (o painel e o pincel novo concordam).
    #[test]
    fn a_marca_de_fabrica_e_o_neutro() {
        let b = BrushSpec::default();
        assert_eq!(LINHAS_DA_MARCA.map(|l| le(&b, l)), MARCA_DE_FABRICA);
        let mut c = b;
        for l in LINHAS_DA_MARCA {
            escreve(&mut c, l, l.omissao());
        }
        assert_eq!(c.dab_jitter, DabJitter::NEUTRO);
        assert_eq!(c.cor_mao, CorDaMao::NEUTRA);
    }

    /// Os ids são os das linhas, pela ordem e pela chave; ler e voltar a escrever é a identidade.
    #[test]
    fn cada_linha_da_marca_tem_o_seu_id_e_volta() {
        for (l, id) in LINHAS_DA_MARCA
            .into_iter()
            .zip(crate::ids::PAINTER_MARCA_FIELDS)
        {
            let pela_chave = ph2d_tool_registry::hash_node_id_runtime(&format!(
                "painter_brush.marca.{}",
                l.chave()
            ));
            assert_eq!(id, pela_chave, "{}", l.chave());
        }
        let mut b = BrushSpec::default();
        for (i, l) in LINHAS_DA_MARCA.into_iter().enumerate() {
            let (lo, hi, _) = l.faixa();
            escreve(&mut b, l, lo + (hi - lo) * (i as f64 + 1.0) / 11.0);
        }
        let b0 = b;
        for l in LINHAS_DA_MARCA {
            let v = le(&b0, l);
            escreve(&mut b, l, v);
        }
        assert_eq!(b, b0);
    }

    /// ⭐ **O painel chega ao TRAÇO**: um `SetValue` da linha Opacity pelo canal do painel muda o pincel com
    /// que o tool pinta (o leitor DECIDE — `dab_jitter` é o que o `Stroke` lê).
    #[test]
    fn a_linha_do_painel_chega_ao_pincel() {
        use ph2d_editor_core::Tool;
        let mut t = super::super::super::PainterTool::default();
        let i = LINHAS_DA_MARCA
            .iter()
            .position(|l| *l == LinhaDaMarca::Across)
            .unwrap();
        t.handle_panel_event(ph2d_editor_core::tool::PanelEvent::SetValue(
            crate::ids::PAINTER_MARCA_FIELDS[i],
            1.25,
        ));
        assert_eq!(t.paint.brush.dab_jitter.across, 1.25);
        assert_eq!(t.marca()[i], 1.25);
        // A cor secundária vem do seletor como «r,g,b».
        t.handle_panel_event(ph2d_editor_core::tool::PanelEvent::SelectOption(
            crate::ids::PAINTER_MARCA_SECONDARY,
            "255,0,51".into(),
        ));
        assert_eq!(t.paint.brush.cor_mao.secondary, [1.0, 0.0, 0.2]);
        assert_eq!(t.brush_settings().marca_secundaria, [1.0, 0.0, 0.2]);
    }
}

#[cfg(test)]
mod pintura {
    use super::*;
    use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

    /// Um recto horizontal na tela branca 200×120 no meio `m`, com a caneta inclinada `tilt`; a tela.
    fn traco(
        m: crate::PaintMedia,
        tilt: [f32; 2],
        f: impl Fn(&mut super::super::super::PainterTool),
    ) -> Vec<u8> {
        let mut t = super::super::super::PainterTool::default();
        t.set_source(vec![255u8; 200 * 120 * 4], 200, 120);
        t.set_paint_media(m);
        f(&mut t);
        let cp = |x: f32, phase| CanvasPointer {
            pos: [x, 60.0],
            pressure: 1.0,
            tilt,
            phase,
        };
        t.on_canvas_pointer(cp(20.0, PointerPhase::Down));
        for k in 1..=30 {
            t.on_canvas_pointer(cp(20.0 + 5.0 * k as f32, PointerPhase::Move));
            t.paint_tick(1.0 / 60.0);
        }
        t.on_canvas_pointer(cp(170.0, PointerPhase::Up));
        for _ in 0..5 {
            t.paint_tick(1.0 / 60.0);
        }
        t.canvas_rgba.to_vec()
    }

    /// As linhas com tinta na coluna do meio (`x = 100`): a altura da faixa.
    fn altura(px: &[u8]) -> usize {
        (0..120).filter(|y| px[(y * 200 + 100) * 4] < 250).count()
    }

    /// ⭐ **A forma da marca CHEGA à tinta** (L4) nos meios que leem a pegada: a redondeza sorteada achata as
    /// marcas no Digital e no Impasto. Vermelho antes da cura (medido 20/10: 0 bytes) — o carimbo lia a pegada
    /// do pincel e não a do dab (`BrushSpec::dab_tilt`).
    #[test]
    fn a_forma_da_marca_chega_a_tinta() {
        for m in [crate::PaintMedia::Digital, crate::PaintMedia::Impasto] {
            let a = traco(m, [0.0, 0.0], |_| {});
            let b = traco(m, [0.0, 0.0], |t| t.set_marca(LinhaDaMarca::Roundness, 0.8));
            let n = a.iter().zip(&b).filter(|(x, y)| x != y).count();
            assert!(n > 1000, "{m:?}: a redondeza mudou só {n} bytes");
        }
    }

    /// ⭐ **A caneta deitada pinta a ELIPSE no Digital** (W6/L2, curado com a L4): deitada ao longo do traço com
    /// a largura 0,3, a faixa fica MAIS FINA que a de pé. Antes da cura o carimbo pintava o disco do raio que
    /// cresceu (o eixo maior): a faixa saía mais LARGA.
    #[test]
    fn a_caneta_deitada_pinta_a_elipse_no_digital() {
        let deitada = |t: &mut super::super::super::PainterTool| {
            t.paint.brush.pen_tilt.width = 0.3;
            t.paint.brush.pen_tilt.length = 2.0;
        };
        let de_pe = altura(&traco(crate::PaintMedia::Digital, [0.0, 0.0], deitada));
        let deit = altura(&traco(crate::PaintMedia::Digital, [0.6, 0.0], deitada));
        assert!(de_pe > 4, "o traço de pé pinta: {de_pe}");
        assert!(deit < de_pe, "deitada {deit} px × de pé {de_pe} px");
    }
}
