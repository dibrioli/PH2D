//! Gates da sonda de curva (plano 25 §9, W6).
//!
//! ⚠️ **O oráculo é sempre uma RETA**, e não por preguiça: um cúbico degenerado
//! `[a, a, b, b]` desenha o segmento `a→b` **com parametrização não-uniforme** (`3t² − 2t³`,
//! o smoothstep). Isso dá as duas coisas de que um gate precisa ao mesmo tempo — a resposta
//! exata tem forma fechada (o pé da perpendicular sobre uma reta), e a amostragem erra por
//! muito, porque as amostras se amontoam nas pontas. Um oráculo sobre curva de verdade teria
//! de re-derivar a resposta com o mesmo maquinário que está sob teste.

use super::*;
use crate::{VecPath, VecVertex};

/// Um segmento cúbico DEGENERADO entre dois pontos: a geometria é a reta `a→b`.
fn line(a: [f64; 2], b: [f64; 2]) -> CubicSeg {
    [a, a, b, b]
}

fn d(a: [f64; 2], b: [f64; 2]) -> f64 {
    dist2(a, b).sqrt()
}

// ------------------------------------------------------- 1. a projeção é REFINADA

/// **O ponto devolvido está sobre a curva, não sobre uma AMOSTRA.**
///
/// Este é o gate que separa esta sonda do [`crate::nearest_point_on_path`]: numa reta de
/// 1000 unidades as 17 amostras caem em `1000·(3t² − 2t³)`, e a mais próxima do pé
/// verdadeiro (531, 0) está em **x = 500** — 31 unidades fora. O Newton fecha para o valor
/// exato. Sem o refino, o snap pousaria o nó a 31 unidades de onde o ímã prometeu, e a um
/// zoom alto isso é a metade da tela.
#[test]
fn the_projection_lands_on_the_curve_not_on_the_nearest_sample() {
    let segs = [line([0.0, 0.0], [1000.0, 0.0])];
    let q = [531.0, 7.0];
    let p = nearest_on_segs(&segs, q, 50.0).expect("a reta passa a 7 unidades");
    assert!(
        (p[0] - 531.0).abs() < 1e-6,
        "o pé da perpendicular é exato: x = {} (esperado 531)",
        p[0]
    );
    assert!(p[1].abs() < 1e-9, "e está SOBRE a reta: y = {}", p[1]);
    // A amostra mais próxima — o que uma sonda sem refino devolveria.
    let best_sample = (0..=SAMPLES)
        .map(|k| {
            #[expect(clippy::cast_precision_loss, reason = "k <= 16")]
            let t = k as f64 / SAMPLES as f64;
            at(&segs[0], t)
        })
        .min_by(|a, b| dist2(*a, q).total_cmp(&dist2(*b, q)))
        .expect("há amostras");
    assert!(
        (best_sample[0] - 531.0).abs() > 20.0,
        "a fixture TEM de conter o fenômeno: a melhor amostra está em x = {}",
        best_sample[0]
    );
}

/// Fora do alcance a sonda **solta**. Sem isto o ímã puxaria de qualquer distância, e o
/// artista perderia o direito de pousar um nó perto de uma curva sem pousar NELA.
#[test]
fn nothing_is_claimed_beyond_the_reach() {
    let segs = [line([0.0, 0.0], [100.0, 0.0])];
    assert!(nearest_on_segs(&segs, [50.0, 4.0], 5.0).is_some());
    assert!(nearest_on_segs(&segs, [50.0, 6.0], 5.0).is_none());
}

/// A caixa do casco de controle é um **superconjunto** da curva, então a rejeição nunca
/// descarta um segmento que poderia vencer — nem quando a curva se afasta muito dos seus
/// pontos de controle.
#[test]
fn a_curve_that_bulges_away_from_its_hull_is_still_found() {
    // Controles muito acima: a curva sobe até ~y = 75 e volta.
    let arch: CubicSeg = [[0.0, 0.0], [0.0, 100.0], [100.0, 100.0], [100.0, 0.0]];
    let p = nearest_on_segs(&[arch], [50.0, 80.0], 10.0).expect("o topo do arco alcança");
    assert!(p[1] > 70.0, "pousou no topo do arco: {p:?}");
}

// ------------------------------------------------------- 2. o cruzamento é EXATO

/// Duas retas em X cruzam em (50, 50), e a sonda devolve **esse** ponto.
#[test]
fn a_crossing_is_found_and_it_is_exact() {
    let segs = [
        line([0.0, 0.0], [100.0, 100.0]),
        line([0.0, 100.0], [100.0, 0.0]),
    ];
    let x = crossings_near(&segs, [52.0, 48.0], 10.0);
    assert_eq!(x.len(), 1, "um cruzamento, não um por amostra: {x:?}");
    assert!(d(x[0], [50.0, 50.0]) < 1e-6, "exato: {:?}", x[0]);
}

