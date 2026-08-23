//! **O DESENHO do gizmo dos deformadores de quadrilátero** — o contorno, os braços e as
//! alças que o `motion.four_point_warp` e o `motion.bezier_warp` passam a ter na tela.
//!
//! A geometria vive em [`super::warp_gizmo`], pura e testada; aqui mora só tinta.
//!
//! ## O que se desenha, e por que cada peça
//!
//! - **O CONTORNO** — as quatro arestas, avaliadas pela cúbica do próprio nó. É a peça
//!   que responde *"que forma é esta?"*, e sem ela doze pontos soltos não dizem nada.
//! - **OS BRAÇOS** — do canto até cada tangente dele. ⚠️ Sem eles, oito pontos à volta
//!   de um quadrilátero não dizem a que aresta pertencem, e o artista arrasta o errado.
//!   É a mesma razão por que o editor de curvas da casa desenha as suas alças ligadas.
//! - **AS ALÇAS** — quadrado para canto, círculo para tangente. ⚠️ **A forma distingue o
//!   TIPO**, e não só a posição: com a fronteira quase recta a tangente nasce a um terço
//!   do canto, e duas marcas iguais ali seriam indistinguíveis.
//!
//! ## ⚠️ Tudo em pixels de TELA
//!
//! O caminho é construído já em coordenadas de tela e traçado com `Affine::IDENTITY`,
//! porque `stroke` **multiplica** a espessura pelo transform: entregar o afim mundo→tela
//! como transform faria a linha engordar com o zoom. É a lei que o `anchor_overlay`
//! escreveu no cabeçalho dele, e ela vale igual aqui.

use super::warp_gizmo::{self, MAX_HANDLES, WarpHandle, WarpHandleKind};
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_vector::{Affine, BezPath, Brush, Color, Point, Stroke, VectorScene};

/// A espessura do contorno, em pixels de tela.
const OUTLINE_PX: f64 = 1.5;
/// A espessura de um braço — mais fina que o contorno: ele é ANDAIME, não a figura.
const ARM_PX: f64 = 1.0;
/// O meio-lado do quadrado de um canto, em pixels.
const CORNER_PX: f64 = 4.5;
/// O raio do círculo de uma tangente, em pixels.
const TANGENT_PX: f64 = 3.5;

/// A cor do contorno e das alças de canto.
const HANDLE_RGBA: [f32; 4] = [0.35, 0.78, 1.0, 1.0];
/// A cor dos braços e das tangentes — a mesma matiz, mais apagada: elas são o segundo
/// nível de leitura, e a hierarquia tem de estar na TINTA e não só no tamanho.
const TANGENT_RGBA: [f32; 4] = [0.35, 0.78, 1.0, 0.55];

/// **Desenha o gizmo do nó de warp seleccionado.** No-op quando não há nenhum, quando a
/// tomada ainda não trouxe a caixa, ou quando o layout é degenerado.
///
/// ⚠️ **O `active` é a tool Motion**, e ele é gateado pelo chamador como no
/// `field_gizmo`: fora dela o canvas é dos sprites, e alças de nó ali seriam alvos que
/// roubam o clique de outra ferramenta.
pub(super) fn draw_warp_gizmo(
    active: bool,
    v: &warp_gizmo::WarpGizmoView,
    param: &dyn Fn(&str) -> f32,
    camera: &Camera2d,
    center_split: ph2d_editor::screens::layout::CenterSplit,
    full_window: WindowSize,
    vector_scene: &mut VectorScene,
) {
    if !active {
        return;
    }
    // ⚠️ **A janela da CENA, resolvida AQUI e não pelo chamador.** Passar a janela cheia
    // desloca e encolhe tudo o que é desenhado em coordenadas de mundo — e, pior, faz a
    // tinta discordar do hit-test, que usa a janela certa. Ver
    // [`super::warp_gizmo::scene_window`], que carrega o relato do defeito.
    let to_screen =
        camera.world_to_screen_affine(warp_gizmo::scene_window(center_split, full_window));
    let pt = |w: [f32; 2]| to_screen * Point::new(f64::from(w[0]), f64::from(w[1]));

    // ── o CONTORNO, já no espaço do que se VÊ (a cadeia de jusante aplicada) ──
    let (ring, corners) = warp_gizmo::view_outline(v, param);
    let mut path = BezPath::new();
    for (i, w) in ring.iter().enumerate() {
        let p = pt(*w);
        if i == 0 {
            path.move_to(p);
        } else {
            path.line_to(p);
        }
    }
    let brush = Brush::Solid(Color::new(HANDLE_RGBA));
    vector_scene.inner_mut().stroke(
        &Stroke::new(OUTLINE_PX),
        Affine::IDENTITY,
        &brush,
        None,
        &path,
    );

    let (hs, n) = warp_gizmo::view_handles(v, param);
    let live: &[WarpHandle] = &hs[..n.min(MAX_HANDLES)];

    // ── os BRAÇOS, antes das alças (elas pousam por cima) ──
    let dim = Brush::Solid(Color::new(TANGENT_RGBA));
    let mut arms = BezPath::new();
    let mut any_arm = false;
    for h in live {
        if let Some(c) = warp_gizmo::tangent_arm(h.kind) {
            arms.move_to(pt(corners[c]));
            arms.line_to(pt(h.world));
            any_arm = true;
        }
    }
    if any_arm {
        vector_scene
            .inner_mut()
            .stroke(&Stroke::new(ARM_PX), Affine::IDENTITY, &dim, None, &arms);
    }

    // ── as ALÇAS ──
    for h in live {
        let c = pt(h.world);
        match h.kind {
            WarpHandleKind::Corner(_) => {
                let mut sq = BezPath::new();
                sq.move_to(Point::new(c.x - CORNER_PX, c.y - CORNER_PX));
                sq.line_to(Point::new(c.x + CORNER_PX, c.y - CORNER_PX));
                sq.line_to(Point::new(c.x + CORNER_PX, c.y + CORNER_PX));
                sq.line_to(Point::new(c.x - CORNER_PX, c.y + CORNER_PX));
                sq.close_path();
                vector_scene.inner_mut().fill(
                    ph2d_vector::Fill::NonZero,
                    Affine::IDENTITY,
                    &brush,
                    None,
                    &sq,
                );
            }
            WarpHandleKind::Tangent(..) => {
                let mut ci = BezPath::new();
                // Um losango: quatro linhas, sem arco — o círculo exacto custaria quatro
                // cúbicas e a distinção que interessa (canto × tangente) é a FORMA, não a
                // suavidade dela.
                ci.move_to(Point::new(c.x, c.y - TANGENT_PX));
                ci.line_to(Point::new(c.x + TANGENT_PX, c.y));
                ci.line_to(Point::new(c.x, c.y + TANGENT_PX));
                ci.line_to(Point::new(c.x - TANGENT_PX, c.y));
                ci.close_path();
                vector_scene.inner_mut().fill(
                    ph2d_vector::Fill::NonZero,
                    Affine::IDENTITY,
                    &dim,
                    None,
                    &ci,
                );
            }
        }
    }
}
