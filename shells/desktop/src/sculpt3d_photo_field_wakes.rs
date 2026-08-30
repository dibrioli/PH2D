//! ⭐⭐⭐ **O CAMPO ACORDA NA PONTA?** — a pergunta que a espec do alvo nomeia como o
//! pré-requisito de qualquer trabalho sobre densidade, e que ninguém tinha medido.
//!
//! ⛔⛔ **A defesa da ponta na cadeia de referência é INDIRECTA, e não é uma verificação:**
//! um espinho geometricamente significativo cria **singularidades no campo**; o traçado
//! parte o retalho ali; e a ponta ganha **fronteira própria**, logo contagem própria de
//! quads. Se o campo **não acordar** — espinho fino demais para o passo do campo, ou campo
//! alisado demais — a ponta cai dentro de um retalho grande e a referência **degrada-se
//! como nós**. *A protecção da ponta é o campo a acordar o traçado.*
//!
//! ⇒ ⭐⭐⭐ **Se o nosso campo não acordar, ter o código deles não resolveria a foto** — e
//! esta sonda é o que responde a isso com número, sem depender de licença nenhuma.
//!
//! ```text
//! \
//!   env PH2D_PIECE=/caminho/peca.obj PH2D_DETAIL=0.85 \
//!   cargo test -p ph2d-host-desktop --release --bins \
//!   does_the_field_wake_up_at_a_thin_tip -- --ignored --nocapture
//! ```

use super::spiked_ball;

/// As cascas radiais, iguais às da régua de cobertura e às do zoom da foto.
const BANDS: [(f32, f32); 4] = [(0.0, 0.5), (0.5, 0.75), (0.75, 0.90), (0.90, 1.01)];

