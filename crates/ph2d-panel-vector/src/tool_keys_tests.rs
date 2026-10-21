use super::*;

/// ⭐ **Toda ferramenta tem a tecla do Illustrator — salvo as quatro que ele não tem.**
#[test]
fn every_tool_has_its_illustrator_key_or_is_keyless_by_decision() {
    for &m in DrawMode::ALL {
        let keyless = KEYLESS.contains(&m);
        match (chord_for_mode(m), keyless) {
            (Some(_), false) | (None, true) => {}
            (Some(c), true) => panic!("{m:?} ganhou a tecla {} fora do idioma", c.label()),
            (None, false) => panic!("{m:?} ficou sem tecla"),
        }
    }
}

/// ⭐ **Nenhuma tecla tem dois donos.**
#[test]
fn no_chord_has_two_owners() {
    for (i, (a, _)) in KEYS.iter().enumerate() {
        for (b, _) in &KEYS[i + 1..] {
            assert_ne!(a, b, "a tecla {} está duas vezes na tabela", a.label());
        }
    }
}

/// O estudo §5, linha a linha — a tabela é a fonte, e esta é a prova de que diz o que o dono decidiu.
#[test]
fn the_table_is_the_one_the_owner_decided() {
    let k = |l| action_for(Chord::key(l));
    assert_eq!(k('v'), Some(KeyAction::Mode(DrawMode::Select)));
    assert_eq!(k('a'), Some(KeyAction::Mode(DrawMode::Node)));
    assert_eq!(k('p'), Some(KeyAction::Mode(DrawMode::Pen)));
    assert_eq!(k('n'), Some(KeyAction::Mode(DrawMode::Pencil)));
    assert_eq!(k('m'), Some(KeyAction::ShapeRect));
    assert_eq!(action_for(Chord::shift('l')), Some(KeyAction::ShapeEllipse));
    assert_eq!(k('t'), Some(KeyAction::Mode(DrawMode::Text)));
    assert_eq!(k('k'), Some(KeyAction::Mode(DrawMode::Bucket)));
    assert_eq!(k('w'), Some(KeyAction::Mode(DrawMode::PickBlend)));
    assert_eq!(k('c'), Some(KeyAction::Mode(DrawMode::Cut)));
    assert_eq!(k('i'), Some(KeyAction::Mode(DrawMode::Eyedropper)));
    assert_eq!(k('x'), Some(KeyAction::ToggleFillStroke));
    assert_eq!(k('d'), Some(KeyAction::DefaultColors));
    let s = |l| action_for(Chord::shift(l));
    assert_eq!(s('m'), Some(KeyAction::Mode(DrawMode::Build)));
    assert_eq!(s('w'), Some(KeyAction::Mode(DrawMode::Width)));
    assert_eq!(s('o'), Some(KeyAction::Mode(DrawMode::Frame)));
    assert_eq!(s('x'), Some(KeyAction::SwapFillStroke));
    let b = |l| action_for(Chord::ctrl_alt(l));
    assert_eq!(b('u'), Some(KeyAction::Union));
    assert_eq!(b('i'), Some(KeyAction::Intersect));
    assert_eq!(b('d'), Some(KeyAction::Subtract));
    assert_eq!(b('x'), Some(KeyAction::Exclude));
    // ⛔ As booleanas SAÍRAM das letras soltas: `U` não é nada, e `I`/`D`/`X` são do idioma.
    assert_eq!(k('u'), None);
}

/// As teclas que já eram de outra coisa no vetor não podem ser roubadas (`Tab` = modo,
/// `]`/`[` = nós): a tabela só tem letras.
#[test]
fn the_table_only_holds_letters() {
    assert!(KEYS.iter().all(|(c, _)| c.letter.is_ascii_lowercase()));
}

/// ⛔ **O `L` é da Timeline em TODO modo** (dono, 21/10: *«L é o atalho da timeline, mas no modo
/// vector tem um conflito com L»*): a cadeia do vetor corre ANTES do atalho global e consumia-o
/// (era a Elipse). Nenhuma tecla do vetor o pode tomar, com ou sem modificador que o global ignore.
#[test]
fn o_l_da_timeline_nao_tem_dono_no_vetor() {
    assert_eq!(action_for(Chord::key('l')), None);
    assert_eq!(action_for(Chord::key('L')), None);
}
