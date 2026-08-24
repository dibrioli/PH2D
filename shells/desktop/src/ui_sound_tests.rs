//! Os gates do **SOM DE UI** — a lei, e o default que a protege.

use crate::prefs::{Prefs, parse, serialize};

/// ⛔ **NASCE DESLIGADO, e o default é a FEATURE.**
///
/// ⚠️ Um app de desenho vive em cima de música, de referências em vídeo e de chamadas. Um som que
/// ninguém pediu desliga-se no primeiro minuto — e leva a feature com ele. O estudo escreve o ⛔ ao
/// lado do item, e este gate é ele.
#[test]
fn the_ui_sound_is_born_off() {
    assert!(!Prefs::default().ui_sound, "a PREFERÊNCIA nasceu ligada");
    // ⚠️ **E o portador VIVO também**, que é uma segunda pergunta: a preferência é o que o disco
    // guarda, o `HeroScreen` é o que o app usa. Eles são semeados um do outro no arranque — mas
    // antes disso (e com `$HOME` por definir, quando a semente é o próprio default) quem responde
    // é o vivo. *Dois defaults para o mesmo facto é um deles a divergir em silêncio.*
    assert!(
        !ph2d_editor::HeroScreen::new(ph2d_editor::NodeId(1)).ui_sound,
        "o portador VIVO nasceu ligado"
    );
    // E um ficheiro que não o menciona deixa-o desligado — a tolerância que dispensa a versão.
    assert!(!parse("motion_character=discreet\nreduced_motion=1\n").ui_sound);
}

/// **A preferência ATRAVESSA o ficheiro, nos dois sentidos.**
///
/// ⚠️ O round-trip com valor **não-default** é a lição que o `ObjectPose` pagou hoje: um campo
/// gravado com o próprio default é indistinguível de um campo que não viaja.
#[test]
fn the_preference_travels_both_ways() {
    let on = Prefs {
        ui_sound: true,
        ..Prefs::default()
    };
    assert!(
        parse(&serialize(&on)).ui_sound,
        "o som ligado não sobreviveu ao ficheiro"
    );
    let off = Prefs::default();
    assert!(!parse(&serialize(&off)).ui_sound);
}

/// ⭐⭐ **AS QUATRO VOZES SÃO DISTINGUÍVEIS, e todas cabem no gesto.**
///
/// ⚠️ As duas metades são a lei do vocabulário: dois sons iguais não são dois sons (o ouvido não os
/// separa, e o vocabulário mente sobre o seu tamanho), e um som mais longo que o gesto **chega
/// depois dele** — deixa de confirmar e passa a comentar.
#[test]
fn the_four_voices_are_distinct_and_shorter_than_the_gesture() {
    use crate::ui_sound::UiSound::{Click, Commit, Refuse, Toggle};
    let all = [Click, Toggle, Commit, Refuse];
    for (i, a) in all.iter().enumerate() {
        let (fa, secs, gain) = a.voice();
        assert!(
            secs <= 0.09,
            "{a:?} dura {secs}s — um som de UI que passa dos 90 ms chega depois do gesto"
        );
        assert!(gain > 0.0 && gain <= 0.25, "{a:?} tem ganho {gain}");
        for b in all.iter().skip(i + 1) {
            assert!(
                (fa - b.voice().0).abs() > 20.0,
                "{a:?} e {b:?} soam à mesma frequência — o vocabulário mente sobre o seu tamanho"
            );
        }
    }
}

/// ⛔ **A RECUSA É A ÚNICA VOZ QUE DESCE.**
///
/// ⚠️ Ela é o som mais importante dos quatro: uma recusa **silenciosa** lê-se como um clique que
/// não funcionou, e o artista repete o gesto em vez de procurar o motivo. E ela tem de ser
/// reconhecível sem a aprender — grave, quando os outros três são agudos.
#[test]
fn the_refusal_is_the_only_voice_that_goes_down() {
    use crate::ui_sound::UiSound::{Click, Commit, Refuse, Toggle};
    let low = Refuse.voice().0;
    for other in [Click, Toggle, Commit] {
        assert!(
            other.voice().0 > low * 2.0,
            "{other:?} soa perto da recusa — o «não» deixa de se reconhecer sem o aprender"
        );
    }
}
