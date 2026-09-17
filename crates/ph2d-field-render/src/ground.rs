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
//! # As três leis, e onde cada uma vive
//!
//! 1. **ONDE o chão está** — [`lowest_point`]: à altura do ponto mais baixo da cena, achado
//!    olhando-a DE BAIXO com o próprio traçador. ⚠️ Quem o **fixa** é a app: a altura é lida uma vez
//!    e fica, para que levantar uma peça a afaste da sombra (é isso que a régua da `W4` pede).
//! 2. **ONDE o raio o toca** — [`Ground::hit`]: um plano horizontal, visto só DE CIMA.
//! 3. **QUANTO ele escurece** — a razão entre a luz que um chão BRANCO recebe com a peça e sem ela
//!    (o `catcher` do [`crate::shade_render`]), com o material de [`catcher_surface`]. Longe da peça
//!    a razão é exactamente `1`, e o fundo sai com os bytes de sempre.
//!
//! ⚠️ **O chão não tem geometria no campo.** Ele não entra na marcha da câmera (uma peça abaixo dele
//! continua a ver-se inteira) nem na de sombra (ele não tapa nada): ele só existe onde um raio de
//! câmera **falha** a peça. *Um chão que tapasse coisas deixava de ser invisível.*

use crate::{Lens, Orbit, trace_with};
use ph2d_field::FieldDoc;
use ph2d_field_eval::hybrid::Registry;

/// A normal do chão, no MUNDO.
pub const UP: [f32; 3] = [0.0, 1.0, 0.0];

/// Os pesos da luminância (Rec. 709) com que a razão do chão se mede — os da
/// `ph2d_color::linear::LinearRgba::luminance`. ⚠️ **Uma razão ESCALAR, de propósito**: uma sombra
/// tingida pela cor de cada luz mudaria de matiz sobre um fundo colorido, e o que o chão entrega é
/// só *quanto* escurece.
pub const LUMA: [f32; 3] = [0.2126, 0.7152, 0.0722];

/// ⭐ **Um plano horizontal no MUNDO**, `y = height`, que só existe para receber luz.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ground {
    pub height: f32,
}

impl Ground {
    /// ⭐⭐ **Onde o raio `(o, d)` toca o chão**, vindo de CIMA — `None` quando não toca.
    ///
    /// ⚠️ **Só de CIMA — e são precisas duas condições, não três.** Um raio que não desce nunca
    /// alcança o plano, e um olho ABAIXO dele (ou nele) que desça dá `t ≤ 0`: a cerca do `t` já o
    /// recusa. ⛔ Uma terceira, `o.y > altura`, foi escrita e **retirada por mutação**: apagá-la não
    /// mudou resposta nenhuma em gate nenhum, porque ela é implicada pelas outras duas. *Uma cerca
    /// que não muda nenhuma resposta é ruído no código.*
    ///
    /// ⚠️ O `y` do ponto é escrito como a **altura**, e não como `o.y + d.y·t`: é o mesmo número a
    /// menos de arredondamento, e é ele que o raio de sombra usa para partir — um ponto meio ULP
    /// abaixo do plano seria um ponto do lado errado dele.
    #[must_use]
    pub fn hit(self, o: [f32; 3], d: [f32; 3]) -> Option<[f32; 3]> {
        // ⚠️ O `is_finite` primeiro: um `NaN` na direcção tem de dar «não toca», e um `d >= 0.0`
        // sozinho responderia `false` sobre ele.
        if !d[1].is_finite() || d[1] >= 0.0 {
            return None;
        }
        let t = (self.height - o[1]) / d[1];
        if !t.is_finite() || t <= 0.0 {
            return None;
        }
        Some([o[0] + d[0] * t, self.height, o[2] + d[2] * t])
    }
}

