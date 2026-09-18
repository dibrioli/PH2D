//! Os gates das DUAS recusas de colisor, que recusam pela mesma razão de motor por **rotas
//! diferentes**: o nome escrito num text param (doc 109 W2, [`super::graph_declares_collider`]) e
//! um EXTERNO que traz a forma do objecto (doc 115 W1, [`super::cook_publishes_collider`]).

use super::{
    GpuOutcome, RECUSA_COLISOR, cook_gpu, cook_publishes_collider, graph_declares_collider,
};
use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::{
    COLLIDER_BOX_COLUMN, COLLIDER_COLUMN, COLLIDER_OFFSET_COLUMN, Column, Stream,
};

/// Um cozedor com UM externo que traz `colunas`.
fn cozedor_com(colunas: &[&str]) -> ph2d_nodegraph::cook::Cook {
    let mut cook = ph2d_nodegraph::cook::Cook::new();
    let mut s = Stream::new(1).with("P", Column::Vec2(vec![[0.0, 0.0]]));
    for c in colunas {
        s = s.with(*c, Column::Vec2(vec![[0.5, 0.5]]));
    }
    cook.set_external("Bola".to_string(), s);
    cook
}

/// ⭐⭐⭐ **AS TRÊS COLUNAS, UMA A UMA** — e o CONTROLO, que é o que impede a cerca de ser
/// incondicional.
///
/// ⚠️ **Uma a uma e não juntas**, de propósito: com as três no mesmo externo, apagar duas da lista
/// da cerca deixaria o gate verde. *Uma cerca que lista N nomes precisa de N casos*, que é a mesma
/// lei do censo com piso — aqui o piso é a população de colunas.
#[test]
fn cada_uma_das_tres_colunas_de_colisor_recusa_o_externo() {
    for c in [COLLIDER_COLUMN, COLLIDER_BOX_COLUMN, COLLIDER_OFFSET_COLUMN] {
        assert!(
            cook_publishes_collider(&cozedor_com(&[c])),
            "um externo com `{c}` tem de recusar o dispositivo"
        );
    }
    // O CONTROLO: as colunas que a membrana de facto publica hoje não recusam nada.
    assert!(
        !cook_publishes_collider(&cozedor_com(&[
            "size",
            "rot",
            "tint",
            "uv_rect",
            "texture_id"
        ])),
        "um objecto SEM colisor tem de continuar a cozer no dispositivo — senão esta cerca \
         derruba toda cena com um Sprite e o §0.0 deixa o caminho lento definir o produto"
    );
    // E um cozedor sem externo nenhum — o chão.
    assert!(!cook_publishes_collider(&ph2d_nodegraph::cook::Cook::new()));
}

/// ⛔⛔⛔ **O FIO, e não a porta — este gate nasceu de uma MUTAÇÃO QUE SOBREVIVEU.**
///
/// Os dois gates acima chamam a [`super::cook_publishes_collider`] **directamente**. Cortado o
/// `return` no `cook_gpu` (a cerca passa a ser perguntada e a resposta deitada fora), os `1 157`
/// testes desta crate ficaram **VERDES** — *um gate que chama a função em vez de percorrer a rota
/// afirma que a lei existe, nunca que o produto a usa*. É a quinta vez que esta casa paga a forma.
///
/// ⚠️ **A rota REAL precisa de um adapter** (o irmão que a percorre é `#[ignore]`, logo o CI nunca
/// o corre — *skip gracioso não é verde*). A metade que o CI corre é esta: o despacho lido por
/// `include_str!`, que **deixa de compilar** se o ficheiro mudar de sítio e reprova alto se alguém
/// apagar a ligação.
///
/// ⭐ E ele exige as DUAS metades da ligação — a pergunta **e** a saída nomeada —, senão um
/// `if … { }` vazio passaria.
#[test]
fn a_cerca_dos_externos_esta_de_facto_ligada_ao_cozimento() {
    const PONTE: &str = include_str!("motion_bridge_gpu.rs");
    let despacho = PONTE
        .split_once("if cook_publishes_collider(&motion.pump.cook) {")
        .map(|(_, resto)| resto)
        .expect(
            "o `cook_gpu` deixou de PERGUNTAR à cerca dos externos — a rota da membrana voltou a \
             atravessar a fronteira sem cerca (doc 115 W1)",
        );
    let ate_ao_fecho = despacho.split_once('}').map(|(x, _)| x).unwrap_or("");
    assert!(
        ate_ao_fecho.contains("return fell(motion, RECUSA_COLISOR_EXTERNO)"),
        "a cerca é perguntada e a resposta não SAI para a CPU — é a mutação que sobreviveu aos \
         dois gates acima: {ate_ao_fecho:?}"
    );
}

/// ⚠️⚠️ **A CERCA NASCE INERTE, e este gate é quem o afirma** — é ele que torna honesta a frase
/// *«este commit não muda um bit do que o artista vê»*.
///
/// A régua é o FICHEIRO da membrana, por `include_str!`: ela publica hoje `P · size · rot · tint ·
/// uv_rect · texture_id · geometry_id` e **nenhuma coluna de colisor**. ⛔ Uma lista escrita à mão
/// aqui seria a segunda cópia daquele conjunto, e divergia no dia em que ele crescesse — que é
/// exactamente o dia que este gate existe para apanhar.
///
/// ⭐ **E ele VAI ficar vermelho, de propósito:** no dia em que a W3 do doc 115 puser a membrana a
/// publicar a forma do objecto. A cura NÃO é apagá-lo — é reescrevê-lo com a morte da premissa
/// visível no diff, depois de confirmar que a W2 (a caixa no dispositivo, ou a recusa medida dela)
/// aterrou. *Sem isto, a rota abre-se e ninguém repara.*
#[test]
fn hoje_nenhum_externo_da_membrana_traz_colisor() {
    const MEMBRANA: &str = include_str!("motion_bridge_objects.rs");
    // O piso: se a varredura deixar de ver o construtor da corrente, ela está a medir o nada.
    assert!(
        MEMBRANA.contains(".with(\n            \"texture_id\","),
        "esta régua deixou de encontrar o construtor do externo — ela mudou de sítio e o gate \
         passou a varrer um ficheiro que não é o da membrana"
    );
    for c in [COLLIDER_COLUMN, COLLIDER_BOX_COLUMN, COLLIDER_OFFSET_COLUMN] {
        assert!(
            !MEMBRANA.contains(c),
            "a membrana passou a publicar `{c}` — a rota de externos ABRIU (doc 115 W3). \
             Confirme que a W2 aterrou (a caixa no dispositivo, ou a recusa MEDIDA dela) e \
             reescreva este gate com a morte da premissa no diff, em vez de o apagar"
        );
    }
}

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
