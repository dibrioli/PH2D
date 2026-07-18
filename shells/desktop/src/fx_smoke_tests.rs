//! Sonda da cena de smoke: a elipse do nível 13 revela-se **progressivamente**?
//!
//! O smoke do Enio disse *"na elipse não vejo nada acontecendo. Na estrela a linha está
//! animada"* — e as duas passam pelo MESMO motor, então a diferença está na forma ou no
//! tempo, não no Trim. Isto mede a cena do PRODUTO (a mesma `shape(ShapeKind::Ellipse, …)`
//! que o `build` empurra), não um fixture conveniente.

use super::*;
use ph2d_vec_scene::ShapeKind;

/// O comprimento do contorno primário de `p`, cozido.
fn cooked_len(p: &ph2d_vec_scene::VecPath) -> f64 {
    let c = p.cooked();
    let n = c.verts.len();
    if n < 2 {
        return 0.0;
    }
    let segs = if c.closed { n } else { n - 1 };
    (0..segs)
        .map(|i| {
            let (a, b) = (&c.verts[i], &c.verts[(i + 1) % n]);
            ph2d_vec_scene::arclen::arclen(&[a.anchor, a.out_handle, b.in_handle, b.anchor])
        })
        .sum()
}

/// **O draw-on RECOMEÇA** — o defeito que o smoke do Enio encontrou não estava no Trim, e
/// sim no tempo: a rampa era one-shot e acabava antes de ele olhar para a janela.
///
/// O gate mora na política de tempo (`draw_on_phase`, função pura) e não na cena, porque é a
/// política que estava errada. [[feedback_ready_to_smoke_example]]
#[test]
fn the_draw_on_loops_instead_of_finishing_once() {
    assert_eq!(draw_on_phase(0), 0.0, "começa vazia");
    assert!(
        draw_on_phase(DRAW_ON_FRAMES / 2) > 0.4,
        "está a meio caminho"
    );
    assert_eq!(draw_on_phase(DRAW_ON_FRAMES), 1.0, "chega ao fim");
    assert_eq!(
        draw_on_phase(DRAW_ON_FRAMES + HOLD_FRAMES / 2),
        1.0,
        "e SEGURA cheia durante o hold"
    );
    assert_eq!(
        draw_on_phase(CYCLE),
        0.0,
        "e RECOMEÇA — sem isto a cena fica parada para sempre e não é smokável"
    );
    assert_eq!(
        draw_on_phase(CYCLE * 7 + 3),
        draw_on_phase(3),
        "é periódica"
    );
}

/// A janela da estrela gira **sem pausa** — ela não desenha, ela corre; se ela parasse junto
/// com o hold da elipse, a cena inteira congelaria uma vez por ciclo.
#[test]
fn the_star_window_never_pauses() {
    let a = spin_phase(DRAW_ON_FRAMES);
    let b = spin_phase(DRAW_ON_FRAMES + HOLD_FRAMES / 2);
    assert!(
        b > a,
        "o giro continuou durante o hold da elipse ({a} -> {b})"
    );
}

/// **A elipse da cena cresce monotonicamente com o `phase`.**
///
/// É o gate que INOCENTOU o motor quando o smoke reprovou a cena: as duas formas passam pelo
/// mesmo Trim, então se a elipse revela certo em headless, o que falhava era o tempo.
#[test]
fn the_smoke_ellipse_reveals_progressively() {
    let mut p = shape(
        ShapeKind::Ellipse,
        [-3.4, -1.2],
        [-0.6, 1.2],
        &[],
        [70, 150, 220],
    );
    let full = cooked_len(&p);
    assert!(full > 0.0, "a elipse da cena tem comprimento");

    let mut prev = -1.0;
    for step in 0..=10 {
        let phase = f64::from(step) / 10.0;
        p.effects = vec![PathEffect::Trim(TrimSpec {
            start: 0.0,
            end: phase,
            offset: 0.0,
        })];
        let got = cooked_len(&p);
        assert!(
            got > prev,
            "phase {phase}: comprimento {got} não cresceu (anterior {prev}) — a revelação \
             não é progressiva"
        );
        prev = got;
    }
    assert!(
        (prev - full).abs() / full < 1e-9,
        "no fim a elipse tem de estar INTEIRA: {prev} vs {full}"
    );
}