/// ⭐⭐⭐ **SONDA — o campo acorda, e o traçado reage?**
///
/// Três colunas por casca, e cada uma responde a uma metade da lei:
///
/// 1. **singularidades** — o campo VIU alguma coisa ali?
/// 2. **arestas de fronteira de patch** — o traçado PARTIU o retalho ali?
/// 3. **quantos patches distintos tocam a casca** — ⛔ se for `1`, o espinho inteiro vive
///    dentro de um retalho só, e é esse o diagnóstico que a espec prevê.
#[test]
#[ignore = "sonda -- o campo acorda na ponta? (PH2D_PIECE=<obj>)"]
fn does_the_field_wake_up_at_a_thin_tip() {
    let Ok(path) = std::env::var("PH2D_PIECE") else {
        eprintln!("sem PH2D_PIECE -- nada a medir");
        return;
    };
    let piece = if let Some(n) = path.strip_prefix("espinhos:") {
        spiked_ball(
            n.parse().unwrap_or(6),
            std::env::var("PH2D_SPIKE_SIGMA")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.10f32),
        )
    } else {
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
        ph2d_mesh::import_obj(&text)
            .unwrap_or_else(|e| panic!("{path} nao e' um OBJ deste leitor: {e:?}"))
            .into_iter()
            .next()
            .unwrap_or_else(|| panic!("{path} nao tem peca dentro"))
            .mesh
    };
    let detail: f32 = std::env::var("PH2D_DETAIL")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.85);

    // ── A MESMA fase zero do botão, pela mesma porta. ⚠️ Medir sobre a malha da cena em vez
    // da preparada mediria uma cadeia que ninguém corre.
    let target = ph2d_quadflow::edge_for_detail_by_count(&piece, detail);
    let work = ph2d_quadchain::phase_zero(&piece, target);
    let dual = ph2d_crossfield::Dual::build(&work);
    let (field, _) = ph2d_crossfield::solve_miq(&dual);
    let index = ph2d_crossfield::vertex_index(&work, &dual, &field);
    let layout = ph2d_trace::trace_patches(&work, &dual, &field);

    let pos = work.positions();
    #[expect(
        clippy::cast_precision_loss,
        reason = "contagem de vertices; o centroide nao precisa de mais que f32"
    )]
    let n = pos.len().max(1) as f32;
    let mut centre = [0.0f32; 3];
    for q in pos {
        for k in 0..3 {
            centre[k] += q[k] / n;
        }
    }
    let radius = |q: &[f32; 3]| -> f32 {
        let d = [q[0] - centre[0], q[1] - centre[1], q[2] - centre[2]];
        d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt()
    };
    let rmax = pos.iter().fold(0.0f32, |acc, q| acc.max(radius(q)));
    let band_of = |r: f32| BANDS.iter().position(|(lo, hi)| r >= *lo && r < *hi);

    // ── Coluna 1: singularidades por casca.
    let mut verts = [0usize; 4];
    let mut singular = [0usize; 4];
    for (v, q) in pos.iter().enumerate() {
        let Some(b) = band_of(radius(q) / rmax.max(f32::MIN_POSITIVE)) else {
            continue;
        };
        verts[b] += 1;
        if index.get(v).copied().unwrap_or(0) != 0 {
            singular[b] += 1;
        }
    }

    // ── Colunas 2 e 3: fronteiras de patch e patches distintos, por casca.
    //
    // ⚠️ Uma aresta é fronteira quando as **duas** faces que a partilham têm patches
    // diferentes — é a mesma definição que o passeio da fronteira usa.
    use std::collections::{BTreeMap, BTreeSet};
    let mut owner: BTreeMap<(u32, u32), Vec<u32>> = BTreeMap::new();
    let mut faces_in: [usize; 4] = [0; 4];
    let mut patches_in: [BTreeSet<u32>; 4] = Default::default();
    for (fi, f) in work.faces().iter().enumerate() {
        let v = f.verts();
        let p = layout.face_patch.get(fi).copied().unwrap_or(u32::MAX);
        let mut c = [0.0f32; 3];
        for &i in v {
            for k in 0..3 {
                c[k] += pos[i as usize][k] / v.len() as f32;
            }
        }
        if let Some(b) = band_of(radius(&c) / rmax.max(f32::MIN_POSITIVE)) {
            faces_in[b] += 1;
            patches_in[b].insert(p);
        }
        for k in 0..v.len() {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            owner
                .entry(if a < b { (a, b) } else { (b, a) })
                .or_default()
                .push(p);
        }
    }
    let mut walls = [0usize; 4];
    for (e, ps) in &owner {
        if ps.len() == 2 && ps[0] != ps[1] {
            let mid = [
                f32::midpoint(pos[e.0 as usize][0], pos[e.1 as usize][0]),
                f32::midpoint(pos[e.0 as usize][1], pos[e.1 as usize][1]),
                f32::midpoint(pos[e.0 as usize][2], pos[e.1 as usize][2]),
            ];
            if let Some(b) = band_of(radius(&mid) / rmax.max(f32::MIN_POSITIVE)) {
                walls[b] += 1;
            }
        }
    }

    eprintln!(
        "CAMPO em {path} (detail {detail:.2}, alvo {target:.5}) -- {} verts preparados, {} patches",
        work.vert_count(),
        patches_in.iter().flatten().collect::<BTreeSet<_>>().len(),
    );
    eprintln!(
        "  {:>16} {:>8} {:>13} {:>8} {:>14} {:>10}",
        "casca r/Rmax", "verts", "singulares", "faces", "arestas-parede", "patches"
    );
    for (b, (lo, hi)) in BANDS.iter().enumerate() {
        if verts[b] == 0 && faces_in[b] == 0 {
            continue;
        }
        eprintln!(
            "  [{lo:.2},{hi:.2}) {:15} {:13} {:8} {:14} {:10}",
            verts[b],
            singular[b],
            faces_in[b],
            walls[b],
            patches_in[b].len(),
        );
    }
    // ⭐⭐⭐ **A leitura**: `singulares == 0` **e** `patches == 1` na casca exterior é o
    // diagnóstico da espec — *a ponta caiu dentro de um retalho grande*. Nesse caso a cura é
    // do CAMPO/TRAÇADO, e ⛔ ter o código da referência não a traria.
    let ponta = BANDS.len() - 1;
    eprintln!(
        "  ⇒ na casca exterior: {} singularidade(s), {} patch(es), {} aresta(s) de parede -- {}",
        singular[ponta],
        patches_in[ponta].len(),
        walls[ponta],
        if singular[ponta] == 0 && patches_in[ponta].len() <= 1 {
            "⛔ O CAMPO NAO ACORDA: a ponta vive dentro de um retalho so'"
        } else {
            "⭐ o campo VE' a ponta -- a cura nao esta' aqui"
        }
    );
}
