//! J13b P6 ([plano](../../../../../../docs/Painter/Arquitetura/J13b_plano_otica_fora.md) §4 portões 2–3): com a
//! thread REAL e o compositor de pé, a tela que as imagens deixam é a composição de casa AO BYTE; o fio da tela não
//! paga a óptica com a água a correr; e uma imagem mais velha do que um dab composto em casa não deixa píxel velho.
//!
//! O interruptor liga-se por [`forca_no_teste`] (do fio do teste, lido quando o worker sobe), nunca pelo global:
//! os outros portões correm em paralelo no mesmo processo.

use super::agua_sobre_oleo_tests::preset;
use super::offthread::otica::forca_no_teste;
use super::*;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
use std::time::{Duration, Instant};

const LADO: usize = 192;
const QUADRO: Duration = Duration::from_millis(16);
const DT_MS: f32 = 1000.0 / 60.0;

fn ponto(x: f32, y: f32, phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos: [x, y],
        pressure: 0.7,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn tique(t: &mut PainterTool) {
    t.on_tick(DT_MS);
    let _ = t.take_preview_arc();
}

fn quadros(t: &mut PainterTool, n: usize) {
    for _ in 0..n {
        tique(t);
        std::thread::sleep(QUADRO);
    }
}

fn traco(t: &mut PainterTool, rel: &str, cor: [u8; 3], y: f32, smooth: f32, veu: bool) {
    t.preset_aplicar(&preset(rel)).expect("o preset");
    t.set_brush_size_px(16.0);
    t.set_brush_color_srgb8(cor);
    t.set_wet_smooth_edges(smooth);
    t.paint.wetpaint.show_wet = veu;
    let lado = LADO as f32;
    let _ = t.on_canvas_pointer(ponto(lado * 0.15, y, PointerPhase::Down));
    for k in 1..=20 {
        for s in 0..3 {
            let f = (k - 1) as f32 + (s + 1) as f32 / 3.0;
            let x = lado * 0.15 + lado * 0.7 * f / 20.0;
            let _ = t.on_canvas_pointer(ponto(x, y + 0.1 * f, PointerPhase::Move));
        }
        tique(t);
        std::thread::sleep(QUADRO);
    }
    let _ = t.on_canvas_pointer(ponto(lado * 0.85, y, PointerPhase::Up));
}

/// A cena do J13 com a thread real: Dabber com Real Oil, depois Wet Drip a correr.
fn cena(fora: bool, smooth: f32, veu: bool) -> PainterTool {
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; LADO * LADO * 4], LADO as u32, LADO as u32);
    t.set_paint_tool_mode("wetpaint");
    forca_no_teste(Some(fora));
    let lado = LADO as f32;
    t.preset_aplicar(&preset("Oil/Dabber/Dabber.ph2d-brush.png"))
        .expect("o preset");
    t.set_wet_oleo_real_ligado(true);
    traco(
        &mut t,
        "Oil/Dabber/Dabber.ph2d-brush.png",
        [40, 90, 220],
        lado * 0.4,
        smooth,
        veu,
    );
    quadros(&mut t, 6);
    let wd = "Watercolor/Fluid/Wet Drip.ph2d-brush.png";
    traco(&mut t, wd, [230, 200, 30], lado * 0.5, smooth, veu);
    assert_eq!(
        sess(&t).tem_compositor(),
        fora,
        "premissa: o compositor existe só com o interruptor"
    );
    t
}

fn sess(t: &PainterTool) -> &WetSession {
    t.paint.wetpaint.session.as_ref().expect("sessao")
}

/// A água PÁRA (o traço do motor aberto, como no P5) com o motor a ir para o worker, e os tiques correm até a tela
/// mostrar o último passo e não haver nada no ar.
fn assenta(t: &mut PainterTool) {
    congela(t);
    let t0 = Instant::now();
    loop {
        tique(t);
        let s = sess(t);
        if !s.otica.pendente() && s.sim_steps() == s.seen_steps && t0.elapsed() > QUADRO * 4 {
            break;
        }
        assert!(
            t0.elapsed() < Duration::from_secs(10),
            "a água não assentou"
        );
        std::thread::sleep(Duration::from_millis(4));
    }
}

fn congela(t: &mut PainterTool) {
    let s = t.paint.wetpaint.session.as_mut().expect("sessao");
    s.bring_home();
    s.engine.pointer_down(1.0, 1.0, None);
    assert!(!s.engine.sim_should_run(), "premissa: a água parada");
}

/// A tela de agora contra a composição de casa da tela inteira, sobre o mesmo motor.
fn confere_com_casa(t: &mut PainterTool, onde: &str) {
    let vista = t.canvas_rgba.to_vec();
    let s = t.paint.wetpaint.session.as_mut().expect("sessao");
    s.bring_home();
    s.engine.mark_dirty_full();
    t.wetpaint_composite();
    let casa = t.canvas_rgba.as_ref();
    let n = vista.iter().zip(casa).filter(|(a, b)| a != b).count();
    if let Some(i) = vista.iter().zip(casa).position(|(a, b)| a != b) {
        let px = i / 4;
        panic!(
            "{onde}: a tela difere da composição de casa em ({}, {}) — tela {:?} casa {:?}; {n} bytes",
            px % LADO,
            px / LADO,
            &vista[px * 4..px * 4 + 4],
            &casa[px * 4..px * 4 + 4],
        );
    }
    assert!(
        casa.chunks_exact(4).any(|p| p != [255; 4]),
        "{onde}: a cena não pintou nada"
    );
}

