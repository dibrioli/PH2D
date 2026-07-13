//! ADR-0114 W6 — **o realce da seleção** (Edit Mode).
//!
//! Uma seleção que não se VÊ não existe: o usuário clica, nada muda na tela, e ele conclui
//! que a ferramenta está quebrada (é a mesma lição do "pintado ≠ populado" — só que aqui o
//! seam é o canvas, não o painel).
//!
//! **É overlay, não render de traço.** O realce é desenhado no `vector_scene` (a cena
//! Vello composta sobre o canvas neste frame, como o anel do pincel em `flip_cursor`), e
//! **não** re-rasterizado pelo passe do Flip. Duas razões:
//!
//! - o realce é **chrome**, não arte: ele não pode entrar no `pack`, não pode ir para o
//!   PNG exportado, e não pode participar do depth do desenho;
//! - a espessura dele é em **px de TELA** e constante — como a do gizmo. Se ele fosse
//!   geometria de documento, aproximar a câmera o engrossaria e ele cobriria a linha que
//!   está tentando destacar.
//!
//! A pose do objeto entra como um **`Affine` só** (`world_to_screen_affine ∘ local→mundo`):
//! nenhum ponto é transformado à mão, então o realce não pode derivar da arte — ele
//! herda a MESMA matriz que o render usa.

use ph2d_flip::FlipDoc;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_vec_scene::Xform;
use ph2d_vector::VectorScene;

/// Espessura do contorno de realce, em px de tela.
const HALO_PX: f64 = 2.0; // LITERAL-PX-OK: chrome de overlay, espessura de tela

/// Cor do realce (âmbar do editor) — chrome, não arte.
const HALO_RGBA: [f32; 4] = [1.0, 0.72, 0.2, 0.95]; // LITERAL-COLOR-OK: overlay de selecao

/// Desenha o contorno de realce sobre cada traço selecionado do desenho VISÍVEL.
///
/// `l2w` é o afim LOCAL→mundo do objeto (a pose do gizmo). Nada é desenhado fora do modo
/// Edit: o realce é a linguagem DESSE modo, e deixá-lo aceso nos outros faria a seleção
/// parecer um estado global que o Draw/Erase respeitam — o que não é verdade (só o Sculpt
/// e o painel a consultam).
#[allow(clippy::too_many_arguments)]
pub(super) fn draw_flip_selection(
    active: bool,
    editing: bool,
    doc: &FlipDoc,
    playhead: &ph2d_core::Playhead,
    active_layer: Option<ph2d_flip::LayerId>,
    l2w: &Xform,
    camera: &Camera2d,
    window: WindowSize,
    vector_scene: &mut VectorScene,
) {
    if !active || !editing {
        return;
    }
    let Some((oid, _lid, did)) = crate::flip_select::visible_drawing(doc, playhead, active_layer)
    else {
        return;
    };
    let Some(drawing) = doc.object(oid).and_then(|o| o.drawing(did)) else {
        return;
    };
    if !drawing.any_selected() {
        return;
    }

    use ph2d_vector::{Affine, BezPath, Brush, Color, Point, Stroke};
    // local → mundo → tela, numa matriz só (a MESMA do render; ver o cabeçalho).
    let [a, b, c, d, e, f] = l2w.0;
    let local_to_world = Affine::new([a, b, c, d, e, f]);
    let to_screen = camera.world_to_screen_affine(window) * local_to_world;

    let color = Color::new(HALO_RGBA);
    for s in drawing.strokes.iter().filter(|s| s.selected) {
        let pos = s.positions();
        if pos.is_empty() {
            continue;
        }
        let mut path = BezPath::new();
        for (i, p) in pos.iter().enumerate() {
            let pt = Point::new(f64::from(p.x), f64::from(p.y));
            if i == 0 {
                path.move_to(pt);
            } else {
                path.line_to(pt);
            }
        }
        // O traço FECHADO (e o contorno de uma região) fecha o realce; um traço aberto
        // NÃO — desenhar o segmento que liga as pontas mostraria uma linha que o usuário
        // não fez, e depois do BUGS #17 sabemos que "fechado" não é o caso comum.
        if s.closed {
            path.close_path();
        }
        vector_scene.inner_mut().stroke(
            &Stroke::new(HALO_PX),
            to_screen,
            &Brush::Solid(color),
            None,
            &path,
        );
    }
}