/// ⭐⭐ **O material com que o chão MEDE a luz** — uma difusa branca, sem especular.
///
/// ⚠️ **Ele não é a cor do chão** (o chão não tem cor: é invisível). É a régua da razão
/// *«luz com a peça / luz sem a peça»*, e ela é escrita com a **mesma** lei do material que pinta a
/// peça — a irradiância do céu e o `N·L` de cada lâmpada saem das mesmas funções, nas mesmas
/// unidades. ⛔ Uma difusa escrita à mão ao lado da `ph2d-material` seria a segunda resposta à
/// pergunta *«quanta luz chega aqui?»*.
///
/// ⚠️ **Sem especular de propósito:** um reflexo depende da direcção de vista, e a sombra de um
/// chão não pode mudar quando a câmera roda.
#[must_use]
pub fn catcher_surface() -> ph2d_material::Surface {
    ph2d_material::OpenPbr {
        base_color: [1.0; 3],
        specular_weight: 0.0,
        ..ph2d_material::OpenPbr::default()
    }
    .prepare()
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

/// ⭐⭐ **O ponto do chão que cada pixel de FUNDO vê** — `None` num pixel de peça, num que não vê o
/// chão, e em todos quando não há chão.
///
/// ⚠️ **O raio é o do pixel**, reconstruído pela MESMA porta que o traçado e o pintor usam
/// ([`crate::Rays::at_plane`] no centro do pixel): um chão lido noutro raio poria a sombra meio pixel
/// ao lado de onde a peça a deita.
#[must_use]
pub fn ground_points(
    cam: &Orbit,
    g: &crate::Gbuffer,
    ground: Option<Ground>,
) -> Vec<Option<[f32; 3]>> {
    let Some(chao) = ground else {
        return Vec::new();
    };
    let screen = crate::Screen::new(g.width, g.height, cam.half_extent);
    let rays = cam.rays();
    let w = g.width as usize;
    (0..g.hit.len())
        .map(|i| {
            (!g.hit[i])
                .then(|| ground_at(&rays, screen, chao, i % w, i / w))
                .flatten()
        })
        .collect()
}

/// O ponto do chão sob o pixel `(x, y)` — a lei de um pixel só, que o pintor lê.
#[must_use]
pub fn ground_at(
    rays: &crate::Rays,
    screen: crate::Screen,
    ground: Ground,
    x: usize,
    y: usize,
) -> Option<[f32; 3]> {
    #[allow(clippy::cast_precision_loss)]
    let (u, v) = screen.plane_at(x as f32 + 0.5, y as f32 + 0.5);
    let (o, d) = rays.at_plane(u, v);
    ground.hit(o, d)
}

/// Quantas alturas a [`ground_sky`] pergunta ao campo por ponto do chão.
pub const GROUND_SKY_SAMPLES: u32 = 6;
/// `α` — até que distância HORIZONTAL uma amostra à altura `h` sente a peça: `α·h`.
pub const GROUND_SKY_SPREAD: f32 = 1.5;
/// `γ` — o peso de cada amostra é `γ^(k−1)`: as de baixo, junto do contacto, mandam mais.
pub const GROUND_SKY_FALLOFF: f32 = 0.8;
/// `κ` — quanto escurece a soma normalizada.
pub const GROUND_SKY_STRENGTH: f32 = 0.80;

/// ⭐⭐⭐ **QUANTO DO CÉU CHEGA A CADA PONTO DO CHÃO** — `1,0` nos pixels que não são chão.
///
/// # A lei
///
/// ```text
/// céu(q) = clamp(1 − κ · Σₖ γ^(k−1) · clamp(1 − d(q + ŷ·hₖ) / (α·hₖ), 0, 1) / Σₖ γ^(k−1), 0, 1)
/// hₖ = alcance · k / N,   k = 1..N,   alcance = OCCLUSION_REACH · half_extent
/// ```
///
/// A oclusão por CAMPO DE DISTÂNCIA, amostrada na vertical: um ponto do chão com a peça por cima ou
/// ao lado tem o campo pequeno logo acima dele.
///
/// # ⛔⛔ Porque NÃO são os cones da peça
///
/// Os `48` cones fixos ([`crate::cone_dir`]) num recetor PLANO desenham **anéis**: cada direcção
/// entra e sai da peça de repente conforme o ponto se afasta, e o chão mostra as `24` direcções de
/// cima como `24` sombras fracas sobrepostas (medido: `17` extremos numa linha do chão, contra `1`
/// da referência de `2 048` cones). Na peça isto não se vê porque a superfície é curva e o material
/// o mistura; num chão liso cinzento vê-se tudo. ⇒ **esta lei é contínua por construção** (o campo é
/// contínuo, não há direcções), não depende da câmera, e custa `N` avaliações por ponto.
///
/// # ⭐ As quatro constantes saíram da REFERÊNCIA, não do olho
///
/// Ajustadas contra a oclusão por **`2 048` cones** (os mesmos cones da peça, convergidos — já sem
/// anéis — e sem a cerca da bola, que num cone é outra descontinuidade), numa esfera e num cubo
/// pousados, `214 308` pixels de chão (`98 805` com a referência abaixo de `0,98`):
///
/// | `N` | `α` | `γ` | expoente | `κ` | erro quadrático (perto) | pior |
/// |---:|---:|---:|---:|---:|---:|---:|
/// | `4` | `1,0` | `1,0` | `1` | `1,07` | `0,082` | `0,226` |
/// | `8` | `2,0` | `1,0` | `1,5` | `0,79` | `0,043` | `0,307` |
/// | **`6`** | **`1,5`** | **`0,8`** | **`1`** | **`0,80`** | **`0,045`** | **`0,245`** |
///
/// ⚠️ A linha escolhida não é a de menor erro: é a mais simples (linear) entre as melhores, e a de
/// menor pior caso do grupo. O gate `a_oclusao_do_chao_segue_os_cones_convergidos` segura-a numa
/// segunda cena, que não entrou no ajuste (uma cruz de três eixos: erro `0,028`, pior `0,114`).
///
/// # ⚠️ A cerca, e o que ela exige do campo
///
/// Uma amostra só é avaliada quando o ponto dela está a menos de `α·h` da bola da peça — fora disso
/// o termo é `0` num campo de distância exacto (a distância à peça é pelo menos a distância à bola).
/// ⇒ longe da peça nenhuma amostra corre e o céu é **exactamente** `1`, que é o que deixa o fundo com
/// os bytes de sempre. O dispositivo usa a MESMA cerca, amostra a amostra.
#[must_use]
pub fn ground_sky(
    doc: &FieldDoc,
    reg: &Registry,
    cam: &Orbit,
    pontos: &[Option<[f32; 3]>],
) -> Vec<f32> {
    use rayon::prelude::*;
    let Some(bola) = ph2d_field_eval::bounds::bounding_ball(doc, reg) else {
        return vec![1.0; pontos.len()];
    };
    let alcance = crate::OCCLUSION_REACH * cam.half_extent;
    let shape = ph2d_field_eval::hybrid::Hybrid::new(doc, reg);
    let n = GROUND_SKY_SAMPLES;
    let mut soma_w = 0.0f32;
    let mut w = 1.0f32;
    for _ in 0..n {
        soma_w += w;
        w *= GROUND_SKY_FALLOFF;
    }
    pontos
        .par_chunks(4096)
        .flat_map_iter(|lote| {
            let mut ev = shape.fork();
            // `(pixel do lote, amostra k)` de cada ponto a avaliar, na ordem do laço.
            let mut quem: Vec<(usize, u32)> = Vec::new();
            let (mut xs, mut ys, mut zs) = (Vec::new(), Vec::new(), Vec::new());
            for (j, q) in lote.iter().enumerate() {
                let Some(q) = q else { continue };
                for k in 1..=n {
                    #[allow(clippy::cast_precision_loss)]
                    let h = alcance * k as f32 / n as f32;
                    let p = [q[0], q[1] + h, q[2]];
                    if !perto_da_bola(&bola, p, GROUND_SKY_SPREAD * h) {
                        continue;
                    }
                    quem.push((j, k));
                    xs.push(p[0]);
                    ys.push(p[1]);
                    zs.push(p[2]);
                }
            }
            let d: Vec<f32> = if xs.is_empty() {
                Vec::new()
            } else {
                ev.eval(&xs, &ys, &zs)
                    .map(<[f32]>::to_vec)
                    .unwrap_or_default()
            };
            let mut termos = vec![0.0f32; lote.len()];
            // ⚠️ Os pesos andam amostra a amostra, na ordem de `k` — a mesma do dispositivo.
            let mut peso_de = vec![1.0f32; n as usize + 1];
            for k in 2..=n as usize {
                peso_de[k] = peso_de[k - 1] * GROUND_SKY_FALLOFF;
            }
            for (m, &(j, k)) in quem.iter().enumerate() {
                let Some(&dk) = d.get(m) else { break };
                #[allow(clippy::cast_precision_loss)]
                let h = alcance * k as f32 / n as f32;
                let t = (1.0 - dk / (GROUND_SKY_SPREAD * h)).clamp(0.0, 1.0);
                termos[j] += peso_de[k as usize] * t;
            }
            lote.iter()
                .zip(termos)
                .map(|(q, s)| {
                    if q.is_none() {
                        return 1.0;
                    }
                    (1.0 - GROUND_SKY_STRENGTH * (s / soma_w)).clamp(0.0, 1.0)
                })
                .collect::<Vec<f32>>()
        })
        .collect()
}

/// O ponto `p` está a menos de `alcance` da bola? — a cerca da [`ground_sky`], amostra a amostra.
fn perto_da_bola(bola: &ph2d_field_eval::bounds::Ball, p: [f32; 3], alcance: f32) -> bool {
    let d = [
        p[0] - bola.center[0],
        p[1] - bola.center[1],
        p[2] - bola.center[2],
    ];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() - bola.radius < alcance
}
