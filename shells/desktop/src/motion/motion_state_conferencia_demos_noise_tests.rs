//! Gates da cena `=37` — **o catálogo de kernels do `value.noise`** (doc 89,
//! folha 15).
//!
//! Irmão do `..._blend_tests.rs`, cortado por ASSUNTO pela mesma razão: aquele
//! mede *em que MODO o resultado é desenhado*, este *que PADRÃO o campo desenha*.
//! O `registry` vem do pai (`pub(super)`) — uma segunda cópia divergiria.
//!
//! ⚠️ **A 1ª versão deste arquivo era VERDE POR VÁCUO, e a mutação a apanhou.**
//! Ela afirmava *"as quatro grades desenham quatro campos"* comparando os quatro
//! streams entre si — mas a cena põe cada grade num `dx` próprio e o ruído é
//! **ESPACIAL** (`space = 1.0`), então as quatro amostram partes diferentes do
//! campo e diferem **por POSIÇÃO**, com ou sem kernel. Forçar `kernel = 0.0` nas
//! quatro deixava o gate verde. A pergunta que isola o kernel da posição é a de
//! baixo: **a MESMA grade, no MESMO lugar, com o param trocado**.

use super::tests::registry;
use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::node::NodeTypeId;
use ph2d_nodegraph::value::CookValue;

/// O TAMANHO que a cena dirige — a coluna que este demo escreve, e por isso a
/// que o gate tem de ler (o `cook` do pai devolve `P`, que aqui é a MESMA grade
/// nas quatro colunas: lê-la faria o gate concordar por vácuo).
fn sizes(doc: &MotionDoc, reg: &NodeRegistry, sink: NodeId) -> Vec<f32> {
    let mut c = Cook::new();
    let out = c.cook(&doc.graph, reg, sink, 0.0).expect("a cena coze");
    let CookValue::Instances(s) = &out[0] else {
        panic!("a saida e um stream")
    };
    match ph2d_nodegraph::attr::Stream::get(s, "size") {
        Some(Column::Vec2(v)) => v.iter().map(|p| p[0]).collect(),
        _ => Vec::new(),
    }
}

/// Os `value.noise` da cena, na ordem em que foram criados (= a ordem das grades).
fn noise_nodes(doc: &MotionDoc) -> Vec<NodeId> {
    let t = NodeTypeId::of("value.noise");
    doc.graph
        .nodes()
        .iter()
        .filter(|n| n.type_id() == t)
        .map(|n| n.id)
        .collect()
}

fn param(doc: &MotionDoc, n: NodeId, k: &str) -> f32 {
    doc.graph
        .node_param_overrides(n)
        .and_then(|p| p.get(k))
        .copied()
        .unwrap_or(0.0)
}

/// **A cena AUTORA os quatro pares distintos.** É a metade declarativa: se o
/// laço parasse de escrever o `kernel` (ou o `feature`), a cena mostraria a mesma
/// coisa quatro vezes e o smoke leria isso como *"ainda não implementaram"*.
#[test]
fn the_kernel_scene_authors_four_distinct_kernels() {
    let reg = registry();
    let mut doc = MotionDoc::new();
    let sinks = build_noise_kernel_demo_document(&mut doc, &reg).expect("cena =37");
    assert_eq!(sinks.len(), 4, "Value / Perlin / Cells / Cracks");

    let ns = noise_nodes(&doc);
    assert_eq!(ns.len(), 4, "um `value.noise` por grade");
    let pairs: Vec<(f32, f32)> = ns
        .iter()
        .map(|&n| (param(&doc, n, "kernel"), param(&doc, n, "feature")))
        .collect();
    assert_eq!(
        pairs,
        vec![(0.0, 0.0), (1.0, 0.0), (2.0, 0.0), (2.0, 1.0)],
        "os quatro pares (kernel, feature) da folha 15"
    );
}

