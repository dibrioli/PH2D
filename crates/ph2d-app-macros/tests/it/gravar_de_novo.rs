//! **Gravar de novo toca no Play de cima** (smoke do dono, 18/10, `docs/Macros/handoffs/
//! HANDOFF_CONTINUACAO_W9_SMOKE_REPROVADO_2026-10-18.md`): o Play da barra de cima corre a cena e só toca os
//! clipes em Autoplay. A 1.ª gravação de um objecto sem outra entrada cria o clipe em Autoplay; uma
//! gravação sobre um objecto que já tem o clipe da macro re-cronometrava-o e deixava-lhe o Autoplay de
//! antes — desligado (a cena `=5`, ou o artista que o desligou), o Play de cima não tocava a gravação
//! nova e a tela mostrava a imagem final: «os dois traços aparecem de uma vez».
//!
//! **A entrada que o artista marcou fica** (decisão de 18/10, handoff W9 §8 item 4): o clipe da macro só
//! ganha o Autoplay se nenhum OUTRO clipe do objecto já é a entrada — na 1.ª tomada e nas seguintes, no
//! Painter e no Motion (as duas passam pelo mesmo `instala`).

use crate::gate7::Banca;
use ph2d_panel_macros::ids;

/// Gravar ▸ duas pinceladas ▸ Parar, pelo painel.
fn grava_duas_pinceladas(b: &mut Banca) {
    b.pinta();
    b.carrega(ids::MACROS_RECORD);
    b.quadro();
    b.pincelada([20.0, 30.0], [100.0, 30.0]);
    b.pincelada([20.0, 80.0], [100.0, 80.0]);
    b.pinta();
    b.carrega(ids::MACROS_STOP);
    b.quadro();
    assert_eq!(b.m.gravando(), None);
}

fn clipes_de(b: &Banca, dono: u64) -> usize {
    b.tl.doc
        .clips()
        .iter()
        .filter(|c| c.owner.is_some_and(|o| o.entity == dono))
        .count()
}

/// Um clipe «Idle» do objecto, marcado como a entrada pelo artista.
fn idle_em_autoplay(b: &mut Banca, dono: u64) -> usize {
    let idle = b.tl.doc.add_clip_for("Idle".into(), dono).expect("o Idle");
    b.tl.doc.set_autoplay(idle, true);
    assert_eq!(b.tl.doc.entry_clip_of(dono), Some(idle));
    idle
}

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

    grava_duas_pinceladas(&mut b);

    assert_eq!(
        clipes_de(&b, b.bits),
        1,
        "gravar de novo re-cronometra o mesmo clipe, não cria outro"
    );
    assert!(
        b.tl.doc.autoplay_entries().contains(&(b.bits, clip)),
        "a gravação nova tem de tocar no Play de cima, como a 1.ª: o clipe sai do Parar em Autoplay"
    );
}

#[test]
fn gravar_de_novo_nao_rouba_a_entrada_que_o_artista_marcou_noutro_clipe() {
    let mut b = Banca::nova();
    let clip =
        b.tl.doc
            .add_macro_clip("Macro".into(), b.bits, 5.0)
            .expect("o clipe da tomada anterior");
    let dono = b.bits;
    let idle = idle_em_autoplay(&mut b, dono);

    grava_duas_pinceladas(&mut b);

    assert_eq!(
        clipes_de(&b, b.bits),
        2,
        "gravar de novo re-cronometra o mesmo clipe, não cria outro"
    );
    assert_eq!(
        b.tl.doc.entry_clip_of(b.bits),
        Some(idle),
        "o Idle que o artista marcou continua a entrada; o clipe da macro ({clip}) não lha rouba"
    );
}

#[test]
fn a_primeira_gravacao_nao_rouba_a_entrada_que_o_artista_marcou() {
    let mut b = Banca::nova();
    let dono = b.bits;
    let idle = idle_em_autoplay(&mut b, dono);

    grava_duas_pinceladas(&mut b);

    assert_eq!(clipes_de(&b, b.bits), 2, "o Parar criou o clipe da macro");
    assert_eq!(b.tl.doc.entry_clip_of(b.bits), Some(idle));
}

#[test]
fn gravar_no_motion_nao_rouba_a_entrada_que_o_artista_marcou() {
    let mut b = Banca::nova();
    let dono = b.objecto_motion(77);
    let idle = idle_em_autoplay(&mut b, dono);

    b.grava_um_passo_do_motion();
    assert_eq!(clipes_de(&b, dono), 2, "o Parar criou o clipe da macro");
    assert_eq!(b.tl.doc.entry_clip_of(dono), Some(idle), "1.ª tomada");

    b.grava_um_passo_do_motion();
    assert_eq!(clipes_de(&b, dono), 2);
    assert_eq!(b.tl.doc.entry_clip_of(dono), Some(idle), "2.ª tomada");
}

#[test]
fn sem_outra_entrada_o_motion_tambem_sai_do_parar_em_autoplay() {
    let mut b = Banca::nova();
    let dono = b.objecto_motion(77);
    for tomada in 1..=2 {
        b.grava_um_passo_do_motion();
        let clip = ph2d_app_macros::instala::clipe_de(&b.tl.doc, dono).expect("o clipe da macro");
        if tomada == 1 {
            // O artista desliga-o entre as tomadas, como a cena `=5`.
            assert_eq!(b.tl.doc.entry_clip_of(dono), Some(clip), "1.ª tomada");
            b.tl.doc.set_autoplay(clip, false);
        } else {
            assert_eq!(b.tl.doc.entry_clip_of(dono), Some(clip), "2.ª tomada");
        }
    }
}

/// O clipe da macro RENOMEADO (o artista dá-lhe outro nome) continua a tocar a macro do objecto: não é
/// uma entrada «do artista» — o Parar seguinte cria o clipe novo e passa-lhe o Autoplay, para o Play de
/// cima tocar a gravação nova com o tempo dela (auditoria do fecho de 18/10).
#[test]
fn o_clipe_da_macro_renomeado_nao_segura_a_entrada_contra_a_gravacao_nova() {
    let mut b = Banca::nova();
    grava_duas_pinceladas(&mut b);
    let velho =
        ph2d_app_macros::instala::clipe_de(&b.tl.doc, b.bits).expect("o clipe da 1.ª tomada");
    assert_eq!(b.tl.doc.entry_clip_of(b.bits), Some(velho));
    b.tl.doc.rename_clip(velho, "Take 1".into());

    grava_duas_pinceladas(&mut b);

    let novo =
        ph2d_app_macros::instala::clipe_de(&b.tl.doc, b.bits).expect("o clipe da 2.ª tomada");
    assert_ne!(novo, velho);
    assert_eq!(
        b.tl.doc.entry_clip_of(b.bits),
        Some(novo),
        "a tomada renomeada é da macro, não do artista: a gravação nova é a entrada"
    );
}
