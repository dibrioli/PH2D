//! A auditoria do undo vivo (06/11): cada portão provado vermelho sem a cura.

use super::agua_sobre_oleo_tests::ponteiro;
use super::undo_tinta_morre_tests::{
    AGUADA, DABBER, grade, linha_y, tela, teto, tinta_viva_na_linha, traco,
};
use super::*;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::PointerPhase;

/// A variante que recompõe os planos da grade inteira, para comparar com a caixa.
pub(super) mod grade_inteira {
    use std::cell::Cell;
    thread_local! { static V: Cell<bool> = const { Cell::new(false) }; }
    pub(in crate::tool::paint) fn get() -> bool {
        V.with(Cell::get)
    }
    pub(in crate::tool::paint) fn set(v: bool) {
        V.with(|c| c.set(v));
    }
}

fn planos_e_tela(t: &mut PainterTool) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    t.wet_bring_home();
    let s = t.paint.wetpaint.session.as_ref().expect("a sessão");
    (
        t.canvas_rgba.to_vec(),
        s.planos.pigment.clone(),
        s.planos.sombra.clone(),
    )
}

fn px_diferentes(a: &[u8], b: &[u8]) -> usize {
    a.iter().zip(b).filter(|(p, q)| p != q).count()
}

/// ⭐ Compor os planos só na caixa do que mudou dá a mesma tela e os mesmos planos que a grade inteira, com a água a
/// correr: para lá do tecto e com fotografia depois de um enterro.
#[test]
fn a_caixa_da_os_mesmos_planos_que_a_grade_inteira() {
    let corrida = |meio: &str, teto_um: bool, inteira: bool| {
        grade_inteira::set(inteira);
        teto::set(teto_um.then_some(1));
        let mut t = tela(800, 500, meio, 40.0);
        let mut fotos = Vec::new();
        traco(&mut t, 40.0, 760.0, 60.0);
        t.ticks_em_blocos(20);
        preenche(&mut t, 2);
        for i in 0..5 {
            traco(&mut t, 40.0, 760.0, 120.0 + 60.0 * i as f32);
            t.ticks_em_blocos(4);
        }
        for _ in 0..4 {
            assert!(t.undo_last());
            fotos.push(planos_e_tela(&mut t));
            t.ticks_em_blocos(6);
            fotos.push(planos_e_tela(&mut t));
        }
        assert!(t.redo_last());
        fotos.push(planos_e_tela(&mut t));
        t.ticks_em_blocos(120);
        fotos.push(planos_e_tela(&mut t));
        teto::set(None);
        grade_inteira::set(false);
        fotos
    };
    for meio in [DABBER, AGUADA] {
        for teto_um in [true, false] {
            let (a, b) = (corrida(meio, teto_um, true), corrida(meio, teto_um, false));
            for (i, (x, y)) in a.iter().zip(&b).enumerate() {
                assert_eq!(
                    px_diferentes(&x.0, &y.0),
                    0,
                    "{meio} tecto {teto_um}: a tela, foto {i}"
                );
                assert_eq!(
                    px_diferentes(&x.1, &y.1),
                    0,
                    "{meio} tecto {teto_um}: o pigmento, foto {i}"
                );
                assert_eq!(
                    px_diferentes(&x.2, &y.2),
                    0,
                    "{meio} tecto {teto_um}: a sombra, foto {i}"
                );
            }
        }
    }
}

/// Dry Canvas e Wet Canvas depois de um traço: o desfazer dele devolve a grade de antes dele, longe dele inclusive.
#[test]
fn a_acao_de_tela_inteira_entra_no_registo_do_traco() {
    for (nome, acao) in [
        (
            "seca",
            PainterTool::wetpaint_dry_canvas as fn(&mut PainterTool),
        ),
        ("molha", PainterTool::wetpaint_wet_canvas),
    ] {
        let mut sem = tela(600, 400, AGUADA, 20.0);
        let mut com = tela(600, 400, AGUADA, 20.0);
        for i in 0..3 {
            traco(&mut sem, 40.0, 560.0, linha_y(i));
            traco(&mut com, 40.0, 560.0, linha_y(i));
        }
        traco(&mut com, 300.0, 340.0, 200.0);
        acao(&mut com);
        assert!(com.undo_last());
        assert!(grade(&mut com) == grade(&mut sem), "{nome}: a grade");
    }
}

