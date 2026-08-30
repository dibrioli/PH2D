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

    // ⭐⭐⭐ **E ONDE ESTÃO AS DOBRAS DO MAPA?** — o suspeito seguinte, depois de o campo e o
    // traçado estarem ilibados. O relatório do botão conta as dobras da peça inteira e
    // **nunca disse onde**; uma dobra é um triângulo cuja imagem no domínio se vira do
    // avesso, e é o mecanismo que produz uma face de `177°` de torção.
    //
    // ⚠️ **Passo UNIFORME de propósito** — é o caminho de fábrica (`Follow Curvature = 0`), e
    // o campo por-vértice vive num módulo privado do botão. *Medir o caminho que o artista
    // tem por omissão vale mais do que medir o que ele tem de ligar.*
    let (cut, _) = ph2d_gridmap::cut_along_patches(&work, &layout);
    let (combed, _) = ph2d_gridmap::comb_patches(&work, &layout, &cut);
    let cones: Vec<u32> = index
        .iter()
        .enumerate()
        .filter(|(_, k)| **k != 0)
        .filter_map(|(v, _)| u32::try_from(v).ok())
        .collect();
    let (map, _) = ph2d_gridmap::round_welded(
        &work,
        &cut,
        &combed,
        ph2d_gridmap::Step::uniform(target),
        ph2d_gridmap::RoundOptions::default(),
        &cones,
    );
    let (tri_idx, uv) = ph2d_gridmap::corner_map(&cut, &map);
    let mut positivos = [0usize; 4];
    let mut negativos = [0usize; 4];
    for (t, w) in tri_idx.iter().zip(uv.iter()) {
        let area = (w[1][0] - w[0][0]).mul_add(
            w[2][1] - w[0][1],
            -((w[2][0] - w[0][0]) * (w[1][1] - w[0][1])),
        );
        let mut c = [0.0f32; 3];
        for &i in t {
            let p = pos[i as usize];
            for k in 0..3 {
                c[k] += p[k] / 3.0;
            }
        }
        let Some(b) = band_of(radius(&c) / rmax.max(f32::MIN_POSITIVE)) else {
            continue;
        };
        if area < 0.0 {
            negativos[b] += 1;
        } else {
            positivos[b] += 1;
        }
    }
    // ⭐⭐⭐ **E O «SoG»: as SINGULARIDADES CAEM EM PONTOS INTEIROS DA GRADE?**
    //
    // ⛔⛔ A literatura dá **três** propriedades a um mapa destes (`PLANO_desdobrar_o_mapa.md`
    // §1): **GP** (costura por rotação de 90° e translação inteira — temos por construção),
    // **det+** (sem dobras — medido acima, e não temos) e **SoG** (*singularity on grid*).
    // ⚠️ **O SoG nunca foi medido nesta casa**, e ele é a causa nomeada do defeito de 29/08: a
    // optimização, para baixar a distorção, converte um índice numa combinação, e essas
    // combinações **geram vértices de valência 1 e 2** — que foram os `19` doublets, todos em
    // pontas finas. *Curámos o sintoma sem nunca medir a propriedade.*
    //
    // ⚠️ **A distância é ao inteiro mais próximo, por CANTO** — um vértice de costura tem
    // coordenadas diferentes em cartas diferentes, e o SoG exige que **todas** sejam inteiras.
    let mut sog: [Vec<f64>; 4] = Default::default();
    let mut meio: [usize; 4] = [0; 4];
    for (t, w) in tri_idx.iter().zip(uv.iter()) {
        for (canto, &v) in t.iter().enumerate() {
            if index.get(v as usize).copied().unwrap_or(0) == 0 {
                continue;
            }
            let Some(b) = band_of(radius(&pos[v as usize]) / rmax.max(f32::MIN_POSITIVE)) else {
                continue;
            };
            let d = w[canto]
                .iter()
                .fold(0.0f64, |acc, x| acc.max((x - x.round()).abs()));
            sog[b].push(d);
            // ⚠️ **Meia célula é o modo de falha que a literatura nomeia** — uma singularidade a
            // `+½` não produz quad nenhum, e lê-se como «quase inteira» numa média.
            if (d - 0.5).abs() < 0.05 {
                meio[b] += 1;
            }
        }
    }
    eprintln!("  SoG -- distancia da SINGULARIDADE ao ponto inteiro (0 = na grade):");
    for (b, (lo, hi)) in BANDS.iter().enumerate() {
        if sog[b].is_empty() {
            continue;
        }
        let mut v = sog[b].clone();
        v.sort_by(f64::total_cmp);
        let p50 = v[v.len() / 2];
        eprintln!(
            "  [{lo:.2},{hi:.2}) {:6} canto(s) singular(es)  p50 {p50:.4}  max {:.4}  a meia celula: {}",
            v.len(),
            v[v.len() - 1],
            meio[b]
        );
    }

    // ⚠️ **As DUAS contagens são impressas** — a dobra é a MINORIA, e uma convenção de sinal
    // invertida leria «tudo dobrado» com toda a confiança do mundo.
    eprintln!("  DOBRAS DO MAPA por casca (a dobra e' a MINORIA; as duas contagens saem):");
    for (b, (lo, hi)) in BANDS.iter().enumerate() {
        let n = positivos[b] + negativos[b];
        if n == 0 {
            continue;
        }
        let dobras = negativos[b].min(positivos[b]);
        #[expect(
            clippy::cast_precision_loss,
            reason = "contagem de triangulos para uma percentagem de diagnostico"
        )]
        let pct = 100.0 * dobras as f64 / n as f64;
        eprintln!(
            "  [{lo:.2},{hi:.2}) {:8} tri  (+{} / -{})  dobras {:5} = {pct:.3} %",
            n, positivos[b], negativos[b], dobras
        );
    }
}
