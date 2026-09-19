//! **A TINTA do gizmo de uma corrente de posições** — a geometria vive em [`super::ponto_gizmo`];
//! aqui mora só o desenho, com o **vocabulário que o artista já aprendeu** (as constantes do
//! `warp_overlay`: a mesma cor, o mesmo casing escuro, a mesma espessura). *Um manipulador que ele
//! já leu noutro sítio não se reaprende.*
//!
//! ## As três feições
//!
//! | feição | o que se vê |
//! |---|---|
//! | **Osso** | a silhueta de armadura: larga na junta que manda, afilada para a ponta, com um anel na junta |
//! | **Corda** | a polilinha dos consecutivos, com uma marca em cada nó |
//! | **Ponto** | uma cruz pequena em cada posição |
//!
//! ⚠️ **O osso é um LOSANGO afilado e não uma linha**, e é isso que faz uma cadeia ler-se como um
//! esqueleto em vez de um arame: a direcção é visível sem se seguir a ordem dos pontos. É a forma
//! que o pincel de pose da escultura já desenha (`ph2d-app-sculpt3d/src/pose_gizmo.rs`), e usá-la
//! aqui é a mesma decisão — *o artista reconhece um osso porque ele tem a forma de um osso*.
//!
//! ## ⚠️ Tudo em pixels de TELA
//!
//! O caminho é construído já em coordenadas de tela e traçado com `Affine::IDENTITY`, porque
//! `stroke` **multiplica** a espessura pelo transform (a lei do cabeçalho do `warp_overlay`). Um
//! osso desenhado em mundo ficaria fino ao afastar a câmara, que é o oposto do que um gizmo quer.

use super::ponto_gizmo::{Feicao, Grupo, PontoGizmoView};
use super::warp_overlay::{CASE_PX, CASE_RGBA, HANDLE_RGBA, OUTLINE_PX, TANGENT_RGBA};
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_vector::{Affine, BezPath, Brush, Circle, Color, Fill, Point, Shape, Stroke, VectorScene};

/// Meia-largura do osso na junta que manda, em pixels de tela.
const OSSO_PX: f64 = 4.0;
/// O raio do anel de uma junta.
const JUNTA_PX: f64 = 2.5;
/// Metade do braço da cruz de um ponto solto.
const CRUZ_PX: f64 = 2.5;
/// A tolerância com que um círculo vira curvas.
const CIRCULO_TOL_PX: f64 = 0.1;

/// ⚠️ **Um osso curto demais desenha-se como uma JUNTA e mais nada.** Sem esta cerca, o losango de
/// um segmento de comprimento ~0 fica com as duas pontas do lado errado da largura e pinta uma
/// gravata — uma cadeia com juntas coincidentes (o caso normal de uma corda em repouso) ficaria
/// coberta de borrões.
const OSSO_MIN_PX: f64 = OSSO_PX * 2.0;

/// **Desenha o gizmo publicado.** No-op sem grupos.
pub fn draw(
    v: &PontoGizmoView,
    camera: &Camera2d,
    center_split: ph2d_editor_core::screens::layout::CenterSplit,
    full_window: WindowSize,
    vector_scene: &mut VectorScene,
) {
    // ⚠️ A janela da CENA, pela porta única — ver `warp_gizmo::scene_window`.
    let to_screen =
        camera.world_to_screen_affine(super::warp_gizmo::scene_window(center_split, full_window));
    let pt = |w: [f32; 2]| to_screen * Point::new(f64::from(w[0]), f64::from(w[1]));
    let case = Brush::Solid(Color::new(CASE_RGBA));
    let brush = Brush::Solid(Color::new(HANDLE_RGBA));
    let dim = Brush::Solid(Color::new(TANGENT_RGBA));

    // ── O QUE SE PREENCHE (os ossos) e O QUE SE TRAÇA (cordas, juntas, cruzes) ──
    // Dois caminhos e não um por grupo: o Vello encoda um caminho de uma vez, e uma cena com
    // treze ilhas (a `=110`) pagaria treze codificações por feição.
    let mut cheios = BezPath::new();
    let mut tracos = BezPath::new();
    let mut houve = false;
    for g in &v.grupos {
        match g.feicao {
            Feicao::Osso => desenha_ossos(g, &pt, &mut cheios, &mut tracos),
            Feicao::Corda => desenha_corda(g, &pt, &mut tracos),
            Feicao::Ponto => desenha_pontos(g, &pt, &mut tracos),
        }
        houve = true;
    }
    if !houve {
        return;
    }
    // O casing PRIMEIRO, mais grosso — a lei do `warp_overlay`.
    for caminho in [&cheios, &tracos] {
        vector_scene.inner_mut().stroke(
            &Stroke::new(OUTLINE_PX + CASE_PX * 2.0),
            Affine::IDENTITY,
            &case,
            None,
            caminho,
        );
    }
    vector_scene
        .inner_mut()
        .fill(Fill::NonZero, Affine::IDENTITY, &dim, None, &cheios);
    for (caminho, cor) in [(&cheios, &brush), (&tracos, &brush)] {
        vector_scene.inner_mut().stroke(
            &Stroke::new(OUTLINE_PX),
            Affine::IDENTITY,
            cor,
            None,
            caminho,
        );
    }
}

