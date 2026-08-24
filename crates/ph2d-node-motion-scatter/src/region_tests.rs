//! Os gates da **REGIÃO** e da **DENSIDADE GRADUADA** deste nó (doc 89, folha 01).
//!
//! ⚠️ **A afirmação que os une é a que a composição não sabe fazer: a contagem não
//! se mexe.** `motion.falloff → motion.cull` sabe recortar um círculo e sabe ralear
//! por probabilidade; o que ela não sabe é entregar **os `count` pontos que o
//! artista pediu** dentro da forma nova. É isso que cada gate aqui mede.

use super::tests::rect;
use super::*;
use ph2d_motion_region::{SHAPE_CIRCLE, SHAPE_RECT, SHAPE_RING};

const N: usize = 240;
const SEED: u32 = 7;

/// ⭐ **O RETÂNGULO CONTINUA SENDO O QUE ERA, AO BIT** — o default não move um
/// ponto, e é isso que torna o param adoptável numa cena já salva.
#[test]
fn the_default_shape_reproduces_the_old_layout_bit_for_bit() {
    let a = scatter(N, &rect(4.0, 3.0), 0.0, SEED);
    // A expressão que o nó tinha escrita à mão, reconstruída aqui.
    let b: Vec<[f32; 2]> = {
        let (w, h) = (4.0_f32, 3.0_f32);
        let mut placed: Vec<[f32; 2]> = Vec::with_capacity(N);
        for i in 0..N {
            let (mut best, mut best_sq) = ([0.0_f32, 0.0], -1.0_f32);
            for k in 0..CANDIDATES {
                let key = i as u32 * CANDIDATES + k;
                let p = [
                    (hash3(SEED, key, 0) - 0.5) * w,
                    (hash3(SEED, key, 1) - 0.5) * h,
                ];
                let d = nearest_sq(p, &placed);
                if d > best_sq {
                    best_sq = d;
                    best = p;
                }
            }
            placed.push(best);
        }
        placed
    };
    assert_eq!(a.len(), b.len());
    for (i, (p, q)) in a.iter().zip(&b).enumerate() {
        assert_eq!(
            p.map(f32::to_bits),
            q.map(f32::to_bits),
            "ponto {i}: {p:?} contra {q:?}"
        );
    }
}

/// ⭐⭐ **O CÍRCULO EMPACOTA — ele não recorta.** A contagem é a que se pediu, e
/// todos os pontos estão no disco.
///
/// ⚠️ **É esta a metade que a cadeia `falloff → cull` não entrega**: ela devolveria
/// `π/4 ≈ 78%` dos pontos, e num anel de buraco `0,8` devolveria **36%**.
#[test]
fn the_circle_packs_the_full_count_where_the_cull_chain_would_lose_a_fifth() {
    let circle = Region::of(SHAPE_CIRCLE as f32, 4.0, 4.0, 0.0);
    let pts = scatter(N, &circle, 0.0, SEED);
    assert_eq!(pts.len(), N, "a contagem e' a que se pediu");
    for p in &pts {
        assert!(circle.contains(*p), "fora do disco: {p:?}");
    }
    // O CONTROLE do que a composição daria: quantos do layout RETANGULAR
    // sobreviveriam ao mesmo corte.
    let culled = scatter(N, &rect(4.0, 4.0), 0.0, SEED)
        .into_iter()
        .filter(|p| circle.contains(*p))
        .count();
    assert!(
        culled < N * 9 / 10,
        "a rota do cull tinha de PERDER pontos: {culled} de {N}"
    );
}

/// E o anel também empacota — com o buraco vazio.
#[test]
fn the_ring_packs_the_full_count_and_leaves_the_hole_empty() {
    let ring = Region::of(SHAPE_RING as f32, 5.0, 5.0, 0.55);
    let pts = scatter(N, &ring, 0.0, SEED);
    assert_eq!(pts.len(), N);
    for p in &pts {
        assert!(ring.contains(*p), "fora do anel: {p:?}");
        assert!(
            ring.radial(*p) >= 0.55 - 1e-3,
            "caiu no buraco: r={}",
            ring.radial(*p)
        );
    }
}

