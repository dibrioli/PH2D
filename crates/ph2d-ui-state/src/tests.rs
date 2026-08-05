//! Gates do **Smart Animate** (plano UI/UX W7).

use super::*;
use crate::StateSets;
use ph2d_vec_scene::{Paint, Rgba8, VecPath, ellipse, rectangle};

fn rect(id: u64) -> VecPath {
    let mut p = rectangle([0.0, 0.0], [2.0, 1.0]);
    p.id = id;
    p
}

fn posed(id: u64) -> ObjectPose {
    ObjectPose::new(id)
}

/// **O CASAMENTO É POR ID, e ele sobrevive a renomear E a reordenar.**
///
/// ⚠️ Os dois gestos são coisas que o artista faz sem pensar, e nenhum deles pode partir uma
/// animação. É o gate que o plano pede nominalmente, e a fragilidade do Figma (que casa por nome)
/// é a razão de ele existir.
#[test]
fn matching_survives_renaming_and_reordering() {
    let mut a = UiState::new("idle");
    a.objects = vec![
        ObjectPose {
            translation: [0.0, 0.0],
            ..posed(7)
        },
        ObjectPose {
            translation: [5.0, 0.0],
            ..posed(9)
        },
    ];

    // O MESMO destino, escrito de duas maneiras: ordem trocada e nome diferente.
    let mut b1 = UiState::new("hover");
    b1.objects = vec![
        ObjectPose {
            translation: [1.0, 0.0],
            ..posed(7)
        },
        ObjectPose {
            translation: [6.0, 0.0],
            ..posed(9)
        },
    ];
    let mut b2 = UiState::new("PRESSED");
    b2.objects = vec![b1.objects[1].clone(), b1.objects[0].clone()];

    let mid1 = Transition::new(&a, &b1).at(0.5);
    let mid2 = Transition::new(&a, &b2).at(0.5);

    let find = |v: &[ObjectPose], id: u64| v.iter().find(|p| p.id == id).unwrap().translation;
    assert_eq!(find(&mid1, 7), [0.5, 0.0]);
    assert_eq!(find(&mid1, 9), [5.5, 0.0]);
    assert_eq!(
        find(&mid1, 7),
        find(&mid2, 7),
        "reordenar a lista ou renomear o estado mudou para onde o objeto 7 vai"
    );
    assert_eq!(find(&mid1, 9), find(&mid2, 9));
}

/// **Sem par no destino SAI; sem par na origem ENTRA. E nenhum dos dois se MOVE.**
///
/// ⚠️ A segunda metade é a que importa: mover algo que só existe de um lado seria inventar a outra
/// ponta do caminho, e o artista veria um objeto a deslizar de um lugar que ele nunca autorou.
#[test]
fn the_unpaired_fade_without_moving() {
    let mut a = UiState::new("idle");
    a.objects = vec![ObjectPose {
        translation: [3.0, 4.0],
        ..posed(1)
    }];
    let mut b = UiState::new("open");
    b.objects = vec![ObjectPose {
        translation: [-2.0, 8.0],
        ..posed(2)
    }];

    let tr = Transition::new(&a, &b);
    let mid = tr.at(0.5);
    let leaving = mid.iter().find(|p| p.id == 1).expect("o que sai");
    let entering = mid.iter().find(|p| p.id == 2).expect("o que entra");

    assert!(
        (leaving.opacity - 0.5).abs() < 1e-6,
        "quem sai nao desvaneceu"
    );
    assert!(
        (entering.opacity - 0.5).abs() < 1e-6,
        "quem entra nao apareceu"
    );
    assert_eq!(leaving.translation, [3.0, 4.0], "quem sai foi MOVIDO");
    assert_eq!(entering.translation, [-2.0, 8.0], "quem entra foi MOVIDO");

    assert!((tr.at(0.0)[0].opacity - 1.0).abs() < 1e-6);
    assert!(tr.at(1.0)[0].opacity.abs() < 1e-6);
}

