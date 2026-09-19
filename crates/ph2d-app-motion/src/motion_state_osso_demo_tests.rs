//! Gates da cena `=125` — **o osso numa cadeia**.
//!
//! ⚠️⚠️ **Como as irmãs, o que passa por um `source.shape` coze `n = 0` num arnês headless** — a
//! geometria é assada pela SHELL. Medido: o sink das duas colunas vestidas sai **sem `P` e sem
//! `rot`**, e a primeira redacção destes gates pediu-os ao sink e reprovou.
//!
//! ⇒ o que se mede é a **CADEIA** (a cabeça de cada coluna, antes do duplicador) e a estrutura do
//! grafo. *A corrente que a forma recebe é a mesma; o que falta headless é a forma.*

use super::*;
use ph2d_node_registry::NodeRegistry;

fn cena() -> (MotionDoc, NodeRegistry, Vec<NodeId>) {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    let mut doc = MotionDoc::default();
    let sinks = build(&mut doc, &reg).expect("a cena monta");
    doc.graph.validate(&reg).expect("bem-tipada");
    (doc, reg, sinks)
}

/// ⭐⭐⭐ **A coluna do MEIO é CURVA — sem isso a cena não pode mostrar orientação nenhuma.**
///
/// ⚠️ **Esta é a metade que a cena existe para ter:** numa cadeia recta todos os ossos apontam
/// para o mesmo lado, e *«180° rodado»* produz a mesma silhueta. A curva é o discriminador, e o
/// gate mede-a pelo produto — a direcção de mundo tem de ANDAR ao longo da cadeia.
#[test]
fn a_coluna_da_esquerda_curva_e_a_do_meio_nao() {
    let (doc, reg, _sinks) = cena();
    let mut cook = ph2d_nodegraph::cook::Cook::new();
    cook.advance_tick(&doc.graph, &reg, 0.0).expect("avanca");

    // As CABEÇAS: os `motion.move` que deslocam cada coluna — é o que entra na porta `1` do
    // duplicador, e é a corrente que a forma de facto veste.
    let cabecas: Vec<NodeId> = doc
        .graph
        .nodes()
        .iter()
        .filter(|n| n.type_name == "motion.move")
        .map(|n| n.id)
        .collect();
    assert_eq!(cabecas.len(), 3, "uma cabeca por coluna");

    let rot = |n: NodeId, cook: &mut ph2d_nodegraph::cook::Cook| {
        let o = cook.cook(&doc.graph, &reg, n, 0.0).expect("coze");
        match o[0].as_stream().get("rot") {
            Some(ph2d_nodegraph::attr::Column::Scalar(v)) => v.clone(),
            _ => panic!("o `rot` tem de existir: e' o que a forma le'"),
        }
    };

    // A ESQUERDA é a curva: o ângulo de mundo ANDA, junta a junta.
    let esquerda = rot(cabecas[0], &mut cook);
    let passos: Vec<f32> = esquerda.windows(2).map(|w| w[1] - w[0]).collect();
    assert!(
        passos.iter().all(|d| (d - CURVA).abs() < 1e-3),
        "cada junta da curva vira {CURVA} graus: {passos:?}"
    );

    // E a do MEIO nasce recta — a curvatura dela vem toda da ONDA, a jusante.
    let meio = rot(cabecas[1], &mut cook);
    assert!(
        meio.windows(2).all(|w| (w[0] - w[1]).abs() < 1e-4),
        "a base da simulacao e' recta: {meio:?}"
    );
}

/// ⚠️ **A DIREITA é o CONTROLO e não pode passar por um duplicador** — se passasse, ela deixava
/// de ser *«as mesmas posições sem forma»* e virava uma terceira coisa.
#[test]
fn a_direita_e_a_cadeia_nua() {
    let (doc, _, sinks) = cena();
    assert_eq!(sinks.len(), 3, "tres colunas");
    let g = &doc.graph;
    let alimenta = |sink: NodeId| -> Vec<String> {
        g.edges()
            .iter()
            .filter(|e| e.to.0 == sink)
            .filter_map(|e| g.nodes().iter().find(|n| n.id == e.from.0))
            .map(|n| n.type_name.clone())
            .collect()
    };
    assert_eq!(
        alimenta(sinks[2]),
        vec!["motion.move".to_string()],
        "o controlo vem da cadeia directamente"
    );
    assert!(
        alimenta(sinks[0]).contains(&"motion.duplicator".to_string()),
        "e as outras duas passam pelo duplicador"
    );
}

/// ⚠️ **As três colunas não se sobrepõem** — senão a comparação que a cena pede é impossível.
#[test]
fn as_tres_colunas_ficam_separadas() {
    let (doc, reg, _) = cena();
    let mut cook = ph2d_nodegraph::cook::Cook::new();
    cook.advance_tick(&doc.graph, &reg, 0.0).expect("avanca");
    let cabecas: Vec<NodeId> = doc
        .graph
        .nodes()
        .iter()
        .filter(|n| n.type_name == "motion.move")
        .map(|n| n.id)
        .collect();
    let faixa = |n: NodeId, cook: &mut ph2d_nodegraph::cook::Cook| {
        let o = cook.cook(&doc.graph, &reg, n, 0.0).expect("coze");
        let Some(ph2d_nodegraph::attr::Column::Vec2(p)) = o[0].as_stream().get("P") else {
            panic!("sem P")
        };
        p.iter().fold((f32::MAX, f32::MIN), |(lo, hi), q| {
            (lo.min(q[0]), hi.max(q[0]))
        })
    };
    let a = faixa(cabecas[0], &mut cook);
    let b = faixa(cabecas[1], &mut cook);
    let c = faixa(cabecas[2], &mut cook);
    assert!(
        a.1 < b.0,
        "a esquerda acaba em {} e o meio comeca em {}",
        a.1,
        b.0
    );
    assert!(
        b.1 < c.0,
        "o meio acaba em {} e a direita comeca em {}",
        b.1,
        c.0
    );
}

