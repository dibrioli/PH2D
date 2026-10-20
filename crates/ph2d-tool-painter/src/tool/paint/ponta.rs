//! **As linhas da PONTA no painel** (L9 · L18 · L19, `docs/Painter/Pinceis/03_MOTORES.md` §4): a forma e o grão do
//! carimbo — a ponta de píxel (cartão Brush, só no Digital), a lei do grão (secção Grain, com um Grain armado).
//! UMA tabela — a chave (a do preset e a do id), a faixa, o valor de fábrica — e duas portas: [`escreve`] e [`le`].
//! O preset grava os campos inteiros no `BrushSpec` (`pixel`, `texture.lei`), não estas chaves.

use ph2d_painter_brush::{BrushSpec, GrainMode, HOSE_MAX, HoseMode, PixelTip};

/// Uma linha numérica da ponta.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinhaDaPonta {
    /// A ponta de píxel (L18): `0` desligada · `1` alinhada · `2` pixel-perfect.
    Pixel,
    /// O modo do grão (L9, [`GrainMode`] pelo índice).
    GrainMode,
    /// A profundidade mínima do grão por dab (L9).
    DepthMin,
    /// O jitter da profundidade do grão por dab (L9).
    DepthJitter,
    /// O controlo da profundidade por dab: `0` sorteada · `1` a pressão (L9).
    DepthPressure,
    /// Como cada dab escolhe a imagem da ponta de N imagens (L19, [`HoseMode`] pelo índice).
    HoseMode,
    /// O peso da imagem `k` no sorteio (L19).
    Peso(u8),
}

/// Onde a linha se pinta.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CartaoDaPonta {
    /// A secção Brush (a ponta de píxel; só no Digital).
    Pincel,
    /// A secção Grain (com um Grain armado).
    Grao,
    /// A secção Shape, com duas imagens ou mais (L19).
    Forma,
}

/// As linhas, pela ordem do painel.
pub const LINHAS_DA_PONTA: [LinhaDaPonta; 14] = [
    LinhaDaPonta::Pixel,
    LinhaDaPonta::GrainMode,
    LinhaDaPonta::DepthMin,
    LinhaDaPonta::DepthJitter,
    LinhaDaPonta::DepthPressure,
    LinhaDaPonta::HoseMode,
    LinhaDaPonta::Peso(0),
    LinhaDaPonta::Peso(1),
    LinhaDaPonta::Peso(2),
    LinhaDaPonta::Peso(3),
    LinhaDaPonta::Peso(4),
    LinhaDaPonta::Peso(5),
    LinhaDaPonta::Peso(6),
    LinhaDaPonta::Peso(7),
];
const _: () = assert!(HOSE_MAX == 8, "uma linha de peso por imagem da ponta");

/// As chaves das linhas de peso (uma por imagem da ponta).
const PESOS: [&str; HOSE_MAX] = [
    "hoseWeight1",
    "hoseWeight2",
    "hoseWeight3",
    "hoseWeight4",
    "hoseWeight5",
    "hoseWeight6",
    "hoseWeight7",
    "hoseWeight8",
];

/// Quantas linhas a ponta tem.
pub const N_LINHAS_DA_PONTA: usize = LINHAS_DA_PONTA.len();

/// O valor de fábrica de cada linha (o neutro: o carimbo de sempre).
pub const PONTA_DE_FABRICA: [f64; N_LINHAS_DA_PONTA] = {
    let mut v = [0.0; N_LINHAS_DA_PONTA];
    let mut i = 0;
    while i < N_LINHAS_DA_PONTA {
        v[i] = LINHAS_DA_PONTA[i].tabela().4;
        i += 1;
    }
    v
};

