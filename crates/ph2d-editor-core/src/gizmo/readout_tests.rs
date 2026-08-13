//! Gates do NÚMERO de um arrasto de gizmo.
//!
//! ⚠️ Vários deles dirigem a `compute_gizmo_transform` REAL e alimentam o resultado dela ao
//! readout, em vez de fabricarem um `now` à mão: é essa cadeia — *o que o produto escreve é o que a
//! ficha diz* — que a wave afirma, e um `now` inventado testaria o formatador contra si próprio.

use super::super::camera::{GizmoCamera, GizmoModifiers, GizmoSnap};
use super::super::drag::{GizmoDragState, GizmoTarget};
use super::super::transform::compute_gizmo_transform;
use super::*;
use crate::project::{DisplayUnit, ProjectSettings};

fn cam() -> GizmoCamera {
    GizmoCamera {
        center: [0.0, 0.0],
        height_world: 10.0,
        window_w: 800.0,
        window_h: 600.0,
    }
}

fn snap(tx: f32, ty: f32) -> TransformSnapshot {
    TransformSnapshot {
        translation: [tx, ty],
        rotation: 0.0,
        scale: [1.0, 1.0],
    }
}

fn drag_at(kind: GizmoDragKind, from: (f32, f32), to: (f32, f32)) -> GizmoDragState {
    let c = cam();
    GizmoDragState {
        kind,
        entity_bits: 1,
        start_screen: from,
        cursor_screen: to,
        start_transform: snap(0.0, 0.0),
        pivot_world: [0.0, 0.0],
        start_cursor_world: c.screen_to_world(from),
        sprite_half_intrinsic: [0.5, 0.5],
        anchor_is_center: false,
        target: GizmoTarget::PrimaryIndividual,
        parent_world: TransformSnapshot::IDENTITY,
        turns: 0,
    }
}

fn plain() -> (GizmoModifiers, GizmoSnap) {
    (
        GizmoModifiers {
            shift: false,
            ctrl: false,
            alt: false,
        },
        GizmoSnap::default(),
    )
}

/// Pixels a 100 px/m — os defaults do projecto, e a régua que o Inspector usa.
fn px() -> crate::LengthDisplay {
    crate::LengthDisplay::of(&ProjectSettings::default())
}

fn meters() -> crate::LengthDisplay {
    crate::LengthDisplay {
        unit: DisplayUnit::Meters,
        pixels_per_meter: 100.0,
    }
}

/// **Mover diz a DIFERENÇA que foi escrita** — e o número sai do resultado do produto.
#[test]
fn a_move_reads_the_difference_the_product_wrote() {
    let d = drag_at(GizmoDragKind::Translate, (400.0, 300.0), (480.0, 300.0));
    let (m, s) = plain();
    let now = compute_gizmo_transform(&d, &cam(), m, s, None);
    let r = gizmo_readout(d.kind, &d.start_transform, &now);
    let GizmoReadout::Moved { dx, dy } = r else {
        panic!("um Translate tem de ler uma diferença, e leu {r:?}")
    };
    assert!(
        (dx - f64::from(now.translation[0])).abs() < 1e-9,
        "a ficha não diz o que foi escrito: {dx} contra {}",
        now.translation[0]
    );
    assert!(dx > 0.0, "arrastar para a direita tem de dar dx positivo");
    assert!(dy.abs() < 1e-9);
}

