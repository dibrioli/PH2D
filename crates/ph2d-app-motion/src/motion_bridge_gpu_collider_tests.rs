//! O gate da recusa do colisor declarado pelo nome (doc 109 W2) — ver [`super::graph_declares_collider`].

use super::{GpuOutcome, RECUSA_COLISOR, cook_gpu, graph_declares_collider};
use crate::motion_state::MotionState;

/// ⭐⭐ **A LIGAÇÃO: a ponte do produto recusa de facto o documento** — o gate abaixo prova a
/// pergunta; este prova que ela é feita no caminho que cozinha.
///
/// ⚠️ Com o CONTROLO: o mesmo grafo sem o nome não pode sair com esta razão, senão a recusa seria
/// incondicional e o gate passaria por ela.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn the_bridge_cooks_a_document_that_names_the_collider_on_the_cpu() {
    use ph2d_nodegraph::graph::Edge;
    let Ok(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) else {
        panic!("sem adapter — este gate mede a ponte do device e nao tem versao de CPU");
    };
    let scopes = ph2d_nodegraph::cook::TimeScopes::new();
    let monta = |com_nome: bool| {
        let mut m = MotionState::new();
        let g = &mut m.doc.graph;
        let grade = g.add_node("motion.grid");
        let out = g.add_node("motion.output");
        g.connect(Edge {
            from: (grade, 0),
            to: (out, 0),
            delayed: false,
        })
        .expect("liga");
        if com_nome {
            let d = g.add_node("motion.drive");
            g.set_text_param(d, "column", "collider");
        }
        m.sinks = vec![out];
        m
    };
    let mut sem = monta(false);
    let _ = cook_gpu(&mut sem, &gpu, 0, 1.0 / 60.0, &scopes);
    assert_ne!(
        sem.route_said,
        Some(RECUSA_COLISOR),
        "o controlo nao pode ser recusado por isto"
    );

    let mut com = monta(true);
    let saida = cook_gpu(&mut com, &gpu, 0, 1.0 / 60.0, &scopes);
    assert!(matches!(saida, GpuOutcome::FellThrough), "{saida:?}");
    assert_eq!(com.route_said, Some(RECUSA_COLISOR));
}

/// ⭐⭐ **Escrever `collider` pelo nome manda o documento para a CPU — e só esse nome.**
///
/// ⚠️ As três metades, porque cada uma sozinha tem cura errada: *«sem nome, dispositivo»* passa
/// com a função a devolver sempre `false`; *«com o nome, CPU»* passa com ela sempre `true`; e *«um
/// nome PARECIDO não conta»* é o que impede a cura preguiçosa por `contains`, que mandaria para a
/// CPU as colunas `collider_around`/`collider_inside` que o shell publica e o nó retira.
#[test]
fn only_the_exact_collider_name_sends_the_document_to_the_cpu() {
    let mut m = MotionState::new();
    let drive = m.doc.graph.add_node("motion.drive");
    assert!(
        !graph_declares_collider(&m.doc.graph),
        "sem text param nenhum"
    );

    m.doc.graph.set_text_param(drive, "column", "tint");
    assert!(!graph_declares_collider(&m.doc.graph), "outra coluna");

    m.doc
        .graph
        .set_text_param(drive, "column", "collider_fit_half");
    assert!(
        !graph_declares_collider(&m.doc.graph),
        "um nome PARECIDO nao conta"
    );

    // As outras duas colunas da declaração (doc 109 §5) contam como o raio.
    for nome in ["collider_box", "collider_offset"] {
        m.doc.graph.set_text_param(drive, "column", nome);
        assert!(
            graph_declares_collider(&m.doc.graph),
            "`{nome}` e' declaracao"
        );
    }

    m.doc.graph.set_text_param(drive, "column", " collider ");
    assert!(
        graph_declares_collider(&m.doc.graph),
        "o nome, com espacos a volta"
    );

    // A pergunta é sobre o NOME e não sobre a chave: um nó que o guarde noutro text param conta.
    let mut outro = MotionState::new();
    let n = outro.doc.graph.add_node("motion.drive");
    outro
        .doc
        .graph
        .set_text_param(n, "qualquer_chave", "collider");
    assert!(graph_declares_collider(&outro.doc.graph));
}
