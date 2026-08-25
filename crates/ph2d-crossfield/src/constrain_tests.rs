//! Os gates da restrição de orientação — o **gate nº6** da
//! `SPEC_restricoes_por_eliminacao.md` §5, e a cerca do conflito.

use super::CONSTRAINT_AGREEMENT;
use crate::{Dual, QUARTER, Rounding, solve_miq_aligned};
use ph2d_mesh::{FeatureEdge, Mesh, shapes};

/// O ângulo entre `d` e a cruz da face `f`, **no círculo de `π/2`** — `0` quando a
/// cruz é paralela a `d`, `π/4` quando está o mais longe possível.
fn gap_to_cross(dual: &Dual, field: &crate::CrossField, f: usize, d: [f32; 3]) -> f32 {
    let c = field.direction(dual, f);
    let fr = dual.frames()[f];
    let ang = |v: [f32; 3]| {
        let k = crate::dot(v, fr.n);
        let t = [
            k.mul_add(-fr.n[0], v[0]),
            k.mul_add(-fr.n[1], v[1]),
            k.mul_add(-fr.n[2], v[2]),
        ];
        let b = crate::cross(fr.n, fr.e);
        crate::dot(t, b)
            .atan2(crate::dot(t, fr.e))
            .rem_euclid(QUARTER)
    };
    let g = (ang(c) - ang(d)).rem_euclid(QUARTER);
    g.min(QUARTER - g)
}

/// As arestas do **rebordo** de um cilindro de eixo `Y`: as duas circunferências onde
/// a parede encontra a tampa. ⚠️ São elas o vinco — as arestas verticais da parede e os
/// raios da tampa não são.
fn rim_edges(mesh: &Mesh, half_height: f32, radius: f32) -> Vec<FeatureEdge> {
    let p = mesh.positions();
    let on_rim = |v: u32| {
        let q = p[v as usize];
        (q[1].abs() - half_height).abs() < 1.0e-3 && (q[0].hypot(q[2]) - radius).abs() < 1.0e-3
    };
    let mut out = std::collections::BTreeSet::new();
    for f in mesh.faces() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            if on_rim(a) && on_rim(b) {
                out.insert([a.min(b), a.max(b)]);
            }
        }
    }
    // ⚠️ **A direcção é a TANGENTE ANALÍTICA da circunferência no meio da aresta**, não a
    // corda: é ela que a detecção estima na peça real, e uma fixtura que use a corda
    // mediria a discretização em vez da lei.
    out.into_iter()
        .map(|e| {
            let (a, b) = (p[e[0] as usize], p[e[1] as usize]);
            let m = [0.5 * (a[0] + b[0]), 0.0, 0.5 * (a[2] + b[2])];
            let l = m[0].hypot(m[2]).max(1.0e-9);
            FeatureEdge {
                verts: e,
                dir: [-m[2] / l, 0.0, m[0] / l],
            }
        })
        .collect()
}

/// ⭐⭐⭐ **GATE Nº6 — O CAMPO OBEDECE À FEIÇÃO**, e o controlo é o que o torna uma
/// medição.
///
/// ⛔⛔ **A armadilha que este gate tem de evitar é a da própria espec (§5.1, armadilha
/// 2): medir o `θ` que acabámos de fixar seria TAUTOLÓGICO.** O que se mede aqui é o
/// **vector em mundo** que a [`crate::CrossField::direction`] reconstrói — depois do
/// CG, do arredondamento guloso e da continuação —, contra a direcção da aresta de
/// feição. Ele atravessa a moldura, a projecção, a eliminação e a reconstrução: uma
/// eliminação que escrevesse o `x` da face em vez do valor fixo reprova aqui.
///
/// ⭐ **E o controlo é o mesmo campo SEM restrição**: sem ele, um número pequeno não
/// diz se a restrição funcionou ou se a cruz já estava paralela por sorte.
#[test]
fn the_field_obeys_the_feature_and_the_unconstrained_field_does_not() {
    let (r, hh) = (0.5f32, 0.75f32);
    let mut mesh = shapes::cylinder(96, r, 2.0 * hh);
    mesh.triangulate();
    let edges = rim_edges(&mesh, hh, r);
    assert!(
        edges.len() > 50,
        "a fixtura tem de ter rebordo: {}",
        edges.len()
    );

    let plain = Dual::build(&mesh);
    let mut held = Dual::build(&mesh);
    let rep = held.constrain(&mesh, &edges);
    assert!(rep.faces > 0, "nenhuma face foi restringida: {rep:?}");

    let (free_field, _) = solve_miq_aligned(&plain, Rounding::default(), crate::ALIGN_WEIGHT);
    let (held_field, _) = solve_miq_aligned(&held, Rounding::default(), crate::ALIGN_WEIGHT);

    // Só as faces que a restrição tocou, e a direcção de cada uma é a da aresta que a
    // tocou — reconstruída aqui, não lida do `Dual`.
    let mut owner: std::collections::BTreeMap<usize, [f32; 3]> = std::collections::BTreeMap::new();
    for (fi, f) in mesh.faces().iter().enumerate() {
        let v = f.verts();
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            let key = [a.min(b), a.max(b)];
            if let Ok(i) = edges.binary_search_by(|e| e.verts.cmp(&key)) {
                owner.insert(fi, edges[i].dir);
            }
        }
    }

    let mean = |field: &crate::CrossField| {
        let mut s = 0.0f64;
        for (&fi, &d) in &owner {
            s += f64::from(gap_to_cross(&held, field, fi, d));
        }
        s / owner.len().max(1) as f64
    };
    let (a, b) = (
        mean(&free_field).to_degrees(),
        mean(&held_field).to_degrees(),
    );
    eprintln!(
        "{} faces de feicao · desvio medio da cruz ao vinco: SEM restricao {a:.2} graus, \
         COM restricao {b:.2} graus ({} faces fixas, {} conflitos)",
        owner.len(),
        rep.faces,
        rep.conflicts
    );
    assert!(
        b < 1.0,
        "⛔ o campo NAO obedece a' feicao: {b:.2} graus de desvio medio nas faces fixas"
    );
    assert!(
        b < a,
        "⛔ a restricao nao melhorou nada: sem ela {a:.2} graus, com ela {b:.2} graus"
    );
}