/// ⭐ **Sob um pai RODADO a ficha diz o delta LOCAL — o mesmo que o campo Position mostra.**
///
/// O gesto é horizontal em MUNDO; com o pai a 90° o que o produto escreve é vertical em LOCAL. Uma
/// ficha que dissesse o delta de mundo contradiria o Inspector que o artista vê a mudar ao lado.
///
/// *Mutação que sangra:* derivar a ficha do cursor (`now_world - start_cursor_world`) em vez do
/// resultado — o eixo troca.
#[test]
fn under_a_rotated_parent_the_chip_says_what_the_inspector_says() {
    let mut d = drag_at(GizmoDragKind::Translate, (400.0, 300.0), (480.0, 300.0));
    d.parent_world = TransformSnapshot {
        translation: [0.0, 0.0],
        rotation: core::f32::consts::FRAC_PI_2,
        scale: [1.0, 1.0],
    };
    let (m, s) = plain();
    let now = compute_gizmo_transform(&d, &cam(), m, s, None);
    let GizmoReadout::Moved { dx, dy } = gizmo_readout(d.kind, &d.start_transform, &now) else {
        panic!("Translate")
    };
    // O mundo andou em +X; o LOCAL, sob um pai a +90°, anda em -Y.
    assert!(
        dx.abs() < 1e-6 && dy < -0.1,
        "a ficha não seguiu o frame LOCAL: dx={dx}, dy={dy}"
    );
    assert!(
        (dx - f64::from(now.translation[0])).abs() < 1e-9
            && (dy - f64::from(now.translation[1])).abs() < 1e-9,
        "a ficha divergiu do `Transform` escrito"
    );
}

/// **O encaixe entra no número.** Com Ctrl a posição é quantizada, e a ficha diz o número
/// QUANTIZADO — não o que o cursor sugeria.
#[test]
fn the_snap_is_inside_the_number() {
    let d = drag_at(GizmoDragKind::Translate, (400.0, 300.0), (480.0, 300.0));
    let now = compute_gizmo_transform(
        &d,
        &cam(),
        GizmoModifiers {
            shift: false,
            ctrl: true,
            alt: false,
        },
        GizmoSnap {
            move_meters: 0.5,
            rotate_deg: 0.0,
        },
        None,
    );
    let GizmoReadout::Moved { dx, .. } = gizmo_readout(d.kind, &d.start_transform, &now) else {
        panic!("Translate")
    };
    let steps = dx / 0.5;
    assert!(
        (steps - steps.round()).abs() < 1e-6,
        "a ficha não caiu na grade do encaixe: {dx} não é múltiplo de 0,5"
    );
}

/// **Escalar diz uma RAZÃO**, não um absoluto: a escala é multiplicativa.
#[test]
fn a_scale_reads_a_ratio() {
    let start = TransformSnapshot {
        translation: [0.0, 0.0],
        rotation: 0.0,
        scale: [2.0, 4.0],
    };
    let now = TransformSnapshot {
        scale: [3.0, 2.0],
        ..start
    };
    let r = gizmo_readout(
        GizmoDragKind::ScaleCorner {
            dx_sign: 1.0,
            dy_sign: 1.0,
        },
        &start,
        &now,
    );
    assert_eq!(
        r,
        GizmoReadout::Scaled { rx: 1.5, ry: 0.5 },
        "a razão tem de ser now/start por eixo"
    );
}

/// Uma escala de partida nula não tem denominador — a ficha diz `1` em vez de infinito.
#[test]
fn a_zero_start_scale_does_not_produce_infinity() {
    let start = TransformSnapshot {
        translation: [0.0, 0.0],
        rotation: 0.0,
        scale: [0.0, 1.0],
    };
    let now = TransformSnapshot {
        scale: [5.0, 1.0],
        ..start
    };
    let r = gizmo_readout(
        GizmoDragKind::ScaleEdge { axis: 0, sign: 1.0 },
        &start,
        &now,
    );
    assert_eq!(r, GizmoReadout::Scaled { rx: 1.0, ry: 1.0 });
    assert!(r.is_idle(), "sem denominador não há nada a dizer");
}

/// **Uma volta e meia lê-se como 540°**, não como 180°.
///
/// O contador de voltas do arrasto entra na rotação escrita, então a diferença carrega-o de graça —
/// e é por isso que a ficha não precisa de o conhecer.
#[test]
fn a_turn_reads_past_one_revolution() {
    let start = snap(0.0, 0.0);
    let now = TransformSnapshot {
        rotation: core::f32::consts::TAU * 1.5,
        ..start
    };
    let GizmoReadout::Turned { degrees } = gizmo_readout(GizmoDragKind::Rotate, &start, &now)
    else {
        panic!("Rotate")
    };
    assert!(
        (degrees - 540.0).abs() < 0.01,
        "a ficha achatou a volta: {degrees}"
    );
}

