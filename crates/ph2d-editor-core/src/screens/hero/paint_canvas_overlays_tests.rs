//! A caixa de objecto só com ALÇAS (F10, `GizmoStateGroup::handles_only`) — pelo quadro inteiro do
//! hero (`paint_hero_screen`), que é o que enche o `hit_index` que o despacho da shell lê.

use super::*;
use crate::gizmo::{gizmo_kind_for_id, ids, is_gizmo_id};

const JANELA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1500.0,
    h: 900.0,
};

fn caixa() -> crate::gizmo::GizmoView {
    crate::gizmo::GizmoView {
        bbox_min_world: [-1.0, -0.8],
        bbox_max_world: [1.0, 0.8],
        rotation: 0.0,
        camera_center: [0.0, 0.0],
        camera_height_world: 10.0,
        window_w: JANELA.w,
        window_h: JANELA.h,
        canvas: JANELA,
        cursor_screen: None,
        pivot_world: [0.0, 0.0],
        pivot_tool_active: false,
        model: crate::gizmo::MODEL_IDENTITY,
    }
}

fn pinta(handles_only: bool) -> HeroScreen {
    let mut hero = HeroScreen::new(ph2d_a11y::NodeId(1));
    hero.gizmo.view = Some(caixa());
    hero.gizmo.handles_only = handles_only;
    let mut ts = TextSystem::without_system_fonts();
    paint_hero_screen(&mut hero, JANELA, &mut VectorScene::new(), &mut ts);
    hero
}

fn centro(h: &HeroScreen, id: ph2d_a11y::NodeId) -> (f32, f32) {
    let r = h.hit_index.rect_for(id).expect("a alça foi registada");
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

/// ⭐ **No modo Forma a caixa é só alças**: o press numa alça de canto é da caixa (escala), e o
/// MIOLO não é de ninguém — o despacho lê-o como canvas e desenha. Sem pivô (pintado ou agarrável).
/// CONTROLO: a caixa inteira regista o miolo e o pivô no mesmo quadro.
#[test]
fn a_handles_only_box_leaves_its_middle_to_the_canvas() {
    let inteira = pinta(false);
    assert!(
        inteira
            .hit_index
            .rect_for(ids::GIZMO_BBOX_INTERIOR)
            .is_some()
    );
    assert!(inteira.hit_index.rect_for(ids::GIZMO_PIVOT).is_some());
    let (mx, my) = centro(&inteira, ids::GIZMO_PIVOT);
    // O controlo do miolo: um ponto do miolo longe do pivô é o Translate da caixa inteira.
    let (qx, qy) = (mx + 60.0, my + 30.0);
    assert_eq!(
        inteira.hit_index.hit(qx, qy),
        Some(ids::GIZMO_BBOX_INTERIOR),
        "controlo: o miolo da caixa inteira não é dela"
    );

    let alcas = pinta(true);
    assert!(alcas.hit_index.rect_for(ids::GIZMO_BBOX_INTERIOR).is_none());
    assert!(alcas.hit_index.rect_for(ids::GIZMO_PIVOT).is_none());
    for (x, y) in [(mx, my), (qx, qy)] {
        assert!(
            alcas.hit_index.hit(x, y).is_none_or(|id| !is_gizmo_id(id)),
            "o miolo em ({x}, {y}) ficou da caixa — o press não desenharia"
        );
    }
    for id in [
        ids::GIZMO_HANDLE_TL,
        ids::GIZMO_HANDLE_BR,
        ids::GIZMO_HANDLE_T,
        ids::GIZMO_HANDLE_L,
    ] {
        let (x, y) = centro(&alcas, id);
        let hit = alcas.hit_index.hit(x, y);
        assert_eq!(hit, Some(id), "a alça {id:?} não é da caixa");
        assert!(
            matches!(
                hit.and_then(gizmo_kind_for_id),
                Some(
                    crate::gizmo::GizmoDragKind::ScaleCorner { .. }
                        | crate::gizmo::GizmoDragKind::ScaleEdge { .. }
                )
            ),
            "a alça {id:?} não abre um arrasto de escala"
        );
        assert_eq!(centro(&inteira, id), (x, y), "a alça mudou de lugar");
    }
}