/// **O cruzamento é REFINADO, não o encontro das CORDAS.**
///
/// ⚠️ O gate acima **não pode ver isto**, e a razão é instrutiva: entre duas RETAS as cordas
/// amostradas estão exactamente sobre as retas, então o candidato já é a resposta e o Newton
/// não tem o que corrigir. É preciso curvatura. Aqui um arco cruza uma horizontal, e o
/// oráculo é **analítico e independente**: para o cúbico `[(0,0),(0,60),(100,60),(100,0)]`
/// vale `y(t) = 180·t·(1−t)` e `x(t) = 100·t²·(3−2t)`, então `y = 30` dá
/// `t = (1 − 1/√3)/2` em forma fechada. O encontro das cordas erra **0,33 unidade**; o
/// refino fecha em `1e-9`.
#[test]
fn a_crossing_between_curves_is_refined_not_merely_sampled() {
    let arch: CubicSeg = [[0.0, 0.0], [0.0, 60.0], [100.0, 60.0], [100.0, 0.0]];
    let segs = [arch, line([0.0, 30.0], [100.0, 30.0])];
    let t = (1.0 - (1.0_f64 / 3.0).sqrt()) / 2.0;
    let want = [100.0 * t * t * (3.0 - 2.0 * t), 30.0];

    let x = crossings_near(&segs, [want[0], 30.0], 5.0);
    assert_eq!(x.len(), 1, "um cruzamento perto da consulta: {x:?}");
    assert!(
        d(x[0], want) < 1e-9,
        "exato contra a forma fechada: {:?} vs {want:?}",
        x[0]
    );
}

/// **A ÂNCORA COMPARTILHADA NÃO É UM CRUZAMENTO.** Dois segmentos vizinhos de um traço
/// dividem a âncora **por construção** — sem esta regra, toda junta do desenho viraria um
/// alvo de cruzamento, e o ímã dos cruzamentos passaria a disputar cada canto com o das
/// âncoras, que já respondem por aquele ponto.
///
/// ⚠️ **Os segmentos aqui têm HANDLES de verdade**, e isso é o gate. Com o cúbico degenerado
/// do helper [`line`] a derivada é ZERO nas duas pontas, então um encontro exactamente na
/// ponta morre no guard de degeneração do Newton — **outra** camada, gateada pelo irmão
/// abaixo. Mutar a regra da ponta sobre a fixture degenerada não sangra, e a defesa parecia
/// testada sem estar.
#[test]
fn a_shared_anchor_is_not_a_crossing() {
    let segs: [CubicSeg; 2] = [
        [[0.0, 0.0], [3.0, 0.0], [7.0, 0.0], [10.0, 0.0]],
        [[10.0, 0.0], [10.0, 3.0], [10.0, 7.0], [10.0, 10.0]],
    ];
    assert!(
        crossings_near(&segs, [10.0, 0.0], 5.0).is_empty(),
        "a junta de dois segmentos vizinhos não é cruzamento"
    );
}

/// A **segunda camada**, e ela é do produto e não teórica: um polígono desenhado com a caneta
/// tem vértices `Corner`, cujos handles ficam recolhidos na âncora — os segmentos são
/// exactamente os cúbicos degenerados de [`line`], sem tangente nas pontas. Ali o sistema de
/// Newton é singular, e o encontro é recusado por esse caminho.
///
/// ⚠️ **O guard explícito de `den ≈ 0` não é observável, e isso foi MEDIDO:** removê-lo deixa
/// os doze gates verdes, porque `0/0` vira `NaN`, o `clamp` o propaga e a aceitação final
/// (`dist2 <= tol`) devolve `false` sobre `NaN`. O guard fica por tornar a intenção visível —
/// depender de propagação de `NaN` é o tipo de coisa que morre em silêncio quando alguém
/// acrescenta um `is_finite` acima —, mas esta nota existe para ninguém o creditar por uma
/// proteção que o gate não prova.
#[test]
fn a_shared_anchor_between_handle_less_corners_is_not_a_crossing_either() {
    let segs = [
        line([0.0, 0.0], [10.0, 0.0]),
        line([10.0, 0.0], [10.0, 10.0]),
    ];
    assert!(crossings_near(&segs, [10.0, 0.0], 5.0).is_empty());
}

