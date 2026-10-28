//! O HUD vivo: as caixas das âncoras e dos rótulos mudam a cada quadro, e o arranjo não se remonta por isso.

use ph2d_hud_layout::{Arranjo, Eixo, Elemento, Forca, Lado, Relacao};

use crate::{bits, caixa, el};

fn relacoes() -> Vec<(Relacao, Forca)> {
    vec![
        (Relacao::LarguraDoConteudo { a: 0 }, Forca::Media),
        (
            Relacao::AoLado {
                a: 1,
                b: 0,
                lado: Lado::Direita,
                folga: 8.0,
            },
            Forca::Requerida,
        ),
        // Quem segue é quem cede: a pontuação é âncora de todos (sem empate de pesos — ótimo único).
        (
            Relacao::NaoInvade {
                a: 2,
                b: 0,
                lado: Lado::Esquerda,
                folga: 8.0,
            },
            Forca::Requerida,
        ),
        (
            Relacao::Centrado {
                a: 1,
                b: 0,
                eixo: Eixo::Y,
            },
            Forca::Forte,
        ),
    ]
}

fn quadro(k: f64) -> Vec<Elemento> {
    // A pontuação cresce com o texto; o minimapa anda com a âncora da direita.
    vec![
        el(16.0, 312.0, 40.0 + 9.75 * k, 32.0, Some(40.0 + 9.75 * k)),
        el(200.0, 316.0, 24.0, 24.0, None),
        el(504.0 - 3.5 * k, 240.0, 120.0, 104.0, None),
    ]
}

#[test]
fn moving_boxes_give_the_bits_of_a_fresh_build() {
    let rel = relacoes();
    let tela = caixa(0.0, 0.0, 640.0, 360.0);
    let mut vivo = Arranjo::new(tela, &quadro(0.0), &rel);
    for k in [0.0, 1.0, 5.0, 17.0, 30.0, 5.0, 0.0] {
        let els = quadro(k);
        let estreita = caixa(0.0, 0.0, 640.0 - 4.0 * k, 360.0);
        let fresco = Arranjo::new(estreita, &els, &rel).atualiza_elementos(estreita, &els);
        assert_eq!(
            bits(&vivo.atualiza_elementos(estreita, &els)),
            bits(&fresco),
            "quadro {k}"
        );
    }
}

#[test]
fn a_box_that_moves_without_relations_is_still_its_own_box() {
    let rel = relacoes();
    let tela = caixa(0.0, 0.0, 640.0, 360.0);
    let mut els = quadro(0.0);
    els.push(el(1.0, 2.0, 3.0, 4.0, None));
    let mut vivo = Arranjo::new(tela, &els, &rel);
    els[3] = el(9.5, 8.5, 7.5, 6.5, None);
    assert_eq!(
        bits(&vivo.atualiza_elementos(tela, &els)[3..]),
        bits(&[els[3].caixa])
    );
    // Elementos a menos não rebentam: quem falta fica com a última caixa.
    assert_eq!(vivo.atualiza_elementos(tela, &els[..1]).len(), 4);
}