/// ⭐⭐⭐ **A COLUNA DO MEIO É UMA SIMULAÇÃO — e ela é a demonstração da lei que esta jornada
/// curou.**
///
/// O oscilador escreve o canal `Rotation` (a coluna `rot`) e o `rig.fk` a seguir **re-resolve**
/// a cadeia. ⚠️ **Isto só funciona pelo degrau 2 da escada do [`fk::local`]:** a corrente que sai
/// do `rig.skeleton` já traz `lrot`, e sem esse degrau o `rot` que o oscilador reescreve seria
/// deitado fora **em silêncio** — a cena ficava parada com todos os números certos.
///
/// ⚠️ **As duas metades:** a fiação tem de existir E as POSIÇÕES têm de andar. Sem a segunda, um
/// oscilador ligado que só rodasse as peças passaria — *e é exactamente esse o defeito que o
/// degrau existe para impedir*.
#[test]
fn a_do_meio_e_uma_simulacao_que_move_as_posicoes() {
    let (doc, reg, _) = cena();
    let g = &doc.graph;
    let tipo = |t: &str| g.nodes().iter().find(|n| n.type_name == t).map(|n| n.id);
    let osc = tipo("motion.oscillator").expect("a simulacao tem um oscilador");
    let fk = tipo("rig.fk").expect("e um `rig.fk` a re-resolver");
    assert!(
        g.edges().iter().any(|e| e.from.0 == osc && e.to.0 == fk),
        "o oscilador tem de alimentar o `rig.fk`, senao ele so' roda as pecas"
    );

    // ⛔ E as POSIÇÕES têm de andar entre dois instantes.
    let ps = |t: f64, cook: &mut ph2d_nodegraph::cook::Cook| {
        cook.advance_tick(g, &reg, t).expect("avanca");
        let o = cook.cook(g, &reg, fk, t).expect("coze");
        match o[0].as_stream().get("P") {
            Some(ph2d_nodegraph::attr::Column::Vec2(v)) => v.clone(),
            _ => panic!("sem P"),
        }
    };
    let mut cook = ph2d_nodegraph::cook::Cook::new();
    let a = ps(0.0, &mut cook);
    let b = ps(0.5, &mut cook);
    let maior = a
        .iter()
        .zip(&b)
        .map(|(p, q)| ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2)).sqrt())
        .fold(0.0f32, f32::max);
    assert!(
        maior > 0.05,
        "meio segundo depois a cadeia tem de estar NOUTRO sitio (maior desvio {maior:e})"
    );
}

/// O roteiro nomeia o que aparece na tela, e nada que não apareça.
///
/// ⛔⛔ **E o passo do PIVÔ é DERIVADO do rótulo registado, não escrito à mão:** um passo que manda
/// arrastar um controlo AFIRMA que ele está na tela com aquele nome, e o dia em que alguém
/// renomear o `Pivot X` no cartão o roteiro passa a mandar o dono procurar uma coisa que não
/// existe. *É a mesma lei que o §5.0 cobra de um passo que nomeia uma LINHA de painel.*
///
/// ⚠️ **O que este gate NÃO alcança** é a PROSA sobre a geometria — e ela já mordeu: até 19/09 o
/// passo (1) dizia que o osso é *«largo do lado para onde a cadeia CRESCE»*, que é o inverso do
/// que ele desenha, e a cláusula do *«deu errado»* descrevia o estado CERTO. O gate ficou verde
/// porque as palavras estavam todas lá. *O facto vive em `o_osso_e_afilado_e_nao_um_losango` e em
/// `a_cabeca_do_osso_cai_sobre_a_posicao`; o que está aqui é só a existência dos nomes.*
#[test]
fn o_roteiro_nomeia_o_que_a_cena_tem() {
    let texto = include_str!("motion_state_osso_demo.rs");
    for nome in ["ESQUERDA", "MEIO", "DIREITA", "Skeleton", "Duplicator"] {
        let achou = texto.contains(nome) || texto.to_lowercase().contains(&nome.to_lowercase());
        assert!(achou, "o roteiro tem de nomear {nome:?}");
    }

    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    let hints = reg
        .param_ui(ph2d_node_motion_shape::MANIFEST.id)
        .expect("o cartao do `source.shape` tem hints");
    let rotulo = |nome: &str| {
        hints
            .iter()
            .find(|h| h.param == nome)
            .unwrap_or_else(|| panic!("o param {nome} tem de ter linha no cartao"))
            .label
    };
    for nome in [
        ph2d_node_motion_shape::param::PIVOT_X,
        ph2d_node_motion_shape::param::PIVOT_Y,
    ] {
        let l = rotulo(nome);
        assert!(
            texto.contains(l),
            "o roteiro manda arrastar o {nome:?}, entao tem de o chamar pelo nome que esta' na \
             tela ({l:?})"
        );
    }
}
