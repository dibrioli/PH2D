//! Gates da [`super::paint_hero_screen_na_escala`].

use super::*;
use crate::gizmo::ids;
use ph2d_tokens::UiScale;

const JANELA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1500.0,
    h: 900.0,
};

/// Uma selecção no meio do mundo, vista por uma janela FÍSICA — o que a shell publica.
fn gizmo_fisico() -> crate::gizmo::GizmoView {
    crate::gizmo::GizmoView {
        bbox_min_world: [-1.0, -0.5],
        bbox_max_world: [1.5, 1.0],
        rotation: 0.3,
        camera_center: [0.2, -0.1],
        camera_height_world: 10.0,
        window_w: JANELA.w,
        window_h: JANELA.h,
        canvas: JANELA,
        cursor_screen: Some((700.0, 400.0)),
        pivot_world: [0.25, 0.25],
        pivot_tool_active: false,
    }
}

const ALCAS: [ph2d_a11y::NodeId; 9] = [
    ids::GIZMO_HANDLE_TL,
    ids::GIZMO_HANDLE_TR,
    ids::GIZMO_HANDLE_BL,
    ids::GIZMO_HANDLE_BR,
    ids::GIZMO_HANDLE_T,
    ids::GIZMO_HANDLE_R,
    ids::GIZMO_HANDLE_B,
    ids::GIZMO_HANDLE_L,
    ids::GIZMO_PIVOT,
];

fn pinta(scale: UiScale) -> (HeroScreen, VectorScene) {
    let mut hero = HeroScreen::new(ph2d_a11y::NodeId(1));
    hero.ui_scale = scale;
    hero.gizmo.view = Some(gizmo_fisico());
    let mut scene = VectorScene::new();
    let mut ts = TextSystem::without_system_fonts();
    paint_hero_screen_na_escala(&mut hero, JANELA, &mut scene, &mut ts, |_, _, _, _| {});
    (hero, scene)
}

/// ⭐ **A `100 %` é o caminho de sempre, byte a byte** — a mesma cena e o mesmo viewport que o
/// `paint_hero_screen` cru.
#[test]
fn a_cem_por_cento_e_o_caminho_de_sempre() {
    let (escala, cena_escala) = pinta(UiScale::P100);
    let mut cru = HeroScreen::new(ph2d_a11y::NodeId(1));
    cru.gizmo.view = Some(gizmo_fisico());
    let mut cena_cru = VectorScene::new();
    paint_hero_screen(
        &mut cru,
        JANELA,
        &mut cena_cru,
        &mut TextSystem::without_system_fonts(),
    );
    assert_eq!(escala.last_viewport, cru.last_viewport);
    assert_eq!(
        cena_escala.inner().encoding().draw_data,
        cena_cru.inner().encoding().draw_data,
        "a 100 % a cena tem de ser a de sempre"
    );
}

/// ⭐⭐ **O mundo cai no MESMO píxel em toda escala** — as alças do gizmo, pintadas e registadas
/// no espaço lógico, voltam ao físico (× `s`) exactamente onde caem a `100 %`; o chrome à volta
/// cresce. E a shell recebe as vistas FÍSICAS de volta (é com elas que ela faz as contas).
#[test]
fn as_vistas_do_mundo_caem_no_mesmo_pixel_em_toda_escala() {
    let (base, _) = pinta(UiScale::P100);
    let centro = |h: &HeroScreen, id| {
        let r = h.hit_index.rect_for(id).expect("a alça foi registada");
        (r.x + r.w * 0.5, r.y + r.h * 0.5)
    };
    for z in [
        UiScale::P80,
        UiScale::P90,
        UiScale::P125,
        UiScale::P150,
        UiScale::P200,
    ] {
        let (h, _) = pinta(z);
        let s = z.factor();
        assert_eq!(
            h.gizmo.view,
            Some(gizmo_fisico()),
            "{z:?}: a vista física não voltou"
        );
        assert!(
            (h.last_viewport.w * s - JANELA.w).abs() < 1e-3,
            "{z:?}: viewport lógico"
        );
        for id in ALCAS {
            let (bx, by) = centro(&base, id);
            let (x, y) = centro(&h, id);
            assert!(
                (x * s - bx).abs() < 1e-2 && (y * s - by).abs() < 1e-2,
                "{z:?} {id:?}: a alça caiu em ({:.2}, {:.2}) e a 100 % cai em ({bx:.2}, {by:.2})",
                x * s,
                y * s
            );
        }
    }
}
