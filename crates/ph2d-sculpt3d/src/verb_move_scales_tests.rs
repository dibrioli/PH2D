//! Gates da **LARGURA DO CAMPO** — a família de escalas do kernel elástico
//! (`Wide` · `Medium` · `Tight`), medida pela porta do PRODUTO.
//!
//! ⚠️ **Irmão do [`super::verb_move_field`] e não parte dele, e o corte é o
//! ASSUNTO:** lá mora *o que o chip `L` troca* (a direção que o campo acrescenta
//! ao puxão, com o `s-mode` de controle em cada gate); aqui mora *quão LARGO o
//! campo é*, que é uma escolha do artista **sobre o modo que ele já escolheu**.
//! Os dois lados variam coisas diferentes e por isso pedem controles diferentes.

use super::verb_move_field::{grab, moved};
use super::*;
use crate::RefMode;
use crate::kelvinlet::Scales;

/// **A LARGURA DO CAMPO CHEGA AO BARRO, E NA DIREÇÃO QUE OS RÓTULOS PROMETEM:
/// a saia acompanha mais no `Wide` que no `Tight`, e a PONTA segue o dedo
/// igual nos três.**
///
/// ⚠️ **As duas metades são o gate, e a segunda é a que dá sentido à primeira.**
/// Um kernel que simplesmente escalasse o deslocamento inteiro passaria pela
/// metade da saia e falharia aqui: a família de escalas **redistribui** o que o
/// campo carrega, ela não muda quanto o dedo leva. Medido pela porta do
/// produto, a ponta vale **0,200000003 nas três** (o `pull` autorado, ao bit) e
/// a saia a meio raio vale **0,136837 / 0,097633 / 0,084063** — o `Wide` carrega
/// **1,63×** o que o `Tight` carrega.
///
/// ⚠️ **E é por isso que o rótulo diz LARGURA e não a aritmética**
/// (`Mono`/`Bi`/`Tri` dizem quantos kelvinlets a soma tem, o que não ajuda
/// ninguém a escolher): o que o artista vê é o quanto a vizinhança acompanha.
#[test]
fn the_field_width_reaches_the_clay_in_the_direction_the_labels_promise() {
    let rest = sphere();
    let pull = [0.2, 0.0, 0.0];
    let run = |sc: Scales| {
        let mut mesh = sphere();
        let mut stroke = SculptStroke::default();
        stroke.begin(&mesh);
        let mut brush = grab(RefMode::L);
        brush.elastic_scales = sc;
        stroke.dab(
            &mut mesh,
            &brush,
            &Dab::pulling([0.0, 0.0, 1.0], 0.5, [0.0, 0.0, -1.0], pull),
            Symmetry::default(),
        );
        mesh
    };
    // A meio raio do bico, ao longo do puxão — onde a saia dos três difere.
    let skirt = [0.5, 0.0, (1.0f32 - 0.25).sqrt()];
    let tip = [0.0, 0.0, 1.0];

    let m: Vec<_> = Scales::ALL.into_iter().map(run).collect();
    let (wide, medium, tight) = (
        moved(&rest, &m[0], skirt),
        moved(&rest, &m[1], skirt),
        moved(&rest, &m[2], skirt),
    );
    assert!(
        wide > medium * 1.15 && medium > tight * 1.05,
        "a largura do campo não chegou ao barro: Wide {wide:.6} · \
         Medium {medium:.6} · Tight {tight:.6}"
    );

    // A metade que impede um escalar global de passar: o dedo leva o mesmo.
    for (sc, mesh) in Scales::ALL.into_iter().zip(&m) {
        let at_tip = moved(&rest, mesh, tip);
        assert!(
            (at_tip - 0.2).abs() < 1e-5,
            "{} moveu a ponta {at_tip:.9} em vez do puxão autorado",
            sc.label()
        );
    }
}
