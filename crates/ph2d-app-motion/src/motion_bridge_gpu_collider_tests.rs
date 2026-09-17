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

/// ⭐⭐⭐ **O COLISOR DECLARADO PELA FORMA NUNCA CHEGA AO DISPOSITIVO** — e é isto que torna
/// impossível a divergência que a wave do doc 114 §12 teria aberto.
///
/// Desde 2026-09-17 o `motion.collide` honra o colisor que a corrente DECLARA (caixas orientadas,
/// pela [`ph2d_contact`]) — e o kernel de WGSL dele continua a separar DISCOS de raio uniforme,
/// com `applicable: None`. ⛔⛔ **Sem uma cerca, o MESMO grafo daria uma pilha de caixas na CPU e
/// um borrão de discos na placa, sem erro nenhum** — a espécie de defeito que este repo caça.
///
/// ⚠️⚠️ **A cerca já existia, e o achado foi esse: são DUAS e cobrem as duas rotas.** O
/// [`super::graph_has_live_vector_source`] apanha o `source.shape` (que é quem declara pelo cartão)
/// e o [`graph_declares_collider`] apanha quem escreva a coluna **pelo nome**. ⛔ E a `applicable`
/// do kernel **não podia** resolver isto: ela recebe só os PARAMS do nó, e a declaração é uma
/// propriedade da CORRENTE que chega.
///
/// ⚠️ **O gate mede a cadeia que o artista escreve**, e não `source.shape` sozinho: é a cadeia
/// inteira que o planeador julga.
#[test]
fn a_cadeia_que_declara_colisor_pela_forma_e_recusada_do_dispositivo() {
    use ph2d_nodegraph::graph::Edge;
    let mut m = MotionState::new();
    let g = &mut m.doc.graph;
    let forma = g.add_node("source.shape");
    g.set_param(forma, ph2d_node_motion_shape::param::COLLIDE, 1.0);
    let clone = g.add_node("motion.clone");
    let sep = g.add_node("motion.collide");
    let out = g.add_node("motion.output");
    for (de, para) in [(forma, clone), (clone, sep), (sep, out)] {
        g.connect(Edge {
            from: (de, 0),
            to: (para, 0),
            delayed: false,
        })
        .expect("fio");
    }
    assert!(
        super::graph_has_live_vector_source(&m.doc.graph, &m.registry),
        "a cadeia do `source.shape` tem de ser recusada do dispositivo — sem isso o \
         `motion.collide` separa CAIXAS na CPU e DISCOS na placa, para o mesmo grafo"
    );

    // ⚠️ **O CONTROLO:** a mesma cadeia sem a forma NÃO pode ser recusada por esta razão, senão a
    // cerca seria incondicional e este gate passaria por ela, não pelo que afirma.
    let mut m2 = MotionState::new();
    let g2 = &mut m2.doc.graph;
    let grade = g2.add_node("motion.grid");
    let sep2 = g2.add_node("motion.collide");
    let out2 = g2.add_node("motion.output");
    for (de, para) in [(grade, sep2), (sep2, out2)] {
        g2.connect(Edge {
            from: (de, 0),
            to: (para, 0),
            delayed: false,
        })
        .expect("fio");
    }
    assert!(
        !super::graph_has_live_vector_source(&m2.doc.graph, &m2.registry),
        "o CONTROLO (grelha, sem forma) nao pode ser recusado — a cerca seria incondicional"
    );
    assert!(
        !graph_declares_collider(&m2.doc.graph),
        "e nem pela outra cerca, que le' os nomes das colunas"
    );
}
