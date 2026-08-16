//! **O SLIDE RELAX** — o único verbo que redistribui a malha sem mudar a forma.
//!
//! ⚠️ **O [`Verb::Smooth`] é o CONTROLE, e ele é obrigatório em quase todo gate
//! daqui:** os dois caminham para a MESMA média do anel, e a única linha que os
//! separa é a subtração da componente normal. Um gate que medisse só o relax
//! diria *"vértices andaram e o raio ficou"* — verdade também para um verbo que
//! não faz nada de útil. O que prova a ferramenta é a **razão** entre as duas
//! colunas: mesma arrumação, uma fracção do custo de forma.
//!
//! ⚠️ **E a fixture não é a esfera lisa**, pelo motivo medido no doc da
//! [`ph2d_mesh::shapes::uv_sphere_shuffled`]: ali a componente tangencial é zero
//! por construção e o relax não teria o que fazer.
//!
//! **5 mutações, 5 sangram** — e cada uma nomeia uma metade diferente da lei:
//!
//! | mutação | sangra |
//! |---|---|
//! | não remover a componente normal (o relax vira Smooth) | 3 gates |
//! | normal do vértice em vez da bissetriz, na beira | a beira |
//! | sem a guarda de valência | **só o gate de CAMADA** |
//! | `relax_normal` devolve `None` sempre (verbo no-op) | 3 gates |
//! | **a FIXTURE lisa** em vez da embaralhada | 2 gates, com o número: `0,132178 -> 0,131972` |

use super::*;
use ph2d_mesh::Mesh;

/// O polo `+z`; o olho olha para `−z`.
const TIP: [f32; 3] = [0.0, 0.0, 1.0];
const EYE: [f32; 3] = [0.0, 0.0, -1.0];
const R: f32 = 0.45;

fn shuffled() -> Mesh {
    ph2d_mesh::shapes::uv_sphere_shuffled(48, 72, 1.0)
}

/// Um traço PARADO no polo — o relax não precisa de percurso para redistribuir,
/// e um traço parado isola a lei do verbo do transporte do caminho.
fn hold(verb: Verb, dabs: usize) -> Mesh {
    let mut mesh = shuffled();
    let brush = Brush {
        verb,
        radius: R,
        strength: 1.0,
        ..Brush::default()
    };
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    for _ in 0..dabs {
        stroke.dab(
            &mut mesh,
            &brush,
            &Dab::pulling(TIP, R, EYE, [0.0; 3]),
            Symmetry::default(),
        );
    }
    mesh
}

/// Os índices dentro da pegada — quem o traço de facto tocou.
fn footprint(mesh: &Mesh) -> Vec<usize> {
    let r2 = R * R;
    mesh.positions()
        .iter()
        .enumerate()
        .filter(|(_, p)| {
            let d = [p[0] - TIP[0], p[1] - TIP[1]];
            p[2] > 0.0 && d[0] * d[0] + d[1] * d[1] <= r2
        })
        .map(|(i, _)| i)
        .collect()
}

/// **A FORMA** — o pior desvio do raio 1, sobre a pegada.
fn radius_drift(mesh: &Mesh, idx: &[usize]) -> f64 {
    idx.iter()
        .map(|&i| {
            let p = mesh.positions()[i];
            (f64::from(p[0])
                .hypot(f64::from(p[1]))
                .hypot(f64::from(p[2]))
                - 1.0)
                .abs()
        })
        .fold(0.0f64, f64::max)
}

/// **A REDISTRIBUIÇÃO** — o coeficiente de variação do comprimento das arestas
/// da pegada. Quanto menor, mais uniforme a malha.
fn edge_cv(mesh: &Mesh, idx: &[usize]) -> f64 {
    let inside: std::collections::BTreeSet<usize> = idx.iter().copied().collect();
    let adj = mesh.adjacency();
    let mut lens = Vec::new();
    for &i in idx {
        for &nb in adj.vert_verts.neighbours(i) {
            if !inside.contains(&(nb as usize)) || (nb as usize) < i {
                continue;
            }
            let (a, b) = (mesh.positions()[i], mesh.positions()[nb as usize]);
            lens.push(
                f64::from(b[0] - a[0])
                    .hypot(f64::from(b[1] - a[1]))
                    .hypot(f64::from(b[2] - a[2])),
            );
        }
    }
    let n = lens.len() as f64;
    let mean = lens.iter().sum::<f64>() / n;
    (lens.iter().map(|l| (l - mean) * (l - mean)).sum::<f64>() / n).sqrt() / mean
}

