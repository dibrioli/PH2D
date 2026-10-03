//! ⭐⭐⭐ **O CHÃO QUE SÓ RECEBE** — a metade da `W4` que esperava uma decisão (`docs/Render3d/07`).
//!
//! # A decisão, e de quem ela é
//!
//! O plano chama à `W4` *«sombras que POUSAM o objecto»*, e a régua dela pede um chão: *um objecto a
//! `0`, `1` e `10 cm` do chão tem de dar três sombras diferentes*. O modelador não tinha nenhum
//! (`docs/Render3d/05` §27.1), e a pergunta foi ao dono. **Resposta de 2026-09-16: um chão
//! INVISÍVEL** — ele não aparece; aparecem a sombra e o escurecimento de contacto que ele recebe. É o
//! *shadow catcher* do KeyShot e do Marmoset: a peça fica pousada sem um piso a ocupar a vista.
//!
//! # Onde ele está
//!
//! [`lowest_point`]: à altura do ponto mais baixo da cena, achado olhando-a DE BAIXO com o próprio
//! traçador. ⚠️ Quem o **fixa** é a app: a altura é lida uma vez e fica, para que levantar uma peça
//! a afaste da sombra. Quem o DESENHA é o desenhista de jogo do modo Render (`ph2d-mesh-forward`);
//! o chão traçado do Render antigo saiu em 03/10.

use crate::{Lens, Orbit, trace_with};
use ph2d_field::FieldDoc;
use ph2d_field_eval::hybrid::Registry;

/// ⭐ **Um plano horizontal no MUNDO**, `y = height`, que só existe para receber luz.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ground {
    pub height: f32,
}

/// ⭐ **Os pixels de lado de cada olhar de baixo.** Três olhares deste tamanho, cada um com `4` pixels
/// do anterior de largura (`16×` mais fino), dão a quina de um cubo a `~5e-5` de uma peça de `0,4`.
pub const LOWEST_SIDE: u32 = 64;

/// Quantos olhares a busca dá — ver [`LOWEST_SIDE`].
///
/// ⛔ **Dois não bastavam, e foi a quina que o disse:** o olhar amostra o CENTRO de cada pixel, e numa
/// quina aguda a altura sobe com a distância ao vértice (a `√2` por unidade num cubo de pé). Com dois
/// olhares o ponto achado ficava `2,9e-4` ACIMA da quina (gate `o_ponto_mais_baixo_de_um_cubo_de_pe_na_quina`).
pub const LOWEST_LOOKS: u32 = 3;

/// ⭐⭐⭐ **A ALTURA DO PONTO MAIS BAIXO DA PEÇA** — `None` num documento sem geometria.
///
/// # ⛔ Porque não é a caixa
///
/// A caixa da [`ph2d_field_eval::bounds::bounding_ball`] é **conservadora**: numa peça rodada, numa
/// mistura suave ou sob um modificador ela desce abaixo da superfície, e um chão pousado nela deixaria
/// a peça a flutuar — com a sombra descolada, que é exactamente o defeito que esta wave existe para
/// curar.
///
/// # ⭐ Como: o traçador, a olhar DE BAIXO
///
/// Uma câmera **paralela** virada para cima, que cobre a caixa: o acerto mais baixo da imagem é o
/// ponto mais baixo da peça, à resolução do pixel. Cada olhar seguinte ([`LOWEST_LOOKS`], hoje mais
/// dois) centra-se no melhor e mede **quatro pixels** do anterior — `16×` mais fino de cada vez. ⚠️ **É a marcha do produto**, com o recorte, a fita e a
/// tolerância de acerto dele — *uma busca escrita à parte seria outra resposta a «onde está a
/// superfície?»*.
///
/// ⚠️ **O erro tem os dois sinais, e é do tamanho do olhar fino.** O acerto de uma marcha fica
/// ligeiramente **abaixo** da superfície (ela pára quando o campo desce abaixo da tolerância), e o
/// centro do pixel mais próximo de uma quina fica ligeiramente **acima** dela. Os dois são da ordem
/// do pixel do último olhar — invisíveis: o chão não se desenha, e uma quina que o atravesse um
/// décimo de milímetro continua a ver-se inteira.
#[must_use]
pub fn lowest_point(doc: &FieldDoc, reg: &Registry) -> Option<f32> {
    let bola = ph2d_field_eval::bounds::bounding_ball(doc, reg)?;
    let (lo, hi) = bola.aabb();
    let largura = (hi[0] - lo[0]).max(hi[2] - lo[2]);
    if !largura.is_finite() || largura <= 0.0 || !lo[1].is_finite() {
        return None;
    }
    // ⚠️ O alvo fica no FUNDO da caixa: a origem da paralela recua `ORTHO_START` a partir dele, logo
    // todo raio nasce abaixo da peça seja qual for a altura dela.
    let olhar = |centro: [f32; 2], meia: f32| -> Option<([f32; 2], f32)> {
        let cam = Orbit {
            half_extent: meia,
            target: [centro[0], lo[1], centro[1]],
            lens: Lens::Ortho,
            ..Orbit::from_yaw_pitch(0.0, -std::f32::consts::FRAC_PI_2)
        };
        let g = trace_with(doc, reg, &cam, LOWEST_SIDE, LOWEST_SIDE, true, false);
        g.point
            .iter()
            .zip(&g.hit)
            .filter(|(_, h)| **h)
            .map(|(p, _)| ([p[0], p[2]], p[1]))
            .min_by(|a, b| a.1.total_cmp(&b.1))
    };
    let mut meia = 0.5 * largura * 1.02;
    let (mut onde, mut baixo) = olhar([0.5 * (lo[0] + hi[0]), 0.5 * (lo[2] + hi[2])], meia)?;
    for _ in 1..LOWEST_LOOKS {
        // A janela seguinte tem QUATRO pixels deste olhar de largura, centrada no melhor.
        #[allow(clippy::cast_precision_loss)]
        let pixel = 2.0 * meia / LOWEST_SIDE as f32;
        meia = 2.0 * pixel;
        if let Some((o, y)) = olhar(onde, meia)
            && y < baixo
        {
            (onde, baixo) = (o, y);
        }
    }
    Some(baixo)
}
