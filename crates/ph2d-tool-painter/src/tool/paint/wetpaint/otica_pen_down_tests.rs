//! J13b P8 ([doc](../../../../../../docs/Painter/Arquitetura/J13b_P8_pen_down.md)): com o compositor de pé, o sujo da
//! água que o motor traz do worker no pen-down vai pela `divida` — a 1.ª composição de casa do traço cobre só o traço,
//! a água é servida pela imagem seguinte, e a tela final é a de casa AO BYTE. Thread real, como os portões do P6.

use super::agua_sobre_oleo_tests::preset;
use super::otica_no_tique_tests::{
    LADO, assenta, cena, confere_com_casa, congela, ponto, quadros, sess, tique,
};
use super::*;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};
use std::time::{Duration, Instant};

fn area(c: (usize, usize, usize, usize)) -> usize {
    (c.2 + 1 - c.0) * (c.3 + 1 - c.1)
}

/// O traço curto no alto da tela, longe da água (`y = 0,1·LADO`, `x` de `0,3` a `0,4`).
const Y: f32 = 0.1;
const X0: f32 = 0.3;
const X1: f32 = 0.4;
/// O raio do dab (o pincel de 16 px) mais a folga do motor, em px.
const MARGEM_PX: f32 = 24.0;

/// A caixa do traço em células: o que a composição de casa do 1.º quadro pode tocar.
fn caixa_do_traco(ratio: u8) -> usize {
    let l = LADO as f32;
    let r = f32::from(ratio.max(1));
    let w = ((l * (X1 - X0) + 2.0 * MARGEM_PX) / r).ceil() + 2.0;
    let h = (2.0 * MARGEM_PX / r).ceil() + 2.0;
    (w * h) as usize
}

