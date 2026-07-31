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
        stroke.dab(mesh, brush, &Dab::at(c, brush.radius), Symmetry::default());
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
                &Dab::at(path[i], radii[i]),
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
    st.dab(&mut mesh, &brush, &Dab::at(c1, r), Symmetry::default());
    // `BTreeSet` e não `HashSet`: a lint estrutural do repo, e aqui ela também
    // torna a varredura do gate reproduzível na mesma ordem.
    let moved_by_first: std::collections::BTreeSet<u32> = st.last_moved().iter().copied().collect();
    assert!(!moved_by_first.is_empty(), "o 1º dab não moveu nada");
    st.dab(&mut mesh, &brush, &Dab::at(c2, r), Symmetry::default());

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
    let dab = Dab::at([0.0, 0.0, 1.0], brush.radius);
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
fn the_plane_offset_lifts_the_plane_the_verbs_project_onto() {
    // O knob que faz de um Flatten um "Clay do Blender" sem um segundo verbo.
    // Sem este gate ele é um número que ninguém confere — e o produto tem quatro
    // verbos que o leem.
    let c = [0.0, 0.0, 1.0];
    let b = Brush {
        verb: Verb::Flatten,
        radius: 0.5,
        strength: 1.0,
        falloff: Falloff::Constant,
        ..Brush::default()
    };
    let height = |offset: f32| {
        let mut mesh = sphere();
        let mut s = SculptStroke::default();
        s.begin(&mesh);
        s.dab(
            &mut mesh,
            &Brush {
                plane_offset: offset,
                ..b
            },
            &Dab::at(c, b.radius),
            Symmetry::default(),
        );
        // A altura do platô: o vértice mais alto da calota achatada.
        mesh.positions()
            .iter()
            .map(|p| p[2])
            .fold(f32::MIN, f32::max)
    };
    let flat = height(0.0);
    let raised = height(0.2);
    let sunk = height(-0.2);
    assert!(
        raised > flat + 0.05,
        "offset positivo devia levantar o plano: {raised} vs {flat}"
    );
    assert!(
        sunk < flat - 0.05,
        "offset negativo devia baixá-lo: {sunk} vs {flat}"
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
    let dab = Dab::at([0.0, 0.0, 1.0], brush.radius);
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
fn draw_lifts_along_one_direction_and_inflate_along_each_vertexs_own() {
    // A distinção Draw×Inflate, medida onde ela EXISTE: numa superfície curva as
    // normais divergem, então o Inflate espalha as direções de deslocamento e o
    // Draw as mantém paralelas. Num plano os dois coincidem — e um fixture plano
    // deixaria este gate verde sobre um Inflate que é Draw.
    let make = |verb| Brush {
        verb,
        radius: 0.6,
        strength: 1.0,
        ..Brush::default()
    };
    let spread = |verb| {
        let mut mesh = sphere();
        let base = snapshot(&mesh);
        let mut stroke = SculptStroke::default();
        stroke.begin(&mesh);
        let b = make(verb);
        stroke.dab(
            &mut mesh,
            &b,
            &Dab::at([0.0, 0.0, 1.0], b.radius),
            Symmetry::default(),
        );
        // Cosseno entre o deslocamento de cada vértice e o do polo: 1 = paralelo.
        let mut worst = 1.0f32;
        let dir = |i: usize| {
            let d = [
                mesh.positions()[i][0] - base[i][0],
                mesh.positions()[i][1] - base[i][1],
                mesh.positions()[i][2] - base[i][2],
            ];
            let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            (l > 1e-5).then(|| [d[0] / l, d[1] / l, d[2] / l])
        };
        let mut reference = None;
        for i in 0..base.len() {
            let Some(u) = dir(i) else { continue };
            let r = *reference.get_or_insert(u);
            worst = worst.min(u[0] * r[0] + u[1] * r[1] + u[2] * r[2]);
        }
        worst
    };
    let draw = spread(Verb::Draw);
    let inflate = spread(Verb::Inflate);
    assert!(
        draw > 0.999,
        "o Draw devia empurrar paralelo: cos mínimo {draw}"
    );
    assert!(
        inflate < 0.9,
        "o Inflate devia divergir com a curvatura: cos mínimo {inflate}"
    );
}

#[test]
fn invert_digs_where_the_verb_lifted() {
    let mut up = sphere();
    let mut down = sphere();
    let base = snapshot(&up);
    let b = Brush {
        verb: Verb::Draw,
        radius: 0.4,
        strength: 1.0,
        ..Brush::default()
    };
    let dab = Dab::at([0.0, 0.0, 1.0], b.radius);
    let mut s = SculptStroke::default();
    s.begin(&up);
    s.dab(&mut up, &b, &dab, Symmetry::default());
    let mut s2 = SculptStroke::default();
    s2.begin(&down);
    s2.dab(
        &mut down,
        &Brush { invert: true, ..b },
        &dab,
        Symmetry::default(),
    );
    let (i, _) = base
        .iter()
        .enumerate()
        .max_by(|a, c| a.1[2].total_cmp(&c.1[2]))
        .unwrap();
    assert!(up.positions()[i][2] > base[i][2], "o normal devia subir");
    assert!(
        down.positions()[i][2] < base[i][2],
        "o invertido devia cavar"
    );
}

#[test]
fn smooth_flattens_a_spike_and_sharpen_deepens_it() {
    let spike_height = |mesh: &Mesh, i: usize| mesh.positions()[i][2];
    let mut mesh = sphere();
    // Faz um pico com um Draw estreito e forte.
    let poke = Brush {
        verb: Verb::Draw,
        radius: 0.12,
        strength: 1.0,
        ..Brush::default()
    };
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    s.dab(
        &mut mesh,
        &poke,
        &Dab::at([0.0, 0.0, 1.0], poke.radius),
        Symmetry::default(),
    );
    let (top, _) = mesh
        .positions()
        .iter()
        .enumerate()
        .max_by(|a, b| a.1[2].total_cmp(&b.1[2]))
        .unwrap();
    let peak = spike_height(&mesh, top);

    let mut smoothed = mesh.clone();
    let mut sharpened = mesh.clone();
    let b = Brush {
        radius: 0.3,
        strength: 1.0,
        ..Brush::default()
    };
    for (m, verb) in [
        (&mut smoothed, Verb::Smooth),
        (&mut sharpened, Verb::Sharpen),
    ] {
        let mut st = SculptStroke::default();
        st.begin(m);
        st.dab(
            m,
            &Brush { verb, ..b },
            &Dab::at([0.0, 0.0, 1.0], b.radius),
            Symmetry::default(),
        );
    }
    assert!(
        spike_height(&smoothed, top) < peak,
        "o Smooth devia baixar o pico ({} vs {peak})",
        spike_height(&smoothed, top)
    );
    assert!(
        spike_height(&sharpened, top) > peak,
        "o Sharpen devia levantá-lo ({} vs {peak})",
        spike_height(&sharpened, top)
    );
}

/// O desvio-padrão da distância ao plano ajustado à calota — quão "plana" ela é.
fn flatness(mesh: &Mesh, center: [f32; 3], radius: f32) -> f32 {
    let pts: Vec<_> = mesh
        .positions()
        .iter()
        .filter(|p| {
            let d = [p[0] - center[0], p[1] - center[1], p[2] - center[2]];
            d[0] * d[0] + d[1] * d[1] + d[2] * d[2] <= radius * radius
        })
        .copied()
        .collect();
    let n = pts.len() as f32;
    let mean = pts.iter().fold(0.0, |a, p| a + p[2]) / n;
    (pts.iter().fold(0.0, |a, p| a + (p[2] - mean).powi(2)) / n).sqrt()
}

#[test]
fn flatten_brings_the_footprint_onto_one_plane() {
    // ⚠️ **Falloff `Constant` de propósito.** O que este gate afirma é a
    // PROJEÇÃO; com um falloff macio só o centro alcança `accum = 1` e o resto
    // fica no meio do caminho — comportamento certo (é o do Blender) que
    // mediria a curva, não o plano. Um dab de peso cheio é o único fixture em
    // que "ficou plano?" tem resposta binária.
    let mut mesh = sphere();
    let c = [0.0, 0.0, 1.0];
    let before = flatness(&mesh, c, 0.4);
    let b = Brush {
        verb: Verb::Flatten,
        radius: 0.5,
        strength: 1.0,
        falloff: Falloff::Constant,
        ..Brush::default()
    };
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    s.dab(&mut mesh, &b, &Dab::at(c, b.radius), Symmetry::default());
    let after = flatness(&mesh, c, 0.4);
    assert!(
        after < before * 0.05,
        "achatou de {before:.4} para {after:.4} — não é um plano"
    );

    // E o contraste que impede o gate de virar "qualquer verbo achata": com o
    // falloff macio o mesmo dab melhora, sem chegar perto de um plano.
    let mut soft = sphere();
    let mut s2 = SculptStroke::default();
    s2.begin(&soft);
    s2.dab(
        &mut soft,
        &Brush {
            falloff: Falloff::Smooth,
            ..b
        },
        &Dab::at(c, b.radius),
        Symmetry::default(),
    );
    let soft_after = flatness(&soft, c, 0.4);
    assert!(
        soft_after < before && soft_after > after * 3.0,
        "macio {soft_after:.4} devia ficar entre {after:.4} e {before:.4}"
    );
}

#[test]
fn fill_only_raises_and_scrape_only_lowers() {
    // O fixture TEM de conter os dois lados: uma calota lisa só tem material
    // acima do plano ajustado, e ali o Fill é legitimamente inerte — um gate
    // sobre ela mediria o vácuo. Então o pico entra de propósito.
    let mut bumpy = sphere();
    let poke = Brush {
        verb: Verb::Draw,
        radius: 0.15,
        strength: 1.0,
        ..Brush::default()
    };
    let mut s = SculptStroke::default();
    s.begin(&bumpy);
    s.dab(
        &mut bumpy,
        &poke,
        &Dab::at([0.0, 0.0, 1.0], poke.radius),
        Symmetry::default(),
    );
    let dip = Brush {
        invert: true,
        ..poke
    };
    let mut s2 = SculptStroke::default();
    s2.begin(&bumpy);
    s2.dab(
        &mut bumpy,
        &dip,
        &Dab::at([0.25, 0.0, 0.97], dip.radius),
        Symmetry::default(),
    );

    let c = [0.1, 0.0, 1.0];
    let b = Brush {
        radius: 0.5,
        strength: 1.0,
        falloff: Falloff::Constant,
        ..Brush::default()
    };
    let mut counts = [0usize; 3];
    for (slot, (verb, sign)) in [
        (Verb::Fill, 1.0f32),
        (Verb::Scrape, -1.0f32),
        (Verb::Flatten, 0.0f32),
    ]
    .into_iter()
    .enumerate()
    {
        let mut mesh = bumpy.clone();
        let base = snapshot(&mesh);
        let mut st = SculptStroke::default();
        st.begin(&mesh);
        st.dab(
            &mut mesh,
            &Brush { verb, ..b },
            &Dab::at(c, b.radius),
            Symmetry::default(),
        );
        for (p, q) in base.iter().zip(mesh.positions()) {
            let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
            let along = d[0] * p[0] + d[1] * p[1] + d[2] * p[2]; // radial ≈ normal
            assert!(
                along * sign >= -1e-5,
                "{} moveu para o lado errado: {along}",
                verb.label()
            );
            if along.abs() > 1e-5 {
                counts[slot] += 1;
            }
        }
    }
    let (fill, scrape, flatten) = (counts[0], counts[1], counts[2]);
    // O oráculo ESTRUTURAL, que é mais forte que um piso escolhido a dedo: todo
    // vértice está de um lado do plano ou do outro, então o Flatten move
    // exatamente a união dos dois — e cada metade tem de ser não-vazia, senão o
    // fixture não contém o fenômeno que o gate afirma.
    assert!(fill > 0 && scrape > 0, "fill {fill}, scrape {scrape}");
    assert_eq!(
        fill + scrape,
        flatten,
        "fill {fill} + scrape {scrape} != flatten {flatten}"
    );
}

#[test]
fn clay_adds_material_where_flatten_conserves_it() {
    let volume_proxy = |mesh: &Mesh, base: &[[f32; 3]]| {
        base.iter()
            .zip(mesh.positions())
            .map(|(a, b)| {
                let l = |p: &[f32; 3]| (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
                f64::from(l(b) - l(a))
            })
            .sum::<f64>()
    };
    let c = [0.0, 0.0, 1.0];
    let b = Brush {
        radius: 0.5,
        strength: 1.0,
        ..Brush::default()
    };
    let mut out = Vec::new();
    for verb in [Verb::Flatten, Verb::Clay] {
        let mut mesh = sphere();
        let base = snapshot(&mesh);
        let mut s = SculptStroke::default();
        s.begin(&mesh);
        s.dab(
            &mut mesh,
            &Brush { verb, ..b },
            &Dab::at(c, b.radius),
            Symmetry::default(),
        );
        out.push(volume_proxy(&mesh, &base));
    }
    assert!(out[0] < 0.0, "o Flatten numa calota REMOVE ({:.4})", out[0]);
    assert!(
        out[1] > out[0],
        "o Clay devia acrescentar sobre o Flatten: {:.4} vs {:.4}",
        out[1],
        out[0]
    );
}

#[test]
fn pinch_pulls_along_the_surface_and_does_not_secretly_flatten() {
    // A divergência deliberada do Blender (ver `tangential`): o deslocamento do
    // Pinch é perpendicular à normal da área, então apertar é apertar.
    let mut mesh = sphere();
    let base = snapshot(&mesh);
    let c = [0.0, 0.0, 1.0];
    let b = Brush {
        verb: Verb::Pinch,
        radius: 0.5,
        strength: 1.0,
        ..Brush::default()
    };
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    s.dab(&mut mesh, &b, &Dab::at(c, b.radius), Symmetry::default());

    // ⚠️ A razão é POR VÉRTICE. Comparar o maior desvio normal com o maior
    // deslocamento lateral compara dois vértices DIFERENTES, e uma mutação que
    // devolve o Pinch do Blender (com componente normal) passa raspando — foi o
    // que aconteceu na primeira rodada.
    let mut worst_ratio = 0.0f32;
    let mut lateral = 0.0f32;
    for (p, q) in base.iter().zip(mesh.positions()) {
        let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
        let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        if len < 1e-5 {
            continue;
        }
        // A normal da área no polo é ~+Z.
        worst_ratio = worst_ratio.max(d[2].abs() / len);
        lateral = lateral.max((d[0] * d[0] + d[1] * d[1]).sqrt());
    }
    assert!(lateral > 0.02, "o Pinch não apertou nada ({lateral})");
    assert!(
        worst_ratio < 0.05,
        "o deslocamento do Pinch tem {:.1}% ao longo da normal — ele está \
         achatando de lambuja",
        worst_ratio * 100.0
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
            &Dab::at([0.0, 0.0, 1.0], b.radius),
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
fn the_mask_verb_writes_its_channel_and_moves_no_geometry() {
    let mut mesh = sphere();
    let base = snapshot(&mesh);
    let b = Brush {
        verb: Verb::Mask,
        radius: 0.5,
        strength: 1.0,
        ..Brush::default()
    };
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    s.dab(
        &mut mesh,
        &b,
        &Dab::at([0.0, 0.0, 1.0], b.radius),
        Symmetry::default(),
    );
    assert_eq!(base, snapshot(&mesh), "o Mask moveu geometria");
    let masks = mesh.masks().expect("o canal foi materializado");
    let peak = masks.iter().copied().fold(0.0f32, f32::max);
    assert!(peak > 0.9, "a máscara mal pintou: pico {peak}");

    // Limpar desfaz pela MESMA aritmética (`lerp(base, alvo, accum)`), e é o
    // `invert` que troca o alvo de 1 para 0.
    let clear = |m: &mut Mesh, falloff| {
        let mut s2 = SculptStroke::default();
        s2.begin(m);
        s2.dab(
            m,
            &Brush {
                invert: true,
                falloff,
                ..b
            },
            &Dab::at([0.0, 0.0, 1.0], b.radius),
            Symmetry::default(),
        );
        m.masks().unwrap().iter().copied().fold(0.0f32, f32::max)
    };

    // ⚠️ Uma limpeza MACIA sobre uma pintura macia deixa resto, e o número não é
    // aproximado: um vértice de peso `w` foi pintado em `w` e limpo para
    // `w(1 − w)`, cujo máximo em `[0,1]` é **exatamente 0,25**. Isto não é
    // defeito — é a aritmética do lerp, e é também o que o Blender faz (por isso
    // ele tem um "Clear Mask" global além do pincel). Pinar o valor é o que
    // impede a próxima pessoa de "consertar" a lei achando que 0,25 é sujeira.
    let mut soft = mesh.clone();
    let residue = clear(&mut soft, Falloff::Smooth);
    assert!(
        (residue - 0.25).abs() < 0.01,
        "o resto de uma limpeza macia é w(1−w) ≤ 0,25; deu {residue}"
    );

    // E com peso cheio na pegada inteira ela limpa ao valor exato.
    let hard = clear(&mut mesh, Falloff::Constant);
    assert_eq!(hard, 0.0, "limpar com peso cheio deixou {hard}");
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
            &Dab::at([0.6, 0.0, 0.8], b.radius),
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
            &Dab::at([x, 0.0, 0.95], b.radius),
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
        Dab::at([9.0, 9.0, 9.0], 0.2),
        Dab::at([0.0, 0.0, 1.0], 0.0),
        Dab {
            center: [0.0, 0.0, 1.0],
            radius: 0.2,
            pressure: 0.0,
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
            &Dab::at([x, 0.1, 0.9], b.radius),
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