fn preenche(t: &mut PainterTool, i: u32) {
    t.set_rect_selection(20 + 45 * (i % 12), 300, 40, 40);
    t.set_brush_color_srgb8([220, 30, 30]);
    t.selection_color_fill();
    t.ticks_em_blocos(1);
    t.clear_selection();
    t.ticks_em_blocos(1);
}

fn retidos(t: &mut PainterTool) -> (usize, usize) {
    t.wet_bring_home();
    let s = t.paint.wetpaint.session.as_ref().expect("a sessão");
    (
        s.engine.bytes_do_historico(0) + super::undo_vivo::bytes_reais_dos_passos(s),
        s.antes_do_passo.len(),
    )
}

/// Fills dentro do Wet: cada enterro retém tela, `base` e planos. O tecto conta-os, senão o undo vivo cresce sem fim.
#[test]
fn o_tecto_conta_o_que_cada_enterro_retem() {
    let mut t = tela(600, 400, DABBER, 20.0);
    traco(&mut t, 40.0, 560.0, 20.0);
    preenche(&mut t, 0);
    let (a, _) = retidos(&mut t);
    preenche(&mut t, 1);
    let (b, _) = retidos(&mut t);
    let um = b.saturating_sub(a).max(1);
    let teto_de_tres = b + 2 * um;
    teto::set(Some(teto_de_tres));
    println!("[a2] um {um} · tecto {teto_de_tres}");
    for i in 2..12 {
        preenche(&mut t, i);
        println!("[a2] {i}: {:?}", retidos(&mut t));
    }
    let (usado, passos) = retidos(&mut t);
    teto::set(None);
    assert!(
        usado <= teto_de_tres + um,
        "{usado} > {teto_de_tres} + {um} ({passos} passos)"
    );
    assert!(passos >= 1, "o passo do topo fica");
}

/// Uma imagem pedida antes de refazer para lá do tecto é de antes do enterro: não se aplica, vai pela dívida.
#[test]
fn refazer_para_la_do_tecto_recusa_a_imagem_no_ar() {
    let mut aplicadas = 0;
    let mut px = 0;
    let mut no_ar = 0;
    for rodada in 0..6 {
        teto::set(Some(1));
        super::offthread::otica::forca_no_teste(Some(true));
        let mut t = tela(600, 400, AGUADA, 40.0);
        t.set_wet_relogio_fixo(false);
        t.set_compor_por_quadro(true);
        // A aguada de cima fica viva e corre para onde o traço desfeito estava.
        traco(&mut t, 40.0, 560.0, 170.0);
        t.set_brush_color_srgb8([220, 200, 40]);
        traco(&mut t, 40.0, 560.0, 200.0);
        traco(&mut t, 40.0, 560.0, 370.0);
        assert!(t.undo_last());
        assert!(t.undo_last());
        for _ in 0..150 {
            std::thread::sleep(std::time::Duration::from_millis(4));
            t.on_tick(1000.0 / 60.0);
            let _ = t.take_preview_arc();
        }
        for _ in 0..300 {
            std::thread::sleep(std::time::Duration::from_millis(1 + rodada % 3));
            t.on_tick(1000.0 / 60.0);
            let _ = t.take_preview_arc();
            let s = t.paint.wetpaint.session.as_ref().expect("a sessão");
            if s.otica.no_ar.is_some() {
                break;
            }
        }
        let s = t.paint.wetpaint.session.as_ref().expect("a sessão");
        let epoca = s.otica.no_ar;
        let antes = s.otica.aplicadas;
        no_ar += usize::from(epoca.is_some());
        assert!(t.redo_last());
        teto::set(None);
        let refeita = t.canvas_rgba.to_vec();
        // Só até a resposta do pedido de antes do enterro chegar: as imagens seguintes são novas.
        for _ in 0..500 {
            std::thread::sleep(std::time::Duration::from_millis(2));
            t.on_tick(1000.0 / 60.0);
            let _ = t.take_preview_arc();
            let s = t.paint.wetpaint.session.as_ref().expect("a sessão");
            if s.otica.no_ar != epoca {
                if s.otica.aplicadas > antes {
                    aplicadas += 1;
                    for (a, b) in t.canvas_rgba.chunks_exact(4).zip(refeita.chunks_exact(4)) {
                        px += usize::from(a.iter().zip(b).any(|(p, q)| p.abs_diff(*q) > 3));
                    }
                }
                break;
            }
        }
        super::offthread::otica::forca_no_teste(None);
    }
    assert!(no_ar > 0, "CONTROLO: nenhuma imagem no ar nas rodadas");
    assert_eq!(
        aplicadas, 0,
        "{aplicadas} de {no_ar} imagens velhas aplicadas: {px} px"
    );
}

