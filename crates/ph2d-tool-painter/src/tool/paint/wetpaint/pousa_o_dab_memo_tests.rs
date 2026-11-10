//! B14c (missão 4K): a silhueta BASE guardada à parte — a bilinear da ponta das cerdas lia-a em quatro células por
//! célula (Flat Bristle 240 a 4K: 7,11 M avaliações por dab → 1,89 M). O oráculo é o caminho de antes (sem a caixa),
//! no mesmo processo: a tela ao byte.

use super::super::agua_sobre_oleo_tests::{mostrada, ponteiro};
use super::super::undo_tinta_morre_tests as m;
use super::{MEMOS_BASE, SEM_MEMO_BASE};
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};

fn pinta(sem_memo: bool, pincel: &str, tamanho: f32) -> (Vec<u8>, usize) {
    SEM_MEMO_BASE.with(|s| s.set(sem_memo));
    MEMOS_BASE.with(|n| n.set(0));
    let mut t = m::tela(320, 160, pincel, tamanho);
    t.on_canvas_pointer(ponteiro(60.0, 80.0, PointerPhase::Down));
    for k in 1..=12 {
        t.on_canvas_pointer(ponteiro(
            60.0 + 16.0 * k as f32,
            80.0 + 3.0 * k as f32,
            PointerPhase::Move,
        ));
        t.ticks_em_blocos(1);
    }
    t.on_canvas_pointer(ponteiro(252.0, 116.0, PointerPhase::Up));
    t.ticks_em_blocos(2);
    let px = mostrada(&mut t);
    SEM_MEMO_BASE.with(|s| s.set(false));
    (px, MEMOS_BASE.with(std::cell::Cell::get))
}

#[test]
fn a_ponta_das_cerdas_com_a_silhueta_base_guardada_da_a_tela_do_oraculo_ao_byte() {
    for (pincel, tamanho) in [
        ("Oil/Flat/Flat Bristle.ph2d-brush.png", 70.0),
        ("Oil/Flat/Thick Impasto.ph2d-brush.png", 60.0),
    ] {
        let (velho, _) = pinta(true, pincel, tamanho);
        let (novo, n) = pinta(false, pincel, tamanho);
        assert!(
            n > 0,
            "{pincel}: nenhum dab com a ponta das cerdas neste fio — o caso não exerce a caixa"
        );
        assert!(
            velho.iter().any(|&b| b != 255),
            "{pincel}: o traço não pintou"
        );
        assert!(
            velho == novo,
            "{pincel}: a tela mudou com a silhueta base guardada"
        );
    }
}
