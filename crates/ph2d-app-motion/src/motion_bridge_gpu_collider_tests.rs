//! Os gates das DUAS recusas de colisor, que recusam pela mesma razão de motor por **rotas
//! diferentes**: o nome escrito num text param (doc 109 W2, [`super::graph_declares_collider`]) e
//! um EXTERNO que traz a forma do objecto (doc 115 W1, [`super::cook_publishes_collider`]).

// ⚠️ **Dois níveis, e a separação é a do ficheiro:** as CERCAS vivem no irmão (`super`) e o
// DESPACHO no pai (`super::super`) — ver o cabeçalho do `motion_bridge_gpu_colisor.rs`.
use super::super::{GpuOutcome, cook_gpu, graph_has_live_vector_source};
use super::{RECUSA_COLISOR, cook_publishes_collider, graph_declares_collider};
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
    // O CONTROLO: as colunas de APARÊNCIA não recusam nada.
    //
    // ⚠️ **A frase que estava aqui continua verdadeira e mudou de dono** (doc 115 W4): ela dizia
    // *«um objecto SEM colisor tem de continuar a cozer no dispositivo — senão esta cerca derruba
    // toda cena com um Sprite e o §0.0 deixa o caminho lento definir o produto»*. Desde a W4 **todo**
    // objecto traz colisor, logo quem garante aquilo já não é esta metade: é a do CONSUMIDOR
    // (`a_cerca_dos_externos_precisa_das_duas_metades`). Aqui fica o que a metade de baixo
    // continua a afirmar sozinha — que ela lê as colunas do colisor e não as da aparência.
    assert!(
        !cook_publishes_collider(&cozedor_com(&[
            "size",
            "rot",
            "tint",
            "uv_rect",
            "texture_id"
        ])),
        "a metade do EXTERNO passou a acusar colunas de aparência — ela deixou de distinguir \
         seja o que for"
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
        .split_once("if cook_publishes_collider(&motion.pump.cook)")
        .map(|(_, resto)| resto)
        .expect(
            "o `cook_gpu` deixou de PERGUNTAR à cerca dos externos — a rota da membrana voltou a \
             atravessar a fronteira sem cerca (doc 115 W1)",
        );
    let ate_ao_fecho = despacho.split_once('}').map(|(x, _)| x).unwrap_or("");
    // ⭐ **A metade do CONSUMIDOR tem de estar no MESMO `if`** (doc 115 W4): sem ela a cerca
    // recusa toda cena com um Sprite, porque desde a W4 todo objecto declara a forma dele.
    assert!(
        ate_ao_fecho.contains("&& graph_reads_declared_collider(&motion.doc.graph"),
        "a cerca voltou a perguntar SÓ pelo externo — e desde a W4 isso recusa toda cena com um \
         objecto, que é o §0.0 ao contrário: {ate_ao_fecho:?}"
    );
    assert!(
        ate_ao_fecho.contains("return fell(motion, RECUSA_COLISOR_EXTERNO)"),
        "a cerca é perguntada e a resposta não SAI para a CPU — é a mutação que sobreviveu aos \
         dois gates acima: {ate_ao_fecho:?}"
    );
}

/// ⭐⭐⭐ **AS DUAS METADES DA CERCA, e cada uma sozinha tem a cura errada** (doc 115 W4).
///
/// - só o EXTERNO: recusa toda cena com um objecto — o `50,9×` do doc 98 aplicado a cenas que não
///   pediram separação nenhuma, que é o §0.0 ao contrário;
/// - só o CONSUMIDOR: um `motion.collide` sobre uma nuvem sem colisor declarado separa discos nos
///   dois lados e **concorda**; recusá-lo seria lento por nada.
///
/// ⚠️ A tabela é a de verdade inteira (`4` células) porque uma cerca de duas metades tem quatro
/// respostas e três delas são *«cozinha no dispositivo»*.
#[test]
fn a_cerca_dos_externos_precisa_das_duas_metades() {
    let com_leitor = |tipo: &str| {
        let mut m = MotionState::new();
        m.doc.graph.add_node(tipo);
        m
    };
    let casos = [
        // (nó no grafo, externo com colisor, recusa?)
        ("motion.collide", true, true),
        ("motion.collide", false, false),
        ("motion.clone", true, false),
        ("motion.clone", false, false),
    ];
    assert_eq!(
        casos.len(),
        4,
        "a tabela de verdade de duas metades tem 4 celulas"
    );
    for (tipo, traz, esperado) in casos {
        let m = com_leitor(tipo);
        let cook = if traz {
            cozedor_com(&[COLLIDER_BOX_COLUMN])
        } else {
            cozedor_com(&["size", "tint"])
        };
        let recusa = cook_publishes_collider(&cook)
            && super::graph_reads_declared_collider(&m.doc.graph, &m.registry);
        assert_eq!(
            recusa, esperado,
            "no' `{tipo}` com externo-traz-colisor={traz}: a cerca respondeu {recusa}"
        );
    }
}