/// ⭐⭐ **A DENSIDADE GRADUADA MOVE OS PONTOS SEM MUDAR QUANTOS SÃO** — a metade da
/// célula que a composição de três nós trocava por uma contagem menor.
///
/// A régua é a fração de pontos no disco interior (metade da ÁREA): uniforme dá
/// ~50%, e com a densidade a cair para a borda tem de subir bem acima disso.
#[test]
fn a_graded_density_moves_the_points_inward_and_keeps_the_count() {
    let circle = Region::of(SHAPE_CIRCLE as f32, 6.0, 6.0, 0.0);
    let inner_half = |pts: &[[f32; 2]]| {
        pts.iter()
            .filter(|p| circle.radial(**p) <= std::f32::consts::FRAC_1_SQRT_2)
            .count() as f32
            / pts.len() as f32
    };
    let flat = scatter(N, &circle, 0.0, SEED);
    let graded = scatter(N, &circle, 1.0, SEED);
    assert_eq!(graded.len(), flat.len(), "a contagem NAO se mexe");
    let (a, b) = (inner_half(&flat), inner_half(&graded));
    assert!(
        (a - 0.5).abs() < 0.08,
        "CONTROLE: sem gradacao a metade da area leva metade dos pontos ({a:.3})"
    );
    assert!(
        b > a + 0.10,
        "a gradacao tinha de puxar os pontos para dentro: {a:.3} -> {b:.3}"
    );
    // E ninguém sai da região.
    for p in &graded {
        assert!(circle.contains(*p), "fora do disco: {p:?}");
    }
}

/// ⚠️ **A gradação não é uma máscara: ela não deixa a borda VAZIA.** O piso da
/// densidade é `0,2`, então a borda continua povoada — mais rala, não deserta.
///
/// *Uma cura que esvaziasse a borda seria o `cull` outra vez, com outro nome.*
#[test]
fn the_graded_edge_is_thinner_never_empty() {
    let circle = Region::of(SHAPE_CIRCLE as f32, 6.0, 6.0, 0.0);
    let graded = scatter(N, &circle, 1.0, SEED);
    let outer = graded.iter().filter(|p| circle.radial(**p) > 0.85).count();
    assert!(
        outer > 0,
        "a borda ficou DESERTA -- isso e' um corte, nao uma densidade"
    );
}

/// A escada do param e a da região andam juntas — um número novo na crate-folha
/// sem rótulo aqui daria um dropdown com um buraco.
#[test]
fn the_shape_slider_reaches_every_shape_the_region_knows() {
    let hint = PARAM_HINTS
        .iter()
        .find(|h| h.param == ph2d_motion_region::SHAPE)
        .expect("o `shape` tem hint");
    let ph2d_node_registry::ParamWidget::Enum { labels } = hint.widget else {
        panic!("o `shape` e' um enum");
    };
    assert_eq!(labels.len(), ph2d_motion_region::SHAPE_LABELS.len());
    assert_eq!(
        hint.max,
        (labels.len() - 1) as f32,
        "o slider tem de alcancar o ultimo rotulo"
    );
    assert_eq!(hint.min, SHAPE_RECT as f32);
}

/// ⚠️ **Os três params novos são DECLARADOS**, senão o painel não os desenha e o
/// documento não os guarda — a costura que já mordeu esta conferência.
#[test]
fn every_new_param_is_declared_and_defaults_to_today() {
    for (name, want) in [(ph2d_motion_region::SHAPE, 0.0), (DENSITY_FALLOFF, 0.0)] {
        let d = MANIFEST.param_default(name).expect("declarado: {name}");
        assert_eq!(d, want, "{name} tem de reduzir ao no' de hoje");
    }
    // O `inner` tem default próprio (um anel de banda a meio), porque ele só é
    // lido no modo que o gate deixa aparecer — um `0` ali daria um anel que é um
    // disco no instante em que alguém escolhe `Ring`.
    assert!(MANIFEST.param_default(ph2d_motion_region::INNER).is_some());
    for h in [
        ph2d_motion_region::SHAPE,
        ph2d_motion_region::INNER,
        DENSITY_FALLOFF,
    ] {
        assert!(
            PARAM_HINTS.iter().any(|x| x.param == h),
            "{h} sem hint de painel"
        );
    }
}
