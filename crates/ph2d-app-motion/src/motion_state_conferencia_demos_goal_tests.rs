//! Os gates da cena `=74` — o alvo e a mira.
//!
//! ⚠️ **Uma cena de FORÇAS não se julga por um cook em `t = 0`**: nada se moveu ainda.
//! O que estes gates defendem é a ESTRUTURA — que a força chega ao integrador pelo laço
//! certo, que as duas metades diferem no número que a linha anuncia e **só** nele, e que
//! nada sai do quadrante. O caminho é do olho do Enio.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::NodeId;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

fn scene() -> (MotionDoc, Vec<NodeId>) {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_goal_demo_document(&mut doc, &reg).expect("a cena monta");
    doc.graph.validate(&reg).expect("bem-tipado");
    (doc, sinks)
}

fn nodes_of(doc: &MotionDoc, ty: &str) -> Vec<NodeId> {
    doc.graph
        .nodes()
        .iter()
        .filter(|n| n.type_name == ty)
        .map(|n| n.id)
        .collect()
}

fn param(doc: &MotionDoc, id: NodeId, name: &str) -> f32 {
    doc.graph
        .node_param_overrides(id)
        .and_then(|m| m.get(name).copied())
        .unwrap_or(f32::NAN)
}

fn band_box(doc: &MotionDoc, reg: &NodeRegistry, sink: NodeId) -> (usize, [f32; 2], [f32; 2]) {
    let mut cook = Cook::new();
    let out = cook.cook(&doc.graph, reg, sink, 0.0).expect("cozinha");
    let s = out[0].as_stream();
    let Some(Column::Vec2(p)) = s.get("P") else {
        panic!("a banda tem de ter geometria")
    };
    assert!(!p.is_empty());
    let mut lo = [f32::INFINITY; 2];
    let mut hi = [f32::NEG_INFINITY; 2];
    for q in p {
        for a in 0..2 {
            lo[a] = lo[a].min(q[a]);
            hi[a] = hi[a].max(q[a]);
        }
    }
    (p.len(), lo, hi)
}

/// **DUAS LINHAS, QUATRO NUVENS, E QUATRO MARCAS DE ALVO.**
///
/// ⚠️ As marcas contam: um alvo que não se vê torna a cena um enigma — as peças
/// convergem para *algum sítio* e o artista adivinha qual.
#[test]
fn the_scene_is_two_rows_with_their_targets_drawn() {
    let (doc, sinks) = scene();
    assert_eq!(sinks.len(), 8, "quatro nuvens + quatro marcas");
    assert_eq!(nodes_of(&doc, "motion.output").len(), 12, "e as 4 legendas");
    assert_eq!(nodes_of(&doc, "force.attractor").len(), 4);
    assert_eq!(nodes_of(&doc, "motion.integrate").len(), 4, "um por nuvem");
}

/// **TODA FORÇA CHEGA AO INTEGRADOR PELO LAÇO.**
///
/// ⚠️ É o gate que mais vezes disparou nas cenas irmãs: uma força sem o `pre` do
/// integrador **não move nada**, e a banda fica parada — que numa cena de smoke lê como
/// *"o knob não faz nada"*.
#[test]
fn every_cloud_wires_its_force_through_the_integrator_loop() {
    let (doc, _) = scene();
    for integ in nodes_of(&doc, "motion.integrate") {
        let edges = doc.graph.edges();
        assert!(
            edges
                .iter()
                .any(|e| e.from.0 == integ && e.delayed && e.to.1 == 0),
            "o `pre` do integrador tem de alimentar a cabeça da cadeia de forças"
        );
        assert!(
            edges.iter().any(|e| e.to == (integ, 1) && !e.delayed),
            "e a ponta tem de alimentar o `forces`"
        );
    }
}

/// **A LINHA 1 DIFERE SÓ NO `target_mode`** — a mesma força, o mesmo alcance.
#[test]
fn the_goal_row_changes_only_where_the_target_comes_from() {
    let (doc, _) = scene();
    let att = nodes_of(&doc, "force.attractor");
    let (esq, dir) = (att[0], att[1]);
    assert_eq!(param(&doc, esq, "target_mode"), 0.0, "esquerda: Point");
    assert_eq!(
        param(&doc, dir, "target_mode"),
        TARGET_STREAM,
        "direita: Stream"
    );
    for k in ["strength", "radius"] {
        assert_eq!(
            param(&doc, esq, k),
            param(&doc, dir, k),
            "`{k}` tem de ser o mesmo — senão a linha mede duas coisas"
        );
    }
    // E só a direita tem fio na porta do alvo.
    let has_port = |a: NodeId| doc.graph.edges().iter().any(|e| e.to == (a, 1));
    assert!(has_port(dir), "o modo Stream precisa do fio");
    assert!(
        !has_port(esq),
        "e o modo Point não pode tê-lo — é o controlo"
    );
}

/// **A LINHA 1 PÕE MAIS DE UM ALVO À DIREITA — senão o «mais próximo» não tem o que
/// escolher.**
///
/// FALSIFICADO por um anel de UM ponto: as duas metades juntar-se-iam num sítio só e a
/// linha ficaria verde e muda.
#[test]
fn the_goal_row_offers_more_than_one_target_on_the_right() {
    let (doc, _) = scene();
    let rings = nodes_of(&doc, "motion.distribute_radial");
    assert_eq!(rings.len(), 1, "só a direita tem anel de alvos");
    assert!(
        param(&doc, rings[0], "count") >= 2.0,
        "um alvo só não dá o que escolher"
    );
    assert_eq!(param(&doc, rings[0], "count"), GOALS);
}

