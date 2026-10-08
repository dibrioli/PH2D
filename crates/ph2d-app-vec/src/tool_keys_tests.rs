use super::*;

fn c(l: char) -> Chord {
    Chord {
        letter: l,
        shift: false,
        ctrl_alt: false,
    }
}

#[test]
fn a_letter_becomes_a_click_on_the_column_button() {
    let mut hold = None;
    let out = key(&mut hold, c('p'), true, false, DrawMode::Select);
    assert_eq!(out, Some(KeyOutcome::Clicks(vec![ids::VECTOR_MODE_PEN])));
}

/// ⭐ Um TOQUE troca de vez: soltar sem ter repetido não volta atrás.
#[test]
fn a_tap_switches_for_good() {
    let mut hold = None;
    key(&mut hold, c('p'), true, false, DrawMode::Select);
    let out = key(&mut hold, c('p'), false, false, DrawMode::Pen);
    assert_eq!(out, Some(KeyOutcome::Swallow));
    assert!(hold.is_none());
}

/// ⭐ SEGURAR (a tecla repetiu) e soltar devolve a ferramenta de antes — a temporária.
#[test]
fn a_held_key_springs_back_on_release() {
    let mut hold = None;
    key(&mut hold, c('a'), true, false, DrawMode::Pen);
    assert_eq!(
        key(&mut hold, c('a'), true, true, DrawMode::Node),
        Some(KeyOutcome::Swallow)
    );
    let out = key(&mut hold, c('a'), false, false, DrawMode::Node);
    assert_eq!(out, Some(KeyOutcome::Clicks(vec![ids::VECTOR_MODE_PEN])));
}

/// O `M` abre a Forma com o RECTÂNGULO escolhido — dois cliques, pela ordem.
#[test]
fn m_arms_the_shape_tool_with_the_rectangle() {
    let mut hold = None;
    let Some(KeyOutcome::Clicks(cliques)) = key(&mut hold, c('m'), true, false, DrawMode::Select)
    else {
        panic!("o M tem de clicar");
    };
    assert_eq!(cliques.first(), Some(&ids::VECTOR_MODE_SHAPE));
    assert_eq!(cliques.len(), 2, "falta o rectângulo do catálogo");
}

/// As booleanas vivem em `Ctrl+Alt` e já não nas letras soltas.
#[test]
fn the_booleans_moved_to_ctrl_alt() {
    let mut hold = None;
    let u = chord(0x55, false, true, true, false).expect("Ctrl+Alt+U é do vetor");
    assert_eq!(
        key(&mut hold, u, true, false, DrawMode::Select),
        Some(KeyOutcome::Boolean(ph2d_vec_boolean::PathfinderOp::Union))
    );
    assert_eq!(chord(0x55, false, true, false, false), None, "Ctrl sozinho não é do vetor");
    assert_eq!(key(&mut hold, c('u'), true, false, DrawMode::Select), None);
}

#[test]
fn only_letters_make_chords() {
    assert!(chord(0x31, false, false, false, false).is_none(), "um dígito não é tecla do vetor");
    assert!(chord(0xF702, false, false, false, false).is_none());
}
