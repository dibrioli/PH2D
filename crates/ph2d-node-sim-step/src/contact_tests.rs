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

// ───────────────────────── §7 · O MATERIAL, PELA PORTA DO PRODUTO ─────────────────────────

use ph2d_nodegraph::attr::{BOUNCE_COLUMN, FRICTION_COLUMN};

/// Uma bola livre a deslizar sobre um OBSTÁCULO (`inv_mass = 0`) que não se move, com o material
/// pedido. `spin` semeia a rotação própria da bola.
fn bola_sobre_obstaculo(atrito: f32, vx: f32, spin: Option<f32>) -> Stream {
    // O obstáculo é um disco grande, a bola um disco pequeno pousado nele com folga mínima.
    let (grande, pequeno) = (2.0_f32, 0.25_f32);
    let s = Stream::new(2)
        .with(
            "P",
            Column::Vec2(vec![[0.0, -grande], [0.0, pequeno - 0.001]]),
        )
        .with("vel", Column::Vec2(vec![[0.0, 0.0], [vx, -0.2]]))
        .with("inv_mass", Column::Scalar(vec![0.0, 1.0]))
        .with("sim_t", Column::Scalar(vec![0.0, 0.0]))
        .with(COLLIDER_COLUMN, Column::Scalar(vec![grande, pequeno]))
        .with(FRICTION_COLUMN, Column::Scalar(vec![atrito, atrito]))
        .with(BOUNCE_COLUMN, Column::Scalar(vec![0.0, 0.0]));
    match spin {
        Some(g) => s.with("spin", Column::Scalar(vec![0.0, g])),
        None => s,
    }
}

fn escalar(s: &Stream, name: &str) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// ⭐⭐⭐ **A COSTURA do material está ligada** (doc 109 §7): o passo tem de entregar ao contacto o
/// DESLIZE (onde a peça estava antes de ele a mover) e o MATERIAL — senão a lei existe na folha e
/// não acontece no produto.
///
/// ⛔⛔ **É este gate, e só este, que morre se o `sim.step` deixar de passar o `antes_do_passo` ou
/// os materiais.** Os gates da `ph2d-contact` chamam o solver directamente e ficariam todos verdes:
/// *a costura é o que se perde num merge, não a matemática.*
#[test]
fn the_step_hands_the_contact_the_slide_and_the_material() {
    let rola = step(
        &bola_sobre_obstaculo(1.0, 1.0, None),
        DT,
        1.0,
        0.0,
        0.0,
        1.0,
    );
    let gelo = step(
        &bola_sobre_obstaculo(0.0, 1.0, None),
        DT,
        1.0,
        0.0,
        0.0,
        1.0,
    );
    let g = escalar(&rola, "rot");
    assert!(
        g.get(1).copied().unwrap_or(0.0) < -1e-4,
        "com atrito a bola tem de RODAR (horario, a deslizar para +x): {g:?}"
    );
    // ⚠️ **A barra é de RUÍDO e não zero, e o número tem mecanismo** (doc 109 §7.1): a alavanca da
    // normal num disco é zero em aritmética exacta, e o `ponto − centro` de uma peça longe da
    // origem é uma subtracção que deixa cancelamento de `f32` — medido aqui, **`9,1e-10`**, contra
    // os `~1e-2` que o atrito produz. *Uma barra de zero mediria a aritmética.*
    let gelado = escalar(&gelo, "rot").get(1).copied().unwrap_or(0.0);
    assert!(
        gelado.abs() < 1e-6,
        "com atrito 0 a bola nao pode rodar, e rodou {gelado}"
    );
}

/// ⭐⭐ **E o passo diz ao contacto quanto o `spin` JÁ rodou neste tique** — uma bola a girar derrapa
/// contra o chão mesmo sem se deslocar, e sem este canal o atrito não a veria.
///
/// ⚠️ O controlo é a MESMA cena sem `spin`: ali o contacto não tem deslize nenhum a opor.
#[test]
fn the_step_tells_the_contact_how_much_the_spin_already_turned() {
    let girando = step(
        &bola_sobre_obstaculo(1.0, 0.0, Some(900.0)),
        DT,
        1.0,
        0.0,
        0.0,
        1.0,
    );
    let parada = step(
        &bola_sobre_obstaculo(1.0, 0.0, Some(0.0)),
        DT,
        1.0,
        0.0,
        0.0,
        1.0,
    );
    // A bola patina: o chão empurra-a para o lado contrário ao varrimento do ponto de baixo.
    let (a, b) = (col(&girando, "P"), col(&parada, "P"));
    assert!(
        a[1][0] < b[1][0] - 1e-6,
        "o atrito tinha de empurrar a bola que patina: {:?} contra {:?}",
        a[1],
        b[1]
    );
}

/// ⭐⭐ **O SALTO da peça chega à velocidade** (doc 109 §7): duas bolas saltitantes a aproximarem-se
/// separam-se com MAIS velocidade do que duas mortas — e com `bounce = 0` a lei é a de sempre.
#[test]
fn the_bounce_of_the_pieces_reaches_the_velocity() {
    let com = |b: f32| {
        par(0.4, 1.0, Some(0.5))
            .with(FRICTION_COLUMN, Column::Scalar(vec![0.0, 0.0]))
            .with(BOUNCE_COLUMN, Column::Scalar(vec![b, b]))
    };
    let morta = step(&com(0.0), DT, 1.0, 0.0, 0.0, 1.0);
    let viva = step(&com(0.9), DT, 1.0, 0.0, 0.0, 1.0);
    let (m, v) = (col(&morta, "vel"), col(&viva, "vel"));
    assert!(
        m[0][0].abs() < 1e-4,
        "morta: a aproximacao e' CANCELADA, e sobrou {:?}",
        m[0]
    );
    assert!(
        v[0][0] < -0.5,
        "viva: ela tem de voltar para tras, e ficou em {:?}",
        v[0]
    );
}