/// **IDÊNTICO não anima — e não custa.**
#[test]
fn an_object_that_does_not_change_is_not_in_the_transition() {
    let mut a = UiState::new("idle");
    a.objects = vec![posed(1), posed(2)];
    let mut b = UiState::new("hover");
    b.objects = vec![
        posed(1),
        ObjectPose {
            opacity: 0.2,
            ..posed(2)
        },
    ];

    let tr = Transition::new(&a, &b);
    assert_eq!(tr.len(), 1, "um objeto inalterado entrou na transicao");
    assert_eq!(tr.at(0.5)[0].id, 2);
}

/// **A ROTAÇÃO vai pelo ARCO MAIS CURTO**, e a volta inteira é o caso degenerado nomeado.
#[test]
fn rotation_takes_the_short_way_around() {
    let deg = |d: f64| d.to_radians();
    let mid = |from: f64, to: f64| {
        let mut a = UiState::new("a");
        a.objects = vec![ObjectPose {
            rotation: deg(from),
            ..posed(1)
        }];
        let mut b = UiState::new("b");
        b.objects = vec![ObjectPose {
            rotation: deg(to),
            ..posed(1)
        }];
        Transition::new(&a, &b).at(0.5)[0].rotation.to_degrees()
    };

    // A METADE de meia volta: um lerp de MATRIZ daria uma forma encolhida, e um lerp ingénuo de
    // ângulo daria isto também — o que este gate separa é o caso do wrap, abaixo.
    assert!((mid(0.0, 180.0) - 90.0).abs() < 1e-9);
    // 350 -> 10 anda +20 (o meio e' 0), NUNCA -340 (o meio seria 180 — do outro lado do circulo).
    let m = mid(350.0, 10.0);
    assert!(
        m.abs() < 1e-9 || (m - 360.0).abs() < 1e-9,
        "a rotacao deu a volta longa: meio em {m} graus"
    );
    // ⚠️ A consequência NOMEADA: uma volta inteira é o mesmo ângulo, logo ela não gira.
    assert!((mid(0.0, 360.0)).abs() < 1e-9);
}

/// **A FORMA interpola pelo motor do Blend, e não por um segundo** — byte a byte contra ele.
#[test]
fn the_shape_goes_through_the_one_blend_engine() {
    let a_geom = rect(1);
    let mut b_geom = ellipse([1.0, 0.5], 1.0, 0.5);
    b_geom.id = 1;

    let mut a = UiState::new("a");
    a.objects = vec![ObjectPose {
        geometry: Some(a_geom.clone()),
        ..posed(1)
    }];
    let mut b = UiState::new("b");
    b.objects = vec![ObjectPose {
        geometry: Some(b_geom.clone()),
        ..posed(1)
    }];

    let tr = Transition::new(&a, &b);
    assert_eq!(tr.plans_built(), 1, "a forma nao foi casada pelo Blend");
    let got = tr.at(0.5)[0].geometry.clone().expect("forma");
    let want = ph2d_vec_blend::morph(&a_geom, &b_geom, 0.5).expect("morph");
    assert_eq!(
        got.verts, want.verts,
        "a forma do Smart Animate divergiu do motor que o artista usa no Morph"
    );
}

/// **Um par SÓ-DE-COR não constrói `Plan` nenhum** — o gate de custo, e o número que ele vale.
///
/// ⚠️ Ele CONTA em vez de cronometrar, que é o que o torna imune à carga da máquina. O relógio
/// está na sonda `measure_what_a_plan_costs` da `ph2d-vec-blend`: 0,64 ms por par mesmo com as
/// formas iguais, contra 0,0001 ms de um passo. Vinte objetos = **12,79 ms**, 77% de um quadro.
#[test]
fn a_colour_only_change_builds_no_plan() {
    let mut a = UiState::new("idle");
    a.objects = vec![ObjectPose {
        fill: Some(Paint::Solid(Rgba8::new(30, 90, 200, 255))),
        ..posed(1)
    }];
    let mut b = UiState::new("hover");
    b.objects = vec![ObjectPose {
        fill: Some(Paint::Solid(Rgba8::new(230, 200, 40, 255))),
        ..posed(1)
    }];

    let tr = Transition::new(&a, &b);
    assert_eq!(
        tr.plans_built(),
        0,
        "uma troca de estado so'-de-cor pagou a busca de fase do Blend"
    );

    // …e a cor ANDA mesmo assim, pela porta OKLab do Blend.
    let Some(Paint::Solid(c)) = tr.at(0.5)[0].fill.clone() else {
        panic!("a tinta do meio nao e' solida")
    };
    assert_ne!(c, Rgba8::new(30, 90, 200, 255), "a cor nao saiu da origem");
    assert_ne!(c, Rgba8::new(230, 200, 40, 255), "a cor ja' chegou ao fim");
}

