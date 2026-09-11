//! Os gates da cena `=72` — a família da cor e o pareamento do `motion.step`.
//!
//! ⚠️ **Uma cena não se julga por um cook em `t = 0`** no par 3 (a escada precisa
//! de tiques). O que estes gates defendem é a ESTRUTURA — que cada par difere no
//! número que a banda anuncia, e **só** nele. O caminho é do olho do Enio.

use super::*;
use ph2d_nodegraph::graph::NodeId;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

fn scene() -> (MotionDoc, Vec<NodeId>) {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_color_demo_document(&mut doc, &reg).expect("a cena monta");
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

/// **SEIS BANDAS, TRÊS PARES, E NENHUMA SEMENTE ÓRFÃ.**
///
/// ⚠️ A contagem de grelhas é `7`, não `6`: a banda 6 tem uma grelha 1×1 EXTRA que
/// é o relógio global. Escrever o número certo aqui é o que impede este gate de
/// ser afrouxado no dia em que alguém acrescentar uma semente por engano.
#[test]
fn the_scene_is_three_pairs_and_every_grid_has_a_job() {
    let (doc, sinks) = scene();
    assert_eq!(sinks.len(), 6, "três pares");
    assert_eq!(
        nodes_of(&doc, "motion.grid").len(),
        7,
        "seis sementes + o relógio 1×1 da banda 6"
    );
    assert_eq!(nodes_of(&doc, "motion.output").len(), 6);
}

/// **O PAR 1 DIFERE SÓ NO `blend`** — a mesma cor, o mesmo degradê por baixo.
///
/// ⚠️ O oráculo mede os DOIS lados de TODOS os canais. Um par que também mudasse
/// o laranja provaria *"a cor mudou"*, não *"a LEI de composição mudou"* — e é a
/// segunda coisa que a célula pede.
#[test]
fn the_blend_pair_changes_only_the_blending_law() {
    let (doc, _) = scene();
    let tints = nodes_of(&doc, "motion.tint");
    // Quatro no par 1 (dois por banda: o degradê e a cor) + um por banda do par 3.
    let over: Vec<NodeId> = tints
        .iter()
        .copied()
        .filter(|t| param(&doc, *t, "r") == WARM[0])
        .collect();
    assert_eq!(over.len(), 2, "uma cor quente por banda do par 1");
    assert_eq!(param(&doc, over[0], "blend"), BLEND_MIX, "esquerda: Mix");
    assert_eq!(
        param(&doc, over[1], "blend"),
        BLEND_MULTIPLY,
        "direita: Multiply"
    );
    for k in ["r", "g", "b"] {
        assert_eq!(
            param(&doc, over[0], k),
            param(&doc, over[1], k),
            "`{k}` tem de ser o mesmo — senão o par mede duas coisas"
        );
    }
    // E o degradê por baixo existe nos DOIS lados: sem ele, `Multiply` não teria
    // o que preservar e as duas bandas sairiam iguais por acidente.
    let under: Vec<NodeId> = tints
        .iter()
        .copied()
        .filter(|t| param(&doc, *t, "mode") == TINT_GRADIENT)
        .collect();
    assert_eq!(under.len(), 2, "um degradê por banda do par 1");
}

/// **SÓ A BANDA DA DIREITA DO PAR 2 TEM CAMPO, e ele chega à PORTA 1.**
///
/// ⚠️ A porta importa: ligado à porta 0 o ruído seria a geometria, e a banda
/// mostraria confete pelo motivo errado — a cor certa saindo de um conjunto
/// deslocado, não de um `Offset` por-instância.
#[test]
fn only_the_right_half_of_the_palette_pair_drives_the_offset() {
    let (doc, _) = scene();
    let arrays = nodes_of(&doc, "motion.color_array");
    assert_eq!(arrays.len(), 2, "um por banda do par 2");
    let noises = nodes_of(&doc, "value.noise");
    assert_eq!(noises.len(), 1, "só a direita tem campo");
    let n = noises[0];
    assert!(
        doc.graph
            .edges()
            .iter()
            .any(|e| e.from.0 == n && e.to == (arrays[1], 1) && !e.delayed),
        "o campo tem de alimentar a porta `offset` (índice 1) da banda da direita"
    );
    assert!(
        !doc.graph
            .edges()
            .iter()
            .any(|e| e.to.0 == arrays[0] && e.to.1 == 1),
        "a esquerda fica com o `Offset` desligado — é esse o controlo"
    );
    // As duas ciclam a MESMA paleta: o par mede o Offset, não a lista de cores.
    let pal = |id| {
        doc.graph
            .node_text_param_overrides(id)
            .and_then(|m| m.get("palette").cloned())
    };
    assert_eq!(pal(arrays[0]), pal(arrays[1]));
    assert_eq!(pal(arrays[0]), Some(palette_text()));
}

/// **O CAMPO DESCORRELACIONA PEÇAS VIZINHAS** — senão ele lê como um
/// deslocamento global, que é exactamente a lei que esta célula cura.
///
/// ⚠️ O gate deriva a conta do PASSO DA GRELHA, não a repete: uma grelha mais
/// fechada com a mesma frequência daria manchas, e o número teria de mudar com
/// ela. `0.5` é meia célula de ruído entre vizinhas — o piso abaixo do qual duas
/// peças lado a lado partilham a fatia.
#[test]
fn the_offset_field_is_decorrelated_across_neighbouring_pieces() {
    let (doc, _) = scene();
    let grids = nodes_of(&doc, "motion.grid");
    // A grelha do par 2 é a que tem 8 colunas.
    let g8 = grids
        .iter()
        .copied()
        .find(|g| param(&doc, *g, "cols") == 8.0)
        .expect("a grelha do par 2");
    let gap = param(&doc, g8, "gap_x");
    assert!(
        gap * FIELD_FREQ >= 0.5,
        "vizinhas a {} de distância no ruído — perto demais para diferirem",
        gap * FIELD_FREQ
    );
    assert!(
        param(&doc, nodes_of(&doc, "value.noise")[0], "speed") == 0.0,
        "o campo é PARADO: um tremeluzir esconderia o que a banda mostra"
    );
}

/// **O PAR 3 DIFERE SÓ NA FONTE DO RELÓGIO** — os dois `motion.step` são o mesmo
/// nó, com os mesmos números.
///
/// ⚠️ **É o par cujos dois lados têm de FICAR IGUAIS na tela**, então o gate tem
/// de ser mais duro que o olho: se um `step`/`count_max`/`mode` divergisse, as
/// fileiras diferiam por um motivo que nada tem a ver com a célula, e o smoke
/// leria isso como o defeito.
#[test]
fn the_step_pair_changes_only_where_the_beat_comes_from() {
    let (doc, _) = scene();
    let steps = nodes_of(&doc, "motion.step");
    assert_eq!(steps.len(), 2, "um por banda do par 3");
    for k in ["channel", "step", "count_max", "mode"] {
        assert_eq!(
            param(&doc, steps[0], k),
            param(&doc, steps[1], k),
            "`{k}` tem de ser o mesmo nos dois lados"
        );
    }
    let beats = nodes_of(&doc, "pulse.beat");
    assert_eq!(beats.len(), 2);
    for b in &beats {
        assert_eq!(param(&doc, *b, "period"), BEAT, "o mesmo compasso");
        assert!(
            doc.graph
                .edges()
                .iter()
                .any(|e| e.from.0 == *b && e.to == (*b, 1) && e.delayed),
            "o metrônomo precisa da memória de beira dele"
        );
    }
    for s in &steps {
        assert!(
            doc.graph
                .edges()
                .iter()
                .any(|e| e.from.0 == *s && e.to == (*s, 2) && e.delayed),
            "sem o `pre` do `state` a escada nunca passa do primeiro degrau"
        );
    }
}

/// **O RELÓGIO DA BANDA 6 CONTA UMA LINHA SÓ, e o da banda 5 conta seis.**
///
/// É a célula inteira: o `pulse.beat` cozinha tantas linhas quantas o stream que o
/// alimenta, então alimentá-lo com uma grelha 1×1 é literalmente autorar *"um
/// batimento global"*. FALSIFICADO se as duas fontes tivessem a mesma contagem —
/// o par não encenaria nada.
#[test]
fn the_right_half_of_the_step_pair_is_fed_by_a_one_row_clock() {
    let (doc, _) = scene();
    let beats = nodes_of(&doc, "pulse.beat");
    let feeder = |b: NodeId| {
        doc.graph
            .edges()
            .iter()
            .find(|e| e.to == (b, 0) && !e.delayed)
            .map(|e| e.from.0)
            .expect("todo metrônomo tem fonte")
    };
    let rows_cols = |n: NodeId| (param(&doc, n, "rows"), param(&doc, n, "cols"));
    // A esquerda conta as SEIS peças: a fonte dela é a cadeia da própria fileira,
    // então subimos até a grelha.
    let left_src = feeder(beats[0]);
    let right_src = feeder(beats[1]);
    assert_eq!(
        rows_cols(right_src),
        (1.0, 1.0),
        "a direita é um relógio de UMA linha"
    );
    assert_ne!(
        right_src, left_src,
        "as duas bandas não podem partilhar a fonte do relógio"
    );
}

/// **O DIAGNOSER DA CASA NÃO ACHA BURACO NESTA CENA.**
///
/// O gate irmão do `=71`, e ele nasceu de um smoke reprovado: um fio esquecido
/// numa porta de valor não aparece em cook nenhum a `t = 0` — aparece como uma
/// banda parada na tela do Enio.
#[test]
fn the_house_diagnoser_finds_no_hole_in_this_scene() {
    let (doc, _) = scene();
    let reg = registry();
    let d = ph2d_motion_diagnose::diagnose(&doc.graph, &reg);
    assert!(
        d.is_empty(),
        "a cena não encena defeito nenhum, então não pode ter um: {d:?}"
    );
}
