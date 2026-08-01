//! Gates do motor de snap (ADR-0108 + plano 25 §9). Irmão de `snap.rs` pelo teto de LOC.
use super::*;
use ph2d_vec_scene::{VecXforms, rectangle};

fn targets(pts: &[[f64; 2]]) -> SnapTargets {
    SnapTargets {
        points: pts.to_vec(),
        ..SnapTargets::default()
    }
}

fn cfg() -> SnapConfig {
    SnapConfig {
        threshold: 1.0,
        ..SnapConfig::default()
    }
}

/// Uma grade quadrada de passo `step`, com um raio de magnetismo — o mesmo
/// contrato do `GridSnapState` real (fora do raio ela não reivindica o ponto).
fn square_grid(step: f64, radius: f64) -> impl FnMut([f64; 2]) -> Option<[f64; 2]> {
    move |p: [f64; 2]| {
        let g = [(p[0] / step).round() * step, (p[1] / step).round() * step];
        let d2 = (g[0] - p[0]).powi(2) + (g[1] - p[1]).powi(2);
        (d2 <= radius * radius).then_some(g)
    }
}

#[test]
fn a_point_inside_the_threshold_snaps_exactly_onto_the_target() {
    let r = snap(&[[9.5, 20.4]], &targets(&[[10.0, 20.0]]), cfg(), None);
    assert_eq!(r.apply([9.5, 20.4]), [10.0, 20.0]);
    assert_eq!(r.x.unwrap().from, SnapSource::Shape);
}

/// O ponto do lema: os eixos são independentes. Aqui só o X está no limiar —
/// o Y desliza livre. Sem isso não dá pra alinhar bordas sem colar o objeto.
#[test]
fn each_axis_snaps_on_its_own() {
    let r = snap(&[[9.8, 50.0]], &targets(&[[10.0, 20.0]]), cfg(), None);
    assert_eq!(r.apply([9.8, 50.0]), [10.0, 50.0]);
    assert!(r.x.is_some() && r.y.is_none());
}

#[test]
fn nothing_snaps_beyond_the_threshold_or_when_disabled() {
    assert!(
        snap(&[[8.0, 20.0]], &targets(&[[10.0, 20.4]]), cfg(), None)
            .x
            .is_none()
    );
    let off = SnapConfig {
        enabled: false,
        ..cfg()
    };
    // Desligado (Alt segurado) ignora forma E grade.
    let mut grid = square_grid(5.0, 1.0);
    let r = snap(
        &[[9.9, 20.0]],
        &targets(&[[10.0, 20.0]]),
        off,
        Some(&mut grid),
    );
    assert_eq!(r.delta(), [0.0, 0.0]);
}

#[test]
fn the_nearest_candidate_wins_each_axis() {
    let r = snap(
        &[[10.0, 10.0]],
        &targets(&[[10.7, 9.4], [10.2, 10.9]]),
        cfg(),
        None,
    );
    assert_eq!(r.x.unwrap().target, [10.2, 10.9], "dx = 0.2 < 0.7");
    assert_eq!(r.y.unwrap().target, [10.7, 9.4], "dy = -0.6, |−0.6| < 0.9");
}

/// Vários pontos-fonte (a bbox de uma seleção): o melhor de todos vence.
#[test]
fn many_sources_offer_the_best_of_all() {
    let bbox = bbox_key_points([0.0, 0.0], [10.0, 10.0]);
    // Alvo colado no canto superior direito.
    let r = snap(&bbox, &targets(&[[10.3, 10.0]]), cfg(), None);
    assert!((r.x.unwrap().delta - 0.3).abs() < 1e-9);
    assert_eq!(
        r.x.unwrap().source,
        [10.0, 0.0],
        "o canto inferior-direito acha primeiro (empate → 1º)"
    );
    assert_eq!(r.y.unwrap().delta, 0.0);
}

/// A grade reivindica os DOIS eixos de uma vez — um ponto de rede é 2D, e
/// decompor por eixo só faria sentido num grid quadrado (existem nove tipos).
#[test]
fn the_grid_claims_both_axes_as_one_lattice_point() {
    let mut grid = square_grid(5.0, 1.0);
    let r = snap(
        &[[9.6, 20.4]],
        &SnapTargets::default(),
        cfg(),
        Some(&mut grid),
    );
    assert_eq!(r.apply([9.6, 20.4]), [10.0, 20.0]);
    assert!(r.x.unwrap().is_grid() && r.y.unwrap().is_grid());
    assert_eq!(
        r.x.unwrap().target,
        r.y.unwrap().target,
        "o mesmo ponto de rede"
    );
}

