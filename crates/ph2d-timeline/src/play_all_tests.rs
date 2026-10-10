use super::*;

fn relogio() -> Playhead {
    Playhead::new(1.0 / 60.0)
}

/// Marcar toca a cena; o Pause NÃO o desmarca (é um modo — 2.º smoke do dono); o mesmo botão
/// desmarca e pausa. (Mutações: o fim pelo relógio de volta; o desmarcar sem `pause` ⇒ RED.)
#[test]
fn o_play_all_e_um_modo_que_so_o_botao_desmarca() {
    let (mut st, mut cena) = (TimelineState::new(), relogio());
    let mut pedido = vec![TimelineIntent::PlayAll, TimelineIntent::Scrub(1.0)];
    take_play_all(&mut st, &mut pedido, &mut cena);
    assert!(st.play_all && cena.is_playing());
    assert_eq!(
        pedido,
        [TimelineIntent::Scrub(1.0)],
        "drenado; o resto fica"
    );

    cena.pause();
    take_play_all(&mut st, &mut Vec::new(), &mut cena);
    assert!(st.play_all, "o Pause não desmarca o modo");
    cena.play();

    take_play_all(&mut st, &mut vec![TimelineIntent::PlayAll], &mut cena);
    assert!(
        !st.play_all && !cena.is_playing(),
        "o mesmo botão desmarca e pausa"
    );
}
