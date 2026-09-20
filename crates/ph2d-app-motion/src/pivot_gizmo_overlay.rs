//! **O DESENHO do gizmo do pivô** — um ALVO por peça, no ponto em que ela gira.
//!
//! A geometria vive em [`super::pivot_gizmo`]; aqui mora só tinta, com o vocabulário que o gizmo
//! do colisor e o do warp já usam (as MESMAS constantes de cor e casing) — *um manipulador que o
//! artista já aprendeu não reaprende a cor*.
//!
//! ⚠️ **A forma do glifo é um ALVO e não um ponto**, e a razão é o que ele tem de dizer: um disco
//! marca *«há aqui uma coisa»*, e a pergunta aqui é *«em torno de QUE ponto»*. O anel dá o sítio e
//! a cruz dá o ponto exacto dentro dele — a mesma leitura de uma mira.
//!
//! ⚠️ **Tudo em pixels de TELA**, e o caminho é traçado com `Affine::IDENTITY`: o `stroke`
//! MULTIPLICA a espessura pelo transform (a lei do cabeçalho do `warp_overlay`), então um alvo
//! construído em mundo engrossaria com o zoom.

use super::pivot_gizmo::PivotGizmoView;
use super::warp_overlay::{CASE_PX, CASE_RGBA, HANDLE_RGBA, OUTLINE_PX};
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_vector::{Affine, BezPath, Brush, Color, Point, Stroke, VectorScene};

/// O raio do anel, em pixels de tela.
///
/// ⚠️⚠️ **O recurso aqui é a OCLUSÃO, e o número saiu de uma foto.** A 1.ª redacção usava o canto
/// do warp dobrado (`9`, com braços de `4`): medido na cena `=125`, isso dá `26 px` de alvo sobre
/// peças de `~40 px` — **65 % da peça tapada**, e o artista deixa de ver a forma que está a
/// reposicionar. A `6` (com braços de `3`) o alvo mede `18 px` e tapa `45 %`, e continua maior que
/// a alça do colisor (`9 px` de lado), que é o que se sabe legível nesta casa.
///
/// *Um gizmo que aponta um ponto e esconde o que está nele responde metade da pergunta.*
const RAIO_PX: f64 = 6.0;

/// Quanto os braços da cruz passam do anel — metade do raio, para a mira se ler como uma mira.
const BRACO_PX: f64 = 3.0;

/// **Desenha o gizmo publicado.** No-op sem pontos.
pub fn draw(
    v: &PivotGizmoView,
    camera: &Camera2d,
    center_split: ph2d_editor_core::screens::layout::CenterSplit,
    full_window: WindowSize,
    vector_scene: &mut VectorScene,
) {
    if v.pontos.is_empty() {
        return;
    }
    // ⚠️ A janela da CENA, pela porta única — ver `warp_gizmo::scene_window`.
    let to_screen =
        camera.world_to_screen_affine(super::warp_gizmo::scene_window(center_split, full_window));
    let mut alvos = BezPath::new();
    for w in &v.pontos {
        let c = to_screen * Point::new(f64::from(w[0]), f64::from(w[1]));
        // O anel, em quatro arcos — o mesmo idioma de círculo-por-curvas do contorno do colisor.
        anel(&mut alvos, c, RAIO_PX);
        // A cruz: dois traços que atravessam o anel e passam dele por `BRACO_PX`.
        let b = RAIO_PX + BRACO_PX;
        alvos.move_to(Point::new(c.x - b, c.y));
        alvos.line_to(Point::new(c.x + b, c.y));
        alvos.move_to(Point::new(c.x, c.y - b));
        alvos.line_to(Point::new(c.x, c.y + b));
    }
    // ⭐ **O CASING primeiro** — o traço escuro e mais grosso por baixo, que é o que faz o alvo
    // ler-se sobre arte clara E sobre arte escura. Sem ele, um alvo branco sobre uma forma branca
    // é invisível exactamente na cena em que ele existe para ajudar.
    vector_scene.inner_mut().stroke(
        &Stroke::new(OUTLINE_PX + CASE_PX * 2.0),
        Affine::IDENTITY,
        &Brush::Solid(Color::new(CASE_RGBA)),
        None,
        &alvos,
    );
    vector_scene.inner_mut().stroke(
        &Stroke::new(OUTLINE_PX),
        Affine::IDENTITY,
        &Brush::Solid(Color::new(HANDLE_RGBA)),
        None,
        &alvos,
    );
}

/// Um círculo em quatro cúbicas, com o comprimento de alça exacto (`KAPPA`).
fn anel(p: &mut BezPath, c: Point, r: f64) {
    const K: f64 = 0.552_284_749_830_793_4;
    let (kr, x, y) = (K * r, c.x, c.y);
    p.move_to(Point::new(x + r, y));
    p.curve_to(
        Point::new(x + r, y + kr),
        Point::new(x + kr, y + r),
        Point::new(x, y + r),
    );
    p.curve_to(
        Point::new(x - kr, y + r),
        Point::new(x - r, y + kr),
        Point::new(x - r, y),
    );
    p.curve_to(
        Point::new(x - r, y - kr),
        Point::new(x - kr, y - r),
        Point::new(x, y - r),
    );
    p.curve_to(
        Point::new(x + kr, y - r),
        Point::new(x + r, y - kr),
        Point::new(x + r, y),
    );
    p.close_path();
}