/// **A ENTREGA DA WAVE, num número:** a MESMA arrumação a **71×** menos custo de
/// forma.
///
/// Medido (32 dabs, força 1, pegada de 119 vértices, base `cv = 0,3324`):
///
/// | verbo | cv das arestas | desvio de raio |
/// |---|---|---|
/// | Slide Relax | 0,161214 | **0,000810** |
/// | Smooth | 0,159858 | **0,057752** |
///
/// ⚠️ **As duas colunas são perguntas DIFERENTES, e é isso que torna o oráculo
/// honesto:** uma razão entre duas grandezas iguais mediria dois doentes. Aqui a
/// primeira diz *arrumou?* (e os dois arrumam, ao 1%) e a segunda *a que preço?*.
#[test]
fn the_relax_tidies_the_mesh_at_a_fraction_of_the_shape_cost() {
    let base = shuffled();
    let idx = footprint(&base);
    let base_cv = edge_cv(&base, &idx);

    let relax = hold(Verb::SlideRelax, 32);
    let smooth = hold(Verb::Smooth, 32);

    let (cv_r, cv_s) = (edge_cv(&relax, &idx), edge_cv(&smooth, &idx));
    assert!(
        cv_r < base_cv * 0.6,
        "o relax tem de ARRUMAR: {base_cv:.6} -> {cv_r:.6}"
    );
    assert!(
        (cv_r / cv_s - 1.0).abs() < 0.10,
        "e tem de arrumar TANTO quanto o Smooth: {cv_r:.6} contra {cv_s:.6}"
    );

    let (dr, ds) = (radius_drift(&relax, &idx), radius_drift(&smooth, &idx));
    assert!(
        dr < ds / 10.0,
        "e por uma fracção do custo de forma: relax {dr:.6} contra smooth {ds:.6}"
    );
}

/// **O relax CONVERGE; o Smooth não.** É a diferença entre uma ferramenta que se
/// pode segurar e uma que corrói enquanto estiver premida.
///
/// Medido no desvio de raio, de 8 para 32 dabs: relax **0,000805 → 0,000810**
/// (+0,6%, um ponto fixo) contra Smooth **0,021611 → 0,057752** (+167%, e sem
/// teto — a esfera está a virar uma poça).
///
/// ⚠️ **Este gate NÃO é redundante com o de cima:** um verbo que arrumasse muito
/// no primeiro dab e depois corroesse devagar passaria naquele e cairia aqui.
#[test]
fn the_relax_saturates_where_the_smooth_keeps_eating_the_shape() {
    let base = shuffled();
    let idx = footprint(&base);

    let r8 = radius_drift(&hold(Verb::SlideRelax, 8), &idx);
    let r32 = radius_drift(&hold(Verb::SlideRelax, 32), &idx);
    let s8 = radius_drift(&hold(Verb::Smooth, 8), &idx);
    let s32 = radius_drift(&hold(Verb::Smooth, 32), &idx);

    assert!(
        r32 < r8 * 1.2,
        "o relax tem de assentar num ponto fixo: {r8:.6} -> {r32:.6}"
    );
    assert!(
        s32 > s8 * 1.5,
        "e o CONTROLE tem de mostrar que a saturação não é do harness: \
         smooth {s8:.6} -> {s32:.6}"
    );
}

/// **A BEIRA DESLIZA AO LONGO DE SI MESMA em vez de ser sugada para dentro** — a
/// segunda metade da lei, e a que a `open_tube3` não consegue testar.
///
/// ⚠️ **Por que o disco, e não o tubo:** o relax troca, numa beira, a normal do
/// vértice pela BISSETRIZ das arestas de borda, e as duas só divergem onde a
/// curva de borda não é perpendicular à superfície. Num tubo as duas são radiais
/// — medido, **0,015°** nos 12 vértices de beira —, então um gate escrito ali
/// passaria com a bissetriz trocada pela normal e não diria nada. Num disco
/// plano elas são ORTOGONAIS, e é isso que separa as duas leis.
///
/// Com a normal da superfície (`+z`), a média dos dois vizinhos de beira — o
/// ponto médio da corda, que está DENTRO do círculo — sobrevive inteira à
/// subtração e a borda encolhe dab a dab. Com a bissetriz sobra só o
/// escorregamento AO LONGO da beira, que é redistribuir sem encolher.
#[test]
fn the_border_slides_along_the_rim_instead_of_being_sucked_inward() {
    let rim_radius = |m: &Mesh| {
        let adj = m.adjacency();
        let (mut sum, mut n) = (0.0f64, 0u32);
        for v in 0..m.vert_count() {
            if !adj.is_border(v) {
                continue;
            }
            let p = m.positions()[v];
            sum += f64::from(p[0]).hypot(f64::from(p[1]));
            n += 1;
        }
        assert_eq!(n, 12, "a fixture tem doze vértices de beira");
        sum / f64::from(n)
    };

    let mut mesh = ph2d_mesh::shapes_open::open_disc();
    let before = rim_radius(&mesh);

    let brush = Brush {
        verb: Verb::SlideRelax,
        radius: 10.0,
        strength: 1.0,
        falloff: Falloff::Constant,
        ..Brush::default()
    };
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    for _ in 0..8 {
        stroke.dab(
            &mut mesh,
            &brush,
            &Dab::at([0.0, 0.0, 0.0], 10.0, [0.0, 0.0, -1.0]),
            Symmetry::default(),
        );
    }
    let after = rim_radius(&mesh);

    assert!(
        (after / before - 1.0).abs() < 0.02,
        "a beira não pode encolher: {before:.6} -> {after:.6}"
    );

    // **E a metade que impede o gate de passar por um no-op:** as pás
    // desiguais do anel de fora TÊM de se ter espalhado. Sem isto, um relax que
    // congelasse toda a borda passaria na asserção acima.
    let spread = |m: &Mesh| {
        let adj = m.adjacency();
        let mut angles: Vec<f64> = (0..m.vert_count())
            .filter(|&v| adj.is_border(v))
            .map(|v| {
                let p = m.positions()[v];
                f64::from(p[1])
                    .atan2(f64::from(p[0]))
                    .rem_euclid(std::f64::consts::TAU)
            })
            .collect();
        angles.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mut gaps: Vec<f64> = angles.windows(2).map(|w| w[1] - w[0]).collect();
        gaps.push(angles[0] + std::f64::consts::TAU - angles[angles.len() - 1]);
        let n = gaps.len() as f64;
        let mean = gaps.iter().sum::<f64>() / n;
        (gaps.iter().map(|g| (g - mean) * (g - mean)).sum::<f64>() / n).sqrt() / mean
    };
    let (s0, s1) = (spread(&ph2d_mesh::shapes_open::open_disc()), spread(&mesh));
    assert!(
        s1 < s0 * 0.9,
        "e as pás desiguais TÊM de se espalhar, senão isto é um no-op: \
         {s0:.6} -> {s1:.6}"
    );
}

