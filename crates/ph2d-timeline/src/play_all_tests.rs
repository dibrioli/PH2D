use super::*;

fn relogio() -> Playhead {
    Playhead::new(1.0 / 60.0)
}

/// Ligar toca a cena; o mesmo pedido outra vez pausa e desliga; o relógio parado por outra via
/// (o Pause) acaba-o. (Mutações: o toggle sem o `pause`; o fim sem o relógio ⇒ RED.)
#[test]
fn o_play_all_liga_desliga_e_acaba_com_o_relogio() {
    let (mut st, mut cena) = (TimelineState::new(), relogio());
    let mut pedido = vec![TimelineIntent::PlayAll, TimelineIntent::Scrub(1.0)];
    take_play_all(&mut st, &mut pedido, &mut cena);
    assert!(st.play_all && cena.is_playing());
    assert_eq!(
        pedido,
        [TimelineIntent::Scrub(1.0)],
        "drenado; o resto fica"
    );

    take_play_all(&mut st, &mut vec![TimelineIntent::PlayAll], &mut cena);
    assert!(
        !st.play_all && !cena.is_playing(),
        "o mesmo botão desliga e pausa"
    );

    take_play_all(&mut st, &mut vec![TimelineIntent::PlayAll], &mut cena);
    cena.pause();
    take_play_all(&mut st, &mut Vec::new(), &mut cena);
    assert!(!st.play_all, "pausado por outra via: acabou");
}
