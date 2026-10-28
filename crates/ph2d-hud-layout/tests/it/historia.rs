//! As duas histórias do dono: a moeda colada à pontuação que cresce; a tela estreita que encolhe o número.

use ph2d_hud_layout::{Forca, Lado, Relacao, resolve};

use crate::{caixa, el};

#[test]
fn the_coin_stays_beside_a_growing_score() {
    let tela = caixa(0.0, 0.0, 640.0, 360.0);
    let rel = [
        (Relacao::LarguraDoConteudo { a: 0 }, Forca::Forte),
        (
            Relacao::AoLado {
                a: 1,
                b: 0,
                lado: Lado::Direita,
                folga: 8.0,
            },
            Forca::Requerida,
        ),
    ];
    // «99» → «12 345»: a largura medida do rótulo cresce.
    for conteudo in [38.5, 51.25, 77.0, 102.75, 128.375] {
        let r = resolve(
            tela,
            &[
                el(16.0, 312.0, 100.0, 32.0, Some(conteudo)),
                el(200.0, 316.0, 24.0, 24.0, None),
            ],
            &rel,
        );
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].x, 16.0, "a pontuação não sai do canto");
        assert_eq!(r[0].w, conteudo, "a pontuação tem a largura do texto");
        assert_eq!(
            r[1].x,
            r[0].x + r[0].w + 8.0,
            "a moeda fica colada, com a folga"
        );
        assert_eq!(
            (r[1].y, r[1].w, r[1].h),
            (316.0, 24.0, 24.0),
            "a moeda só anda em x"
        );
    }
}

#[test]
fn a_narrow_screen_shrinks_the_score_before_it_overlaps() {
    let rel = [
        (Relacao::LarguraDoConteudo { a: 0 }, Forca::Media),
        (Relacao::LarguraDoConteudo { a: 1 }, Forca::Forte),
        (
            Relacao::NaoInvade {
                a: 0,
                b: 1,
                lado: Lado::Direita,
                folga: 8.0,
            },
            Forca::Requerida,
        ),
    ];
    let els = [
        el(16.0, 312.0, 100.0, 32.0, Some(200.0)),
        el(504.0, 240.0, 120.0, 104.0, Some(120.0)),
    ];
    let mut encolheu = false;
    for largura in [640.0, 400.0, 300.0, 250.0] {
        let tela = caixa(0.0, 0.0, largura, 360.0);
        let r = resolve(tela, &els, &rel);
        let (pont, mapa) = (r[0], r[1]);
        assert!(
            pont.x + pont.w + 8.0 <= mapa.x + 1e-9,
            "tela {largura}: a pontuação invadiu o minimapa: {r:?}"
        );
        assert!(
            mapa.x + mapa.w <= largura + 1e-9 && mapa.w == 120.0,
            "tela {largura}: o minimapa é que cedeu: {r:?}"
        );
        assert!(pont.w <= 200.0, "nunca maior que o texto");
        encolheu |= pont.w < 200.0;
    }
    assert!(encolheu, "a tela mais estreita tinha de encolher o número");
}