/// ⭐⭐⭐ **A TERCEIRA CERCA: o passe ARMADO cozinha na CPU** (doc 115 W5).
///
/// Ela é a pergunta que a §10.3 daquele doc prescreveu por escrito antes de haver um passe — *«a
/// pergunta certa deixa de ser «alguém declara?» e passa a ser «a separação está ARMADA?»»* — e a
/// razão é de motor: o passe corre no fim do cozimento **da CPU**, e na rota do dispositivo não
/// existe corrente de CPU nenhuma para separar.
///
/// ⚠️ **As três metades:** armado recusa · desarmado **não** recusa (senão a cerca seria
/// incondicional e toda cena do produto caía) · e o número de varreduras **sozinho** não arma nada,
/// que é o que impede uma leitura que ignore o interruptor de passar aqui.
#[test]
fn o_passe_armado_recusa_o_dispositivo() {
    let monta = |collide: f32, varreduras: f32| {
        let mut m = MotionState::new();
        let sink = m.doc.graph.add_node("motion.output");
        m.doc.graph.set_param(sink, "collide", collide);
        m.doc
            .graph
            .set_param(sink, "collide_iterations", varreduras);
        m.sinks = vec![sink];
        m
    };
    let armado = monta(1.0, 8.0);
    assert!(
        super::sink_arma_a_separacao(&armado.doc.graph, &armado.sinks),
        "com o interruptor ligado a cerca tem de disparar"
    );
    let desarmado = monta(0.0, 8.0);
    assert!(
        !super::sink_arma_a_separacao(&desarmado.doc.graph, &desarmado.sinks),
        "desarmado NAO pode recusar — senao toda cena do produto cozinha na CPU"
    );
    let so_o_numero = monta(0.0, 64.0);
    assert!(
        !super::sink_arma_a_separacao(&so_o_numero.doc.graph, &so_o_numero.sinks),
        "o numero de varreduras sozinho nao arma nada"
    );
    // E o chão: um documento sem sink nenhum.
    let vazio = MotionState::new();
    assert!(!super::sink_arma_a_separacao(
        &vazio.doc.graph,
        &vazio.sinks
    ));
}

/// ⛔⛔⛔ **O FIO da terceira cerca** — irmão do `a_cerca_dos_externos_esta_de_facto_ligada_ao_cozimento`
/// e pela mesma razão: os gates acima chamam a porta, e **cortar o `return` no `cook_gpu` deixa-os
/// todos verdes**. A metade que o CI corre é o despacho lido por `include_str!`.
#[test]
fn a_cerca_do_passe_esta_de_facto_ligada_ao_cozimento() {
    const PONTE: &str = include_str!("motion_bridge_gpu.rs");
    let despacho = PONTE
        .split_once("if sink_arma_a_separacao(&motion.doc.graph, &motion.sinks)")
        .map(|(_, resto)| resto)
        .expect(
            "o `cook_gpu` deixou de PERGUNTAR se o passe esta' armado — a mesma cena sairia \
             separada na CPU e sobreposta na placa (doc 115 W5)",
        );
    let ate_ao_fecho = despacho.split_once('}').map(|(x, _)| x).unwrap_or("");
    assert!(
        ate_ao_fecho.contains("return fell(motion, RECUSA_PASSE)"),
        "a cerca e' perguntada e a resposta nao SAI para a CPU: {ate_ao_fecho:?}"
    );
}

/// ⭐⭐ **O CENSO dos leitores da declaração** — a bandeira do registo não pode ficar por pôr.
///
/// A porta única que lê o colisor declarado é o `ph2d_contact::colisores`; quem lhe chama num
/// GRAFO tem de se registar, senão a cerca acima não o vê e o documento dele cozinha no
/// dispositivo — com o kernel de discos — enquanto a CPU honra caixas. ⛔ Uma lista escrita na
/// shell envelheceria em silêncio e do lado errado.
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
        ("ph2d-node-sim-step", Some("sim.step")),
        // A shell: o gizmo do cartão desenha o colisor e as cenas de pilha medem-no. Nenhum
        // deles vive num grafo, logo nenhum pode divergir entre CPU e dispositivo.
        ("ph2d-app-motion", None),
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
                if std::fs::read_to_string(&p).is_ok_and(|t| t.contains("ph2d_contact::colisores("))
                    && let Ok(rel) = p.strip_prefix(raiz)
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
                 a cerca do `cook_gpu` não o vê, e o documento dele coze no dispositivo com o \
                 kernel de discos enquanto a CPU honra caixas"
            );
        }
    }
    // O CONTROLO: a bandeira não é universal — senão ela não distinguiria nada.
    assert!(
        !reg.reads_declared_collider(ph2d_nodegraph::node::NodeTypeId::of("motion.clone")),
        "a bandeira ficou universal e a cerca voltou a recusar toda cena com um objecto"
    );
}

