//! **As linhas da PONTA no painel** (L9 · L18 · L19 · L6, `docs/Painter/Pinceis/03_MOTORES.md` §4): a forma e o grão do
//! carimbo — a ponta de píxel (cartão Brush, só no Digital), a lei do grão (secção Grain, com um Grain armado), a 2.ª
//! ponta do dual brush (secção Shape).
//! UMA tabela — a chave (a do preset e a do id), a faixa, o valor de fábrica — e duas portas: [`escreve`] e [`le`].
//! O preset grava os campos inteiros no `BrushSpec` (`pixel`, `texture.lei`), não estas chaves.

use ph2d_painter_brush::{
    BrushSpec, DabJitter, DualMode, DualTip, GrainMode, HOSE_MAX, HoseMode, PixelTip, TEX_SIZE_MAX, TEX_SIZE_MIN,
    TextureKind,
};

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
    /// O dual brush (L6): `0` desligado, `1 + k` o [`DualMode`] `k`.
    DualMode,
    /// A forma da 2.ª ponta ([`TextureKind`] pelo índice; `Image` = a imagem dela).
    DualTip,
    /// A 2.ª ponta segue o traço: `0` não · `1` Rake · `2` Flow (as riscas ao longo).
    DualFollow,
    /// O diâmetro da 2.ª ponta em diâmetros da 1.ª.
    DualSize,
    /// O passo da 2.ª ponta (em diâmetros dela).
    DualSpacing,
    /// Espalhar a 2.ª ponta (em diâmetros dela).
    DualScatter,
    /// Espalhar a 2.ª ponta só através do traço.
    DualAcross,
    /// Marcas da 2.ª ponta por ponto do caminho.
    DualCount,
    /// O ângulo da 2.ª ponta (graus).
    DualAngle,
    /// A rotação sorteada da 2.ª ponta.
    DualRotation,
    /// A dureza do disco da 2.ª ponta.
    DualHardness,
    /// O tamanho do padrão da forma da 2.ª ponta.
    DualPattern,
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
    /// A secção Shape, o bloco Dual brush (L6).
    Dual,
}

