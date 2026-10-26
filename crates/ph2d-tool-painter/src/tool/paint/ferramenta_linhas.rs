//! ⭐ **O que cada ferramenta do trilho NUNCA lê** — a tabela única que o painel do pincel pergunta (dono 23/10:
//! *«o inspector do painter não se adequa à tool selecionada e várias opções do pincel de pintura aparecem e não
//! são usadas em outras tools»*). Medido pelo censo `censo_ferramenta_x_linha` (o mesmo pincel partilhado, uma
//! ponta de imagem, Digital): cada linha daqui muda o traço do Pincel e NADA na ferramenta. Uma linha que só às
//! vezes é lida não entra aqui — essa esmaece com o porquê ([`super::inercia`]).

use super::BrushSettings;

/// A ferramenta do trilho que o pincel serve agora.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FerramentaDoPincel {
    Pincel,
    Borracha,
    Blur,
    /// O Smear do trilho e a Faca do Impasto (a mesma rota, `PaintMode::smears`).
    Smear,
    Clone,
    Mascara,
    /// A cura (Inpaint): o resto da aparência também sai (`fora` no painel).
    Cura,
    /// O Sculpt do Impasto: mexe no relevo, nunca na cor.
    Escultura,
    /// O conta-gotas: lê a cor da tela; o pincel por baixo é o do Pincel.
    ContaGotas,
    /// As ferramentas da água (o trilho com o Wet Paint, dono 25/10): Smear · Blend · Wet · Dry · Blow.
    SmearMolhado,
    BlendMolhado,
    Molhar,
    Secar,
    Soprar,
}

impl FerramentaDoPincel {
    /// A ferramenta da água `t` (o Pincel da água é o [`FerramentaDoPincel::Pincel`]).
    #[must_use]
    pub fn da_agua(t: crate::WetTool) -> Self {
        match t {
            crate::WetTool::Paint => F::Pincel,
            crate::WetTool::Smear => F::SmearMolhado,
            crate::WetTool::Blend => F::BlendMolhado,
            crate::WetTool::Wet => F::Molhar,
            crate::WetTool::Dry => F::Secar,
            crate::WetTool::Blow => F::Soprar,
        }
    }

    /// A ferramenta da água que esta é (`None` fora das cinco).
    #[must_use]
    pub fn ferramenta_da_agua(self) -> Option<crate::WetTool> {
        crate::WetTool::DO_TRILHO
            .into_iter()
            .find(|t| F::da_agua(*t) == self)
    }
}

/// Uma linha (ou um bloco) do painel do pincel que depende da ferramenta.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinhaDaFerramenta {
    Blend,
    Cor,
    /// A secção Randomize Color.
    Randomize,
    /// O cartão Composite Brush.
    Composite,
    /// A secção Symmetry.
    Simetria,
    /// O «Solid» do cartão Line (o traço vira forma cheia).
    Solido,
    /// «Use Texture Colors» / «Per-Layer Color» da Shape.
    CorPorCamada,
    /// O traço como CAMADA (Glaze, Wet/Burnt edges, Edge width) e a cauda que ele re-pinta (End, Tip end).
    TracoComoCamada,
    /// O Continuous Spacing (a cápsula entre dabs, que só as rotas do carimbo lêem).
    Continuo,
    /// O preview vivo do pincel (e o pino dele): sai das ferramentas que só se vêem sobre tinta que já lá
    /// está (dono 24/10: *«vamos retirar o preview para Mask, Blur e Smear»*, e depois o Clone).
    Preview,
    /// A aguada (o meio Watercolor): a secção Watercolor, o «Automatic» da Shape e o «Same as Paper» do Grain.
    /// Quem a lê é a rota óptica (`watercolor_render_active`), que a Borracha não toma: na Watercolor ela apaga
    /// byte a byte como no Digital (medido 24/10, `borracha_por_meio_tests`).
    Aguada,
    /// O que o pincel do Wet Paint POUSA e como: o volume Opacity, Pigment, Pickup, o pó do pincel, a ponta
    /// reta (Square tip), o Tooth e o Texture on Paint do papel. A borracha da água levanta tinta viva e não pousa nada
    /// (censo 24/10, riscas secas e molhadas: `|Δ| = 0`).
    DepositoMolhado,
    /// O volume Water do Wet Paint: o Pincel e a ferramenta Wet lêem-no (a água que pousam); as outras não.
    AguaDoVolume,
    /// O Flow (a taxa por dab): as ferramentas que movem tinta que já lá está (Blur, Smear/Faca, Clone) e a cura não
    /// o lêem; a Borracha, a Máscara e a Escultura sim (medido 24/10, `o_flow_a_vista_e_o_flow_que_age`). O meio
    /// também decide: [`BrushSettings::le_o_flow`].
    Flow,
    /// A silhueta do carimbo: na Shape o Falloff, a Hardness, a textura e o que dela depende; o Shape Tone; no Jitter
    /// o que só uma ponta não redonda vê (Roundness, Rotation, Angle per stroke, Flip X/Y). As ferramentas da água
    /// carimbam o redondo delas e lêem só o Taper, o Flatten e o giro (`o_carimbo_das_ferramentas_da_agua`, 25/10).
    Forma,
    /// A orientação pelo gesto: o Follow da Shape e o menu Dab rotation do cartão Pen.
    Orientacao,
    /// O Speed → Opacity do cartão Pen.
    OpacidadePelaVelocidade,
    /// O Live Water do Wet Paint (a água corre com o traço aberto).
    AguaViva,
}

use FerramentaDoPincel as F;
use LinhaDaFerramenta as L;

