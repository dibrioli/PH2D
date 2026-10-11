//! Os gates das recusas de colisor, que recusam pela mesma razão de motor por **rotas
//! diferentes**: o nome escrito num text param (doc 109 W2, [`super::graph_declares_collider`]) e o
//! botão `Collide` de uma peça — a `source.shape` e o `source.object` —, que desde o doc 115 §33
//! arma o passe automático ([`super::peca_declara_colisor`]).

// ⚠️ **Dois níveis, e a separação é a do ficheiro:** as CERCAS vivem no irmão (`super`) e o
// DESPACHO no pai (`super::super`) — ver o cabeçalho do `motion_bridge_gpu_colisor.rs`.
use super::super::{GpuOutcome, cook_gpu};
use super::{RECUSA_COLISOR, RECUSA_PASSE, graph_declares_collider, peca_declara_colisor};
use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::{COLLIDER_BOX_COLUMN, COLLIDER_COLUMN, COLLIDER_OFFSET_COLUMN};
use ph2d_nodegraph::graph::{Edge, NodeId};

/// Um documento `<fonte> → motion.output` com o `Collide` da fonte em `collide` (`None` = sem
/// override, o default do cartão).
fn fonte_na_saida(tipo: &str, collide: Option<f32>) -> (MotionState, NodeId) {
    let mut m = MotionState::new();
    let g = &mut m.doc.graph;
    let fonte = g.add_node(tipo);
    let out = g.add_node("motion.output");
    g.connect(Edge {
        from: (fonte, 0),
        to: (out, 0),
        delayed: false,
    })
    .expect("liga");
    if let Some(v) = collide {
        g.set_param(fonte, "collide", v);
    }
    m.sinks = vec![out];
    (m, fonte)
}

/// ⭐⭐⭐ **A PERGUNTA DO PASSE AUTOMÁTICO: toda peça que declara a arma, e só ela** (doc 115 §33).
///
/// Desde o doc 130 o passe corre também NA PLACA, e a resposta deixou de recusar: arma a cerca do
/// fim do cozimento ([`super::cerca_do_passe`]). A pergunta continua a ser só *«alguma peça
/// declara?»*, pelos DOIS cartões que têm o botão.
///
/// ⚠️ **A tabela inteira, com os controlos:** cada cartão desligado (e sem override) **não** recusa —
/// ⭐ é a frase que a W1 escreveu e que continua a valer: *«um objecto SEM colisor tem de continuar a
/// cozer no dispositivo — senão a cerca derruba toda cena com um Sprite e o §0.0 deixa o caminho
/// lento definir o produto»*. E as varreduras do sink **sozinhas** não armam nada.
#[test]
fn uma_peca_que_declara_arma_o_passe() {
    let casos = [
        // (fonte, collide, recusa?)
        ("source.shape", Some(1.0), true),
        ("source.shape", Some(0.0), false),
        ("source.shape", None, false),
        ("source.object", Some(1.0), true),
        ("source.object", Some(0.0), false),
        ("source.object", None, false),
    ];
    assert_eq!(casos.len(), 6, "dois cartoes x tres estados");
    for (tipo, collide, esperado) in casos {
        let (mut m, _) = fonte_na_saida(tipo, collide);
        assert_eq!(
            peca_declara_colisor(&mut m, 0.0),
            esperado,
            "`{tipo}` com collide={collide:?}"
        );
    }
    // As varreduras do sink sozinhas não armam nada — o Output já não tem interruptor.
    let (mut m, _) = fonte_na_saida("source.object", None);
    let out = m.sinks[0];
    m.doc.graph.set_param(out, "collide_iterations", 64.0);
    assert!(!peca_declara_colisor(&mut m, 0.0));
    // E o chão: um documento vazio.
    assert!(!peca_declara_colisor(&mut MotionState::new(), 0.0));
}