/// ⭐⭐⭐ **A PREMISSA DESTE GATE MORREU EM 2026-09-17, e ele está aqui reescrito e não apagado.**
///
/// ## O que ele dizia, e é o que o diff tem de mostrar
///
/// Ele chamava-se `hoje_nenhum_externo_da_membrana_traz_colisor` e varria o ficheiro da membrana
/// por `include_str!` a exigir que **nenhuma** das três colunas lá aparecesse — *«a cerca nasce
/// INERTE, e este gate é quem o afirma»*. A W4 pôs a membrana a declarar a forma de todo objecto
/// (`motion_bridge_objects_collider.rs`) e ele ficou vermelho **no dia previsto e pelo motivo
/// previsto**; a própria mensagem dele mandava confirmar que a W2 tinha aterrado (aterrou, por
/// recusa MEDIDA — doc 115 §9.5) e reescrevê-lo em vez de o apagar.
///
/// ## O que ele afirma agora, e é a metade que passou a ser frágil
///
/// A rota abriu, logo *«ninguém declara»* deixou de ser verdade e a pergunta útil mudou de sítio:
/// **a declaração tem de ser CENTRALIZADA numa porta.** Enquanto ela viver num só sítio, a lei
/// (`size / 2` em mundo, quadrado unitário em geometria) é gateável de uma vez; espalhada por
/// `.with(COLLIDER_BOX_COLUMN, …)` em cada construtor, o quarto médio herda-a errada em silêncio.
///
/// ⚠️ **A régua é textual de propósito e é um CENSO:** a ESCRITA da coluna só pode aparecer no
/// ficheiro da porta. Um `.with` novo noutro ficheiro da membrana reprova aqui.
///
/// ⛔⛔ **E a 1.ª redacção dela varria a PROSA:** `COLLIDER_COLUMN` é literalmente `"collider"`, e
/// procurar o VALOR da constante acusa qualquer comentário que use a palavra — o gate reprovou
/// sobre o ficheiro pai, que só a diz em inglês. ⇒ a varredura é pelas duas formas com que se
/// **escreve** uma coluna (o identificador da constante, ou um literal com a aspa a abrir), que é
/// o que separa código de prosa. *A mesma família do censo textual que não separa os dois e mente
/// nos dois sentidos.*
#[test]
fn a_declaracao_da_membrana_mora_numa_porta_so() {
    // Os ficheiros da membrana que constroem correntes de aparência.
    const PORTA: &str = include_str!("motion_bridge_objects_collider.rs");
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
    assert!(
        PORTA.contains(COLLIDER_BOX_COLUMN),
        "a PORTA deixou de declarar a caixa — a W4 do doc 115 foi desfeita, e todo objecto da \
         cena voltou a não ter forma nenhuma"
    );
    // ⚠️ **A régua pergunta exactamente o que o gate AFIRMA: a coluna é ESCRITA aqui?** — as duas
    // portas de escrita de um `Stream` (`with` / `set`) × as duas formas de a nomear (a constante
    // ou um literal). ⛔ Procurar só o nome acusa um link de doc — foi a 2.ª reprovação desta
    // régua, depois de a 1.ª acusar a palavra em prosa inglesa.
    let escreve = |texto: &str, valor: &str, ident: &str| {
        ["with(", "set("].iter().any(|porta| {
            texto.contains(&format!("{porta}{ident}"))
                || texto.contains(&format!("{porta}\"{valor}\""))
        })
    };
    let colunas = [
        (COLLIDER_COLUMN, "COLLIDER_COLUMN"),
        (COLLIDER_BOX_COLUMN, "COLLIDER_BOX_COLUMN"),
        (COLLIDER_OFFSET_COLUMN, "COLLIDER_OFFSET_COLUMN"),
    ];
    // O CONTROLO da própria régua: ela TEM de acusar a porta, senão não acusa nada.
    assert!(
        escreve(PORTA, COLLIDER_BOX_COLUMN, "COLLIDER_BOX_COLUMN"),
        "a régua não reconhece a escrita nem no ficheiro que de facto a faz"
    );
    for (nome, texto) in [
        ("pai", PAI),
        ("streams", STREAMS),
        ("shift", SHIFT),
        ("lod", LOD),
    ] {
        for (valor, ident) in colunas {
            assert!(
                !escreve(texto, valor, ident),
                "o ficheiro `{nome}` da membrana escreve `{valor}` — a declaração saiu da porta \
                 (`motion_bridge_objects_collider.rs`) e a lei passou a estar escrita em dois \
                 sítios, que é onde o quarto médio a herda errada"
            );
        }
    }
    // ⛔ E o RAIO continua a não ser declarado por ninguém: um objecto é um quadro, e a porta da
    // leitura já declara que a caixa ganha. (A metade que o `a_membrana_declara_a_forma_e_…`
    // mede no valor; esta mede-a no TEXTO, que é o que apanha um `.with` novo antes de correr.)
    assert!(
        !escreve(PORTA, COLLIDER_COLUMN, "COLLIDER_COLUMN"),
        "a porta passou a declarar um RAIO — um objecto é um quadro"
    );
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
        graph_has_live_vector_source(&m.doc.graph, &m.registry),
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
        !graph_has_live_vector_source(&m2.doc.graph, &m2.registry),
        "o CONTROLO (grelha, sem forma) nao pode ser recusado — a cerca seria incondicional"
    );
    assert!(
        !graph_declares_collider(&m2.doc.graph),
        "e nem pela outra cerca, que le' os nomes das colunas"
    );
}