/// Sem cor: as ferramentas que não depositam pigmento.
const SEM_COR: [L; 4] = [L::Blend, L::Cor, L::Randomize, L::Composite];

/// A cura: sem cor e sem Flow.
const CURA: [L; 5] = [L::Blend, L::Cor, L::Randomize, L::Composite, L::Flow];

/// As ferramentas da água não pousam cor nem tinta nem lêem a forma; o Water do topo só a Wet lê, e o Blow sopra pelo
/// gesto: sem rotação, sem opacidade, a água já corre (lido pelo censo `cada_ferramenta_da_agua_so_ve_o_que_le`).
/// O Per-Layer Color sai com a [`L::Forma`] (mora na parte da Shape que a textura arma).
const DA_AGUA: [L; 11] = [
    L::Blend,
    L::Cor,
    L::Randomize,
    L::Composite,
    L::Solido,
    L::TracoComoCamada,
    L::Aguada,
    L::Preview,
    L::DepositoMolhado,
    L::Flow,
    L::Forma,
];
const DA_AGUA_SEM_AGUA: [L; 12] = [
    L::Blend,
    L::Cor,
    L::Randomize,
    L::Composite,
    L::Solido,
    L::TracoComoCamada,
    L::Aguada,
    L::Preview,
    L::DepositoMolhado,
    L::Flow,
    L::Forma,
    L::AguaDoVolume,
];
const SOPRO: [L; 15] = [
    L::Blend,
    L::Cor,
    L::Randomize,
    L::Composite,
    L::Solido,
    L::TracoComoCamada,
    L::Aguada,
    L::Preview,
    L::DepositoMolhado,
    L::Flow,
    L::Forma,
    L::AguaDoVolume,
    L::Orientacao,
    L::OpacidadePelaVelocidade,
    L::AguaViva,
];

/// ⭐ A tabela.
pub const NAO_LE: [(FerramentaDoPincel, &[LinhaDaFerramenta]); 14] = [
    (F::Pincel, &[]),
    (
        F::Borracha,
        &[
            L::Blend,
            L::Cor,
            L::Randomize,
            L::Composite,
            L::Solido,
            L::TracoComoCamada,
            L::Aguada,
            L::DepositoMolhado,
            L::AguaDoVolume,
        ],
    ),
    (
        F::Blur,
        &[
            L::Blend,
            L::Cor,
            L::Randomize,
            L::Composite,
            L::Solido,
            L::CorPorCamada,
            L::TracoComoCamada,
            L::Continuo,
            L::Preview,
            L::Flow,
        ],
    ),
    (
        F::Smear,
        &[
            L::Blend,
            L::Cor,
            L::Randomize,
            L::Composite,
            L::Solido,
            L::CorPorCamada,
            L::TracoComoCamada,
            L::Continuo,
            L::Preview,
            L::Flow,
        ],
    ),
    (
        F::Clone,
        &[
            L::Blend,
            L::Cor,
            L::Randomize,
            L::Composite,
            L::Simetria,
            L::Solido,
            L::CorPorCamada,
            L::TracoComoCamada,
            L::Continuo,
            L::Preview,
            L::Flow,
        ],
    ),
    (
        F::Mascara,
        &[
            L::Blend,
            L::Cor,
            L::Randomize,
            L::Composite,
            L::Solido,
            L::TracoComoCamada,
            L::Preview,
        ],
    ),
    (F::Cura, &CURA),
    (F::Escultura, &SEM_COR),
    // Dono 24/10: «sem preview no painel» — a amostra vive ao lado do cursor.
    (F::ContaGotas, &[L::Preview]),
    (F::SmearMolhado, &DA_AGUA_SEM_AGUA),
    (F::BlendMolhado, &DA_AGUA_SEM_AGUA),
    (F::Molhar, &DA_AGUA),
    (F::Secar, &DA_AGUA_SEM_AGUA),
    (F::Soprar, &SOPRO),
];

impl BrushSettings {
    /// A ferramenta do trilho, pelo instantâneo.
    #[must_use]
    pub fn ferramenta(&self) -> FerramentaDoPincel {
        if self.is_sculpt {
            F::Escultura
        } else if self.is_inpaint {
            F::Cura
        } else if self.is_mask {
            F::Mascara
        } else if self.is_clone {
            F::Clone
        } else if self.is_blur {
            F::Blur
        } else if self.is_smear {
            F::Smear
        } else if self.eraser {
            F::Borracha
        } else if self.is_eyedropper {
            F::ContaGotas
        } else if self.wetpaint {
            F::da_agua(self.wet_tool)
        } else {
            F::Pincel
        }
    }

    /// **A ferramenta de agora lê esta linha?** — a pergunta única do painel.
    #[must_use]
    pub fn le(&self, linha: LinhaDaFerramenta) -> bool {
        let f = self.ferramenta();
        !NAO_LE
            .iter()
            .any(|(g, nao)| *g == f && nao.contains(&linha))
    }

    /// **O Flow à vista?** A ferramenta lê-o ([`LinhaDaFerramenta::Flow`]) e o meio do traço também: a aguada
    /// (`watercolor_active`) e o pincel do Wet Paint (o que pousa por volumes, [`LinhaDaFerramenta::DepositoMolhado`])
    /// não o lêem; a Borracha nesses meios lê (medido 24/10, `o_flow_a_vista_e_o_flow_que_age`).
    #[must_use]
    pub fn le_o_flow(&self) -> bool {
        self.le(L::Flow)
            && !self.watercolor_active
            && !(self.wetpaint && self.le(L::DepositoMolhado))
    }
}