/// **A COR atravessa em OKLab, não em sRGB** — o meio de dois matizes opostos NÃO é cinza.
///
/// ⚠️ Oráculo de APARÊNCIA: a distância do meio ao cinza de mesma luminosidade. Um lerp por canal
/// de azul para amarelo passa por um cinza lamacento; o caminho perceptual não.
#[test]
fn the_colour_path_is_perceptual_not_muddy() {
    let blue = Rgba8::new(0, 40, 220, 255);
    let yellow = Rgba8::new(240, 220, 0, 255);
    let mut a = UiState::new("a");
    a.objects = vec![ObjectPose {
        fill: Some(Paint::Solid(blue)),
        ..posed(1)
    }];
    let mut b = UiState::new("b");
    b.objects = vec![ObjectPose {
        fill: Some(Paint::Solid(yellow)),
        ..posed(1)
    }];

    let Some(Paint::Solid(mid)) = Transition::new(&a, &b).at(0.5)[0].fill.clone() else {
        panic!("sem tinta no meio")
    };

    let naive = [
        f32::midpoint(f32::from(blue.r), f32::from(yellow.r)),
        f32::midpoint(f32::from(blue.g), f32::from(yellow.g)),
        f32::midpoint(f32::from(blue.b), f32::from(yellow.b)),
    ];
    let d = (f32::from(mid.r) - naive[0]).powi(2)
        + (f32::from(mid.g) - naive[1]).powi(2)
        + (f32::from(mid.b) - naive[2]).powi(2);
    assert!(
        d.sqrt() > 20.0,
        "o meio caiu em cima do lerp por canal (distancia {:.1}) — a cor nao esta' a ir por OKLab",
        d.sqrt()
    );
}

/// **`t` fora de `[0, 1]` encosta na ponta e para** — um ease com overshoot não quebra a pose.
#[test]
fn overshoot_clamps_instead_of_breaking() {
    let mut a = UiState::new("a");
    a.objects = vec![posed(1)];
    let mut b = UiState::new("b");
    b.objects = vec![ObjectPose {
        translation: [10.0, 0.0],
        ..posed(1)
    }];
    let tr = Transition::new(&a, &b);
    assert_eq!(tr.at(1.4)[0].translation, [10.0, 0.0]);
    assert_eq!(tr.at(-0.3)[0].translation, [0.0, 0.0]);
}