/// ⭐⭐ **A CERCA DO CONFLITO** — duas arestas do MESMO triângulo pedem `60°` uma da
/// outra, e nenhuma cruz é paralela às duas.
///
/// ⚠️ **Ela é a metade JUSTA da cerca:** sem ela, uma implementação que aceitasse
/// sempre a primeira leitura passaria todos os outros gates e poria a cruz a apontar
/// para um dos dois vincos, escolhido pela ordem do `BTreeMap`.
#[test]
fn two_edges_of_the_same_triangle_are_a_conflict_and_the_face_is_dropped() {
    let mut mesh = shapes::uv_sphere(24, 32, 1.0);
    mesh.triangulate();
    let v = mesh.faces()[0].verts().to_vec();
    assert_eq!(v.len(), 3, "a fixtura tem de estar triangulada");
    let p = mesh.positions();
    let along = |a: u32, b: u32| {
        let d = crate::sub(p[b as usize], p[a as usize]);
        let l = crate::dot(d, d).sqrt().max(1.0e-9);
        FeatureEdge {
            verts: [a.min(b), a.max(b)],
            dir: [d[0] / l, d[1] / l, d[2] / l],
        }
    };
    let one = [along(v[0], v[1])];
    let two = [along(v[0], v[1]), along(v[1], v[2])];

    let mut a = Dual::build(&mesh);
    let ra = a.constrain(&mesh, &one);
    assert!(
        a.constrained(0).is_some(),
        "uma aresta so' NAO e' conflito: {ra:?}"
    );

    let mut b = Dual::build(&mesh);
    let rb = b.constrain(&mesh, &two);
    assert!(
        b.constrained(0).is_none(),
        "⛔ a face 0 aceitou DUAS leituras a 60 graus uma da outra: {rb:?}"
    );
    assert!(rb.conflicts >= 1, "o conflito tem de ser CONTADO: {rb:?}");
}

/// ⚠️ **A cerca tem de deixar passar o que é a MESMA cruz:** `90°` é a identidade de um
/// campo de quatro braços, e duas feições perpendiculares num canto **concordam**.
#[test]
fn perpendicular_features_agree_because_a_cross_has_four_arms() {
    assert!(
        super::quarter_gap(0.0, QUARTER) <= CONSTRAINT_AGREEMENT,
        "⛔ um quarto de volta tem de ser distancia ZERO no circulo da cruz"
    );
    assert!(
        super::quarter_gap(0.0, QUARTER * 0.5) > CONSTRAINT_AGREEMENT,
        "⛔ 45 graus e' o mais longe possivel, e tem de ser conflito"
    );
}

/// ⭐⭐ **O GAUGE DO `θ` É ESCRITO UMA VEZ POR COMPONENTE — e uma face restringida já o é.**
///
/// ⚠️ **Este gate é ESTRUTURAL de propósito, e a razão está medida:** a cura que ele defende
/// move o resultado em `111` ⇒ `109` singularidades sobre uma resposta certa de `25`, ou
/// seja, **nada**. ⛔ *Uma correcção que nenhum número apanha não pode ser defendida por um
/// número* — o que se pode afirmar é a forma: com restrição, ninguém mais é âncora.
#[test]
fn the_gauge_is_written_once_per_component() {
    let mut mesh = shapes::uv_sphere(24, 32, 1.0);
    mesh.triangulate();

    let free = Dual::build(&mesh);
    let seeds = crate::solve::gauge_seeds(&free);
    assert_eq!(
        seeds.iter().filter(|s| **s).count(),
        1,
        "⛔ sem restricao o calibre e' UM, e e' a face 0 — mudar isto mexe em toda malha"
    );
    assert!(seeds[0], "⛔ e a face ancorada tem de continuar a ser a 0");

    let v = mesh.faces()[0].verts().to_vec();
    let p = mesh.positions();
    let d = crate::sub(p[v[1] as usize], p[v[0] as usize]);
    let l = crate::dot(d, d).sqrt().max(1.0e-9);
    let mut held = Dual::build(&mesh);
    held.constrain(
        &mesh,
        &[FeatureEdge {
            verts: [v[0].min(v[1]), v[0].max(v[1])],
            dir: [d[0] / l, d[1] / l, d[2] / l],
        }],
    );
    let seeds = crate::solve::gauge_seeds(&held);
    assert_eq!(
        seeds.iter().filter(|s| **s).count(),
        0,
        "⛔ a esfera e' UMA componente e ela ja' tem uma face restringida — prender a face 0 \
         alem dela e' escrever uma segunda referencia, que e' uma equacao que ninguem pediu"
    );
}
