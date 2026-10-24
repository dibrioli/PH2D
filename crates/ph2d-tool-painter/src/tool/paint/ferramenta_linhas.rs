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
}

use FerramentaDoPincel as F;
use LinhaDaFerramenta as L;

/// Sem cor: as ferramentas que não depositam pigmento.
const SEM_COR: [L; 4] = [L::Blend, L::Cor, L::Randomize, L::Composite];

/// ⭐ A tabela.
pub const NAO_LE: [(FerramentaDoPincel, &[LinhaDaFerramenta]); 8] = [
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
        ],
    ),
    (F::Cura, &SEM_COR),
    (F::Escultura, &SEM_COR),
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
}