/// ⛔⛔⛔ **O FIO da cerca do passe** (doc 130 §5) — os gates acima chamam a porta, e **cortar o
/// `return` no `cook_gpu` deixa-os todos verdes** (*um gate que chama a função em vez de percorrer a
/// rota afirma que a lei existe, nunca que o produto a usa*). A metade que o CI corre é o despacho
/// lido por `include_str!`: a pergunta ANTES do plano, as varreduras entregues à placa, e a cerca
/// do fim nas DUAS rotas (a inteira e a híbrida). A que percorre a rota é o irmão `#[ignore]`.
#[test]
fn a_cerca_do_passe_esta_de_facto_ligada_ao_cozimento() {
    const PONTE: &str = include_str!("motion_bridge_gpu.rs");
    let (antes, depois) = PONTE
        .split_once("let declara = peca_declara_colisor(motion, target as f64 * fixed_dt);")
        .expect("o `cook_gpu` deixou de PERGUNTAR se alguma peca declara colisor (doc 115 §33)");
    assert!(
        antes.contains("fn cook_gpu("),
        "a pergunta e' do `cook_gpu`"
    );
    let plano = depois
        .find("plano_do_produto(")
        .expect("o cook_gpu planeia");
    let passe = depois
        .find(".set_passe(colisor::varreduras_do_passe(motion));")
        .expect("as varreduras do cartao Output chegam a placa — sem elas o passe nao corre");
    assert!(
        passe < plano,
        "as varreduras entregam-se antes do cozimento"
    );
    let cerca = "if colisor::cerca_do_passe(motion, declara) {\n                return fell(motion, RECUSA_PASSE);";
    assert_eq!(
        depois.matches(cerca).count(),
        2,
        "a cerca do fim tem de SAIR para a CPU nas duas rotas (inteira e hibrida)"
    );
    // ⛔ E as cercas que a §33 matou não voltam: com o passe automático uma declaração tem
    // sempre leitor, e a pergunta «e alguém a lê?» seria uma segunda porta para a mesma lei.
    for morta in ["graph_reads_declared_collider", "cook_publishes_collider"] {
        assert!(
            !PONTE.contains(morta),
            "`{morta}` voltou ao despacho — a lei «uma declaracao lida recusa a placa» tem UMA \
             porta (`peca_declara_colisor`)"
        );
    }
}

/// ⭐⭐⭐ **A ROTA REAL: um objecto com `Collide` FICA na placa** (doc 130). Sem objecto publicado a
/// corrente é vazia — nada a separar dos dois lados —, e a cerca do fim não pode confundir isso
/// com «a placa perdeu o colisor» (ela dispararia em toda cena com um objecto por carregar).
/// Percorre o `cook_gpu` com a placa — o irmão do gate textual.
#[test]
#[ignore = "placa: precisa de adapter de GPU"]
fn a_ponte_separa_na_placa_o_objecto_que_declara_e_so_esse() {
    let Ok(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) else {
        panic!("sem adapter — este gate mede a ponte do device e nao tem versao de CPU");
    };
    let scopes = ph2d_nodegraph::cook::TimeScopes::new();
    let (mut com, _) = fonte_na_saida("source.object", Some(1.0));
    let saida = cook_gpu(&mut com, &gpu, 0, 1.0 / 60.0, &scopes);
    assert_ne!(com.route_said, Some(RECUSA_PASSE), "{saida:?}");
    if matches!(saida, GpuOutcome::Handled) {
        assert_eq!(
            com.gpu_cook.passe_correu(),
            1,
            "a saida vazia conta como vista"
        );
    }
    // ⚠️ O CONTROLO: o mesmo documento sem o botão também não sai por esta razão.
    let (mut sem, _) = fonte_na_saida("source.object", None);
    let _ = cook_gpu(&mut sem, &gpu, 0, 1.0 / 60.0, &scopes);
    assert_ne!(sem.route_said, Some(RECUSA_PASSE));
}