#[test]
fn o_pen_down_manda_a_agua_herdada_pela_divida() {
    let mut t = cena(true, 0.0, false);
    quadros(&mut t, 10);
    // Um pedido no ar e o tique parado: o worker anda sem ser servido e o sujo da água acumula no motor.
    let t0 = Instant::now();
    loop {
        tique(&mut t);
        if sess(&t).otica.pendente() {
            break;
        }
        assert!(t0.elapsed() < Duration::from_secs(5), "nenhum pedido saiu");
        std::thread::sleep(Duration::from_millis(4));
    }
    let p0 = sess(&t).sim_steps();
    std::thread::sleep(Duration::from_millis(150));
    assert!(
        sess(&t).sim_steps() > p0,
        "premissa: a água andou na espera"
    );
    t.preset_aplicar(&preset("Oil/Dabber/Dabber.ph2d-brush.png"))
        .expect("o preset");
    t.set_brush_size_px(16.0);
    t.set_brush_color_srgb8([200, 40, 40]);
    // O motor em casa e a imagem do pedido no ar recebida antes do traço: nenhuma imagem descartada pelo traço cobre a
    // região herdada — só a dívida a pode mostrar (sem isto a mutação «largar o sujo» passava).
    t.paint
        .wetpaint
        .session
        .as_mut()
        .expect("sessao")
        .bring_home();
    let t0 = Instant::now();
    while sess(&t).otica.pendente() {
        t.wet_recebe_imagens();
        assert!(
            t0.elapsed() < Duration::from_secs(10),
            "a imagem não chegou"
        );
        std::thread::sleep(Duration::from_millis(4));
    }
    // O sujo herdado, medido sem o tirar do motor (o mesmo que o pen-down encontra).
    let s = t.paint.wetpaint.session.as_mut().expect("sessao");
    let herdado = offthread::otica::caixa_da_grade(s.engine.take_dirty(), s.grid)
        .expect("premissa: a água sujou o motor desde a última imagem");
    let i = |v: usize| i32::try_from(v).expect("cabe");
    s.engine
        .merge_dirty_otica(i(herdado.0), i(herdado.1), i(herdado.2), i(herdado.3));
    let limite = caixa_do_traco(s.ratio);
    assert!(
        area(herdado) > 2 * limite,
        "premissa: o sujo da água ({} células) é muito maior que o traço ({limite})",
        area(herdado)
    );
    let _ = super::composite_contagem::take();
    let antes_do_traco = t.canvas_rgba.to_vec();
    let l = LADO as f32;
    let _ = t.on_canvas_pointer(ponto(l * X0, l * Y, PointerPhase::Down));
    let mut por_evento = vec![super::composite_contagem::take()];
    for k in 1..=3 {
        let x = l * (X0 + (X1 - X0) * k as f32 / 3.0);
        let _ = t.on_canvas_pointer(ponto(x, l * Y, PointerPhase::Move));
        por_evento.push(super::composite_contagem::take());
    }
    let passos = sess(&t).sim_steps();
    tique(&mut t);
    por_evento.push(super::composite_contagem::take());
    eprintln!(
        "[p8] herdado {} células · (composições, motor, render) por evento do 1.º quadro: {por_evento:?}",
        area(herdado)
    );
    let chamadas: u64 = por_evento.iter().map(|e| e.0).sum();
    let motor = por_evento.iter().map(|e| e.1).max().unwrap_or(0);
    assert!(chamadas >= 1, "premissa: o traço compôs em casa");
    assert!(
        motor as usize <= limite,
        "uma composição de casa do 1.º quadro do traço levou {motor} células do motor (o traço tem {limite}; a água herdada {})",
        area(herdado)
    );
    for k in 0..6 {
        let x = l * (X1 - 0.01 * k as f32);
        let _ = t.on_canvas_pointer(ponto(x, l * Y, PointerPhase::Move));
        tique(&mut t);
        std::thread::sleep(Duration::from_millis(16));
    }
    // Com o traço aberto a água não anda: a região herdada mostra o último estado composto, e é o certo.
    assert_eq!(
        sess(&t).sim_steps(),
        passos,
        "a água andou com o traço aberto"
    );
    assert!(
        sess(&t).otica.pendente(),
        "premissa: a água herdada espera o pen-up (o motor fica em casa com o traço aberto)"
    );
    // O motor fica em casa do `Up` ao `congela`: sem isso o worker dá um passo à chegada e a corrida
    // contra o `bring_home` decide a premissa (17 contra 16 na suíte, load 70–100).
    offthread::segura_a_entrega(true);
    let _ = t.on_canvas_pointer(ponto(l * X1, l * Y, PointerPhase::Up));
    congela(&mut t);
    offthread::segura_a_entrega(false);
    // A água pára já: só a dívida pode mostrar a região herdada (sem passo novo a cobri-la).
    assenta(&mut t);
    assert_eq!(
        sess(&t).sim_steps(),
        passos,
        "premissa: o motor não andou desde o pen-down"
    );
    // A região herdada (abaixo do traço e do alcance dele) mudou na tela: foi a dívida que a mostrou.
    let r = usize::from(sess(&t).ratio.max(1));
    let y0 = ((herdado.1 - 1) * r).max((l * Y) as usize + 64);
    let mudou = (y0..(herdado.3 * r).min(LADO))
        .flat_map(|y| {
            ((herdado.0 - 1) * r..(herdado.2 * r).min(LADO)).map(move |x| (y * LADO + x) * 4)
        })
        .filter(|&i| antes_do_traco[i..i + 4] != t.canvas_rgba[i..i + 4])
        .count();
    eprintln!(
        "[p8] píxeis da região herdada que a dívida mudou: {mudou} · aplicadas {} tocadas {} vistos {}",
        sess(&t).otica.aplicadas,
        sess(&t).otica.tocadas,
        sess(&t).seen_steps
    );
    assert!(
        mudou > 0,
        "a região herdada não mudou na tela: a dívida não a mostrou"
    );
    confere_com_casa(&mut t, "água herdada pela dívida e traço noutra zona");
}