/// Fora do raio de magnetismo a grade não reivindica nada — o arrasto segue
/// liso entre pontos de rede (comportamento Figma/Blender).
#[test]
fn outside_the_magnetism_radius_the_grid_stays_quiet() {
    let mut grid = square_grid(5.0, 1.0);
    let r = snap(
        &[[12.5, 12.5]],
        &SnapTargets::default(),
        cfg(),
        Some(&mut grid),
    );
    assert!(!r.any());
}

/// Forma vence régua, **por eixo**: o X encaixa na âncora vizinha e o Y fica
/// com o ponto de rede. É o que deixa alinhar com o desenho sem perder a grade.
#[test]
fn shape_points_override_the_grid_axis_by_axis() {
    let mut grid = square_grid(5.0, 1.0);
    // X: âncora em 9.9 (delta −0.1) bate o grid 10.0 (delta +0.1)? Não — quem
    // decide não é a distância, é a precedência: o ponto SEMPRE sobrescreve.
    let r = snap(
        &[[9.8, 20.4]],
        &targets(&[[9.5, 99.0]]),
        cfg(),
        Some(&mut grid),
    );
    assert_eq!(r.apply([9.8, 20.4]), [9.5, 20.0]);
    assert!(!r.x.unwrap().is_grid(), "X veio da forma");
    assert!(r.y.unwrap().is_grid(), "Y veio da grade");
}

/// Com "Shapes" desligado no painel, só a grade opera.
#[test]
fn with_points_off_only_the_grid_speaks() {
    let mut grid = square_grid(5.0, 1.0);
    let c = SnapConfig {
        to_points: false,
        ..cfg()
    };
    let r = snap(&[[9.6, 20.4]], &targets(&[[9.5, 20.5]]), c, Some(&mut grid));
    assert_eq!(r.apply([9.6, 20.4]), [10.0, 20.0]);
}

#[test]
fn bbox_key_points_are_corners_mids_and_center() {
    let k = bbox_key_points([0.0, 0.0], [10.0, 4.0]);
    assert_eq!(k.len(), 9);
    assert!(k.contains(&[0.0, 0.0]) && k.contains(&[10.0, 4.0]));
    assert!(k.contains(&[5.0, 2.0]), "centro");
    assert!(
        k.contains(&[5.0, 0.0]) && k.contains(&[0.0, 2.0]),
        "meios de aresta"
    );
}

#[test]
fn collect_targets_skips_the_moving_paths_and_the_moving_anchors() {
    let mut scene = VecScene::new();
    let a = scene.push_path(rectangle([0.0, 0.0], [10.0, 10.0]));
    let b = scene.push_path(rectangle([20.0, 0.0], [30.0, 10.0]));

    // Nada excluído: 4 âncoras + 9 pontos de bbox, por path.
    let all = collect_targets(&scene, &VecXforms::new(), &[], &[], false);
    assert_eq!(all.points.len(), 2 * (4 + 9));

    // `a` inteiro fora (é ele que está sendo movido).
    let no_a = collect_targets(&scene, &VecXforms::new(), &[a], &[], false);
    assert_eq!(no_a.points.len(), 4 + 9);
    assert!(!no_a.points.contains(&[0.0, 0.0]));

    // Uma âncora de `b` em movimento: ela sai E a bbox de `b` sai junto —
    // uma caixa que está sendo deformada não serve de referência.
    let deforming = collect_targets(&scene, &VecXforms::new(), &[], &[(b, 0)], false);
    assert_eq!(deforming.points.len(), (4 + 9) + 3);
    assert!(!deforming.points.contains(&[20.0, 0.0]));
}

// ══════════════════════════════════════ a reivindicação 2-D (plano 25 §9, W6)

/// Um segmento cúbico degenerado — a geometria é a reta `a→b`.
fn seg(a: [f64; 2], b: [f64; 2]) -> ph2d_vec_scene::curve_probe::CubicSeg {
    [a, a, b, b]
}