/// ⭐⭐ **O CENSO dos leitores da declaração** — a bandeira do registo não pode ficar por pôr.
///
/// A porta única que lê o colisor declarado é o `ph2d_contact::colisores`; quem lhe chama num
/// GRAFO tem de se registar, senão o diagnóstico (`ColisorMudo`) não distingue uma simulação que vê
/// a forma declarada de uma que não a vê. ⛔ Uma lista escrita na shell envelheceria em silêncio.
///
/// ⚠️ **As duas metades:** a tabela tem de conter TODA crate que chama a porta (varrido da
/// árvore, com piso de população) **e** cada nó dela tem de ter a bandeira. A 1.ª sozinha deixa a
/// bandeira por pôr; a 2.ª sozinha fica verde no dia do quarto leitor.
#[test]
fn todo_leitor_do_colisor_declarado_se_regista() {
    // A tabela: crate → o nó que ela regista, ou `None` com a razão de não ser um nó.
    const TABELA: &[(&str, Option<&str>)] = &[
        ("ph2d-node-motion-collide", Some("motion.collide")),
        ("ph2d-node-sim-collide", Some("sim.collide")),
        // O corpo mole lê o colisor das peças da porta `shape` (doc 124 §6) — só na CPU.
        ("ph2d-node-motion-soft-body", Some("motion.soft_body")),
        // A corda lê o colisor das peças da porta `shape` (doc 125 §1) — só na CPU.
        ("ph2d-node-motion-verlet-rope", Some("motion.verlet_rope")),
        // O `sim.step` lê-o pelo mundo de contacto (doc 121 §9.20, `colisores_locais`).
        ("ph2d-contact-world", Some("sim.step")),
        // A shell: o gizmo do cartão desenha o colisor e as cenas de pilha medem-no. Nenhum
        // deles vive num grafo, logo nenhum pode divergir entre CPU e dispositivo.
        ("ph2d-app-motion", None),
        // O passe na PLACA (doc 130): o kernel lê as MESMAS colunas em WGSL; a porta aparece só no
        // gate de paridade dele, que mede o alcance da fixtura com ela. Não é um nó de grafo.
        ("ph2d-gpu-cook", None),
    ];
    let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/");
    let mut vistas = std::collections::BTreeSet::new();
    let mut ficheiros = 0usize;
    let mut pilha = vec![raiz.to_path_buf()];
    while let Some(dir) = pilha.pop() {
        let Ok(entradas) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in entradas.flatten() {
            let p = e.path();
            if p.is_dir() {
                pilha.push(p);
            } else if p.extension().is_some_and(|x| x == "rs") {
                ficheiros += 1;
                if std::fs::read_to_string(&p).is_ok_and(|t| {
                    t.contains("ph2d_contact::colisores(")
                        || t.contains("ph2d_contact::colisores_locais(")
                }) && let Ok(rel) = p.strip_prefix(raiz)
                    && let Some(c) = rel.components().next()
                {
                    vistas.insert(c.as_os_str().to_string_lossy().into_owned());
                }
            }
        }
    }
    // ⚠️ **O PISO**, nas duas grandezas: uma varredura partida devolve zero e `is_empty()` sobre
    // uma lista vazia é trivialmente verdadeiro — é a falha MUDA que o HOWTO §2.7 nomeia.
    assert!(
        ficheiros > 5_000,
        "a varredura viu {ficheiros} ficheiros .rs — ela deixou de olhar para `crates/`"
    );
    let esperadas: std::collections::BTreeSet<String> =
        TABELA.iter().map(|(c, _)| (*c).to_string()).collect();
    assert_eq!(
        vistas, esperadas,
        "mudou quem chama `ph2d_contact::colisores`. Uma crate NOVA na lista de vistas é um \
         leitor do colisor declarado: se ele for um nó, registe-o com \
         `register_declared_collider_reader` e ponha-o nesta tabela; se não for, ponha-o com \
         `None` e a razão"
    );
    let reg = MotionState::new().registry;
    for (crate_, no) in TABELA {
        if let Some(no) = no {
            assert!(
                reg.reads_declared_collider(ph2d_nodegraph::node::NodeTypeId::of(no)),
                "`{no}` (da `{crate_}`) lê o colisor declarado e NÃO tem a bandeira do registo — \
                 o diagnóstico acusaria de mudo um `Collide` que ele lê"
            );
        }
    }
    // O CONTROLO: a bandeira não é universal — senão ela não distinguiria nada.
    assert!(
        !reg.reads_declared_collider(ph2d_nodegraph::node::NodeTypeId::of("motion.clone")),
        "a bandeira ficou universal e deixou de distinguir quem le' a declaracao"
    );
}

