//! **A tinta viva sobrevive** ([plano](../../../../../../../docs/Painter/TINTA_VIVA_2026-11-06.md)): o caso `vivo` da Borracha
//! é a régua; sair do Wet e voltar não pode mudar nem um pixel dele.

use super::agua_sobre_oleo_tests::{H, W, com_oleo, mostrada, ponteiro, preset};
use super::borracha_de_fabrica_tests::{apaga, diferentes};
use super::*;
use crate::tool::paint::PaintMedia;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};

const OUTROS: [PaintMedia; 3] = [
    PaintMedia::Digital,
    PaintMedia::Watercolor,
    PaintMedia::Impasto,
];

/// O traço de um outro meio, de `(x0, y0)` a `(x1, y1)`, a vermelho.
pub(super) fn traco_de_outro_meio(t: &mut PainterTool, a: (f32, f32), b: (f32, f32)) {
    t.set_brush_size_px(16.0);
    t.set_brush_color_srgb8([220, 30, 30]);
    t.on_canvas_pointer(ponteiro(a.0, a.1, PointerPhase::Down));
    for k in 1..=20 {
        let f = k as f32 / 20.0;
        t.on_canvas_pointer(ponteiro(
            a.0 + (b.0 - a.0) * f,
            a.1 + (b.1 - a.1) * f,
            PointerPhase::Move,
        ));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(ponteiro(b.0, b.1, PointerPhase::Up));
    t.on_tick(1000.0 / 60.0);
}

pub(super) fn borracha_viva() -> Vec<u8> {
    let mut t = com_oleo();
    t.set_paint_tool_mode("eraser");
    apaga(&mut t)
}

pub(super) fn volta_e_apaga(t: &mut PainterTool) -> Vec<u8> {
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_paint_tool_mode("eraser");
    apaga(t)
}

/// ⭐ O defeito do dono: Dabber → outro meio → Wet → Borracha apaga como se nunca tivesse saído.
#[test]
fn sair_do_wet_e_voltar_nao_muda_a_borracha() {
    let vivo = borracha_viva();
    for meio in OUTROS {
        let mut t = com_oleo();
        t.set_paint_media(meio);
        let r = volta_e_apaga(&mut t);
        assert_eq!(diferentes(&vivo, &r), 0, "{meio:?}");
    }
}

/// Adormecer e acordar não mexem na tela.
#[test]
fn sair_e_voltar_nao_muda_a_tela() {
    let mut t = com_oleo();
    let antes = mostrada(&mut t);
    t.set_paint_media(PaintMedia::Digital);
    assert_eq!(diferentes(&antes, &mostrada(&mut t)), 0, "ao sair");
    t.set_paint_media(PaintMedia::WetPaint);
    assert_eq!(diferentes(&antes, &mostrada(&mut t)), 0, "ao voltar");
    t.ticks_em_blocos(60);
    assert_eq!(diferentes(&antes, &mostrada(&mut t)), 0, "a tinta assente");
}

/// Fora do Wet não há sessão nem fio: o tique não a vê.
#[test]
fn a_tinta_adormecida_nao_tem_sessao_nem_fio() {
    let mut t = com_oleo();
    t.set_paint_media(PaintMedia::Digital);
    assert!(t.paint.wetpaint.session.is_none());
    let a = t.paint.wetpaint.adormecidas.first().expect("adormecida");
    assert!(a.sess.lock().expect("trinco").worker.is_none());
    t.set_paint_media(PaintMedia::WetPaint);
    assert!(t.paint.wetpaint.adormecidas.is_empty());
    assert!(t.paint.wetpaint.session.is_some());
}

/// Um traço longe do óleo fica, e a Borracha sobre o óleo é a mesma.
#[test]
fn traco_longe_nao_muda_a_borracha_e_fica_na_tela() {
    let longe = ((20.0, 150.0), (220.0, 150.0));
    let mut controlo = com_oleo();
    controlo.set_paint_tool_mode("eraser");
    let vivo = apaga(&mut controlo);
    for meio in OUTROS {
        let mut t = com_oleo();
        t.set_paint_media(meio);
        traco_de_outro_meio(&mut t, longe.0, longe.1);
        let com_traco = mostrada(&mut t);
        let r = volta_e_apaga(&mut t);
        let zona =
            |p: &[u8], y0: usize, y1: usize| -> Vec<u8> { p[y0 * W * 4..y1 * W * 4].to_vec() };
        let (v, q) = (zona(&vivo, 0, 125), zona(&r, 0, 125));
        if matches!(meio, PaintMedia::Watercolor) {
            // A aguada deixa o vidro na camada: daí em diante o composite do Wet segue a lei dele (doc 48), o que
            // a sessão faria também sem adormecer. Medido: 6 574 px, Δ máx 64, 229 com Δ ≥ 10.
            let forte = v
                .chunks(4)
                .zip(q.chunks(4))
                .filter(|(a, b)| (0..4).any(|k| a[k].abs_diff(b[k]) >= 10))
                .count();
            assert!(forte < 300, "a borracha sob o vidro: {forte} px com Δ ≥ 10");
        } else {
            assert_eq!(diferentes(&v, &q), 0, "{meio:?}: a borracha sobre o óleo");
        }
        assert_eq!(
            diferentes(&zona(&com_traco, 140, H), &zona(&r, 140, H)),
            0,
            "{meio:?}: o traço longe"
        );
    }
}

const PASSADAS_ATE_O_PAPEL: usize = 8;

/// ⭐ Decisão 2 do dono: um traço por cima do óleo vai-se com o óleo. Sem resto vermelho nem azul no rasto.
#[test]
fn a_borracha_leva_o_traco_e_o_oleo_juntos() {
    let vermelho = |p: &[u8]| p[0] > 150 && p[1] < 90 && p[3] > 40;
    let azul = |p: &[u8]| p[2] > 120 && p[0] < 110 && p[3] > 40;
    let mut alfa_digital: Option<Vec<[u8; 4]>> = None;
    for meio in OUTROS {
        let mut t = com_oleo();
        t.set_paint_media(meio);
        traco_de_outro_meio(&mut t, (130.0, 60.0), (130.0, 120.0));
        let antes = mostrada(&mut t);
        let conta = |p: &[u8], f: &dyn Fn(&[u8]) -> bool| p.chunks(4).filter(|c| f(c)).count();
        assert!(conta(&antes, &vermelho) > 100, "{meio:?}: CONTROLO");
        t.set_paint_media(PaintMedia::WetPaint);
        t.ticks_em_blocos(10);
        let acordada = mostrada(&mut t);
        assert!(
            diferentes(&antes, &acordada) < 400,
            "{meio:?}: acordar não apaga o traço de cima ({} px)",
            diferentes(&antes, &acordada)
        );
        t.set_paint_tool_mode("eraser");
        // Passadas no mesmo lugar, aos poucos: o rasto fica sem tinta nenhuma.
        let mut r = Vec::new();
        for _ in 0..PASSADAS_ATE_O_PAPEL {
            r = apaga(&mut t);
        }
        let rasto = |p: &[u8]| -> Vec<[u8; 4]> {
            (70..110usize)
                .map(|y| {
                    let x = 120 + (y - 60) * 7 / 20;
                    let i = (y * W + x) * 4;
                    [p[i], p[i + 1], p[i + 2], p[i + 3]]
                })
                .collect()
        };
        let resto = rasto(&r);
        let v = resto.iter().filter(|c| vermelho(&c[..])).count();
        let a = resto.iter().filter(|c| azul(&c[..])).count();
        let alfa = rasto(&t.paint.wetpaint.session.as_ref().expect("sessão").base);
        if matches!(meio, PaintMedia::Impasto) {
            // O vermelho mostrado sob o Impasto passa pela luz do relevo, que agora segue o alfa (B06.1): a mesma
            // tinta que no Digital (alfa ao bit, medido) lia-se «vermelha» por 1 nível (G 89 × 90). Mede-se a tinta.
            let dig = alfa_digital.as_ref().expect("o Digital vem antes");
            let d = alfa.iter().zip(dig).map(|(p, q)| p[3].abs_diff(q[3])).max();
            assert!(
                d <= Some(2),
                "Impasto: o rasto fica com a tinta do Digital (Δα {d:?})"
            );
            assert_eq!(a, 0, "Impasto: restou azul no rasto");
        } else {
            assert_eq!((v, a), (0, 0), "{meio:?}: restou tinta no rasto");
        }
        if matches!(meio, PaintMedia::Digital) {
            alfa_digital = Some(alfa);
        }
    }
}

/// O traço por cima desfeito antes de voltar: nenhum sinal dele.
#[test]
pub(super) fn desfazer_o_traco_antes_de_voltar_mantem_a_tinta_viva() {
    let vivo = borracha_viva();
    let mut t = com_oleo();
    t.set_paint_media(PaintMedia::Digital);
    traco_de_outro_meio(&mut t, (130.0, 60.0), (130.0, 120.0));
    assert!(t.undo_last(), "o undo aplica-se");
    let r = volta_e_apaga(&mut t);
    assert_eq!(diferentes(&vivo, &r), 0);
}

/// ⭐ Comprimida na RAM: o mesmo, ao bit. A Borracha e a tela depois de acordar são as de quem nunca comprimiu.
#[test]
pub(super) fn comprimir_a_tinta_adormecida_nao_muda_nada() {
    let vivo = borracha_viva();
    let mut t = com_oleo();
    t.set_paint_media(PaintMedia::Digital);
    let a = t.paint.wetpaint.adormecidas.first().expect("adormecida");
    let cru = a.sess.lock().expect("trinco").bytes_retidos();
    a.encolhe_ja();
    let z = a.bytes.load(std::sync::atomic::Ordering::Relaxed);
    // Na grade 1: numa grade menor pesa mais a `base`, do tamanho da tela e mal comprimida.
    assert!(
        super::undo_tinta_morre_tests::razoes::grade() > 1 || z < cru / 4,
        "comprime: {cru} → {z}"
    );
    assert_eq!(diferentes(&vivo, &volta_e_apaga(&mut t)), 0);
}

/// ⭐ Voltar ao Wet com a tinta comprimida não espera: a troca de modo volta logo, e o quadro acorda-a quando pronta.
#[test]
fn voltar_com_a_tinta_comprimida_nao_engasga() {
    let vivo = borracha_viva();
    let mut t = com_oleo();
    t.set_paint_media(PaintMedia::Digital);
    t.paint.wetpaint.adormecidas[0].encolhe_ja();
    // Não engasgar é do relógio da parede; no fixo o acordar espera o fio (B18.5).
    t.set_wet_relogio_fixo(false);
    t.set_paint_media(PaintMedia::WetPaint);
    assert!(
        t.paint.wetpaint.session.is_none(),
        "a troca de modo não descomprime"
    );
    for _ in 0..5000 {
        t.on_tick(1000.0 / 60.0);
        if t.paint.wetpaint.session.is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    assert!(t.paint.wetpaint.session.is_some(), "o quadro acorda-a");
    t.set_wet_relogio_fixo(true);
    t.set_paint_tool_mode("eraser");
    assert_eq!(diferentes(&vivo, &apaga(&mut t)), 0);
}

/// Pen-down logo a seguir: o traço espera a tinta, nunca cai na Borracha da tinta morta.
#[test]
fn o_pen_down_espera_a_tinta_comprimida() {
    let vivo = borracha_viva();
    let mut t = com_oleo();
    t.set_paint_media(PaintMedia::Digital);
    t.paint.wetpaint.adormecidas[0].encolhe_ja();
    assert_eq!(diferentes(&vivo, &volta_e_apaga(&mut t)), 0);
}

/// Acordada depois de comprimir, a grade é a mesma ao bit, plano a plano, e a água continua a correr igual.
#[test]
pub(super) fn a_agua_comprimida_corre_igual() {
    let cena = |comprime: bool| -> Vec<u8> {
        let mut t = PainterTool::default();
        t.abre_a_sprite(vec![255u8; W * H * 4], W as u32, H as u32);
        t.set_paint_tool_mode("wetpaint");
        t.set_wet_relogio_fixo(true);
        t.preset_aplicar(&preset("Watercolor/Fluid/Wet Round.ph2d-brush.png"))
            .expect("o meio");
        t.set_brush_size_px(30.0);
        t.on_canvas_pointer(ponteiro(40.0, 80.0, PointerPhase::Down));
        for k in 1..=30 {
            t.on_canvas_pointer(ponteiro(40.0 + 5.0 * k as f32, 80.0, PointerPhase::Move));
            t.on_tick(1000.0 / 60.0);
        }
        t.on_canvas_pointer(ponteiro(190.0, 80.0, PointerPhase::Up));
        t.ticks_em_blocos(5);
        t.set_paint_media(PaintMedia::Digital);
        if comprime {
            t.paint.wetpaint.adormecidas[0].encolhe_ja();
        }
        t.set_paint_media(PaintMedia::WetPaint);
        // Acorda pela porta do pen-down: pelo quadro, a comprimida ganharia um quadro de água a mais.
        t.wet_acorda();
        assert!(t.paint.wetpaint.session.is_some(), "acordada");
        t.ticks_em_blocos(30);
        mostrada(&mut t)
    };
    assert_eq!(diferentes(&cena(false), &cena(true)), 0);
}

/// A água parada ao sair volta a correr ao voltar.
#[test]
pub(super) fn a_agua_volta_a_correr() {
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.preset_aplicar(&preset("Watercolor/Fluid/Wet Round.ph2d-brush.png"))
        .expect("o meio");
    t.set_brush_size_px(30.0);
    t.on_canvas_pointer(ponteiro(40.0, 80.0, PointerPhase::Down));
    for k in 1..=30 {
        t.on_canvas_pointer(ponteiro(40.0 + 5.0 * k as f32, 80.0, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(ponteiro(190.0, 80.0, PointerPhase::Up));
    t.ticks_em_blocos(5);
    t.set_paint_media(PaintMedia::Digital);
    let parada = mostrada(&mut t);
    t.ticks_em_blocos(30);
    assert_eq!(
        diferentes(&parada, &mostrada(&mut t)),
        0,
        "fora do Wet pára"
    );
    t.set_paint_media(PaintMedia::WetPaint);
    t.ticks_em_blocos(30);
    assert!(
        diferentes(&parada, &mostrada(&mut t)) > 50,
        "de volta ao Wet, corre"
    );
}

/// ⭐ Trocar de camada e voltar: a tinta da camada acorda com ela.
#[test]
fn trocar_de_camada_e_voltar_nao_muda_a_borracha() {
    let vivo = borracha_viva();
    let mut t = com_oleo();
    let camada = t.layers.active().expect("a camada");
    let outra = t.add_raster_layer("outra").expect("a outra camada");
    t.select_layer(outra);
    assert_ne!(t.layers.active(), Some(camada));
    t.on_tick(1000.0 / 60.0);
    t.select_layer(camada);
    t.set_paint_tool_mode("eraser");
    let r = apaga(&mut t);
    // A camada de cima, vazia, não muda a imagem.
    assert_eq!(diferentes(&vivo, &r), 0);
}
