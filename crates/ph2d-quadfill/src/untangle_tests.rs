//! Os gates de [`super::untangle_bowties`].

use ph2d_mesh::{Face, Mesh};

/// Uma grade de quads sobre um plano — a superfície é ela própria.
fn grade(n: usize) -> Mesh {
    let mut pos: Vec<[f32; 3]> = Vec::new();
    for j in 0..=n {
        for i in 0..=n {
            #[allow(clippy::cast_precision_loss)]
            let (x, y) = (i as f32 / n as f32, j as f32 / n as f32);
            pos.push([x - 0.5, y - 0.5, 0.0]);
        }
    }
    let mut faces: Vec<Face> = Vec::new();
    #[allow(clippy::cast_possible_truncation)]
    let w = (n + 1) as u32;
    #[allow(clippy::cast_possible_truncation)]
    let nn = n as u32;
    for j in 0..nn {
        for i in 0..nn {
            faces.push(Face::quad(
                j * w + i,
                j * w + i + 1,
                (j + 1) * w + i + 1,
                (j + 1) * w + i,
            ));
        }
    }
    Mesh::from_parts(pos, faces).expect("a grade e' valida")
}

/// ⭐ **A gravata sintética:** um vértice interior atirado para o outro lado da diagonal, que é
/// exactamente o que torna dois quads auto-cruzados sem mexer na topologia.
fn grade_com_gravata(n: usize, empurrao: f32) -> (Mesh, Mesh) {
    let plano = grade(n);
    let mut m = plano.clone();
    let alvo = {
        let w = n + 1;
        (n / 2) * w + n / 2
    };
    // ⚠️ **Passar POR CIMA do vizinho, e não pela diagonal:** um vértice atirado ao longo da
    // diagonal deixa o quad **concavo** (ou inteiramente invertido, que a lei lê como
    // `Convex` porque a normal de Newell vira com ele). O que cruza dois lados é o vértice
    // passar para lá do vizinho — medido a construir esta fixtura.
    {
        let pos = m.positions_mut();
        let p = pos[alvo];
        pos[alvo] = [p[0] + empurrao, p[1] + empurrao * 0.35, p[2]];
    }
    m.rebuild();
    (m, plano)
}

#[test]
fn sem_gravata_a_malha_fica_igual_ao_bit() {
    let plano = grade(8);
    let mut m = plano.clone();
    let n = super::untangle_bowties(&mut m, &plano, crate::EXTRACT_TRAVEL);
    assert_eq!(n, 0, "nao havia gravatas para desfazer");
    assert_eq!(m.positions(), plano.positions(), "a malha limpa foi mexida");
    assert_eq!(m.face_count(), plano.face_count());
}

#[test]
fn a_gravata_desfaz_se_e_a_topologia_nao_muda() {
    let (mut m, plano) = grade_com_gravata(8, 0.14);
    let antes = crate::local_shape(&m).0.bowties;
    assert!(antes > 0, "a fixtura tem de ter gravata: {antes}");
    let (v, f) = (m.vert_count(), m.face_count());
    let curadas = super::untangle_bowties(&mut m, &plano, crate::EXTRACT_TRAVEL);
    let depois = crate::local_shape(&m).0.bowties;
    assert_eq!(
        curadas,
        antes - depois,
        "o numero devolvido tem de ser o que desapareceu: {antes} -> {depois}"
    );
    assert_eq!(depois, 0, "sobraram gravatas: {antes} -> {depois}");
    assert_eq!(
        (m.vert_count(), m.face_count()),
        (v, f),
        "a topologia mudou"
    );
}

/// ⛔ **A cerca de viagem MANDA** — com ela a zero-e-tal o vértice não pode voltar ao sítio, e a
/// porta tem de **repor** a malha em vez de entregar um alisamento a meio.
#[test]
fn uma_cerca_apertada_repoe_a_malha_em_vez_de_entregar_meio_caminho() {
    let (mut m, plano) = grade_com_gravata(8, 0.14);
    let antes = m.positions().to_vec();
    let curadas = super::untangle_bowties(&mut m, &plano, 1.0e-4);
    assert_eq!(curadas, 0, "com a cerca fechada nada se cura");
    assert_eq!(m.positions(), &antes[..], "a malha nao voltou ao que era");
}
