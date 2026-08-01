//! Gates da LEI DO TRAÇO e dos verbos.
//!
//! O gate que decide a wave é
//! `the_stroke_is_a_fact_of_the_path_not_of_how_finely_it_was_sampled` — o
//! irmão 3D do `the_trench_is_a_fact_of_the_path_not_of_the_dab_spacing` que a
//! `line/Painter` escreveu depois de pagar a mesma doença quatro vezes.

use super::*;
use crate::brush::Falloff;
use ph2d_mesh::{Mesh, shapes};

fn sphere() -> Mesh {
    shapes::uv_sphere(32, 48, 1.0)
}

/// Um dab visto **de fora, olhando direto para o centro dele** — o caso normal
/// de toda fixture desta suíte.
///
/// ⚠️ Ele existe para o conjunto FRONTAL não mudar o sentido de nenhum gate
/// escrito antes dele: com o olho apontando para o centro, a calota inteira sob
/// o pincel é frontal, que é a situação que todas estas fixtures descrevem. Um
/// olho fixo (`[0,0,-1]`) faria a pegada de um dab fora do eixo ficar
/// parcialmente de costas, e o culling passaria a mexer em gates que não são
/// sobre ele.
fn dab_at(center: [f32; 3], radius: f32) -> Dab {
    let l = (center[0] * center[0] + center[1] * center[1] + center[2] * center[2]).sqrt();
    let eye = if l > 1e-6 {
        [-center[0] / l, -center[1] / l, -center[2] / l]
    } else {
        [0.0, 0.0, -1.0]
    };
    Dab::at(center, radius, eye)
}

fn snapshot(mesh: &Mesh) -> Vec<[f32; 3]> {
    mesh.positions().to_vec()
}

/// O maior deslocamento em relação a `before`.
fn max_shift(before: &[[f32; 3]], mesh: &Mesh) -> f32 {
    before
        .iter()
        .zip(mesh.positions())
        .map(|(a, b)| {
            let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
        })
        .fold(0.0f32, f32::max)
}

/// Passa o pincel do ponto `a` ao ponto `b` em `samples` dabs, num traço só.
fn sweep(mesh: &mut Mesh, brush: &Brush, a: [f32; 3], b: [f32; 3], samples: usize) {
    let mut stroke = SculptStroke::default();
    stroke.begin(mesh);
    for i in 0..samples {
        let u = if samples == 1 {
            0.0
        } else {
            i as f32 / (samples - 1) as f32
        };
        let c = [
            a[0] + (b[0] - a[0]) * u,
            a[1] + (b[1] - a[1]) * u,
            a[2] + (b[2] - a[2]) * u,
        ];
        stroke.dab(mesh, brush, &dab_at(c, brush.radius), Symmetry::default());
    }
}

#[test]
fn the_stroke_is_a_fact_of_the_path_not_of_how_finely_it_was_sampled() {
    // O MESMO caminho, amostrado 8× e 64×. Sob a lei do envelope o resultado
    // converge; sob um produto por-dab ele CRESCE com a taxa de amostragem, e
    // "passar devagar deposita mais" vira uma propriedade do mouse.
    // ⚠️ **Força BAIXA de propósito, e é o que dá dentes ao gate.** Com força
    // alta um acumulador somado satura em `1` nas DUAS densidades e o gate fica
    // verde sobre a doença — foi exatamente o que uma mutação `+=` clampada
    // provou. O regime em que "somar" e "envelopar" divergem é o não-saturado.
    let brush = Brush {
        verb: Verb::Draw,
        radius: 0.30,
        strength: 0.08,
        ..Brush::default()
    };
    let (a, b) = ([0.0, -0.2, 1.0], [0.0, 0.2, 1.0]);

    let mut coarse = sphere();
    let base = snapshot(&coarse);
    sweep(&mut coarse, &brush, a, b, 8);
    let coarse_shift = max_shift(&base, &coarse);

    let mut fine = sphere();
    sweep(&mut fine, &brush, a, b, 64);
    let fine_shift = max_shift(&base, &fine);

    let ratio = fine_shift / coarse_shift;
    assert!(
        (ratio - 1.0).abs() < 0.05,
        "8 dabs deram {coarse_shift:.5} e 64 deram {fine_shift:.5} ({ratio:.3}×) — \
         o traço virou função do ESPAÇAMENTO"
    );
    // E o traço de fato aconteceu: sem isto o gate ficaria verde com um pincel
    // que não move nada (0/0 controlado, o vácuo que deixa razão sadia sobre
    // dois doentes).
    assert!(
        coarse_shift > 1e-3,
        "o pincel não moveu nada: {coarse_shift}"
    );
}

