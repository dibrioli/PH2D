//! ⭐⭐ **QUÃO GRANDE é o assado de um conjunto de caminhos** — sem assar nada.
//!
//! Irmão do [`crate::motion_object_bake`] e separado dele por RESPONSABILIDADE (e pelo tecto de
//! LOC): aquele produz **pixels** e precisa de uma placa; isto responde **quantos**, e é geometria
//! em CPU.
//!
//! ⭐ A separação não é cosmética — é o que torna a resposta alcançável por quem **coloca** um
//! padrão. Sem ela, saber o aspecto de um grupo parecia exigir um render + readback, e a nota que o
//! adiava dizia exactamente isso. *Uma ausência afirmada sem olhar a API é um palpite com cara de
//! medição.*

use crate::motion_object_bake::{BAKE_DPI, MAX_TILE_SIDE, bake_camera};
use ph2d_vec_render::LiveGeometry;
use ph2d_vec_scene::{VecPathId, VecScene, VecXforms};

/// A **UNIÃO** das caixas dos `ids`, na câmara do assado.
fn union_box(
    scene: &VecScene,
    xforms: &VecXforms,
    live: &LiveGeometry,
    ids: &[VecPathId],
) -> Option<(f64, f64, f64, f64)> {
    let camera = bake_camera();
    let mut caixa: Option<(f64, f64, f64, f64)> = None;
    for &id in ids {
        if let Some(b) = ph2d_vec_render::path_screen_bounds(scene, xforms, live, id, camera) {
            caixa = Some(caixa.map_or(b, |a: (f64, f64, f64, f64)| {
                (a.0.min(b.0), a.1.min(b.1), a.2.max(b.2), a.3.max(b.3))
            }));
        }
    }
    caixa
}

/// **A caixa, as dimensões em pixels e a ESCALA que faz o assado caber no tecto.**
///
/// # ⛔⛔ O tecto RECORTAVA, e a nota do módulo dizia que ele reamostrava
///
/// A redacção anterior fazia `clamp(1, MAX_TILE_SIDE)` **em cada eixo, independentemente**, e o
/// afim do assado é uma translação pura ⇒ acima do tecto o artista via **um canto da arte**, sem
/// mensagem nenhuma. E o doc do [`MAX_TILE_SIDE`] prometia o contrário (*"a coarser effective
/// DPI"*). *Uma afirmação que descreve o que se queria e não o que o código faz é a pior espécie de
/// comentário: ela impede a próxima pessoa de olhar.*
///
/// ⭐⭐ **E o recorte trazia o report de volta.** Medido na auditoria desta wave, num grupo de razão
/// geométrica `3,000`: a `4 x 12` unidades de mundo a razão medida cai para `2,000`, e a `8 x 24`
/// para **`1,000` — o quadrado do report original**, porque os dois eixos saturam no mesmo número.
/// ⚠️ E chega-se lá depressa: uma caixa `1 x 1` com traço de largura `1` já mede **5 unidades**
/// (o `miter_limit` da casa infla a caixa em `2 x width` por lado).
///
/// ⇒ a escala é **uniforme**, calculada do lado MAIOR: o aspecto sobrevive por construção, e o que
/// se perde é resolução — que é exactamente o que a nota do tecto sempre prometeu.
///
/// ⚠️ O tamanho de MUNDO do ladrilho **não muda** com a escala (ele é a caixa a dividir pelo
/// [`BAKE_DPI`]): o que encolhe é a contagem de pixels, não o desenho.
/// **O que se sabe de um assado antes de o assar.**
///
/// ⚠️ Os três são factos DIFERENTES e o clippy tinha razão a recusar o tuplo: a `caixa` é geometria
/// de mundo, os `px` são o alvo de render, e a `escala` é a concessão ao tecto. Confundi-los é
/// exactamente o defeito que esta wave curou (os pixels a decidirem o aspecto).
#[derive(Copy, Clone, Debug)]
pub(crate) struct BakeFit {
    /// A união das caixas, na câmara do assado (`x0, y0, x1, y1`).
    pub caixa: (f64, f64, f64, f64),
    /// O alvo de render, em pixels — já com a escala do tecto aplicada.
    pub px: [u32; 2],
    /// `1.0` no caminho comum; abaixo dele, quanto o tecto obrigou a reamostrar.
    pub escala: f64,
}

#[must_use]
pub(crate) fn bake_fit(
    scene: &VecScene,
    xforms: &VecXforms,
    live: &LiveGeometry,
    ids: &[VecPathId],
) -> Option<BakeFit> {
    let caixa = union_box(scene, xforms, live, ids)?;
    let (x0, y0, x1, y1) = caixa;
    let (w, h) = ((x1 - x0).max(1.0), (y1 - y0).max(1.0));
    let escala = (f64::from(MAX_TILE_SIDE) / w.max(h)).min(1.0);
    let px = |v: f64| ((v * escala).ceil() as u32).clamp(1, MAX_TILE_SIDE);
    Some(BakeFit {
        caixa,
        px: [px(w), px(h)],
        escala,
    })
}

/// ⭐⭐⭐ **QUANTOS PIXELS o assado destes `ids` vai ter** — sem assar nada.
///
/// # Porque isto é uma porta, e não uma conta repetida
///
/// O `bake_rgba_many` já calculava este número e **deitava-o fora** para quem chama. Quem coloca um
/// padrão precisa dele para saber o **aspecto** da arte — e sem ele um grupo alto nasce
/// **achatado** num `size` quadrado (report do Enio, 2026-08-30). ⛔ Uma segunda conta daria um
/// aspecto ao ladrilho e outro à colocação, e a arte sairia esticada por uma razão que nenhuma das
/// duas fórmulas mostraria sozinha.
///
/// ⭐⭐ **E não precisa de GPU.** A minha nota do dia anterior dizia que as dimensões de um grupo
/// *"exigem o assado de GPU"* — falso, e no ponto que decidia o preço: a caixa é
/// [`ph2d_vec_render::path_screen_bounds`], que é **geometria em CPU**. A placa só é precisa para os
/// PIXELS. *Uma ausência afirmada sem olhar a API é um palpite com cara de medição.*
#[must_use]
pub(crate) fn bake_dims(
    scene: &VecScene,
    xforms: &VecXforms,
    live: &LiveGeometry,
    ids: &[VecPathId],
) -> Option<[u32; 2]> {
    bake_fit(scene, xforms, live, ids).map(|f| f.px)
}

/// O tamanho de MUNDO de um assado — invariante à escala do tecto, de propósito.
#[must_use]
pub(crate) fn world_size(caixa: (f64, f64, f64, f64)) -> [f32; 2] {
    let (x0, y0, x1, y1) = caixa;
    #[allow(clippy::cast_possible_truncation)]
    [((x1 - x0) / BAKE_DPI) as f32, ((y1 - y0) / BAKE_DPI) as f32]
}