/// Os dois pontos coincidem? Uma reivindicação de POSIÇÃO sai de um refino de Newton, então
/// ela é exacta até o épsilon da máquina e não bit a bit — ao contrário do alinhamento, cuja
/// aritmética é uma subtração só.
fn near(a: [f64; 2], b: [f64; 2]) {
    assert!(
        (a[0] - b[0]).abs() < 1e-9 && (a[1] - b[1]).abs() < 1e-9,
        "{a:?} != {b:?}"
    );
}

/// Config com as duas reivindicações de posição armadas.
fn cfg2d() -> SnapConfig {
    SnapConfig {
        to_path: true,
        to_crossings: true,
        ..cfg()
    }
}

/// **A REGRESSÃO PINADA:** com os interruptores DESLIGADOS, ter geometria na lista de alvos
/// não muda nada. É o que torna a wave segura — todo encaixe que já shipava continua a ser
/// resolvido pelo caminho do alinhamento, termo a termo.
#[test]
fn geometry_in_the_target_list_changes_nothing_while_the_toggles_are_off() {
    let with_geometry = SnapTargets {
        points: vec![[10.0, 20.0]],
        segs: vec![seg([0.0, 0.0], [100.0, 0.0])],
    };
    let plain = targets(&[[10.0, 20.0]]);
    for q in [[9.5, 20.4], [9.8, 50.0], [0.2, 0.3]] {
        assert_eq!(
            snap(&[q], &with_geometry, cfg(), None),
            snap(&[q], &plain, cfg(), None),
            "q = {q:?}"
        );
    }
}

/// A reivindicação de posição pousa o ponto **SOBRE** a curva — os dois eixos de uma vez.
/// Um encaixe por eixo aqui não significaria nada: todo X na faixa da curva é o X de algum
/// ponto dela.
#[test]
fn a_position_claim_lands_the_point_on_the_curve_and_claims_both_axes() {
    let t = SnapTargets {
        points: Vec::new(),
        segs: vec![seg([0.0, 0.0], [100.0, 0.0])],
    };
    let q = [40.0, 0.6];
    let r = snap(&[q], &t, cfg2d(), None);
    near(r.apply(q), [40.0, 0.0]);
    assert_eq!(r.x.unwrap().from, SnapSource::Curve);
    assert_eq!(r.y.unwrap().from, SnapSource::Curve);
}

/// **VÉRTICE VENCE CURVA** — a lei que mantém as quinas alcançáveis.
///
/// A curva passa POR CIMA da âncora, então perto de um vértice as duas espécies competem.
/// Aqui o alinhamento pousa exactamente sobre (10, 10) e a reivindicação de posição, que
/// levaria o ponto para (10.3, 10.0), **se retira**. Sem esta lei nenhum gesto do artista
/// alcançaria o canto: ele pousaria sempre a uma fração de pixel dele.
#[test]
fn a_vertex_beats_the_curve_that_passes_through_it() {
    let t = SnapTargets {
        points: vec![[10.0, 10.0], [20.0, 10.0]],
        segs: vec![seg([10.0, 10.0], [20.0, 10.0])],
    };
    let q = [10.3, 10.2];
    let r = snap(&[q], &t, cfg2d(), None);
    assert_eq!(r.apply(q), [10.0, 10.0], "pousou NA âncora");
    assert_eq!(r.x.unwrap().from, SnapSource::Shape);
}

/// O alinhamento **PARCIAL** não se retira — ele não é uma coincidência. Aqui só o Y tem
/// vizinho no limiar, e a curva ainda reivindica o ponto: se o alinhamento de um eixo só
/// bastasse para calar a posição, encaixar sobre uma linha ficaria impossível sempre que
/// houvesse qualquer vizinha alinhada em algum eixo — quase sempre.
#[test]
fn a_half_alignment_does_not_silence_the_position_claim() {
    let t = SnapTargets {
        points: vec![[500.0, 0.4]],
        segs: vec![seg([0.0, 0.0], [100.0, 0.0])],
    };
    let q = [40.0, 0.6];
    let r = snap(&[q], &t, cfg2d(), None);
    near(r.apply(q), [40.0, 0.0]);
    assert_eq!(r.y.unwrap().from, SnapSource::Curve);
}