#[test]
fn smoothing_is_a_fact_of_the_path_too_because_the_ring_is_read_frozen() {
    // O irmão do gate acima para os verbos que leem a VIZINHANÇA. Se o Smooth
    // lesse as posições vivas, cada dab suavizaria sobre o que o anterior já
    // suavizou — o produto por-dab entrando pela porta dos fundos, e a
    // superfície derretendo mais quanto mais devagar a mão passa.
    let brush = Brush {
        verb: Verb::Smooth,
        radius: 0.30,
        strength: 0.15,
        ..Brush::default()
    };
    let (a, b) = ([0.0, -0.25, 0.97], [0.0, 0.25, 0.97]);

    let mut coarse = shapes::uv_sphere(24, 36, 1.0);
    let base = snapshot(&coarse);
    sweep(&mut coarse, &brush, a, b, 6);
    let coarse_shift = max_shift(&base, &coarse);

    let mut fine = shapes::uv_sphere(24, 36, 1.0);
    sweep(&mut fine, &brush, a, b, 48);
    let fine_shift = max_shift(&base, &fine);

    assert!(
        coarse_shift > 1e-4,
        "o Smooth não moveu nada: {coarse_shift}"
    );
    let ratio = fine_shift / coarse_shift;
    assert!(
        (ratio - 1.0).abs() < 0.10,
        "6 dabs deram {coarse_shift:.6} e 48 deram {fine_shift:.6} ({ratio:.3}×) — \
         o Smooth virou função do ESPAÇAMENTO"
    );
}

#[test]
fn the_envelope_is_order_free_where_the_footprint_cannot_move() {
    // O `max` é comutativo e todo alvo sai do estado congelado, então a máquina
    // do envelope **não tem histórico**: a mesma lista de dabs em qualquer ordem
    // dá o mesmo resultado, ao bit.
    //
    // ⚠️ **Medido no verbo de MÁSCARA, e a escolha é a coisa importante deste
    // gate.** Os verbos de geometria não podem prometer isto e não é defeito da
    // lei: a PEGADA é consultada nas posições VIVAS — o pincel age onde a
    // superfície está agora, que é o que o artista vê e o que Blender e SculptGL
    // fazem — então mover a superfície muda quem cai sob o dab seguinte. O
    // acoplamento entra pela CONSULTA, nunca pelo acumulador. O Mask não move
    // geometria, logo ali a pegada é fixa e a afirmação vira exata.
    let path = [
        [0.00, -0.30, 0.95],
        [0.00, -0.10, 1.00],
        [0.00, 0.10, 1.00],
        [0.00, 0.30, 0.95],
        [0.20, 0.00, 0.97],
    ];
    // Pesos deliberadamente DESIGUAIS: com todos iguais o desempate por "o
    // primeiro vence" tornaria a ordem observável mesmo na lei correta, e o gate
    // estaria afirmando algo falso.
    let radii = [0.34f32, 0.22, 0.30, 0.26, 0.38];
    let orders: [[usize; 5]; 3] = [[0, 1, 2, 3, 4], [4, 3, 2, 1, 0], [2, 0, 4, 1, 3]];

    let mut results = Vec::new();
    for order in orders {
        let mut mesh = sphere();
        let mut st = SculptStroke::default();
        st.begin(&mesh);
        for i in order {
            st.dab(
                &mut mesh,
                &Brush {
                    verb: Verb::Mask,
                    radius: radii[i],
                    strength: 0.9,
                    ..Brush::default()
                },
                &dab_at(path[i], radii[i]),
                Symmetry::default(),
            );
        }
        results.push(mesh.masks().expect("o canal foi pintado").to_vec());
    }
    for (k, r) in results.iter().enumerate().skip(1) {
        assert_eq!(r, &results[0], "a ordem {k} deu outro resultado");
    }
    // E o traço fez algo: sem isto três máscaras vazias seriam "iguais".
    assert!(results[0].iter().copied().fold(0.0f32, f32::max) > 0.5);
}

