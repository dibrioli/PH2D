//! **O que o canal custa — medido, porque o número que ele herdaria não foi.**
//!
//! O `ph2d-script::messaging` declara num doc-comment *"100.000 mensagens/quadro em ≤ 1,5 ms
//! (HR-4)"*. Aquele número **nunca foi medido neste repo**: a crate não tem `benches/`, o
//! módulo não tem um único `#[test]`, e o único teste que existe é um proptest de determinismo
//! de intern. Herdá-lo seria carregar uma aspiração como se fosse um piso.
//!
//! Então esta sonda mede o que ESTA saída faz, em três regimes, e diz qual deles é o produto:
//!
//! - **produto** — o que o app de fato publica: unidades de sinais por quadro, 2 consumidores;
//! - **pesado** — uma cena de gameplay densa (centenas de contatos marcados por quadro);
//! - **o teto** — quantos sinais/quadro cabem em 1% de um quadro de 60 fps.
//!
//! ⚠️ **A latência sinal→consumidor NÃO se mede com relógio**: os dois produtores e o dreno
//! rodam em linha reta na mesma função, então o atraso é **zero quadros por construção** — e
//! quem prova isso é o `both_producers_land_in_one_outbox_in_one_frame` mais o arch-gate de
//! ordem do shell, não um cronômetro.
//!
//! Rode com `cargo test -p ph2d-runtime --release -- --ignored --nocapture`.

use std::time::Instant;

use ph2d_runtime::{Signal, SignalOutbox, SignalReader};

/// Um quadro: vira a janela, publica `n` sinais, e `readers` consumidores drenam.
fn frame(outbox: &mut SignalOutbox, readers: &mut [SignalReader], n: usize) -> usize {
    outbox.advance_frame();
    for i in 0..n {
        if i % 2 == 0 {
            outbox.publish(Signal::from_timeline("footstep", 1.0));
        } else {
            outbox.publish(Signal::from_contact("door", 11, 42));
        }
    }
    let mut seen = 0;
    for r in readers.iter_mut() {
        seen += outbox.read(r).count();
    }
    seen
}

fn measure(n: usize, consumers: usize, frames: usize) -> f64 {
    let mut outbox = SignalOutbox::new();
    let mut readers = vec![SignalReader::new(); consumers];
    // A 1ª volta paga alocação e caminho frio — descartada, como toda sonda desta casa.
    let _ = frame(&mut outbox, &mut readers, n);
    let t0 = Instant::now();
    let mut sink = 0;
    for _ in 0..frames {
        sink += frame(&mut outbox, &mut readers, n);
    }
    let per_frame = t0.elapsed().as_secs_f64() / frames as f64;
    assert!(sink > 0 || n == 0, "a sonda não consumiu nada");
    per_frame * 1.0e6 // µs por quadro
}

#[test]
#[ignore = "sonda: mede o custo do canal; rode com `--release -- --ignored --nocapture`"]
fn what_the_signal_channel_costs_per_frame() {
    // O orçamento de um quadro de 60 fps, e o 1% dele que uma saída de eventos pode pedir sem
    // discussão. Não é um teto de recurso: é a régua contra a qual os números abaixo se leem.
    const FRAME_US: f64 = 16_666.0;

    eprintln!("[canal] sinais/quadro x consumidores -> us/quadro (% de um quadro de 60 fps)");
    for (n, consumers, rotulo) in [
        (0usize, 2usize, "quieto  (o caso comum: nada acontece)"),
        (4, 2, "produto (o smoke: 2 markers + 2 contatos)"),
        (32, 2, "denso   (uma cena de gameplay movimentada)"),
        (256, 2, "pesado  (centenas de contatos marcados)"),
        (256, 8, "pesado  x 8 consumidores"),
        (4096, 2, "extremo (para achar a forma da curva)"),
    ] {
        let us = measure(n, consumers, 2000);
        eprintln!(
            "  {n:>5} x {consumers} -> {us:>8.3} us  ({:>6.3}% do quadro)   {rotulo}",
            us / FRAME_US * 100.0
        );
    }

    // O regime do PRODUTO tem de ser ruído contra um quadro. É a única afirmação numérica que
    // esta wave faz — e ela é sobre o app que existe, não sobre um alvo herdado.
    let produto = measure(4, 2, 5000);
    assert!(
        produto < FRAME_US * 0.001,
        "o canal custa {produto:.3} us/quadro no regime do produto (4 sinais, 2 consumidores) — \
         mais de 0,1% de um quadro de 60 fps para entregar meia dúzia de nomes. Isso não é um \
         canal de eventos, é um problema."
    );

    // **UM CONSUMIDOR É QUASE DE GRAÇA, e este é o número que decide o R3.** O custo mora no
    // PRODUTOR (a alocação do `Arc<str>` do nome, ~39 ns por sinal); ler é percorrer um `Vec`
    // com um cursor. Quadruplicar os consumidores não pode aproximar-se de quadruplicar o
    // custo — se aproximar, o desenho passou a copiar alguma coisa por leitor.
    //
    // ⚠️ É uma RAZÃO, não um wall-clock: a barra tem de sobreviver a uma máquina disputada.
    let dois = measure(256, 2, 2000);
    let oito = measure(256, 8, 2000);
    eprintln!("[canal] 8 consumidores / 2 consumidores = {:.2}x", oito / dois);
    assert!(
        oito < dois * 1.5,
        "8 consumidores custaram {oito:.3} us contra {dois:.3} us de 2 — {:.2}x. Ler devia ser \
         um cursor a percorrer um `Vec`; se um leitor a mais pesa, alguém passou a COPIAR por \
         consumidor, e o 'todos' (áudio + Luau + UI) deixou de ser grátis.",
        oito / dois
    );
}
