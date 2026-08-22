//! ⭐ **Os gates do número digitado** (W26).
//!
//! ⚠️ **A aritmética é a metade fácil.** O que falha em silêncio é a costura: a tecla chega? o rato
//! cede? o que vai ao mundo é o TOTAL ou mais um incremento por cima do que o ponteiro já tinha
//! dado? Os gates de baixo dirigem `typed_key`, que é o caminho de produção inteiro — só o `App` (a
//! tradução da tecla) fica de fora, e essa é uma linha ao lado de três idênticas.

use super::*;
use crate::field3d_gizmo::{Anchor, Handle, Mode, Motion};
use crate::field3d_smoke::{Drag, Grip, Smoke, set_armed_by_panel, with_smoke};

const AXES: [[f32; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
const ENTITY: u64 = 7;

fn armed<R>(f: impl FnOnce(&mut Smoke) -> R) -> R {
    set_armed_by_panel(true);
    with_smoke(f).expect("o módulo está armado")
}

/// Uma alça **agarrada**, com `applied` já a valer alguma coisa — que é o estado real de quem
/// arrastou um bocado antes de escrever o número.
fn holding(handle: Handle, applied: Motion) {
    armed(|s| {
        s.area = Some(ph2d_editor::zones::Rect::new(0.0, 0.0, 800.0, 600.0));
        s.cam = ph2d_field_render::Orbit::default();
        s.gizmo_mode = Mode::Move;
        let anchor = Anchor {
            entity: ENTITY,
            origin: [0.0; 3],
            axes: AXES,
        };
        s.gizmo = Some(anchor);
        s.drag = Some(Drag::Gizmo(handle));
        s.drag_grip = Some(Grip {
            anchor,
            from: [400.0, 300.0],
            applied,
        });
        s.pending_move = Some((ENTITY, applied));
        s.typed = None;
        s.last_pointer = (400.0, 300.0);
    });
}

/// O que o mundo recebeu deste gesto, somado.
fn banked() -> Option<Motion> {
    armed(|s| s.pending_move.map(|(_, m)| m))
}

fn type_in(text: &str) {
    armed(|s| {
        for c in text.chars() {
            let stroke = match c {
                '.' => Stroke::Dot,
                '-' => Stroke::Sign,
                d => Stroke::Digit(d.to_digit(10).expect("dígito") as u8),
            };
            crate::field3d_input::typed_key(s, stroke);
        }
    });
}

/// ⭐ **O GATE-MÃE: o que se digita é o total, e é exactamente isso que a peça anda.**
///
/// ⚠️ **A fixture começa com o ponteiro já a ter aplicado 0,37**, e é o que torna este gate uma
/// medição em vez de uma tautologia: se o número digitado fosse tratado como um **incremento**, o
/// mundo teria recebido 0,87 — e a peça pararia num sítio que o artista não pediu, com a ficha a
/// dizer o número certo.
#[test]
fn a_typed_number_is_the_total_not_one_more_step() {
    holding(Handle::Axis(0), Motion::Translate([0.37, 0.0, 0.0]));
    type_in("0.5");
    match banked().expect("alguma coisa foi ao mundo") {
        Motion::Translate(d) => {
            assert!(
                (d[0] - 0.5).abs() < 1e-5,
                "o total tem de ser 0,5 — o mundo recebeu {}",
                d[0]
            );
            assert!(
                d[1].abs() < 1e-6 && d[2].abs() < 1e-6,
                "e só no eixo da alça"
            );
        }
        other => panic!("o verbo mudou: {other:?}"),
    }
}

/// ⭐ **Com um número em cima da mesa, o rato CEDE.**
///
/// ⚠️ Sem esta lei o gesto seria inutilizável e o sintoma leria como *"digitar não faz nada"*: o
/// dedo nunca está completamente parado, e o movimento seguinte sobrescreveria o número no quadro a
/// seguir a alguém o escrever.
#[test]
fn the_pointer_gives_way_while_a_number_is_open() {
    holding(Handle::Axis(0), Motion::Translate([0.0; 3]));
    type_in("2");
    let after_typing = banked();
    armed(|s| {
        crate::field3d_input::advance(s, 600.0, 300.0);
    });
    assert_eq!(
        banked(),
        after_typing,
        "o ponteiro mexeu-se 200 px e não pode ter mudado nada"
    );
}

/// ⭐ **`Esc` põe a peça de volta onde ela estava** — o gesto inteiro, não a última tecla.
///
/// ⚠️ O inverso é escrito com a **própria álgebra** (`neutral().since(applied)`): uma segunda conta
/// de *"como se desfaz um giro"* divergiria da primeira no dia em que um verbo novo entrasse.
#[test]
fn escape_puts_the_piece_back_where_it_was() {
    holding(Handle::Axis(1), Motion::Translate([0.0; 3]));
    type_in("1.5");
    armed(|s| crate::field3d_input::typed_key(s, Stroke::Cancel));

    match banked().expect("o desfazer também vai pelo mesmo cano") {
        Motion::Translate(d) => assert!(
            d.iter().all(|v| v.abs() < 1e-5),
            "a soma do gesto tem de voltar a zero; ficou {d:?}"
        ),
        other => panic!("o verbo mudou: {other:?}"),
    }
    armed(|s| {
        assert!(s.drag.is_none(), "e o gesto acabou");
        assert!(s.typed.is_none(), "…e a entrada com ele");
    });
}

/// **Uma tecla numérica sem alça agarrada NÃO é deste módulo** — ela tem de passar adiante.
///
/// ⚠️ É a guarda que separa um atalho de um sequestro: sem ela, escrever `5` num campo de texto do
/// app com o rato por cima da janela 3D perderia o cinco.
#[test]
fn a_number_is_only_taken_while_a_handle_is_held() {
    armed(|s| {
        s.drag = None;
        s.typed = None;
        assert!(
            !crate::field3d_input::typed_key(s, Stroke::Digit(5)),
            "sem alça agarrada a tecla é de outra pessoa"
        );
        // …e nem sequer com um gesto de CÂMERA em curso.
        s.drag = Some(Drag::Orbit);
        assert!(!crate::field3d_input::typed_key(s, Stroke::Digit(5)));
        // …nem numa alça onde um número não tem significado.
        s.drag = Some(Drag::Gizmo(Handle::Plane(2)));
        assert!(!crate::field3d_input::typed_key(s, Stroke::Digit(5)));
    });
}

/// ⭐ **As unidades são as da FICHA** — e é o gate que impede a segunda verdade.
///
/// ⚠️ A ficha escreve graus numa argola e um fator no punho ([`crate::field3d_gizmo_paint`]). Um
/// número que se digita em radianos e se lê em graus seria invisível até alguém medir a peça.
#[test]
fn the_typed_number_speaks_the_units_of_the_readout() {
    let anchor = Anchor {
        entity: ENTITY,
        origin: [0.0; 3],
        axes: AXES,
    };
    let fwd = [0.0, 0.0, -1.0];

    match total(Handle::Axis(2), &anchor, fwd, 0.25) {
        Some(Motion::Translate(d)) => assert!((d[2] - 0.25).abs() < 1e-6, "unidades de mundo"),
        other => panic!("{other:?}"),
    }
    match total(Handle::Ring(1), &anchor, fwd, 90.0) {
        Some(Motion::Rotate { angle, axis }) => {
            assert!(
                (angle - std::f32::consts::FRAC_PI_2).abs() < 1e-5,
                "90 digitado tem de ser 90 GRAUS, e deu {} rad",
                angle
            );
            assert_eq!(axis, AXES[1]);
        }
        other => panic!("{other:?}"),
    }
    match total(Handle::Grip, &anchor, fwd, 2.0) {
        Some(Motion::Scale(f)) => assert!((f - 2.0).abs() < 1e-6, "o punho fala em FATOR"),
        other => panic!("{other:?}"),
    }
    // ⛔ E um fator não-positivo não é um tamanho.
    assert!(total(Handle::Grip, &anchor, fwd, 0.0).is_none());
    assert!(total(Handle::Grip, &anchor, fwd, -1.0).is_none());
}

/// **As alças em que um número não tem UM significado recusam-no.**
#[test]
fn a_plane_handle_takes_no_number() {
    for h in [
        Handle::Plane(0),
        Handle::Plane(1),
        Handle::Plane(2),
        Handle::View,
    ] {
        assert!(!accepts(h), "{h:?} não pode aceitar um número sozinho");
    }
    for h in [
        Handle::Axis(0),
        Handle::Ring(2),
        Handle::ViewRing,
        Handle::Grip,
    ] {
        assert!(accepts(h), "{h:?} tem um significado só — tem de aceitar");
    }
}

/// **A entrada edita-se como um campo numérico** — e sai sozinha quando fica vazia.
#[test]
fn the_entry_edits_like_a_number_field() {
    assert_eq!(edit("", Stroke::Digit(4)).as_deref(), Some("4"));
    assert_eq!(edit("4", Stroke::Dot).as_deref(), Some("4."));
    // Um segundo ponto não é um número.
    assert_eq!(edit("4.2", Stroke::Dot).as_deref(), Some("4.2"));
    // O ponto sozinho ganha o zero que o torna legível (e parseável).
    assert_eq!(edit("", Stroke::Dot).as_deref(), Some("0."));
    // O sinal TROCA — não escreve um traço.
    assert_eq!(edit("4.2", Stroke::Sign).as_deref(), Some("-4.2"));
    assert_eq!(edit("-4.2", Stroke::Sign).as_deref(), Some("4.2"));
    // Apagar o último caractere SAI da entrada: um campo vazio com o rato mudo prendia o gesto.
    assert_eq!(edit("4", Stroke::Backspace), None);
    assert_eq!(edit("42", Stroke::Backspace).as_deref(), Some("4"));

    // E um texto que ainda não é um número não manda nada ao mundo.
    assert_eq!(value_of(""), None);
    assert_eq!(value_of("-"), None);
    assert_eq!(value_of("0.5"), Some(0.5));
    assert_eq!(value_of("-2"), Some(-2.0));
}

/// **A ficha mostra o TEXTO com a unidade da alça** — enquanto se escreve, não há valor.
#[test]
fn the_readout_shows_what_is_being_written() {
    assert_eq!(label(Handle::Axis(0), "0."), "X 0.");
    assert_eq!(label(Handle::Axis(2), "-1"), "Z -1");
    assert_eq!(label(Handle::Ring(1), "-"), "-°");
    assert_eq!(label(Handle::Grip, "2"), "x 2");
}