#[test]
fn the_smooth_target_is_the_frozen_neighbourhood_not_the_moved_one() {
    // Oráculo **ANALÍTICO**, e ele existe porque os gates de comportamento não
    // alcançam esta propriedade: o `continue` do envelope descarta o dab fraco,
    // então ler a vizinhança viva só diverge quando um dab FORTE chega depois de
    // os vizinhos já terem se mexido — uma coincidência que nenhuma varredura
    // garante. Aqui a resposta certa é CALCULADA e comparada.
    let r = 0.30f32;
    let (c1, c2) = ([0.0, -0.12, 0.99], [0.0, 0.05, 1.0]);
    let brush = Brush {
        verb: Verb::Smooth,
        radius: r,
        strength: 1.0,
        ..Brush::default()
    };
    let mut mesh = sphere();
    let base = snapshot(&mesh);
    let mut st = SculptStroke::default();
    st.begin(&mesh);
    st.dab(&mut mesh, &brush, &dab_at(c1, r), Symmetry::default());
    // `BTreeSet` e não `HashSet`: a lint estrutural do repo, e aqui ela também
    // torna a varredura do gate reproduzível na mesma ordem.
    let moved_by_first: std::collections::BTreeSet<u32> = st.last_moved().iter().copied().collect();
    assert!(!moved_by_first.is_empty(), "o 1º dab não moveu nada");
    st.dab(&mut mesh, &brush, &dab_at(c2, r), Symmetry::default());

    // Um vértice que o SEGUNDO dab venceu e cujo anel o PRIMEIRO já tinha
    // mexido: é o único lugar onde "congelado" e "vivo" dão respostas
    // diferentes, e a fixture tem de conter esse vértice ou o gate é vácuo.
    let mut checked = 0;
    for &v in st.last_moved() {
        let ring = mesh.adjacency().vert_verts.neighbours(v as usize);
        if !ring.iter().any(|n| moved_by_first.contains(n)) {
            continue;
        }
        let bv = base[v as usize];
        let mut avg = [0.0f32; 3];
        for &n in ring {
            for k in 0..3 {
                avg[k] += base[n as usize][k];
            }
        }
        let inv = 1.0 / ring.len() as f32;
        let d = [bv[0] - c2[0], bv[1] - c2[1], bv[2] - c2[2]];
        let dist = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        let w = Falloff::Smooth.weight(dist / r) * brush.strength;
        for k in 0..3 {
            let want = bv[k] + (avg[k] * inv - bv[k]) * w;
            let got = mesh.positions()[v as usize][k];
            assert!(
                (want - got).abs() < 1e-5,
                "vértice {v} eixo {k}: previsto {want} pelo anel CONGELADO, \
                 obtido {got}"
            );
        }
        checked += 1;
    }
    assert!(
        checked > 10,
        "só {checked} vértices continham o fenômeno — a fixture é fraca"
    );
}

#[test]
fn re_stamping_the_same_dab_list_changes_nothing() {
    // Idempotência sob re-stamp — a propriedade que permitiria editar
    // parâmetros do traço DEPOIS dele, olhando o resultado.
    let brush = Brush {
        verb: Verb::Draw,
        radius: 0.3,
        strength: 0.7,
        ..Brush::default()
    };
    let mut mesh = sphere();
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    let dab = dab_at([0.0, 0.0, 1.0], brush.radius);
    stroke.dab(&mut mesh, &brush, &dab, Symmetry::default());
    let once = snapshot(&mesh);
    for _ in 0..12 {
        stroke.dab(&mut mesh, &brush, &dab, Symmetry::default());
    }
    assert_eq!(once, snapshot(&mesh), "o mesmo dab repetido intensificou");
    // E ele não faz TRABALHO: `last_moved` é o que alimenta o refit do octree e
    // o upload incremental, então um empate que "vencesse" mandaria a pegada
    // inteira para a GPU a cada frame sem um pixel mudar.
    assert!(
        stroke.last_moved().is_empty(),
        "o dab repetido re-escreveu {} vértices",
        stroke.last_moved().len()
    );
}

#[test]
fn a_new_stroke_forgets_the_previous_envelope_and_builds_on_top() {
    // O outro lado da idempotência: soltar e desenhar de novo TEM de somar,
    // senão a ferramenta fica presa num teto que o artista não pediu.
    let brush = Brush {
        verb: Verb::Draw,
        radius: 0.3,
        strength: 0.7,
        ..Brush::default()
    };
    let mut mesh = sphere();
    let base = snapshot(&mesh);
    let dab = dab_at([0.0, 0.0, 1.0], brush.radius);
    let mut stroke = SculptStroke::default();

    stroke.begin(&mesh);
    stroke.dab(&mut mesh, &brush, &dab, Symmetry::default());
    let after_one = max_shift(&base, &mesh);

    stroke.begin(&mesh);
    stroke.dab(&mut mesh, &brush, &dab, Symmetry::default());
    let after_two = max_shift(&base, &mesh);

    assert!(
        after_two > after_one * 1.8,
        "dois traços deram {after_two:.4} contra {after_one:.4} de um só"
    );
}

#[test]
fn a_masked_vertex_is_not_moved_by_any_verb() {
    for verb in Verb::ALL {
        if verb.paints_mask() {
            continue;
        }
        let mut mesh = sphere();
        mesh.masks_mut().fill(1.0);
        let base = snapshot(&mesh);
        let b = Brush {
            verb,
            radius: 0.5,
            strength: 1.0,
            ..Brush::default()
        };
        let mut s = SculptStroke::default();
        s.begin(&mesh);
        s.dab(
            &mut mesh,
            &b,
            &dab_at([0.0, 0.0, 1.0], b.radius),
            Symmetry::default(),
        );
        assert_eq!(
            base,
            snapshot(&mesh),
            "{} atravessou a máscara",
            verb.label()
        );
    }
}