/// **Trocar o kernel numa grade MUDA o campo que ela desenha.** Este é o oráculo
/// que a versão anterior não tinha: a grade fica onde está, o `dx` não se move, o
/// seed não se move — **só o param muda**, então o que sobra é o kernel.
///
/// Ele cobre também o `feature`, que é o 2º canal e cai no mesmo modo de falha
/// (um `Cracks` que cozinhasse como `Cells` seria uma grade a mais do mesmo).
#[test]
fn swapping_the_kernel_on_one_grid_changes_the_field_it_draws() {
    let reg = registry();
    let mut doc = MotionDoc::new();
    let sinks = build_noise_kernel_demo_document(&mut doc, &reg).expect("cena =37");
    let ns = noise_nodes(&doc);

    // A 1ª grade (Value) serve de bancada: o kernel varre 0 -> 1 -> 2 no MESMO
    // lugar, e cada troca tem de mover o campo.
    let mut seen: Vec<Vec<f32>> = Vec::new();
    for k in [0.0_f32, 1.0, 2.0] {
        doc.graph.set_param(ns[0], "kernel", k);
        let f = sizes(&doc, &reg, sinks[0]);
        assert!(!f.is_empty(), "kernel {k} nao escreveu `size`");
        // CONTROLE: o campo VARIA. Um kernel que devolvesse constante daria
        // grades regulares e os `assert` de diferença ainda passariam por ruído.
        let (lo, hi) = (
            f.iter().cloned().fold(f32::MAX, f32::min),
            f.iter().cloned().fold(f32::MIN, f32::max),
        );
        assert!(hi - lo > 0.05, "kernel {k}: campo chato, span {}", hi - lo);
        seen.push(f);
    }
    for a in 0..seen.len() {
        for b in (a + 1)..seen.len() {
            let same = seen[a]
                .iter()
                .zip(&seen[b])
                .filter(|(x, y)| (**x - **y).abs() < 1e-5)
                .count();
            assert!(
                same * 5 < seen[a].len(),
                "os kernels {a} e {b} desenham o mesmo campo em {same}/{} celulas",
                seen[a].len()
            );
        }
    }

    // E o 2º canal: no CELULAR, `Cells` e `Cracks` sao campos diferentes.
    doc.graph.set_param(ns[0], "kernel", 2.0);
    doc.graph.set_param(ns[0], "feature", 0.0);
    let cells = sizes(&doc, &reg, sinks[0]);
    doc.graph.set_param(ns[0], "feature", 1.0);
    let cracks = sizes(&doc, &reg, sinks[0]);
    let same = cells
        .iter()
        .zip(&cracks)
        .filter(|(x, y)| (**x - **y).abs() < 1e-5)
        .count();
    assert!(
        same * 5 < cells.len(),
        "Cells e Cracks coincidem em {same}/{} celulas",
        cells.len()
    );
}

/// **Toda grade fica VISÍVEL.** O canal dirigido é o TAMANHO, que não tem metade
/// negativa — a 1ª versão da cena mandava um campo bipolar para lá e saía com
/// tamanhos até `-0.26`, com a grade do Cracks (a de média mais baixa) quase
/// invisível. O gate afirma o piso e a faixa, não um número escolhido.
#[test]
fn every_grid_in_the_kernel_scene_is_visible() {
    let reg = registry();
    let mut doc = MotionDoc::new();
    let sinks = build_noise_kernel_demo_document(&mut doc, &reg).expect("cena =37");
    for (k, &s) in sinks.iter().enumerate() {
        let f = sizes(&doc, &reg, s);
        let (lo, hi) = (
            f.iter().cloned().fold(f32::MAX, f32::min),
            f.iter().cloned().fold(f32::MIN, f32::max),
        );
        assert!(lo > 0.0, "grade {k}: tamanho NEGATIVO, min {lo}");
        assert!(
            hi - lo > 0.05,
            "grade {k}: faixa invisivel, span {}",
            hi - lo
        );
    }
}

/// Sonda: o que a cena `=37` de facto monta. Roda com
/// `cargo test -p ph2d-host-desktop probe_noise_kernel_scene -- --ignored --nocapture`.
#[test]
#[ignore = "sonda de medicao"]
fn probe_noise_kernel_scene() {
    let reg = registry();
    let mut doc = MotionDoc::new();
    let sinks = build_noise_kernel_demo_document(&mut doc, &reg).expect("cena =37");
    for (k, &s) in sinks.iter().enumerate() {
        let f = sizes(&doc, &reg, s);
        let (lo, hi) = (
            f.iter().cloned().fold(f32::MAX, f32::min),
            f.iter().cloned().fold(f32::MIN, f32::max),
        );
        let mean = f.iter().sum::<f32>() / f.len() as f32;
        let name = ["Value", "Perlin", "Cells", "Cracks"][k];
        eprintln!(
            "{name:>7}: n={} size=[{lo:.3}, {hi:.3}] media={mean:.3}",
            f.len()
        );
    }
}
