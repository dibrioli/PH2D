//! **A CANETA NÃO CHEGA — e este é o preço, num teste que roda.**
//!
//! O Flip shipou `Min Width` e `Response` (a dinâmica de pressão, integrada em 2026-07-27) e o
//! Vector oferece a fonte `Pen`. Nenhum dos dois recebe pressão nesta shell, que é a **única que
//! existe** (`shells/` tem só `desktop`).
//!
//! # O levantamento, medido e não suposto (2026-07-31)
//!
//! A nota que circulava dizia que ligar a caneta *"custa uma função"*. **É falso no winit 0.30**, e
//! a lista completa de `WindowEvent` diz por quê — não existe evento de caneta:
//!
//! - `Touch { force: Option<Force> }` é **touchscreen**, não estilete;
//! - `TouchpadPressure` é force-touch de trackpad da Apple (o próprio doc do winit o diz);
//! - `CursorMoved` não carrega pressão nenhuma;
//! - `AxisMotion { axis: AxisId, value }` é o **valuator CRU do XInput2**, e o `AxisId` é um `u32`
//!   opaco: não há API para perguntar *qual* eixo é a pressão nem qual a faixa dele. Sem isso o
//!   número não é interpretável;
//! - e no backend **Wayland** não há nada: o `zwp_tablet_v2` não está implementado (a máquina do
//!   Enio roda Wayland).
//!
//! ⇒ A cura é uma destas duas, e nenhuma é uma função:
//!
//! 1. **subir o winit** (0.31+ reformou a API de ponteiro com estilete/força/tilt) — é a fundação
//!    de janela e evento do app inteiro, compartilhada com todo módulo: cross-line, classe ADR;
//! 2. **um caminho de tablet por plataforma** (libinput / XInput2 / `zwp_tablet_v2` ao lado do
//!    winit) — dep nova, `unsafe` por plataforma, e uma segunda fonte de eventos correndo com a do
//!    winit.
//!
//! # Por que isto é um GATE e não um comentário
//!
//! O modo de falha de uma nota é envelhecer em silêncio — foi o que aconteceu com a do
//! `painter_canvas_input.rs`, que prometia *"real pressure arrives on the iPad shell"* para uma
//! shell que nunca existiu. Este gate fica VERMELHO no instante em que alguém liga a caneta, que é
//! exatamente o instante em que os dois sliders precisam ser **re-calibrados contra pressão de
//! verdade** (hoje ninguém sabe se `Response 0.5` é uma curva boa: ela nunca foi exercida).
//!
//! Quando isso acontecer: leia a mensagem, refaça a calibração, e **apague este arquivo**.

use std::fs;

/// A dinâmica de pressão é **inerte** na pressão que esta shell entrega, e isto é aritmética
/// fechada: `min + (1 − min)·1^γ = 1` para todo `min` e todo `γ`.
///
/// ⚠️ **A LEI em si já é pinada na crate dela** (`ph2d-tool-flip`,
/// `pressure_width_factor_floor_full_and_response_curve`, em três pontos). O que este acrescenta é
/// a **varredura da grade inteira de sliders** — que é o que transforma *"em pressão 1,0 o fator é
/// 1"* em *"NENHUMA combinação dos dois controles move um pixel"* — mais o contraste com pressão de
/// verdade. Ele existe aqui, junto do levantamento, para quem ler o gate não ter de caçar a
/// consequência noutra crate.
#[test]
fn the_pressure_sliders_cannot_move_a_pixel_at_the_pressure_this_shell_delivers() {
    let mut pior = 0.0_f32;
    for i in 0..=20 {
        for j in 0..=20 {
            let (min, resp) = (i as f32 / 20.0, j as f32 / 20.0);
            let f = ph2d_tool_flip::pressure_width_factor(1.0, min, resp);
            pior = pior.max((f - 1.0).abs());
        }
    }
    assert!(
        pior < 1e-6,
        "a lei mudou: em pressao 1,0 o fator deixou de ser 1,0 (pior desvio {pior}). \
         Se foi de proposito, os dois sliders passaram a mexer no traco SEM caneta — o que \
         nao e' o que eles prometem."
    );
    // E o contraste: com pressão de verdade eles fazem exatamente o que dizem.
    let leve = ph2d_tool_flip::pressure_width_factor(0.3, 0.05, 0.5);
    assert!(
        leve < 0.5,
        "com pressao 0,3 o fator deveria afinar bem o traco, e deu {leve}"
    );
}

/// **A porta de entrada do traço do Flip não tem por onde a pressão passar** — e é esta a forma do
/// buraco, não um literal `1.0` escondido em algum lugar.
///
/// Mutação que sangra: dar um parâmetro de pressão ao `flip_canvas_down`/`flip_canvas_move` (que é
/// literalmente o primeiro passo de quem for ligar a caneta).
#[test]
fn the_flip_canvas_entry_points_carry_no_pressure() {
    let src = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/flip_draw.rs"))
        .expect("flip_draw.rs");
    for porta in [
        "fn flip_canvas_down(&mut self, x: f32, y: f32)",
        "fn flip_canvas_move(&mut self, x: f32, y: f32)",
    ] {
        assert!(
            src.contains(porta),
            "`{porta}` mudou de forma.\n\n\
             Se foi para RECEBER PRESSAO: otimo, o buraco fechou — mas leia o cabecalho deste \
             arquivo antes de apagar o gate. Os sliders `Min Width` e `Response` do Flip nunca \
             foram exercidos com pressao de verdade (a shell sempre entregou 1,0), entao os \
             defaults 0,05 / 0,5 sao um palpite herdado, nao uma calibracao. O mesmo vale para a \
             fonte `Pen` do Vector.\n\n\
             Se foi so' um refactor de assinatura: ajuste a agulha aqui."
        );
    }
}
