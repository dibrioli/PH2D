//! Gates dos VERBOS, um a um.
//!
//! A **LEI** do traço (envelope, `pre` congelado, ordem, undo) mora no irmão
//! `stroke_tests.rs`; aqui está o que cada verbo FAZ. O corte é o mesmo que o
//! produto faz: uma expressão só, e o verbo escolhe de onde vem o alvo.
//!
//! Este módulo é FILHO de `tests`, então `use super::*` alcança as fixtures
//! compartilhadas (`sphere`, `snapshot`, `max_shift`, `sweep`) — duplicá-las
//! seria como os dois arquivos passam a medir malhas diferentes sem ninguém
//! notar.

use super::*;

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
fn draw_lifts_along_one_direction_and_inflate_along_each_vertex_own() {
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