#[test]
fn a_agua_pelo_compositor_e_a_tela_de_casa_ao_byte() {
    for (fora, smooth, veu) in [(false, 0.0, false), (true, 0.0, false), (true, 0.6, true)] {
        let mut t = cena(fora, smooth, veu);
        quadros(&mut t, 30);
        assenta(&mut t);
        let s = sess(&t);
        if fora {
            assert!(
                s.otica.aplicadas >= 5,
                "só {} imagens aplicadas — a cena não passou pelo compositor",
                s.otica.aplicadas
            );
            assert!(
                t.paint.wetpaint.oleo_na_tela,
                "o Real Oil não chegou à tela pela imagem"
            );
        } else {
            assert_eq!(
                s.otica.aplicadas, 0,
                "controlo: sem compositor, nenhuma imagem"
            );
        }
        confere_com_casa(&mut t, &format!("fora {fora} smooth {smooth} véu {veu}"));
    }
}

#[test]
fn o_fio_da_tela_nao_paga_a_otica() {
    for fora in [true, false] {
        let mut t = cena(fora, 0.0, false);
        // O pendente do último quadro do traço compõe-se em casa: fica fora da janela.
        quadros(&mut t, 3);
        let _ = super::composite_contagem::take();
        let antes = sess(&t).otica.aplicadas;
        let p0 = sess(&t).sim_steps();
        quadros(&mut t, 40);
        let (chamadas, _, _) = super::composite_contagem::take();
        let s = sess(&t);
        assert!(s.sim_steps() > p0 + 5, "premissa: a água andou");
        if fora {
            assert_eq!(
                chamadas, 0,
                "o fio da tela compôs a óptica {chamadas} vezes"
            );
            assert!(s.otica.aplicadas > antes + 5, "as imagens não chegaram");
        } else {
            assert!(
                chamadas > 5,
                "controlo: sem o compositor o fio da tela compõe ({chamadas})"
            );
        }
    }
}

#[test]
fn a_imagem_velha_nao_deixa_pixel_velho_debaixo_do_dab() {
    let mut t = cena(true, 0.0, false);
    quadros(&mut t, 10);
    // Um pedido no ar, servido pelo worker durante a espera: a imagem fica no canal, mais velha que o dab.
    let t0 = Instant::now();
    loop {
        tique(&mut t);
        if sess(&t).otica.pendente() {
            break;
        }
        assert!(t0.elapsed() < Duration::from_secs(5), "nenhum pedido saiu");
        std::thread::sleep(Duration::from_millis(4));
    }
    std::thread::sleep(Duration::from_millis(150));
    // O dab sobre a água, composto em casa no tique seguinte; a água pára antes dele.
    let lado = LADO as f32;
    t.preset_aplicar(&preset("Oil/Dabber/Dabber.ph2d-brush.png"))
        .expect("o preset");
    t.set_brush_color_srgb8([200, 40, 40]);
    let _ = t.on_canvas_pointer(ponto(lado * 0.4, lado * 0.5, PointerPhase::Down));
    for k in 0..6 {
        let x = lado * (0.4 + 0.03 * k as f32);
        let _ = t.on_canvas_pointer(ponto(x, lado * 0.5, PointerPhase::Move));
    }
    let _ = t.on_canvas_pointer(ponto(lado * 0.58, lado * 0.5, PointerPhase::Up));
    congela(&mut t);
    let d0 = sess(&t).otica.tocadas;
    tique(&mut t);
    assert!(
        sess(&t).otica.tocadas > d0,
        "premissa: a imagem velha chegou depois do dab e tocou a composição de casa"
    );
    assenta(&mut t);
    confere_com_casa(&mut t, "depois do dab");
}

/// A mesma lei, sem corrida: a água parada, o motor muda SEM compor (o Fast dry do motor, a tela inteira suja), a
/// imagem disso fica no canal, um dab pequeno compõe-se em casa e a imagem chega depois dele. Descartá-la é obrigatório
/// (é mais velha que o dab) e a caixa dela TEM de ir para a dívida (fora do dab, a tela ainda mostra a água molhada).
#[test]
fn a_divida_leva_a_caixa_da_imagem_descartada() {
    let mut t = cena(true, 0.0, false);
    quadros(&mut t, 10);
    assenta(&mut t);
    let s = t.paint.wetpaint.session.as_mut().expect("sessao");
    s.bring_home();
    s.engine.fast_dry_now();
    // O motor não andou: só a dívida (uma célula) faz o tique pedir.
    s.otica.divida = Some((1, 1, 1, 1));
    tique(&mut t);
    assert!(sess(&t).otica.pendente(), "premissa: o pedido saiu");
    std::thread::sleep(Duration::from_millis(150));
    let lado = LADO as f32;
    t.preset_aplicar(&preset("Oil/Dabber/Dabber.ph2d-brush.png"))
        .expect("o preset");
    t.set_brush_color_srgb8([200, 40, 40]);
    let _ = t.on_canvas_pointer(ponto(lado * 0.5, lado * 0.45, PointerPhase::Down));
    let _ = t.on_canvas_pointer(ponto(lado * 0.52, lado * 0.45, PointerPhase::Move));
    let _ = t.on_canvas_pointer(ponto(lado * 0.52, lado * 0.45, PointerPhase::Up));
    congela(&mut t);
    let d0 = sess(&t).otica.tocadas;
    tique(&mut t);
    assert!(
        sess(&t).otica.tocadas > d0,
        "premissa: a imagem do Fast dry chegou depois do dab"
    );
    assenta(&mut t);
    confere_com_casa(&mut t, "Fast dry e dab");
}