/// ⭐ **Um gesto que não fez nada não tem número** — e sem isto a ficha piscaria a cada clique de
/// selecção, porque um pick de canvas ABRE um arrasto de Translate.
///
/// *Mutação que sangra:* fazer `is_idle` devolver sempre `false`.
#[test]
fn a_gesture_that_did_nothing_has_no_number() {
    let s = TransformSnapshot {
        translation: [3.0, -1.0],
        rotation: 0.7,
        scale: [2.0, 0.5],
    };
    for kind in [
        GizmoDragKind::Translate,
        GizmoDragKind::MovePivot,
        GizmoDragKind::Rotate,
        GizmoDragKind::ScaleCorner {
            dx_sign: 1.0,
            dy_sign: -1.0,
        },
        GizmoDragKind::ScaleEdge { axis: 1, sign: 1.0 },
    ] {
        assert!(
            gizmo_readout(kind, &s, &s).is_idle(),
            "{kind:?} com o mundo parado tem de ficar calado"
        );
    }
    // …e o menor movimento acorda-a.
    let moved = TransformSnapshot {
        translation: [3.000_01, -1.0],
        ..s
    };
    assert!(!gizmo_readout(GizmoDragKind::Translate, &s, &moved).is_idle());
}

/// ⭐ **O texto passa pela porta de unidade do projecto** — a mesma do Inspector e da régua.
///
/// *Mutação que sangra:* formatar o delta directamente em metros — a ficha passa a dizer `+1.2`
/// enquanto o campo Position ao lado diz `+120`.
#[test]
fn the_text_goes_through_the_projects_unit_door() {
    let r = GizmoReadout::Moved { dx: 1.2, dy: -0.5 };
    let in_px = r.text(px(), 100.0);
    let in_m = r.text(meters(), 100.0);
    assert!(
        in_px.contains("120") && in_px.ends_with("px"),
        "em pixels a ficha tem de dizer 120: {in_px}"
    );
    assert!(
        in_m.contains("1.2") && in_m.ends_with('m'),
        "em metros a ficha tem de dizer 1.2: {in_m}"
    );
}

/// O delta carrega o SINAL — sem ele é indistinguível de um absoluto.
#[test]
fn the_delta_wears_its_sign() {
    let t = GizmoReadout::Moved { dx: 1.2, dy: -0.5 }.text(px(), 100.0);
    assert!(t.starts_with('+'), "delta positivo sem sinal: {t}");
    assert!(t.contains(", -"), "delta negativo sem sinal: {t}");
    let turn = GizmoReadout::Turned { degrees: 45.0 }.text(px(), 100.0);
    assert_eq!(turn, "+45.0\u{b0}");
}

/// Uma escala UNIFORME diz um número só — repetir o mesmo valor é ruído.
#[test]
fn a_uniform_scale_says_one_number() {
    let uni = GizmoReadout::Scaled { rx: 1.2, ry: 1.2 }.text(px(), 100.0);
    assert_eq!(uni, "\u{d7}1.20");
    let non = GizmoReadout::Scaled { rx: 1.2, ry: 0.8 }.text(px(), 100.0);
    assert_eq!(non, "\u{d7}1.20, \u{d7}0.80");
}

/// As casas decimais seguem o ZOOM, como no rótulo do smart guide: num zoom grosso um milímetro não
/// é uma leitura, é ruído.
#[test]
fn the_decimals_follow_the_zoom() {
    let r = GizmoReadout::Moved {
        dx: 1.234_5,
        dy: 0.0,
    };
    let fine = r.text(meters(), 2000.0);
    let coarse = r.text(meters(), 2.0);
    assert!(
        fine.len() > coarse.len(),
        "o zoom fino tem de dar mais casas: fino {fine}, grosso {coarse}"
    );
}
