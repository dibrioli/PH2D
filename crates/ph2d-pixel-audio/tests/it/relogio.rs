//! P13 T: o som manda no relógio do Play; sem som, o Play é o de hoje. Sem dispositivo: o
//! `AudioRenderer` corre à mão (nenhum som sai).

use ph2d_audio::{AudioEngine, AudioFormat, AudioRenderer, ChannelLayout, SampleData};
use ph2d_pixel_audio::{Sons, Tocador, dt_do_play, picos};
use ph2d_pixel_core::play::{Play, PlayOptions};
use ph2d_pixel_core::{DocOp, FaixaDeAudio, NewSprite, Op, Sprite, frames};

const OUT: u32 = 48_000;
const BLOCO: usize = 256;

/// `n` quadros de 100 ms (a duração de fábrica).
fn sprite(n: usize) -> Sprite {
    let mut s = Sprite::new(NewSprite::default()).expect("sprite");
    for _ in 1..n {
        let f = s.frames().last().expect("quadro").id;
        let op = frames::new_frame(&mut s, f, frames::NewFrame::Empty)
            .expect("novo")
            .0;
        ph2d_pixel_core::apply(&mut s, op).expect("aplica");
    }
    s
}

/// `seg` segundos de um som mono a 44,1 kHz (outra taxa que o mixer: a voz reamostra).
fn som(seg: usize) -> SampleData {
    let rate = 44_100;
    SampleData::from_fn(
        seg * rate as usize,
        AudioFormat {
            sample_rate: rate,
            channels: ChannelLayout::Mono,
        },
        |i| ((i % 441) as f32 / 441.0 - 0.5) * 0.2,
    )
}

fn faixa(asset: &str, inicio_ms: i32, mudo: bool) -> FaixaDeAudio {
    FaixaDeAudio {
        asset: asset.into(),
        inicio_ms,
        ganho: 0.8,
        mudo,
    }
}

/// O host a 60 Hz com relógio que deriva (+2 %) e tremido (±3 ms): o `dt` que ele DIZ.
fn dt_do_host(i: usize) -> u32 {
    let tremor = [0_i32, 3, -2, 1, -3, 2][i % 6];
    u32::try_from(17 + tremor).unwrap_or(17)
}

/// Corre `seg` segundos de tempo REAL a 60 Hz; o mixer rende o que o tempo real pede. Devolve o pior
/// desvio (ms) entre o instante do Play e o do som, e o quadro final.
fn corre(com_relogio: bool, seg: usize) -> (f64, usize) {
    let s = sprite(7);
    let mut sons = Sons::default();
    sons.insere("a.wav", som(seg + 2));
    sons.insere("b.wav", som(seg + 2));
    let faixas = vec![
        faixa("a.wav", -250, false),
        faixa("b.wav", 400, false),
        faixa("a.wav", 0, true),
    ];
    let (mut engine, mut renderer): (AudioEngine, AudioRenderer) =
        AudioEngine::new(AudioFormat::stereo(OUT));
    let mut t = Tocador::default();
    t.tocar(&mut engine, &faixas, &sons, 0).expect("toca");
    assert_eq!(t.faixas_a_tocar(), 2, "a faixa muda não entra no mixer");
    let mut p = Play::start(&s, 0, PlayOptions::default());
    let (mut rendidos, mut play_ms, mut pior) = (0_u64, 0_u64, 0.0_f64);
    let mut bloco = vec![0.0_f32; BLOCO * 2];
    for i in 1..=seg * 60 {
        let real_ms = i as f64 * 1000.0 / 60.0;
        while (rendidos as f64) * 1000.0 / f64::from(OUT) < real_ms {
            t.bombear();
            renderer.render(&mut bloco, BLOCO);
            engine.collect_returns();
            rendidos += BLOCO as u64;
        }
        let rel = if com_relogio { t.relogio() } else { None };
        let dt = dt_do_play(rel, dt_do_host(i));
        p.advance(&s, dt);
        play_ms += u64::from(dt);
        let som_ms = rendidos as f64 * 1000.0 / f64::from(OUT);
        pior = pior.max((play_ms as f64 - som_ms).abs());
        // O quadro mostrado é o do instante do Play.
        assert_eq!(
            p.index,
            (play_ms / 100) as usize % 7,
            "quadro em {play_ms} ms"
        );
    }
    (pior, p.index)
}

/// **O som manda no relógio:** com o host a derivar +2 % e tremido, o quadro segue o playhead do
/// mixer — desvio ≤ 1 quadro a 60 Hz em 10 s. O controlo: o relógio do host sozinho afasta-se.
#[test]
fn o_som_manda_no_relogio() {
    let quadro_60 = 1000.0 / 60.0;
    let (pior, _) = corre(true, 10);
    assert!(
        pior <= quadro_60,
        "com som: desvio {pior:.2} ms > {quadro_60:.2}"
    );
    let (sem, _) = corre(false, 10);
    assert!(
        sem > 5.0 * quadro_60,
        "o controlo não mede: o host sozinho só se afastou {sem:.2} ms"
    );
}

/// **Sem som, o Play é o de hoje**, byte a byte: só faixas mudas (ou nenhuma) ⇒ sem relógio, e a
/// sequência de quadros é a mesma do `advance` com o `dt` do host.
#[test]
fn sem_som_o_play_e_o_de_hoje() {
    let mut s = sprite(5);
    let op = frames::set_duration(&s, &[s.frames()[2].id], 37).expect("dur");
    ph2d_pixel_core::apply(&mut s, op).expect("aplica");
    let mut sons = Sons::default();
    sons.insere("a.wav", som(1));
    let (mut engine, _r) = AudioEngine::new(AudioFormat::stereo(OUT));
    for faixas in [
        vec![],
        vec![faixa("a.wav", 0, true)],
        vec![faixa("falta.wav", 0, false)],
    ] {
        let op = Op::Doc(Box::new(DocOp::SetAudio(faixas.clone())));
        ph2d_pixel_core::apply(&mut s, op).expect("aplica");
        let mut t = Tocador::default();
        t.tocar(&mut engine, s.audio(), &sons, 0).expect("toca");
        assert!(t.relogio().is_none(), "{faixas:?}");
        let (mut hoje, mut com) = (
            Play::start(&s, 0, PlayOptions::default()),
            Play::start(&s, 0, PlayOptions::default()),
        );
        let (mut a, mut b) = (Vec::new(), Vec::new());
        for i in 0..600 {
            hoje.advance(&s, dt_do_host(i));
            com.advance(&s, dt_do_play(t.relogio(), dt_do_host(i)));
            a.push(hoje.index);
            b.push(com.index);
        }
        assert_eq!(a, b);
    }
}

/// Os picos: um por 10 ms, e o máximo é o da fonte.
#[test]
fn os_picos_sao_um_por_janela() {
    let d = som(1);
    let p = picos(&d);
    assert_eq!(p.len(), 100);
    let max = d.samples().iter().copied().fold(0.0_f32, f32::max);
    assert!((p.iter().map(|x| x.max).fold(0.0_f32, f32::max) - max).abs() < 1e-6);
}
