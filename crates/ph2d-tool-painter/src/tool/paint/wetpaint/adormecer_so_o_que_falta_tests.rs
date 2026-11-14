//! **Sair do Wet com uma imagem no ar compõe só o que falta** (dono 06/11, [onda 2](../../../../../../../docs/Painter/handoffs/HANDOFF_CONTINUACAO_2026-11-06_tinta_viva_onda2.md)):
//! a tela inteira custava 1,6 s a 4K e re-iluminava a orla das células enterradas.

use super::borracha_defeitos_tests::{Caminho, oleo, quadros};
use super::composite_contagem as contagem;
use super::otica_no_tique_tests::{assenta, cena, confere_com_casa, tique};
use super::*;
use crate::tool::paint::PaintMedia;
use std::cell::Cell;
use std::time::Duration;

thread_local! {
    /// `true` = a porta de antes: recompõe a tela inteira (a régua e o controlo).
    pub(super) static SAI_INTEIRA: Cell<bool> = const { Cell::new(false) };
}

/// Uma imagem no ar, servida: a dívida de uma célula faz o tique pedir e o worker tem tempo de a levar.
fn imagem_no_ar(t: &mut PainterTool) {
    let s = t.paint.wetpaint.session.as_mut().expect("sessão");
    s.bring_home();
    s.otica.divida = vec![(1, 1, 1, 1)];
    tique(t);
    let s = t.paint.wetpaint.session.as_ref().expect("sessão");
    assert!(s.otica.pendente(), "premissa: o pedido saiu");
    std::thread::sleep(Duration::from_millis(150));
}

/// O portão de custo, por caixa: a saída compõe o sujo, e não as células todas da grade.
#[test]
fn sair_com_imagem_no_ar_nao_recompoe_a_tela_inteira() {
    let mut t = cena(true, 0.0, false);
    quadros(&mut t, Caminho::App, 10);
    assenta(&mut t);
    let (gw, gh) = t.paint.wetpaint.session.as_ref().expect("sessão").grid;
    imagem_no_ar(&mut t);
    let _ = contagem::take();
    t.set_paint_media(PaintMedia::Digital);
    let (n, motor, _) = contagem::take();
    assert_eq!(n, 1, "a saída compõe uma vez");
    assert!(
        motor < (gw * gh / 16) as u64,
        "a saída compôs {motor} de {} células",
        gw * gh
    );
    assert_eq!(t.paint.wetpaint.adormecidas.len(), 1, "premissa: adormeceu");

    // O controlo: a porta de antes compõe a grade toda, e é isto que o portão reprova.
    let mut t = cena(true, 0.0, false);
    quadros(&mut t, Caminho::App, 10);
    assenta(&mut t);
    imagem_no_ar(&mut t);
    let _ = contagem::take();
    SAI_INTEIRA.set(true);
    t.set_paint_media(PaintMedia::Digital);
    SAI_INTEIRA.set(false);
    assert_eq!(contagem::take().1, (gw * gh) as u64, "o controlo");
}

/// O que a imagem levou do motor chega à tela sem ela: a saída é a composição inteira, ao byte.
#[test]
fn sair_com_imagem_no_ar_deixa_a_tela_da_composicao_inteira() {
    // A água a correr, à saída; e o motor mudado sem compor, com tudo no sujo que a imagem levou.
    for mudo in [false, true] {
        let mut t = cena(true, 0.0, false);
        if mudo {
            quadros(&mut t, Caminho::App, 10);
            assenta(&mut t);
            let s = t.paint.wetpaint.session.as_mut().expect("sessão");
            s.bring_home();
            s.engine.fast_dry_now();
            imagem_no_ar(&mut t);
        } else {
            let mut n = 0;
            while !t
                .paint
                .wetpaint
                .session
                .as_ref()
                .is_some_and(|s| s.otica.pendente())
            {
                tique(&mut t);
                std::thread::sleep(Duration::from_millis(2));
                n += 1;
                assert!(n < 2000, "premissa: nenhum pedido saiu");
            }
        }
        t.set_paint_media(PaintMedia::Digital);
        assert_eq!(t.paint.wetpaint.adormecidas.len(), 1, "premissa: adormeceu");
        t.set_paint_media(PaintMedia::WetPaint);
        t.wet_acorda();
        confere_com_casa(&mut t, if mudo { "motor mudo" } else { "água a correr" });
    }
}

/// Com células enterradas a tela inteira re-ilumina a orla: a saída não muda nem um píxel.
#[test]
fn sair_com_imagem_no_ar_nao_reilumina_o_enterrado() {
    let mudados = |inteira: bool| {
        let mut t = oleo(240, 160, Caminho::App);
        t.set_paint_media(PaintMedia::Digital);
        super::adormecida_tests::traco_de_outro_meio(&mut t, (130.0, 60.0), (130.0, 120.0));
        t.set_paint_media(PaintMedia::WetPaint);
        t.wet_acorda();
        quadros(&mut t, Caminho::App, 30);
        imagem_no_ar(&mut t);
        let antes = t.canvas_rgba.to_vec();
        SAI_INTEIRA.set(inteira);
        t.set_paint_media(PaintMedia::Digital);
        SAI_INTEIRA.set(false);
        antes
            .chunks(4)
            .zip(t.canvas_rgba.chunks(4))
            .filter(|(a, b)| a != b)
            .count()
    };
    assert!(mudados(true) > 0, "o controlo: a tela inteira re-ilumina");
    assert_eq!(mudados(false), 0);
}
