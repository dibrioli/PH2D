//! Os gates do contacto no passo (doc 109 W2) — pela porta do produto, o [`crate::step`].

use crate::step;
use ph2d_nodegraph::attr::{
    COLLIDER_BOX_COLUMN, COLLIDER_COLUMN, Column, INV_INERTIA_COLUMN, Stream,
};

/// Duas peças em `x = ±meio`, com velocidades, relógio e (opcional) colisor de raio `r`.
fn par(meio: f32, v: f32, r: Option<f32>) -> Stream {
    let s = Stream::new(2)
        .with("P", Column::Vec2(vec![[-meio, 0.0], [meio, 0.0]]))
        .with("vel", Column::Vec2(vec![[v, 0.0], [-v, 0.0]]))
        .with("sim_t", Column::Scalar(vec![0.0, 0.0]));
    match r {
        Some(r) => s.with(COLLIDER_COLUMN, Column::Scalar(vec![r, r])),
        None => s,
    }
}

fn col(s: &Stream, name: &str) -> Vec<[f32; 2]> {
    match s.get(name) {
        Some(Column::Vec2(v)) => v.clone(),
        _ => panic!("sem {name}"),
    }
}

const DT: f32 = 1.0 / 60.0;

/// ⭐ **Um colisor de raio ZERO passa exactamente como nenhum** — posições e velocidades ao bit.
#[test]
fn a_zero_collider_steps_exactly_like_no_collider() {
    let sem = step(&par(0.1, 1.0, None), DT, 1.0, 0.0, 0.0, 1.0);
    let zero = step(&par(0.1, 1.0, Some(0.0)), DT, 1.0, 0.0, 0.0, 1.0);
    for c in ["P", "vel"] {
        let (a, b) = (col(&sem, c), col(&zero, c));
        for i in 0..2 {
            assert_eq!(
                (a[i][0].to_bits(), a[i][1].to_bits()),
                (b[i][0].to_bits(), b[i][1].to_bits()),
                "{c}[{i}]"
            );
        }
    }
}

/// ⭐ **Duas peças sobrepostas saem à soma dos raios** (`size` ausente ⇒ escala 1).
#[test]
fn overlapping_pieces_are_pushed_apart_to_their_radii() {
    let s = step(&par(0.1, 0.0, Some(0.5)), DT, 1.0, 0.0, 0.0, 1.0);
    let p = col(&s, "P");
    assert!(((p[1][0] - p[0][0]) - 1.0).abs() < 1e-4, "{p:?}");
}

/// ⭐⭐ **O contacto NUNCA acrescenta velocidade** — duas peças que nascem sobrepostas, paradas,
/// separam-se em posição e continuam paradas. Somar `Δp/dt` inteiro faria delas uma explosão.
#[test]
fn the_contact_never_adds_speed() {
    let s = step(&par(0.1, 0.0, Some(0.5)), DT, 1.0, 0.0, 0.0, 1.0);
    assert_eq!(col(&s, "vel"), vec![[0.0, 0.0], [0.0, 0.0]]);
}

/// ⭐⭐ **A aproximação é CANCELADA, não reflectida** — duas peças que se encostam a `±2 u/s`
/// ficam encostadas e param; nenhuma ressalta para trás.
#[test]
fn an_approach_is_cancelled_not_reflected() {
    // Encostadas (distância 1 = a soma dos raios) e a vir uma para a outra.
    let s = step(&par(0.5, 2.0, Some(0.5)), DT, 1.0, 0.0, 0.0, 1.0);
    let (p, v) = (col(&s, "P"), col(&s, "vel"));
    assert!(
        ((p[1][0] - p[0][0]) - 1.0).abs() < 1e-4,
        "encostadas: {p:?}"
    );
    for (i, vi) in v.iter().enumerate() {
        assert!(
            vi[0].abs() < 1e-3,
            "a peca {i} parou em x, sem ressalto: {vi:?}"
        );
    }
}

/// ⭐⭐ **Duas CAIXAS encostam pela FACE e param** — a distância é a soma das meias larguras, não a
/// dos círculos à volta delas (o `41 %` de ar do report do doc 109 §5), e a aproximação é cancelada.
#[test]
fn two_boxes_rest_face_to_face_and_stop() {
    let s = Stream::new(2)
        .with("P", Column::Vec2(vec![[-0.3, 0.0], [0.3, 0.0]]))
        .with("vel", Column::Vec2(vec![[1.0, 0.0], [-1.0, 0.0]]))
        .with("sim_t", Column::Scalar(vec![0.0, 0.0]))
        .with(
            COLLIDER_BOX_COLUMN,
            Column::Vec2(vec![[0.5, 2.0], [0.5, 2.0]]),
        );
    let out = step(&s, DT, 1.0, 0.0, 0.0, 1.0);
    let (p, v) = (col(&out, "P"), col(&out, "vel"));
    assert!(((p[1][0] - p[0][0]) - 1.0).abs() < 1e-4, "{p:?}");
    for (i, vi) in v.iter().enumerate() {
        assert!(vi[0].abs() < 1e-3, "a caixa {i} parou em x: {vi:?}");
    }
}

/// ⭐⭐⭐ **Uma caixa apanhada FORA DO CENTRO tomba** (doc 109 §6 — *«precisa destravar a rot»*), e a
/// coluna `inv_inertia` a zero (o botão `Lock Rotation` do cartão) trava-a.
///
/// ⚠️ **As duas metades num gate:** *«roda»* passa com uma peça que gira sempre, e *«trava»* passa
/// com uma que nunca gira. E travada **nem a coluna `rot` nasce** — uma cena sem rotação sai como
/// sempre saiu.
#[test]
fn a_box_caught_off_centre_turns_unless_the_column_locks_it() {
    let angulos = |travada: bool| -> Option<Vec<f32>> {
        let s = Stream::new(2)
            .with("P", Column::Vec2(vec![[0.0, 0.0], [0.9, 0.25]]))
            .with("vel", Column::Vec2(vec![[0.0, 0.0], [0.0, 0.0]]))
            .with("sim_t", Column::Scalar(vec![0.0, 0.0]))
            // A prancha é um pino; a caixa livre pousa na ponta dela.
            .with("inv_mass", Column::Scalar(vec![0.0, 1.0]))
            .with(
                COLLIDER_BOX_COLUMN,
                Column::Vec2(vec![[1.0, 0.1], [0.25, 0.25]]),
            );
        let s = if travada {
            s.with(INV_INERTIA_COLUMN, Column::Scalar(vec![0.0, 0.0]))
        } else {
            s
        };
        match step(&s, DT, 1.0, 0.0, 0.0, 1.0).get("rot") {
            Some(Column::Scalar(v)) => Some(v.clone()),
            _ => None,
        }
    };
    let solta = angulos(false).expect("sem travar, o passo escreve o angulo");
    assert_eq!(solta[0], 0.0, "o pino nao roda: {solta:?}");
    assert!(solta[1].abs() > 1.0, "a caixa da ponta tombou: {solta:?}");
    assert!(angulos(true).is_none(), "travada, nem a coluna `rot` nasce");
}

/// ⭐ **Uma peça sem colisor ATRAVESSA** — só as duas com colisor se afastam.
#[test]
fn a_piece_without_a_collider_passes_through() {
    let s = par(0.1, 0.0, None).with(COLLIDER_COLUMN, Column::Scalar(vec![0.5, 0.0]));
    let out = step(&s, DT, 1.0, 0.0, 0.0, 1.0);
    assert_eq!(col(&out, "P"), vec![[-0.1, 0.0], [0.1, 0.0]]);
}