/// **CRUZAMENTO VENCE CURVA.** Perto de um cruzamento as duas curvas passam por ali, então
/// as distâncias empatam a menos de ruído de `f64` — decidir por proximidade seria
/// cara-ou-coroa. O cruzamento é o ponto que o desenho produziu, e ganha por posto.
#[test]
fn a_crossing_beats_the_curves_that_make_it() {
    let t = SnapTargets {
        points: Vec::new(),
        segs: vec![
            seg([0.0, 0.0], [100.0, 100.0]),
            seg([0.0, 100.0], [100.0, 0.0]),
        ],
    };
    let q = [50.4, 50.0];
    let r = snap(&[q], &t, cfg2d(), None);
    near(r.apply(q), [50.0, 50.0]);
    assert_eq!(r.x.unwrap().from, SnapSource::Crossing);
}

/// Cada interruptor governa a sua espécie, e a fixture TEM de conter as duas.
///
/// ⚠️ A primeira versão deste gate só olhava uma curva SOLTA, e por isso não podia falhar
/// pelo motivo que alega: sem cruzamento nenhum na cena, ligar ou desligar "Crossings" dá o
/// mesmo resultado por vácuo. Aqui há um X — com "Crossings" desligado o encaixe pousa
/// **sobre a linha** (50,5 / 50,5) e não no cruzamento (50 / 50), que é a diferença que o
/// interruptor promete.
#[test]
fn each_toggle_governs_only_its_own_species() {
    let x_shape = SnapTargets {
        points: Vec::new(),
        segs: vec![
            seg([0.0, 0.0], [100.0, 100.0]),
            seg([0.0, 100.0], [100.0, 0.0]),
        ],
    };
    let q = [50.4, 50.6];

    let only_crossings = SnapConfig {
        to_crossings: true,
        ..cfg()
    };
    let r = snap(&[q], &x_shape, only_crossings, None);
    near(r.apply(q), [50.0, 50.0]);
    assert_eq!(r.x.unwrap().from, SnapSource::Crossing);

    let only_path = SnapConfig {
        to_path: true,
        ..cfg()
    };
    let r = snap(&[q], &x_shape, only_path, None);
    near(r.apply(q), [50.5, 50.5]); // sobre a linha, NÃO no cruzamento
    assert_eq!(r.x.unwrap().from, SnapSource::Curve);

    // E uma curva sem cruzamento nenhum não reivindica nada sob "Crossings" sozinho.
    let lone = SnapTargets {
        points: Vec::new(),
        segs: vec![seg([0.0, 0.0], [100.0, 0.0])],
    };
    assert!(!snap(&[[40.0, 0.6]], &lone, only_crossings, None).any());
}

/// O Alt segurado (`enabled: false`) desliga **também** as reivindicações de posição. Um
/// escape que não escapa de tudo não é escape.
#[test]
fn alt_kills_the_position_claim_too() {
    let t = SnapTargets {
        points: Vec::new(),
        segs: vec![seg([0.0, 0.0], [100.0, 0.0])],
    };
    let off = SnapConfig {
        enabled: false,
        ..cfg2d()
    };
    assert_eq!(snap(&[[40.0, 0.6]], &t, off, None).delta(), [0.0, 0.0]);
}

/// A geometria só é recolhida quando pedida — quem não ligou os interruptores não paga por
/// percorrer os contornos da cena a cada movimento do gizmo.
#[test]
fn collect_targets_gathers_the_geometry_only_when_asked() {
    let mut scene = VecScene::new();
    scene.push_path(rectangle([0.0, 0.0], [10.0, 10.0]));
    let xf = VecXforms::new();
    assert!(
        collect_targets(&scene, &xf, &[], &[], false)
            .segs
            .is_empty()
    );
    assert_eq!(
        collect_targets(&scene, &xf, &[], &[], true).segs.len(),
        4,
        "um retângulo fechado oferece os quatro lados"
    );
}

/// Uma forma cuja geometria está sendo deformada não é referência — nem a caixa dela, nem as
/// CURVAS. A regra já valia para a bbox; estendê-la às curvas é o que impede o nó arrastado
/// de encaixar na aresta que ele próprio está movendo.
#[test]
fn a_deforming_path_offers_neither_its_box_nor_its_curves() {
    let mut scene = VecScene::new();
    let a = scene.push_path(rectangle([0.0, 0.0], [10.0, 10.0]));
    scene.push_path(rectangle([20.0, 0.0], [30.0, 10.0]));
    let t = collect_targets(&scene, &VecXforms::new(), &[], &[(a, 0)], true);
    assert_eq!(t.segs.len(), 4, "só os lados do retângulo INTACTO");
    assert!(
        t.segs.iter().all(|s| s[0][0] >= 20.0),
        "e são os dele: {:?}",
        t.segs
    );
}