/// Passos que o undo do documento já descartou não seguem no tecto do undo vivo.
#[test]
fn o_passo_que_o_documento_ja_largou_sai_do_tecto() {
    let mut t = tela(600, 400, DABBER, 20.0);
    t.undo.set_max_bytes(1);
    for i in 0..8 {
        traco(&mut t, 40.0, 560.0, linha_y(i));
    }
    let fundo = t.undo.undo_depth();
    assert!(fundo < 8, "CONTROLO: o documento largou passos ({fundo})");
    t.on_canvas_pointer(ponteiro(40.0, 390.0, PointerPhase::Down));
    t.on_canvas_pointer(ponteiro(40.0, 390.0, PointerPhase::Up));
    t.wet_bring_home();
    let s = t.paint.wetpaint.session.as_ref().expect("a sessão");
    let passos = s.antes_do_passo.len();
    assert!(
        passos <= fundo + 1,
        "{passos} passos, o documento alcança {fundo}"
    );
}

/// As células de uma camada com tinta sem alfa no plano, na faixa de linhas `y0..y1`.
fn invisiveis(t: &mut PainterTool, y0: usize, y1: usize) -> usize {
    t.wet_bring_home();
    let s = t.paint.wetpaint.session.as_ref().expect("a sessão");
    let (gw, _) = s.grid;
    let g = &s.engine.layers[0].grid;
    let r = usize::from(s.ratio);
    let mut n = 0;
    for cy in y0 / r..y1 / r {
        for cx in 0..gw {
            let k = cy * gw + cx;
            let c = (cy + 1) * g.s + cx + 1;
            n += usize::from(s.planos.pigment[k * 4 + 3] == 0 && g.sett[c] + g.susp[c] > 0.0);
        }
    }
    n
}

/// Só a orla do traço desfeito acaba: a da aguada longe dele continua viva, ao bit.
#[test]
fn para_la_do_tecto_a_tinta_longe_fica_ao_bit() {
    teto::set(Some(1));
    let mut t = tela(900, 700, AGUADA, 40.0);
    traco(&mut t, 60.0, 840.0, 80.0);
    // Sem água: a que ainda corre durante o traço desfeito muda a tela e enterra-se com ele.
    t.ticks_em_blocos(60);
    t.wetpaint_dry_canvas();
    t.ticks_em_blocos(2);
    let assentada = t.canvas_rgba.to_vec();
    traco(&mut t, 60.0, 840.0, 500.0);
    traco(&mut t, 60.0, 840.0, 620.0);
    assert!(t.undo_last(), "o passo do topo, com fotografia");
    t.wet_bring_home();
    t.paint
        .wetpaint
        .session
        .as_mut()
        .expect("a sessão")
        .engine
        .drain_step();
    let n = invisiveis(&mut t, 0, 250);
    assert!(n > 0, "CONTROLO: a aguada longe tem orla sem alfa");
    let s = t.paint.wetpaint.session.as_ref().expect("a sessão");
    let g = &s.engine.layers[0].grid;
    let corte = 250 / usize::from(s.ratio) * g.s;
    let antes = (g.sett[..corte].to_vec(), g.susp[..corte].to_vec());
    let corrida = t.canvas_rgba[..250 * 900 * 4]
        .iter()
        .zip(&assentada)
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(corrida, 0, "CONTROLO: a aguada de cima ainda corria");
    assert!(t.undo_last(), "para lá do tecto");
    teto::set(None);
    assert!(tinta_viva_na_linha(&mut t, 80) > 0.0);
    t.wet_bring_home();
    let s = t.paint.wetpaint.session.as_ref().expect("a sessão");
    let g = &s.engine.layers[0].grid;
    let difere = g.sett[..corte]
        .iter()
        .zip(&antes.0)
        .chain(g.susp[..corte].iter().zip(&antes.1))
        .filter(|(a, b)| a.to_bits() != b.to_bits())
        .count();
    let zeradas = g.sett[..corte]
        .iter()
        .zip(&antes.0)
        .chain(g.susp[..corte].iter().zip(&antes.1))
        .filter(|(a, b)| a.to_bits() != b.to_bits() && **a == 0.0)
        .count();
    assert_eq!(
        difere, 0,
        "{difere} células longe mudaram, {zeradas} a zero ({n} sem alfa)"
    );
}