/// A silhueta de cada osso mais o anel de cada junta.
fn desenha_ossos(
    g: &Grupo,
    pt: &impl Fn([f32; 2]) -> Point,
    cheios: &mut BezPath,
    tracos: &mut BezPath,
) {
    for [de, para] in &g.segmentos {
        let (a, b) = (pt(g.pontos[*de]), pt(g.pontos[*para]));
        let (dx, dy) = (b.x - a.x, b.y - a.y);
        let comp = dx.hypot(dy);
        if comp < OSSO_MIN_PX {
            continue; // ver `OSSO_MIN_PX`
        }
        // A normal unitária, para abrir o losango na junta que manda.
        let (nx, ny) = (-dy / comp * OSSO_PX, dx / comp * OSSO_PX);
        // O ombro fica a um quinto do caminho: é onde a armadura do referencial o põe, e é o
        // que dá a direcção sem engordar a cadeia inteira.
        let ombro = Point::new(a.x + dx * 0.2, a.y + dy * 0.2);
        cheios.move_to(a);
        cheios.line_to(Point::new(ombro.x + nx, ombro.y + ny));
        cheios.line_to(b);
        cheios.line_to(Point::new(ombro.x - nx, ombro.y - ny));
        cheios.close_path();
    }
    for p in &g.pontos {
        anel(tracos, pt(*p), JUNTA_PX);
    }
}

/// A polilinha dos consecutivos, com uma marca em cada nó.
fn desenha_corda(g: &Grupo, pt: &impl Fn([f32; 2]) -> Point, tracos: &mut BezPath) {
    let mut aberto = false;
    for [de, para] in &g.segmentos {
        if !aberto {
            tracos.move_to(pt(g.pontos[*de]));
            aberto = true;
        }
        tracos.line_to(pt(g.pontos[*para]));
    }
    for p in &g.pontos {
        anel(tracos, pt(*p), JUNTA_PX * 0.7);
    }
}

/// Uma cruz em cada posição. ⚠️ **Uma cruz e não um ponto cheio**: um disco pequeno some sobre
/// fundo claro e vira um borrão quando são milhares; uma cruz tem duas direcções e lê-se nos dois.
fn desenha_pontos(g: &Grupo, pt: &impl Fn([f32; 2]) -> Point, tracos: &mut BezPath) {
    for p in &g.pontos {
        let c = pt(*p);
        tracos.move_to(Point::new(c.x - CRUZ_PX, c.y));
        tracos.line_to(Point::new(c.x + CRUZ_PX, c.y));
        tracos.move_to(Point::new(c.x, c.y - CRUZ_PX));
        tracos.line_to(Point::new(c.x, c.y + CRUZ_PX));
    }
}

fn anel(caminho: &mut BezPath, c: Point, r: f64) {
    caminho.extend(Circle::new(c, r).path_elements(CIRCULO_TOL_PX));
}
