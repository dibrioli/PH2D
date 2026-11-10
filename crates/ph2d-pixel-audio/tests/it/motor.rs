//! P13 T: o Play pede o som ao mixer do host (`motor::quadro`), sem dispositivo (nenhum som sai):
//! Play ⇒ tocar a partir do instante do quadro onde começa; Stop ⇒ parar; parado, mudar de quadro
//! com o cursor ⇒ o pedaço desse quadro (o scrub). Sem mixer, o `dt` é o do host.

use ph2d_audio::{AudioEngine, AudioFormat, ChannelLayout, SampleData};
use ph2d_pixel_audio::canal;
use ph2d_pixel_audio::motor::{self, Acao, Vista};
use ph2d_pixel_core::{DocOp, FaixaDeAudio, NewSprite, Op, Sprite, frames};

/// `n` quadros de 100 ms com uma faixa de 2 s que começa no 0.
fn sprite(n: usize, com_som: bool) -> Sprite {
    let mut s = Sprite::new(NewSprite::default()).expect("sprite");
    for _ in 1..n {
        let f = s.frames().last().expect("quadro").id;
        let op = frames::new_frame(&mut s, f, frames::NewFrame::Empty)
            .expect("novo")
            .0;
        ph2d_pixel_core::apply(&mut s, op).expect("aplica");
    }
    let faixas = if com_som {
        vec![FaixaDeAudio {
            asset: "motor.wav".into(),
            inicio_ms: 0,
            ganho: 1.0,
            mudo: false,
        }]
    } else {
        Vec::new()
    };
    ph2d_pixel_core::apply(&mut s, Op::Doc(Box::new(DocOp::SetAudio(faixas)))).expect("som");
    let fmt = AudioFormat {
        sample_rate: 48_000,
        channels: ChannelLayout::Mono,
    };
    let dados = SampleData::from_fn(96_000, fmt, |i| (i % 7) as f32 * 0.01);
    canal::com_sons(|c| c.insere("motor.wav", dados));
    s
}

fn vista(s: &Sprite, quadro: usize, tocando: bool) -> Vista<'_> {
    Vista {
        chave: 7,
        sprite: s,
        quadro,
        tocando,
    }
}

#[test]
fn o_play_pede_o_som_ao_mixer() {
    let s = sprite(6, true);
    let (mut engine, _r) = AudioEngine::new(AudioFormat::stereo(48_000));
    // Parado no quadro 3; Play ⇒ tocar a partir de 300 ms (o instante onde o quadro 3 começa).
    motor::quadro(Some(&mut engine), Some(vista(&s, 3, false)), 16);
    motor::quadro(Some(&mut engine), Some(vista(&s, 3, true)), 16);
    assert_eq!(motor::ultima_acao(), (Some(Acao::Tocar { t_ms: 300 }), 1));
    // A tocar: não volta a pedir (o mesmo Play continua no mixer).
    motor::quadro(Some(&mut engine), Some(vista(&s, 4, true)), 16);
    assert_eq!(motor::ultima_acao(), (Some(Acao::Tocar { t_ms: 300 }), 1));
    // Stop ⇒ parar, e o mixer fica vazio.
    motor::quadro(Some(&mut engine), Some(vista(&s, 3, false)), 16);
    assert_eq!(motor::ultima_acao(), (Some(Acao::Parar), 0));
    // Sem mixer: o `dt` é o do host e nada se pede.
    assert_eq!(motor::quadro(None, Some(vista(&s, 3, true)), 23), 23);
    assert_eq!(motor::ultima_acao().0, Some(Acao::Parar));
}

#[test]
fn o_scrub_pede_o_pedaco_do_quadro() {
    let s = sprite(6, true);
    let (mut engine, _r) = AudioEngine::new(AudioFormat::stereo(48_000));
    motor::quadro(Some(&mut engine), Some(vista(&s, 0, false)), 16);
    motor::quadro(Some(&mut engine), Some(vista(&s, 0, false)), 16);
    assert_ne!(
        motor::ultima_acao().0,
        Some(Acao::Pedaco {
            t_ms: 0,
            dur_ms: 100
        }),
        "o quadro não mudou: nada toca"
    );
    // O cursor arrastado para o quadro 2 ⇒ o pedaço de 200..300 ms.
    motor::quadro(Some(&mut engine), Some(vista(&s, 2, false)), 16);
    assert_eq!(
        motor::ultima_acao(),
        (
            Some(Acao::Pedaco {
                t_ms: 200,
                dur_ms: 100
            }),
            1
        )
    );
    // Sem som no sprite, o scrub fica calado.
    let mudo = sprite(6, false);
    motor::quadro(Some(&mut engine), Some(vista(&mudo, 4, false)), 16);
    assert_eq!(
        motor::ultima_acao().0,
        Some(Acao::Pedaco {
            t_ms: 200,
            dur_ms: 100
        })
    );
}