/// Quão longe do que a tela mudou fica a tinta invisível que um traço desfeito deixa: o alcance do enterro.
#[test]
#[ignore = "sonda: corre por nome com --ignored --nocapture"]
fn sonda_alcance_da_orla() {
    for meio in [AGUADA, DABBER] {
        for tam in [20.0f32, 60.0, 160.0] {
            for espera in [2u32, 120, 600, 3000] {
                let mut t = tela(1000, 700, meio, tam);
                let antes = t.canvas_rgba.to_vec();
                traco(&mut t, 200.0, 800.0, 350.0);
                t.ticks_em_blocos(espera);
                t.wet_bring_home();
                t.wetpaint_composite();
                t.wet_bring_home();
                let s = t.paint.wetpaint.session.as_ref().expect("a sessão");
                let (gw, gh) = s.grid;
                let r = usize::from(s.ratio);
                let mut mudou = vec![false; gw * gh];
                for y in 0..700 {
                    for x in 0..1000 {
                        let o = (y * 1000 + x) * 4;
                        if antes[o..o + 4] != t.canvas_rgba[o..o + 4] {
                            mudou[(y / r) * gw + x / r] = true;
                        }
                    }
                }
                let g = &s.engine.layers[0].grid;
                // Distância de Chebyshev à célula mudada mais próxima: varredura de dois passes.
                let mut d = vec![usize::MAX / 2; gw * gh];
                for k in 0..gw * gh {
                    if mudou[k] {
                        d[k] = 0;
                    }
                }
                let viz = |x: isize, y: isize| -> Option<usize> {
                    (x >= 0 && y >= 0 && (x as usize) < gw && (y as usize) < gh)
                        .then(|| y as usize * gw + x as usize)
                };
                for y in 0..gh as isize {
                    for x in 0..gw as isize {
                        let k = y as usize * gw + x as usize;
                        for (a, b) in [(-1, 0), (-1, -1), (0, -1), (1, -1)] {
                            if let Some(v) = viz(x + a, y + b) {
                                d[k] = d[k].min(d[v] + 1);
                            }
                        }
                    }
                }
                for y in (0..gh as isize).rev() {
                    for x in (0..gw as isize).rev() {
                        let k = y as usize * gw + x as usize;
                        for (a, b) in [(1, 0), (1, 1), (0, 1), (-1, 1)] {
                            if let Some(v) = viz(x + a, y + b) {
                                d[k] = d[k].min(d[v] + 1);
                            }
                        }
                    }
                }
                let (mut alcance, mut n) = (0usize, 0usize);
                for k in 0..gw * gh {
                    let c = (k / gw + 1) * g.s + k % gw + 1;
                    if !mudou[k] && s.planos.pigment[k * 4 + 3] == 0 && g.sett[c] + g.susp[c] > 0.0
                    {
                        n += 1;
                        alcance = alcance.max(d[k]);
                    }
                }
                println!(
                    "[orla] {meio:.6} tamanho {tam} espera {espera}: razão {r} · {n} células sem alfa fora da mudança · alcance {alcance} células"
                );
            }
        }
    }
}