/// A busca é **LOCALIZADA**: um cruzamento longe do cursor não é devolvido. É esta
/// propriedade que dispensa o cache por gesto que o plano previa — nada envelhece porque
/// nada é guardado.
#[test]
fn only_the_crossings_near_the_query_come_back() {
    let segs = [
        line([0.0, 0.0], [100.0, 100.0]),
        line([0.0, 100.0], [100.0, 0.0]),
        line([900.0, 0.0], [1000.0, 100.0]),
        line([900.0, 100.0], [1000.0, 0.0]),
    ];
    let near = crossings_near(&segs, [50.0, 50.0], 10.0);
    assert_eq!(near.len(), 1, "só o cruzamento de perto: {near:?}");
    assert!(d(near[0], [50.0, 50.0]) < 1e-6);
    let far = crossings_near(&segs, [950.0, 50.0], 10.0);
    assert_eq!(far.len(), 1);
    assert!(d(far[0], [950.0, 50.0]) < 1e-6);
}

/// Curvas que não se tocam não inventam cruzamento.
#[test]
fn parallel_lines_never_cross() {
    let segs = [
        line([0.0, 0.0], [100.0, 0.0]),
        line([0.0, 10.0], [100.0, 10.0]),
    ];
    assert!(crossings_near(&segs, [50.0, 5.0], 20.0).is_empty());
}

// ------------------------------------------------------- 3. a geometria é a COZIDA

/// **`world_segs` percorre a geometria COZIDA.** Numa quina com raio vivo (ADR-0121) o
/// documento guarda o vértice AFIADO e o mundo consome o arredondado; encaixar na fonte
/// deixaria o ímã a um raio de distância da linha que o artista vê.
///
/// O oráculo é a distância do PONTO DA QUINA à geometria: com raio, a curva passa longe
/// dele (`r·(√2 − 1)` num canto reto); sem raio, ela passa exactamente por ele — e o
/// controle é o que torna o número acima significativo.
#[test]
fn world_segs_walks_the_cooked_geometry_not_the_authored_corner() {
    let corner = [10.0, 0.0];
    let verts: Vec<VecVertex> = [[0.0, 0.0], corner, [10.0, 10.0], [0.0, 10.0]]
        .map(VecVertex::corner)
        .to_vec();

    let sharp = VecPath {
        verts: verts.clone(),
        closed: true,
        ..VecPath::default()
    };
    let mut segs = Vec::new();
    world_segs(&sharp, &Xform::IDENTITY, &mut segs);
    let hit = nearest_on_segs(&segs, corner, 5.0).expect("a quina afiada está no traço");
    assert!(
        d(hit, corner) < 1e-9,
        "CONTROLE: sem raio a geometria passa pela quina ({hit:?})"
    );

    let mut rounded_verts = verts;
    rounded_verts[1].corner_radius = 3.0;
    let rounded = VecPath {
        verts: rounded_verts,
        closed: true,
        ..VecPath::default()
    };
    let mut segs = Vec::new();
    world_segs(&rounded, &Xform::IDENTITY, &mut segs);
    let hit = nearest_on_segs(&segs, corner, 5.0).expect("a curva arredondada passa perto");
    assert!(
        d(hit, corner) > 0.5,
        "com raio a curva se AFASTA da quina autorada — distância {:.3}",
        d(hit, corner)
    );
}

/// O `Xform` é aplicado: os segmentos saem em MUNDO. Um alvo de snap em espaço local
/// encaixaria onde a forma **não está** (ADR-0111).
#[test]
fn world_segs_are_in_world_space() {
    let path = VecPath {
        verts: [[0.0, 0.0], [10.0, 0.0]].map(VecVertex::corner).to_vec(),
        closed: false,
        ..VecPath::default()
    };
    let shifted = Xform([1.0, 0.0, 0.0, 1.0, 100.0, 50.0]);
    let mut segs = Vec::new();
    world_segs(&path, &shifted, &mut segs);
    assert_eq!(segs.len(), 1);
    assert_eq!(segs[0][0], [100.0, 50.0]);
    assert_eq!(segs[0][3], [110.0, 50.0]);
}

/// Um contorno FECHADO tem tantos segmentos quantos vértices (o de volta ao começo conta);
/// um aberto tem um a menos. É o que faz o lado de fechamento de um retângulo ser
/// encaixável como qualquer outro.
#[test]
fn a_closed_contour_offers_the_closing_segment_too() {
    let verts = [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0]]
        .map(VecVertex::corner)
        .to_vec();
    for (closed, want) in [(false, 2), (true, 3)] {
        let path = VecPath {
            verts: verts.clone(),
            closed,
            ..VecPath::default()
        };
        let mut segs = Vec::new();
        world_segs(&path, &Xform::IDENTITY, &mut segs);
        assert_eq!(segs.len(), want, "closed = {closed}");
    }
}
