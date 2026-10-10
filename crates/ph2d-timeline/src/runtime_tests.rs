use super::*;

/// Um relógio PARADO — o `Playhead::new` nasce a tocar, e um teste do «Play põe a tocar» sobre ele
/// não distinguiria nada (a mutação que tirava o `play` sobreviveu assim).
fn relogio() -> Playhead {
    let mut p = Playhead::new(1.0 / 60.0);
    p.pause();
    p
}

/// O Play de cima liga a corrida e toca a cena; o Pause de cima (só o relógio) não a desliga; o Reset
/// de cima desliga-a. (Mutações: `RunScene` sem `play`; o fim pelo relógio ⇒ RED.)
#[test]
fn o_play_de_cima_corre_a_cena_e_o_reset_sai() {
    let (mut st, mut cena) = (TimelineState::new(), relogio());
    let mut pedido = vec![TimelineIntent::RunScene, TimelineIntent::Scrub(1.0)];
    take_runtime(&mut st, &mut pedido, &mut cena);
    assert!(st.runtime && cena.is_playing());
    assert_eq!(
        pedido,
        [TimelineIntent::Scrub(1.0)],
        "drenado; o resto fica"
    );

    cena.pause();
    take_runtime(&mut st, &mut Vec::new(), &mut cena);
    assert!(st.runtime, "o Pause de cima não sai da corrida");

    take_runtime(&mut st, &mut vec![TimelineIntent::StopScene], &mut cena);
    assert!(!st.runtime, "o Reset de cima sai");
}

/// ⭐ O transporte da TIMELINE é de edição: com a cena a correr, o ▶/⏸ dela sai da corrida, pára o
/// relógio da cena e segue para a vista. (Mutação: sem a saída ⇒ RED.)
#[test]
fn o_transporte_da_timeline_sai_da_corrida() {
    let (mut st, mut cena) = (TimelineState::new(), relogio());
    take_runtime(&mut st, &mut vec![TimelineIntent::RunScene], &mut cena);
    let mut edita = vec![TimelineIntent::TogglePlay];
    take_runtime(&mut st, &mut edita, &mut cena);
    assert!(!st.runtime && !cena.is_playing());
    assert_eq!(
        edita,
        [TimelineIntent::TogglePlay],
        "a vista recebe o seu Play"
    );
}
