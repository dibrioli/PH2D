//! P14a-3: `repeat` N do Aseprite conta PASSADAS; a porta `tag_do_aseprite` traduz para ciclos e,
//! N ímpar, o último ciclo é meio e pára na ponta de chegada (lei do Trial, `ph2d_pixel_core::play`).

use ph2d_ecs::{SpriteAnimator, advance, tag_do_aseprite};

/// O que o ecrã mostra, um quadro de 100 ms de cada vez, até parar.
fn sequencia(direcao: u8, n: u16) -> Vec<u32> {
    let tag = tag_do_aseprite("run", 2, 4, direcao, n, &[100; 6]);
    let mut a = SpriteAnimator::new("run");
    a.playing = true;
    a.rewind(Some(&tag));
    let mut frame = if direcao == 3 { 4 } else { 2 };
    let mut vista = vec![frame];
    for _ in 0..40 {
        if !a.playing {
            break;
        }
        advance(&mut a, &tag, &mut frame, 6, 100_000);
        if a.playing || vista.last() != Some(&frame) {
            vista.push(frame);
        }
    }
    vista.dedup();
    assert!(!a.playing, "N = {n} tem fim");
    vista
}

#[test]
fn pingpong_com_passadas_impares_acaba_na_ponta_de_chegada() {
    assert_eq!(sequencia(2, 1), [2, 3, 4], "N=1: uma passada");
    assert_eq!(sequencia(2, 2), [2, 3, 4, 3, 2], "N=2: ida e volta");
    assert_eq!(sequencia(2, 3), [2, 3, 4, 3, 2, 3, 4], "N=3: pára em 4");
    assert_eq!(
        sequencia(3, 3),
        [4, 3, 2, 3, 4, 3, 2],
        "PingPongReverse N=3: pára em 2"
    );
    // Forward conta passadas = ciclos, como sempre.
    assert_eq!(sequencia(0, 2), [2, 3, 4, 2, 3, 4]);
}