#[test]
fn every_verb_inherits_symmetry_from_the_one_place_it_is_expanded() {
    for verb in Verb::ALL {
        let mut mesh = sphere();
        let b = Brush {
            verb,
            radius: 0.4,
            strength: 1.0,
            ..Brush::default()
        };
        let mut s = SculptStroke::default();
        s.begin(&mesh);
        // Dab fora do plano X = 0, para que o espelho caia noutro lugar.
        s.dab(
            &mut mesh,
            &b,
            &dab_at([0.6, 0.0, 0.8], b.radius),
            Symmetry::MIRROR_X,
        );
        let touched = s.touched().len();
        assert!(touched > 0, "{} não tocou nada", verb.label());
        // O conjunto tocado tem de ser simétrico em X: para cada vértice tocado
        // existe o espelho dele. Um verbo que "esquecesse" a simetria falharia
        // aqui sem precisar de um gate por verbo.
        let mut left = 0;
        let mut right = 0;
        for &v in s.touched() {
            if mesh.positions()[v as usize][0] > 0.0 {
                right += 1;
            } else {
                left += 1;
            }
        }
        assert!(
            left > 0 && right > 0,
            "{}: {left} à esquerda e {right} à direita — o espelho não saiu",
            verb.label()
        );
    }
}

#[test]
fn the_undo_window_is_the_touched_list_and_restoring_the_base_is_exact() {
    let mut mesh = sphere();
    let pristine = snapshot(&mesh);
    let b = Brush {
        verb: Verb::Draw,
        radius: 0.35,
        strength: 1.0,
        ..Brush::default()
    };
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    for k in 0..6 {
        let x = -0.3 + 0.12 * k as f32;
        s.dab(
            &mut mesh,
            &b,
            &dab_at([x, 0.0, 0.95], b.radius),
            Symmetry::default(),
        );
    }
    assert_ne!(pristine, snapshot(&mesh));

    // O undo é *exatamente* isto — não há um segundo sistema a construir.
    let (touched, base) = (s.touched().to_vec(), s.base_positions().to_vec());
    for (&v, p) in touched.iter().zip(&base) {
        mesh.positions_mut()[v as usize] = *p;
    }
    assert_eq!(pristine, snapshot(&mesh), "a janela não cobria o traço");
}

#[test]
fn a_dab_that_touches_nothing_is_a_no_op() {
    let mut mesh = sphere();
    let base = snapshot(&mesh);
    let b = Brush {
        radius: 0.2,
        ..Brush::default()
    };
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    for dab in [
        dab_at([9.0, 9.0, 9.0], 0.2),
        dab_at([0.0, 0.0, 1.0], 0.0),
        Dab {
            pressure: 0.0,
            ..dab_at([0.0, 0.0, 1.0], 0.2)
        },
    ] {
        assert_eq!(s.dab(&mut mesh, &b, &dab, Symmetry::default()), 0);
    }
    assert_eq!(base, snapshot(&mesh));
    assert!(s.touched().is_empty(), "capturou sem mover");
}

#[test]
fn the_normals_after_a_stroke_are_what_a_full_rebuild_would_give() {
    let mut mesh = sphere();
    let b = Brush {
        verb: Verb::Draw,
        radius: 0.3,
        strength: 1.0,
        falloff: Falloff::Sphere,
        ..Brush::default()
    };
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    for k in 0..8 {
        let x = -0.4 + 0.1 * k as f32;
        s.dab(
            &mut mesh,
            &b,
            &dab_at([x, 0.1, 0.9], b.radius),
            Symmetry::MIRROR_X,
        );
    }
    let incremental = mesh.normals().to_vec();
    mesh.rebuild();
    for (i, (a, c)) in incremental.iter().zip(mesh.normals()).enumerate() {
        for k in 0..3 {
            assert!(
                (a[k] - c[k]).abs() < 1e-5,
                "vértice {i}: incremental {a:?} vs rebuild {c:?}"
            );
        }
    }
}

/// Os verbos, um a um — a LEI mora no arquivo irmão.
#[path = "verb_tests.rs"]
mod verbs;

/// O Crease, que tem fixtures próprias e caras — ver o cabeçalho dele.
#[path = "verb_crease_tests.rs"]
mod verb_crease;

/// O conjunto FRONTAL, que tem fixtures de silhueta — ver o cabeçalho dele.
#[path = "verb_culling_tests.rs"]
mod verb_culling;