/// **A forma que sai do `Plan` usa a tinta da POSE, não a que o `Plan` interpolou por conta.**
///
/// ⚠️ São dois números para a mesma pergunta, e quem está a jusante não pode ter de escolher. A
/// tinta é sempre a da pose (uma porta), e a geometria sai daqui **auto-consistente** com ela.
#[test]
fn the_morphed_shape_wears_the_poses_own_paint() {
    let mut a_geom = rect(1);
    // Uma tinta DELIBERADAMENTE diferente da da pose: se o `Plan` mandasse, ela apareceria.
    a_geom.fill = Some(Paint::Solid(Rgba8::new(255, 0, 0, 255)));
    let mut b_geom = ellipse([1.0, 0.5], 1.0, 0.5);
    b_geom.id = 1;
    b_geom.fill = Some(Paint::Solid(Rgba8::new(0, 255, 0, 255)));

    let mut a = UiState::new("a");
    a.objects = vec![ObjectPose {
        geometry: Some(a_geom),
        fill: Some(Paint::Solid(Rgba8::new(10, 10, 10, 255))),
        ..posed(1)
    }];
    let mut b = UiState::new("b");
    b.objects = vec![ObjectPose {
        geometry: Some(b_geom),
        fill: Some(Paint::Solid(Rgba8::new(240, 240, 240, 255))),
        ..posed(1)
    }];

    let mid = Transition::new(&a, &b).at(0.5).remove(0);
    assert_eq!(
        mid.geometry.as_ref().and_then(|g| g.fill.clone()),
        mid.fill,
        "a forma morfada saiu com uma tinta diferente da da pose — duas respostas para a mesma \
         pergunta"
    );
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// A TABELA que viaja no documento
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// **Update State preserva o NOME.** O artista está a corrigir a pose, não a re-baptizar.
#[test]
fn updating_a_state_keeps_the_name_the_artist_gave_it() {
    let mut sets = StateSets::default();
    let i = sets.push(7, UiState::new("hover"));

    let mut fresh = UiState::new("sem nome");
    fresh.objects = vec![posed(1)];
    assert!(sets.replace_pose(7, i, fresh));

    assert_eq!(sets.get(7)[i].name, "hover", "o Update State roubou o nome");
    assert_eq!(sets.get(7)[i].objects.len(), 1, "a pose nao foi gravada");
}

/// **Um hospedeiro sem estado nenhum SAI da tabela.**
///
/// ⚠️ Sem isto o documento carregaria entradas vazias para sempre, e o `is_empty` mentiria — o
/// save levaria uma tabela que descreve nada.
#[test]
fn a_host_with_no_states_left_leaves_the_table() {
    let mut sets = StateSets::default();
    sets.push(7, UiState::new("idle"));
    sets.push(7, UiState::new("hover"));
    assert!(sets.remove(7, 1));
    assert!(!sets.is_empty(), "a tabela esvaziou cedo demais");
    assert!(sets.remove(7, 0));
    assert!(sets.is_empty(), "um hospedeiro vazio ficou na tabela");
    assert_eq!(sets.hosts().count(), 0);
}

/// **Apagar a forma esquece os estados dela.**
#[test]
fn deleting_the_shape_forgets_its_states() {
    let mut sets = StateSets::default();
    sets.push(7, UiState::new("idle"));
    sets.push(9, UiState::new("idle"));
    sets.retain_hosts(|id| id == 9);
    assert_eq!(
        sets.get(7).len(),
        0,
        "os estados de uma forma apagada sobreviveram"
    );
    assert_eq!(sets.get(9).len(), 1);
}

/// **A tabela atravessa o postcard, e a ordem é DETERMINISTA.**
///
/// ⚠️ A segunda metade é a que importa e não é decorativa: um `HashMap` faria dois saves do mesmo
/// documento diferirem em bytes, e — pior — faria o **diff do undo** registrar um passo espúrio
/// sobre um estado que ninguém tocou. É o mesmo mecanismo que o `canonicalize` do `WorldSnapshot`
/// existe para matar.
#[test]
fn the_table_survives_the_wire_in_a_deterministic_order() {
    let mut a = StateSets::default();
    for host in [30_u64, 7, 19] {
        a.push(host, UiState::new(format!("s{host}")));
    }
    let mut b = StateSets::default();
    for host in [19_u64, 30, 7] {
        b.push(host, UiState::new(format!("s{host}")));
    }
    let (wa, wb) = (
        postcard::to_allocvec(&a).expect("wire a"),
        postcard::to_allocvec(&b).expect("wire b"),
    );
    assert_eq!(
        wa, wb,
        "a mesma tabela escrita noutra ordem deu bytes diferentes"
    );
    let back: StateSets = postcard::from_bytes(&wa).expect("volta");
    assert_eq!(back, a);
    assert_eq!(back.hosts().collect::<Vec<_>>(), vec![7, 19, 30]);
}