impl LinhaDaPonta {
    /// `(chave, min, max, passo, fábrica)`.
    const fn tabela(self) -> (&'static str, f64, f64, f64, f64) {
        match self {
            Self::Pixel => ("pixel", 0.0, (PixelTip::COUNT - 1) as f64, 1.0, 0.0),
            Self::GrainMode => ("grainMode", 0.0, (GrainMode::COUNT - 1) as f64, 1.0, 0.0),
            Self::DepthMin => ("depthMin", 0.0, 1.0, 0.01, 0.0),
            Self::DepthJitter => ("depthJitter", 0.0, 1.0, 0.01, 0.0),
            Self::DepthPressure => ("depthPressure", 0.0, 1.0, 1.0, 0.0),
            Self::HoseMode => ("hoseMode", 0.0, (HoseMode::COUNT - 1) as f64, 1.0, 0.0),
            Self::Peso(k) => (PESOS[k as usize], 0.0, 100.0, 1.0, 1.0),
        }
    }

    /// Onde a linha se pinta.
    #[must_use]
    pub fn cartao(self) -> CartaoDaPonta {
        match self {
            Self::Pixel => CartaoDaPonta::Pincel,
            Self::HoseMode | Self::Peso(_) => CartaoDaPonta::Forma,
            _ => CartaoDaPonta::Grao,
        }
    }

    /// A chave da linha (o id do painel é `painter_brush.ponta.<chave>`).
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

    /// ⭐ As opções da linha quando ela é um MENU (os modos), pelas chaves i18n; vazio = um campo numérico.
    #[must_use]
    pub fn opcoes(self) -> &'static [&'static str] {
        match self {
            Self::Pixel => &[
                "panel.painter_ponta.pixel.off",
                "panel.painter_ponta.pixel.aligned",
                "panel.painter_ponta.pixel.perfect",
            ],
            Self::GrainMode => &[
                "panel.painter_ponta.grain_mode.multiply",
                "panel.painter_ponta.grain_mode.subtract",
                "panel.painter_ponta.grain_mode.darken",
                "panel.painter_ponta.grain_mode.color_burn",
                "panel.painter_ponta.grain_mode.height",
                "panel.painter_ponta.grain_mode.linear_height",
            ],
            Self::DepthPressure => &[
                "panel.painter_ponta.depth_control.random",
                "panel.painter_ponta.depth_control.pressure",
            ],
            Self::HoseMode => &[
                "panel.painter_ponta.hose_mode.random",
                "panel.painter_ponta.hose_mode.sequence",
                "panel.painter_ponta.hose_mode.pressure",
                "panel.painter_ponta.hose_mode.angle",
            ],
            _ => &[],
        }
    }

    /// As casas decimais que o campo mostra (os modos são inteiros).
    #[must_use]
    pub fn casas(self) -> usize {
        if self.faixa().2 >= 1.0 { 0 } else { 2 }
    }

    /// O valor de fábrica (o neutro).
    #[must_use]
    pub fn omissao(self) -> f64 {
        self.tabela().4
    }

    /// `v` dentro da faixa e no passo dos inteiros; NaN = o valor de fábrica.
    #[must_use]
    pub fn limpa(self, v: f64) -> f64 {
        let (lo, hi, passo) = self.faixa();
        if v.is_nan() {
            return self.omissao();
        }
        let v = v.clamp(lo, hi);
        if passo >= 1.0 { v.round() } else { v }
    }
}

/// ⭐ **A lei**: a linha `l` com o valor `v` no campo do pincel que ela governa.
pub(crate) fn escreve(b: &mut BrushSpec, l: LinhaDaPonta, v: f64) {
    let v = l.limpa(v);
    let lei = &mut b.texture.lei;
    match l {
        LinhaDaPonta::Pixel => b.pixel = PixelTip::from_u8(v as u8),
        LinhaDaPonta::GrainMode => lei.mode = GrainMode::from_u8(v as u8),
        LinhaDaPonta::DepthMin => lei.depth_min = v as f32,
        LinhaDaPonta::DepthJitter => lei.depth_jitter = v as f32,
        LinhaDaPonta::DepthPressure => lei.depth_pressure = v >= 0.5,
        LinhaDaPonta::HoseMode => b.hose.mode = HoseMode::from_u8(v as u8),
        LinhaDaPonta::Peso(k) => b.hose.pesos[usize::from(k)] = v as u8,
    }
}