/// As linhas, pela ordem do painel.
pub const LINHAS_DA_PONTA: [LinhaDaPonta; 26] = [
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
    LinhaDaPonta::DualMode,
    LinhaDaPonta::DualTip,
    LinhaDaPonta::DualFollow,
    LinhaDaPonta::DualSize,
    LinhaDaPonta::DualSpacing,
    LinhaDaPonta::DualScatter,
    LinhaDaPonta::DualAcross,
    LinhaDaPonta::DualCount,
    LinhaDaPonta::DualAngle,
    LinhaDaPonta::DualRotation,
    LinhaDaPonta::DualHardness,
    LinhaDaPonta::DualPattern,
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
            Self::DualMode => ("dualMode", 0.0, DualMode::COUNT as f64, 1.0, 0.0),
            Self::DualTip => ("dualTip", 0.0, (TextureKind::COUNT - 1) as f64, 1.0, 0.0),
            Self::DualFollow => ("dualFollow", 0.0, 2.0, 1.0, 0.0),
            Self::DualSize => {
                let d = DualTip::DE_FABRICA.escala as f64;
                ("dualSize", DualTip::ESCALA_MIN as f64, DualTip::ESCALA_MAX as f64, 0.01, d)
            }
            Self::DualSpacing => {
                let d = DualTip::DE_FABRICA.passo as f64;
                ("dualSpacing", DualTip::PASSO_MIN as f64, DualTip::PASSO_MAX as f64, 0.01, d)
            }
            Self::DualScatter => ("dualScatter", 0.0, 1.0, 0.01, 0.0),
            Self::DualAcross => ("dualAcross", 0.0, DabJitter::ACROSS_MAX as f64, 0.01, 0.0),
            Self::DualCount => (
                "dualCount",
                1.0,
                ph2d_painter_brush::stroke::spray::SPRAY_COUNT_MAX as f64,
                1.0,
                1.0,
            ),
            Self::DualAngle => ("dualAngle", 0.0, 359.0, 1.0, 0.0),
            Self::DualRotation => ("dualRotation", 0.0, 1.0, 0.01, 0.0),
            Self::DualHardness => ("dualHardness", 0.0, 1.0, 0.01, DualTip::DE_FABRICA.dureza as f64),
            Self::DualPattern => ("dualPattern", TEX_SIZE_MIN as f64, TEX_SIZE_MAX as f64, 0.01, 1.0),
        }
    }

    /// Onde a linha se pinta.
    #[must_use]
    pub fn cartao(self) -> CartaoDaPonta {
        match self {
            Self::Pixel => CartaoDaPonta::Pincel,
            Self::HoseMode | Self::Peso(_) => CartaoDaPonta::Forma,
            Self::DualMode
            | Self::DualTip
            | Self::DualFollow
            | Self::DualSize
            | Self::DualSpacing
            | Self::DualScatter
            | Self::DualAcross
            | Self::DualCount
            | Self::DualAngle
            | Self::DualRotation
            | Self::DualHardness
            | Self::DualPattern => CartaoDaPonta::Dual,
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
            Self::DualMode => &[
                "panel.painter_ponta.dual_mode.off",
                "panel.painter_ponta.dual_mode.multiply",
                "panel.painter_ponta.dual_mode.darken",
                "panel.painter_ponta.dual_mode.subtract",
                "panel.painter_ponta.dual_mode.overlay",
                "panel.painter_ponta.dual_mode.color_burn",
                "panel.painter_ponta.dual_mode.height",
                "panel.painter_ponta.dual_mode.linear_height",
            ],
            Self::DualTip => FORMAS_DO_DUAL.as_slice(),
            Self::DualFollow => &[
                "panel.painter_ponta.dual_follow.off",
                "panel.painter_ponta.dual_follow.rake",
                "panel.painter_ponta.dual_follow.flow",
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

/// As formas da 2.ª ponta no menu: os nomes dos padrões, pela ordem do [`TextureKind`] (os mesmos da Shape).
static FORMAS_DO_DUAL: std::sync::LazyLock<[&'static str; TextureKind::COUNT as usize]> =
    std::sync::LazyLock::new(|| std::array::from_fn(|k| TextureKind::from_u8(k as u8).name_key()));

/// ⭐ **A lei**: a linha `l` com o valor `v` no campo do pincel que ela governa.
pub(crate) fn escreve(b: &mut BrushSpec, l: LinhaDaPonta, v: f64) {
    let v = l.limpa(v);
    if escreve_o_dual(b, l, v) {
        return;
    }
    let lei = &mut b.texture.lei;
    match l {
        LinhaDaPonta::Pixel => b.pixel = PixelTip::from_u8(v as u8),
        LinhaDaPonta::GrainMode => lei.mode = GrainMode::from_u8(v as u8),
        LinhaDaPonta::DepthMin => lei.depth_min = v as f32,
        LinhaDaPonta::DepthJitter => lei.depth_jitter = v as f32,
        LinhaDaPonta::DepthPressure => lei.depth_pressure = v >= 0.5,
        LinhaDaPonta::HoseMode => b.hose.mode = HoseMode::from_u8(v as u8),
        LinhaDaPonta::Peso(k) => b.hose.pesos[usize::from(k)] = v as u8,
        _ => {}
    }
}

/// As linhas do dual brush (L6): o modo arma ou desliga a 2.ª ponta; as outras só existem com ela armada. `false` =
/// a linha não é do dual.
fn escreve_o_dual(b: &mut BrushSpec, l: LinhaDaPonta, v: f64) -> bool {
    if l == LinhaDaPonta::DualMode {
        b.dual = (v >= 1.0).then(|| DualTip {
            modo: DualMode::from_u8(v as u8 - 1),
            ..b.dual.unwrap_or_default()
        });
        return true;
    }
    if l.cartao() != CartaoDaPonta::Dual {
        return false;
    }
    let Some(d) = &mut b.dual else {
        return true;
    };
    let f = v as f32;
    match l {
        LinhaDaPonta::DualTip => d.muda_a_forma(TextureKind::from_u8(v as u8)),
        LinhaDaPonta::DualFollow => (d.forma.rake, d.forma.flow) = (v == 1.0, v == 2.0),
        LinhaDaPonta::DualSize => d.escala = f,
        LinhaDaPonta::DualSpacing => d.passo = f,
        LinhaDaPonta::DualScatter => d.espalhar = f,
        LinhaDaPonta::DualAcross => d.atraves = f,
        LinhaDaPonta::DualCount => d.contagem = v as u32,
        LinhaDaPonta::DualAngle => d.angulo = v as u16,
        LinhaDaPonta::DualRotation => d.rotacao = f,
        LinhaDaPonta::DualHardness => d.dureza = f,
        LinhaDaPonta::DualPattern => d.forma.size = [f, f],
        _ => {}
    }
    true
}

/// O valor de uma linha do dual (desligado: a 2.ª ponta de fábrica, que as linhas mostram ao armar).
fn le_o_dual(b: &BrushSpec, l: LinhaDaPonta) -> f32 {
    let d = b.dual.unwrap_or_default();
    match l {
        LinhaDaPonta::DualMode => b.dual.map_or(0.0, |d| f32::from(d.modo.to_u8() + 1)),
        LinhaDaPonta::DualTip => f32::from(d.forma.kind.to_u8()),
        LinhaDaPonta::DualFollow => f32::from(u8::from(d.forma.rake) + 2 * u8::from(d.forma.flow)),
        LinhaDaPonta::DualSize => d.escala,
        LinhaDaPonta::DualSpacing => d.passo,
        LinhaDaPonta::DualScatter => d.espalhar,
        LinhaDaPonta::DualAcross => d.atraves,
        LinhaDaPonta::DualCount => d.contagem as f32,
        LinhaDaPonta::DualAngle => f32::from(d.angulo),
        LinhaDaPonta::DualRotation => d.rotacao,
        LinhaDaPonta::DualHardness => d.dureza,
        _ => d.forma.size[0],
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
        dual => le_o_dual(b, dual),
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
        // Sem a ponta de píxel o 1 px (`PIXEL_SIZE_MIN_PX`) não se cumpre: o raio volta ao piso da ponta com AA.
        if l == LinhaDaPonta::Pixel && !self.pixel_vivo(&self.paint.brush) {
            let r = &mut self.paint.brush.radius_px;
            *r = r.max(super::BRUSH_SIZE_MIN_PX);
        }
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
            // A imagem da 2.ª ponta (L6): o seletor de ficheiro da shell.
            PanelEvent::Click(id) if *id == crate::ids::PAINTER_DUAL_IMAGE => {
                self.request_brush_dual_image();
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