/// **A LINHA 2 DIFERE SÓ NO `lead`, e as DUAS estão em modo `Stream`.**
///
/// ⚠️ A antecipação precisa da velocidade do ALVO, e um par de params não tem
/// velocidade — uma metade em `Point` mediria o modo, não a mira.
#[test]
fn the_aim_row_changes_only_the_prediction() {
    let (doc, _) = scene();
    let att = nodes_of(&doc, "force.attractor");
    let (esq, dir) = (att[2], att[3]);
    for a in [esq, dir] {
        assert_eq!(
            param(&doc, a, "target_mode"),
            TARGET_STREAM,
            "as duas perseguem um STREAM"
        );
    }
    assert_eq!(param(&doc, esq, "lead"), 0.0, "esquerda: sem antecipação");
    assert_eq!(param(&doc, dir, "lead"), LEAD, "direita: antecipa");
    for k in ["strength", "radius"] {
        assert_eq!(param(&doc, esq, k), param(&doc, dir, k));
    }
}

/// **O ALVO DA LINHA 2 MEXE-SE E PUBLICA A VELOCIDADE DELE.**
///
/// ⚠️ **Sem a coluna `vel` a antecipação lê zero e o knob fica MUDO** — as duas
/// fileiras fariam o mesmo caminho e o smoke leria isso como *"o Predict não faz
/// nada"*, que é a conclusão errada sobre o código certo. E o `motion.velocity`
/// carrega estado: sem o `pre` self-loop ele não tem o quadro anterior para comparar.
#[test]
fn the_aim_rows_target_moves_and_publishes_its_velocity() {
    let (doc, _) = scene();
    let osc = nodes_of(&doc, "motion.oscillator");
    assert_eq!(osc.len(), 2, "um alvo a balançar por metade");
    for o in &osc {
        assert!(
            param(&doc, *o, "amplitude") > 0.0,
            "um alvo parado não lidera"
        );
        assert!(param(&doc, *o, "frequency") > 0.0);
    }
    let vels = nodes_of(&doc, "motion.velocity");
    assert_eq!(vels.len(), 2);
    for v in &vels {
        assert!(
            doc.graph
                .edges()
                .iter()
                .any(|e| e.from.0 == *v && e.to == (*v, 1) && e.delayed),
            "o medidor de velocidade precisa da memória dele"
        );
        // …e ele alimenta a porta do alvo do atrator daquela metade.
        assert!(
            doc.graph
                .edges()
                .iter()
                .any(|e| e.from.0 == *v && e.to.1 == 1 && !e.delayed),
            "a velocidade medida tem de chegar à porta `target`"
        );
    }
}

/// **NENHUMA NUVEM NASCE FORA DO QUADRANTE DELA.**
///
/// ⚠️ Em `t = 0` nada se moveu ainda, então a caixa é a da grelha em repouso — é
/// exactamente aí que o defeito de colocação da `=73` aparecia. A folga cobre o raio da
/// peça, que não entra no `P`.
#[test]
fn no_cloud_is_born_outside_its_slot() {
    let (doc, sinks) = scene();
    let reg = registry();
    for (i, sink) in sinks.iter().take(4).enumerate() {
        let (row, right) = (i / 2, i % 2 == 1);
        let (w, h) = footprint(row);
        let cx = if right { COL_X } else { -COL_X };
        let (n, lo, hi) = band_box(&doc, &reg, *sink);
        let slack = 0.12;
        for (a, (c, size)) in [(cx, w), (ROW_Y[row], h)].into_iter().enumerate() {
            assert!(
                lo[a] >= c - size * 0.5 - slack && hi[a] <= c + size * 0.5 + slack,
                "nuvem {} eixo {a}: [{:.2}..{:.2}] fora do quadrante em {c:.2} — n={n}",
                i + 1,
                lo[a],
                hi[a]
            );
        }
    }
}

/// **OS ALVOS NASCEM NO MESMO QUADRANTE DA NUVEM QUE OS PERSEGUE.**
///
/// ⚠️ Um alvo deixado na origem enquanto a nuvem está no quadrante daria uma cena em
/// que as quatro nuvens correm para o meio da tela — verde nos params, ilegível na
/// imagem.
#[test]
fn every_target_is_born_in_the_quadrant_of_the_cloud_that_chases_it() {
    let (doc, sinks) = scene();
    let reg = registry();
    for (i, sink) in sinks.iter().skip(4).enumerate() {
        let (row, right) = (i / 2, i % 2 == 1);
        let cx = if right { COL_X } else { -COL_X };
        let (_, lo, hi) = band_box(&doc, &reg, *sink);
        let mid = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5];
        assert!(
            (mid[0] - cx).abs() < 0.6 && (mid[1] - ROW_Y[row]).abs() < 0.6,
            "a marca {} está em {mid:?}, longe do quadrante ({cx:.2}, {:.2})",
            i + 1,
            ROW_Y[row]
        );
    }
}

/// **O DIAGNOSER DA CASA NÃO ACHA BURACO NESTA CENA.**
#[test]
fn the_house_diagnoser_finds_no_hole_in_this_scene() {
    let (doc, _) = scene();
    let reg = registry();
    let d = ph2d_motion_diagnose::diagnose(&doc.graph, &reg);
    assert!(d.is_empty(), "a cena não encena defeito nenhum: {d:?}");
}
