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

/// B14c-cont3: a tela e as leituras da silhueta base `(avaliadas, as que deram 0)`, com ou sem o corte além de `t = 1`.
fn pinta_contando(sem_corte: bool, pincel: &str, tamanho: f32) -> (Vec<u8>, [u64; 2]) {
    super::SEM_CORTE_FORA.with(|s| s.set(sem_corte));
    crate::wet_diag::CONTA_SIL.store(true, std::sync::atomic::Ordering::Relaxed);
    let _ = crate::wet_diag::take_sil();
    let (px, _) = pinta(false, pincel, tamanho);
    let s = crate::wet_diag::take_sil();
    crate::wet_diag::CONTA_SIL.store(false, std::sync::atomic::Ordering::Relaxed);
    super::SEM_CORTE_FORA.with(|s| s.set(false));
    (px, [s[2], s[5]])
}

#[test]
fn a_silhueta_alem_da_pegada_nao_e_avaliada_e_a_tela_e_a_do_oraculo_ao_byte() {
    for (pincel, tamanho) in [
        ("Oil/Flat/Flat Bristle.ph2d-brush.png", 70.0),
        ("Oil/Flat/Thick Impasto.ph2d-brush.png", 60.0),
    ] {
        let (velho, [n0, z0]) = pinta_contando(true, pincel, tamanho);
        let (novo, [n1, z1]) = pinta_contando(false, pincel, tamanho);
        assert!(
            velho.iter().any(|&b| b != 255),
            "{pincel}: o traço não pintou"
        );
        assert!(
            velho == novo,
            "{pincel}: a tela mudou com o corte além da pegada"
        );
        assert!(
            z0 * 5 > n0,
            "{pincel}: o oráculo não exerce o corte ({z0} zeros em {n0} avaliações)"
        );
        assert!(
            z1 * 4 < z0 && n1 + z0 / 2 < n0,
            "{pincel}: a silhueta base ainda é avaliada além da pegada — {n0} → {n1} avaliações, zeros {z0} → {z1}"
        );
    }
}