/// **Quem tem dois vizinhos fica onde está.** Um vértice de valência 2 não tem
/// beira com dois lados nem anel que defina um plano; qualquer resposta ali é
/// inventada, e a inventada barata — o ponto médio dos dois vizinhos — puxaria a
/// ponta de uma tira para a corda.
///
/// ⚠️ **A [`ph2d_mesh::shapes_open::pillow`] não é de BORDA**, e é isso que a
/// torna necessária: pela regra do anel que não fecha, cada vértice dela é
/// *interior*. Uma fixture que confundisse os dois fenômenos deixaria a correção
/// de um passar pela do outro.
#[test]
fn a_two_neighbour_vertex_is_frozen() {
    let mut mesh = ph2d_mesh::shapes_open::pillow();
    let before = mesh.positions().to_vec();

    let brush = Brush {
        verb: Verb::SlideRelax,
        radius: 10.0,
        strength: 1.0,
        falloff: Falloff::Constant,
        ..Brush::default()
    };
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    stroke.dab(
        &mut mesh,
        &brush,
        &Dab::at([0.3, 0.0, 0.3], 10.0, [0.0, 1.0, 0.0]),
        Symmetry::default(),
    );

    for (i, (a, b)) in before.iter().zip(mesh.positions()).enumerate() {
        assert_eq!(a, b, "o vértice {i} tem valência 2 e não pode ter andado");
    }
}

/// **A CAMADA, exercitada directamente** — porque a de baixo a torna redundante.
///
/// ⚠️ **Medido:** apagar a guarda de valência do [`SculptStroke::relax_normal`]
/// deixa a suíte inteira da crate VERDE (249/249), incluindo o gate irmão acima:
/// o [`ph2d_mesh::ring_average`] guarda o mesmo predicado uma chamada abaixo e
/// devolve a própria base, então o delta é zero pelas duas vias. Um gate de
/// PRODUTO não consegue separá-las por construção — é preciso perguntar à
/// camada, e é isto.
#[test]
fn the_relax_refuses_to_answer_for_a_ring_that_defines_no_plane() {
    let mesh = ph2d_mesh::shapes_open::pillow();
    let stroke = SculptStroke::default();
    for v in 0..mesh.vert_count() {
        assert_eq!(
            mesh.adjacency().valence(v),
            2,
            "a fixture inteira tem de ser de valência 2, senão o gate mede outra coisa"
        );
        assert!(
            stroke
                .relax_normal(&mesh, v as u32, mesh.positions()[v])
                .is_none(),
            "um anel de dois não define plano: o vértice {v} não tem resposta"
        );
    }

    // **O CONTROLE**, sem o qual isto passaria com um `relax_normal` que
    // devolvesse `None` sempre — e aí o verbo inteiro seria um no-op.
    let sphere = ph2d_mesh::shapes::uv_sphere(12, 16, 1.0);
    assert!(
        stroke
            .relax_normal(&sphere, 20, sphere.positions()[20])
            .is_some(),
        "num anel cheio a resposta EXISTE"
    );
}