/// ⭐⭐⭐ **A PREMISSA DESTE GATE MUDOU DUAS VEZES, e o diff tem de o mostrar.**
///
/// - até 2026-09-17 (`hoje_nenhum_externo_da_membrana_traz_colisor`): a membrana não declarava nada;
/// - da W4 do doc 115 até 2026-10-11 (`a_declaracao_da_membrana_mora_numa_porta_so`): a membrana
///   declarava a caixa de TODO objecto, por UMA porta;
/// - desde a §33 (a decisão do dono, *«automático é melhor»*): **a declaração é do CARTÃO** — o
///   `Collide` do `source.object` — e a membrana volta a não escrever colisor nenhum. Se ela o
///   escrevesse, todo objecto da cena colidiria sem o artista o pedir, e a cerca do passe não o veria
///   (ela pergunta pelos botões).
///
/// ⚠️ **A régua é textual e é um CENSO** pelas duas formas com que se ESCREVE uma coluna (o
/// identificador da constante, ou um literal com a aspa a abrir) — procurar o VALOR acusaria a
/// prosa, que diz `collider` em inglês.
#[test]
fn a_membrana_nao_escreve_colisor_nenhum() {
    const PORTA_DO_NO: &str = include_str!("../../ph2d-node-source-object/src/colisor.rs");
    const COLISOR: &str = include_str!("motion_bridge_objects_collider.rs");
    const PAI: &str = include_str!("motion_bridge_objects.rs");
    const STREAMS: &str = include_str!("motion_bridge_objects_streams.rs");
    const SHIFT: &str = include_str!("motion_bridge_objects_shift.rs");
    const LOD: &str = include_str!("motion_bridge_objects_lod.rs");
    // O piso: se a varredura deixar de ver o construtor da corrente, ela está a medir o nada.
    assert!(
        PAI.contains(".with(\n            \"texture_id\","),
        "esta régua deixou de encontrar o construtor do externo — ela mudou de sítio e o gate \
         passou a varrer um ficheiro que não é o da membrana"
    );
    let escreve = |texto: &str, valor: &str, ident: &str| {
        ["with(", "set("].iter().any(|porta| {
            texto.contains(&format!("{porta}{ident}"))
                || texto.contains(&format!("{porta}\"{valor}\""))
        })
    };
    // O CONTROLO da própria régua: ela TEM de acusar a porta do nó, que de facto escreve a caixa.
    assert!(
        escreve(PORTA_DO_NO, COLLIDER_BOX_COLUMN, "COLLIDER_BOX_COLUMN"),
        "a régua não reconhece a escrita nem no ficheiro que de facto a faz"
    );
    for (nome, texto) in [
        ("colisor", COLISOR),
        ("pai", PAI),
        ("streams", STREAMS),
        ("shift", SHIFT),
        ("lod", LOD),
    ] {
        for (valor, ident) in [
            (COLLIDER_COLUMN, "COLLIDER_COLUMN"),
            (COLLIDER_BOX_COLUMN, "COLLIDER_BOX_COLUMN"),
            (COLLIDER_OFFSET_COLUMN, "COLLIDER_OFFSET_COLUMN"),
        ] {
            assert!(
                !escreve(texto, valor, ident),
                "o ficheiro `{nome}` da membrana escreve `{valor}` — a declaração é do `Collide` \
                 do `source.object` (doc 115 §33), e um objecto não colide sem o artista o pedir"
            );
        }
    }
    // ⛔ E a porta do nó continua a não declarar um RAIO: um objecto é um quadro.
    assert!(
        !escreve(PORTA_DO_NO, COLLIDER_COLUMN, "COLLIDER_COLUMN"),
        "a porta passou a declarar um RAIO — um objecto é um quadro"
    );
}

/// ⭐⭐ **A LIGAÇÃO: a ponte do produto recusa de facto o documento** — o gate abaixo prova a
/// pergunta; este prova que ela é feita no caminho que cozinha.
///
/// ⚠️ Com o CONTROLO: o mesmo grafo sem o nome não pode sair com esta razão, senão a recusa seria
/// incondicional e o gate passaria por ela.
#[test]
#[ignore = "placa: precisa de adapter de GPU"]
fn the_bridge_cooks_a_document_that_names_the_collider_on_the_cpu() {
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
/// ⚠️⚠️ **A cerca já existia, e o achado foi esse: são DUAS e cobrem as duas rotas.** A do
/// `source.shape` (que é quem declara pelo cartão) e o [`graph_declares_collider`], que apanha quem
/// escreva a coluna **pelo nome**.
///
/// ⛔ **A do cartão MUDOU de pergunta duas vezes:** no doc 121 W3 era *«a forma declara E o grafo
/// lê a declaração?»*; desde o doc 115 §33 o sink lê toda declaração sozinho, e a pergunta é só
/// *«alguma peça declara?»* ([`peca_declara_colisor`]). ⛔ E a `applicable` do kernel **não podia**
/// resolver isto: ela recebe só os PARAMS do nó, e a declaração é da CORRENTE que chega.
///
/// ⚠️ **O gate mede a cadeia que o artista escreve**, e não `source.shape` sozinho: é a cadeia
/// inteira que o planeador julga.
#[test]
fn a_cadeia_que_declara_colisor_pela_forma_e_recusada_do_dispositivo() {
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
        peca_declara_colisor(&mut m, 0.0),
        "a cadeia do `source.shape` com `Collide` tem de ser recusada do dispositivo — sem isso \
         o `motion.collide` separa CAIXAS na CPU e DISCOS na placa, para o mesmo grafo"
    );
    // ⚠️ **O CONTROLO do cartão:** a mesma forma SEM `Collide` não declara — senão a cerca
    // recusaria toda forma seguida de um `motion.collide`, que é a cerca do TIPO outra vez.
    m.doc
        .graph
        .set_param(forma, ph2d_node_motion_shape::param::COLLIDE, 0.0);
    assert!(!peca_declara_colisor(&mut m, 0.0));

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
        !peca_declara_colisor(&mut m2, 0.0),
        "o CONTROLO (grelha, sem forma) nao pode ser recusado — a cerca seria incondicional"
    );
    assert!(
        !graph_declares_collider(&m2.doc.graph),
        "e nem pela outra cerca, que le' os nomes das colunas"
    );
}