/// O valor da linha `l` lido do pincel (o inverso de [`escreve`]), pelo menor texto que volta ao mesmo `f32`.
#[must_use]
pub(crate) fn le(b: &BrushSpec, l: LinhaDaPonta) -> f64 {
    let lei = &b.texture.lei;
    let v: f32 = match l {
        LinhaDaPonta::Pixel => f32::from(b.pixel.to_u8()),
        LinhaDaPonta::GrainMode => f32::from(lei.mode.to_u8()),
        LinhaDaPonta::DepthMin => lei.depth_min,
        LinhaDaPonta::DepthJitter => lei.depth_jitter,
        LinhaDaPonta::DepthPressure => f32::from(u8::from(lei.depth_pressure)),
        LinhaDaPonta::HoseMode => f32::from(b.hose.mode.to_u8()),
        LinhaDaPonta::Peso(k) => f32::from(b.hose.pesos[usize::from(k)]),
    };
    format!("{v}").parse().unwrap_or(f64::from(v))
}

impl super::PainterTool {
    /// O valor de cada linha da ponta (o que o painel mostra).
    #[must_use]
    pub fn ponta(&self) -> [f64; N_LINHAS_DA_PONTA] {
        LINHAS_DA_PONTA.map(|l| le(&self.paint.brush, l))
    }

    /// **Uma linha da ponta** no pincel de agora.
    pub fn set_ponta(&mut self, l: LinhaDaPonta, v: f64) {
        escreve(&mut self.paint.brush, l, v);
    }

    /// As linhas da ponta do canal do painel: `SetValue` (os campos) e `SelectOption` «i» (os menus).
    pub(crate) fn route_ponta_event(&mut self, event: &ph2d_editor_core::tool::PanelEvent) -> bool {
        use ph2d_editor_core::tool::PanelEvent;
        // Os botões da ponta de N imagens (L19): «Add image» pede o seletor à shell, «Clear images» tira as outras.
        match event {
            PanelEvent::Click(id) if *id == crate::ids::PAINTER_SHAPE_HOSE_ADD => {
                self.request_brush_shape_hose_image();
                return true;
            }
            PanelEvent::Click(id) if *id == crate::ids::PAINTER_SHAPE_HOSE_CLEAR => {
                self.clear_brush_shape_hose();
                return true;
            }
            _ => {}
        }
        let (id, v) = match event {
            PanelEvent::SetValue(id, v) => (id, *v),
            PanelEvent::SelectOption(id, s) => match s.parse::<f64>() {
                Ok(v) => (id, v),
                Err(_) => return false,
            },
            _ => return false,
        };
        let Some(i) = crate::ids::PAINTER_PONTA_FIELDS.iter().position(|f| f == id) else {
            return false;
        };
        self.set_ponta(LINHAS_DA_PONTA[i], v);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// O valor de fábrica de cada linha é o neutro do pincel (o painel e o pincel novo concordam), e ler/escrever
    /// é a identidade.
    #[test]
    fn a_ponta_de_fabrica_e_o_neutro_e_volta() {
        let b = BrushSpec::default();
        assert_eq!(LINHAS_DA_PONTA.map(|l| le(&b, l)), PONTA_DE_FABRICA);
        let mut c = b;
        for (i, l) in LINHAS_DA_PONTA.into_iter().enumerate() {
            let (lo, hi, _) = l.faixa();
            escreve(&mut c, l, lo + (hi - lo) * (i as f64 + 1.0) / 7.0);
        }
        let c0 = c;
        for l in LINHAS_DA_PONTA {
            let v = le(&c0, l);
            escreve(&mut c, l, v);
        }
        assert_eq!(c, c0);
        for (l, id) in LINHAS_DA_PONTA.into_iter().zip(crate::ids::PAINTER_PONTA_FIELDS) {
            let pela_chave = ph2d_tool_registry::hash_node_id_runtime(&format!(
                "painter_brush.ponta.{}",
                l.chave()
            ));
            assert_eq!(id, pela_chave, "{}", l.chave());
        }
    }
}
