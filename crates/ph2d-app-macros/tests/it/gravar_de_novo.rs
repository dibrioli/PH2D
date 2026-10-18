//! **Gravar de novo toca no Play de cima** (smoke do dono, 18/10, `docs/Macros/handoffs/
//! HANDOFF_CONTINUACAO_W9_SMOKE_REPROVADO_2026-10-18.md`): o Play da barra de cima corre a cena e só toca os
//! clipes em Autoplay. A 1.ª gravação de um objecto cria o clipe em Autoplay; uma gravação sobre um objecto
//! que já tem o clipe da macro re-cronometrava-o e deixava-lhe o Autoplay de antes — desligado (a cena
//! `=5`, ou o artista que o desligou), o Play de cima não tocava a gravação nova e a tela mostrava a imagem
//! final: «os dois traços aparecem de uma vez». O Parar dá SEMPRE o mesmo clipe, haja tomada antes ou não.

use crate::gate7::Banca;
use ph2d_panel_macros::ids;

#[test]
fn parar_por_cima_de_um_clipe_sem_autoplay_deixa_a_gravacao_nova_a_tocar_no_play_de_cima() {
    let mut b = Banca::nova();
    // O objecto já tem o clipe da macro, sem Autoplay — como a cena `=5` o monta.
    let clip =
        b.tl.doc
            .add_macro_clip("Macro".into(), b.bits, 5.0)
            .expect("o clipe da tomada anterior");
    b.tl.doc.set_autoplay(clip, false);
    assert!(!b.tl.doc.autoplay_entries().contains(&(b.bits, clip)));

    b.pinta();
    b.carrega(ids::MACROS_RECORD);
    b.quadro();
    b.pincelada([20.0, 30.0], [100.0, 30.0]);
    b.pincelada([20.0, 80.0], [100.0, 80.0]);
    b.pinta();
    b.carrega(ids::MACROS_STOP);
    b.quadro();

    assert_eq!(b.m.gravando(), None);
    assert_eq!(
        b.tl.doc
            .clips()
            .iter()
            .filter(|c| c.owner.is_some_and(|o| o.entity == b.bits))
            .count(),
        1,
        "gravar de novo re-cronometra o mesmo clipe, não cria outro"
    );
    assert!(
        b.tl.doc.autoplay_entries().contains(&(b.bits, clip)),
        "a gravação nova tem de tocar no Play de cima, como a 1.ª: o clipe sai do Parar em Autoplay"
    );
}
