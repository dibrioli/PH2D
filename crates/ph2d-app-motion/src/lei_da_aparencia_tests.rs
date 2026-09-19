//! Os portões da [`super`] — a LEI (*uma corrente que não veio de uma forma não vira pixel*) e o
//! AVISO que a torna legível ao artista (ordem do dono, 2026-09-19).

use crate::motion_state::MotionState;
use ph2d_nodegraph::port::Domain;

/// Os nós que o registo marcou como fontes de posições, por nome e ordenados.
fn marcados(m: &MotionState) -> Vec<&'static str> {
    let mut v: Vec<_> = m
        .registry
        .manifests()
        .filter(|man| m.registry.so_posicoes(man.id))
        .map(|man| man.name)
        .collect();
    v.sort_unstable();
    v
}

/// ⭐⭐⭐ **O AVISO NOMEIA EXACTAMENTE QUEM SÓ ENTREGA POSIÇÕES** — a população é DERIVADA do
/// manifesto, e este gate volta a derivá-la aqui, à mão, contra a que o registo guarda.
///
/// ⚠️ **A régua não pode ser a mesma função**, senão ela concorda consigo mesma: o gate escreve
/// o predicado outra vez (*emite instâncias · não recebe instâncias*) e compara os dois
/// conjuntos. É a lei que a casa cobra de todo censo derivado — *uma lista escrita à mão ficaria
/// muda no primeiro nó novo*, e é por isso que ninguém escreve a lista.
///
/// **Mutação que deve sangrar:** inverter o `!` do `inputs` na
/// [`ph2d_node_registry::NodeRegistry::marca_as_fontes_de_posicoes`] — o conjunto passa a ser o
/// dos nós de PASSAGEM, e o aviso aparece em todo `motion.move` do grafo.
#[test]
fn o_aviso_nomeia_exactamente_as_fontes_de_posicoes() {
    use ph2d_nodegraph::port::Dim;
    let m = MotionState::new();
    let recebe = |p: &ph2d_nodegraph::node::PortSpec| p.ty.domain == Domain::Instances;
    let emite_posicoes = |p: &ph2d_nodegraph::node::PortSpec| {
        p.ty.domain == Domain::Instances && p.ty.dim == Dim::Vec2
    };
    let mut esperado: Vec<&'static str> = m
        .registry
        .manifests()
        .filter(|man| {
            man.outputs.iter().any(emite_posicoes)
                && !man.inputs.iter().any(recebe)
                && !m.registry.is_object_source(man.id)
                && !m.registry.is_live_vector_source(man.id)
        })
        .map(|man| man.name)
        .collect();
    esperado.sort_unstable();
    assert_eq!(
        marcados(&m),
        esperado,
        "o registo tem de marcar EXACTAMENTE quem emite instancias e nao recebe nenhuma"
    );
}

/// ⭐⭐ **O PISO DE POPULAÇÃO, e ele é metade do gate acima.**
///
/// ⚠️ Sem ele, o irmão fica **trivialmente verde** no dia em que a marcação varrer zero nós —
/// dois conjuntos vazios são iguais. É o modo de falha MUDO que o CLAUDE.md §5.0 nomeia para os
/// censos que passam a varrer nada, e a cura é sempre a mesma: um piso dentro do próprio gate.
///
/// O número sai da medição: a regra das TRÊS cláusulas marca **9** nós (das `14` que a sonda
/// `quem_e_como_o_grid` conta com a regra larga, saem `value.number` e `debug.const` por serem
/// escalares e as três origens de aparência por serem a cura).
/// ⛔ **Não é um teto:** uma fonte nova entra sozinha, e o gate não tem porque a barrar.
#[test]
fn a_populacao_do_aviso_nao_pode_esvaziar_se() {
    const PISO: usize = 6;
    let m = MotionState::new();
    let n = marcados(&m).len();
    assert!(
        n >= PISO,
        "so {n} fontes de posicoes marcadas (piso {PISO}) -- a derivacao varreu quase nada, e \
         dois conjuntos vazios sao iguais: o gate irmao ficaria verde a afirmar nada"
    );
}

/// ⭐⭐⭐ **OS DOIS LADOS NOMEADOS** — o gate que fala a língua do report do dono.
///
/// ⚠️ **A metade NEGATIVA vale metade:** o `motion.duplicator` é a CURA que a frase nomeia, e o
/// `motion.move` é passagem — se algum deles ganhasse o aviso, a frase passaria a acender no nó
/// que a resolve e em todo transformador do grafo. *Um aviso que soa sempre é ruído.*
#[test]
fn a_grelha_ganha_o_aviso_e_a_passagem_nao() {
    let m = MotionState::new();
    let marcou = |t: &str| {
        m.registry
            .manifests()
            .find(|man| man.name == t)
            .map(|man| m.registry.so_posicoes(man.id))
    };
    assert_eq!(
        marcou("motion.grid"),
        Some(true),
        "a `motion.grid` e' a fonte do proprio report do dono"
    );
    for passagem in ["motion.move", "motion.scale", "motion.duplicator"] {
        assert_eq!(
            marcou(passagem),
            Some(false),
            "`{passagem}` recebe uma corrente de instancias -- ela nao e' uma fonte de posicoes"
        );
    }
    // ⛔⛔ **A cláusula que a MEDIÇÃO comprou.** Estes três emitem posições e não recebem
    // instâncias — passam nas duas primeiras cláusulas — e são **exactamente o que o aviso manda
    // ir buscar**. Marcá-los poria a frase *«precisa de um Duplicator + uma forma»* no cartão do
    // nó que É a forma, e ela viraria mentira no clique seguinte (no valor de fábrica eles ainda
    // não trazem aparência, o que é o que torna este erro tão fácil de não ver).
    for origem in ["source.object", "source.shape", "source.text"] {
        assert_eq!(
            marcou(origem),
            Some(false),
            "`{origem}` E' uma origem de aparencia -- o aviso no cartao dele seria uma mentira"
        );
    }
    // ⛔ E os NÚMEROS por elemento: eles emitem `Instances` mas em `Scalar`, e um aviso sobre
    // duplicadores num nó de valor não quer dizer nada.
    for valor in ["value.number", "debug.const"] {
        assert_eq!(
            marcou(valor),
            Some(false),
            "`{valor}` emite numeros por elemento, nao posicoes"
        );
    }
}

/// ⭐⭐⭐ **A LEI CHEGA AO SISTEMA DE ALERTA** — a pergunta que o CLAUDE.md §5.0 diz que nenhum
/// instrumento desta casa faz: *o valor chega ao consumidor?*
///
/// ⚠️⚠️ **Este gate substituiu um irmão que media OUTRA costura, e a troca é ordem do dono**
/// (2026-09-19): *«o módulo tem um sistema de alerta. não era para colocar a mensagem no próprio
/// nó»*. O que existia era o `a_bandeira_chega_ao_cartao` — ele media a bandeira a viajar do
/// registo para o `GraphNodeView`, e essa costura **deixou de existir** (a nota permanente saiu
/// do cartão, e com ela o campo). *Um gate cujo sujeito foi apagado não se afrouxa: substitui-se
/// pelo que mede a costura NOVA.*
///
/// **Mutação que deve sangrar:** cravar `false` no `reg.so_posicoes(ty)` do `diagnose`, ou
/// apagar o `register_veste_as_posicoes` do `motion.duplicator` (a metade negativa).
#[test]
fn a_lei_chega_ao_sistema_de_alerta() {
    use ph2d_motion_diagnose::{Deficit, diagnose};
    use ph2d_nodegraph::graph::{Edge, Graph};
    let m = MotionState::new();
    let liga = |g: &mut Graph, a: ph2d_nodegraph::graph::NodeId, b, porta| {
        g.connect(Edge {
            from: (a, 0),
            to: (b, porta),
            delayed: false,
        })
        .expect("liga");
    };
    let acusa = |g: &Graph, n: ph2d_nodegraph::graph::NodeId| {
        diagnose(g, &m.registry)
            .iter()
            .any(|d| d.node == n && d.deficit == Deficit::SemQuemVista)
    };

    // (a) A grelha sozinha, a desenhar directo no sink: o ecrã mostra MARCAS.
    let mut g = Graph::new();
    let grelha = g.add_node("motion.grid");
    let saida = g.add_node("motion.output");
    liga(&mut g, grelha, saida, 0);
    assert!(
        acusa(&g, grelha),
        "uma grelha que desenha directo no sink tem de acusar -- e' o report do dono a letra"
    );

    // (b) A MESMA grelha com um duplicador vestido: calada.
    //
    // ⚠️ **A porta `points` é a `1`** (o manifesto do duplicador declara `shape` na `0`), e a
    // forma tem de estar LIGADA: sem ela o duplicador acusa `MissingInput("shape")` — nele, que
    // é onde a cura mora — e a grelha continua calada na mesma, porque o que ela pergunta é se
    // há alguém a jusante que veste, não se esse alguém já tem a forma.
    let mut g = Graph::new();
    let grelha = g.add_node("motion.grid");
    let dup = g.add_node("motion.duplicator");
    let forma = g.add_node("source.shape");
    let saida = g.add_node("motion.output");
    liga(&mut g, forma, dup, 0);
    liga(&mut g, grelha, dup, 1);
    liga(&mut g, dup, saida, 0);
    assert!(
        !acusa(&g, grelha),
        "com um duplicador vestido a jusante a grelha nao tem do que se queixar"
    );

    // (c) ⭐⭐ **A JUNÇÃO — a metade que um passeio só para a frente nunca veria.** A arte entra
    // por um IRMÃO da grelha, não por um descendente dela: sem o fecho a partir das fontes, este
    // caso acusava um grafo perfeitamente certo.
    let mut g = Graph::new();
    let grelha = g.add_node("motion.grid");
    let objecto = g.add_node("source.object");
    let juncao = g.add_node("motion.merge");
    let saida = g.add_node("motion.output");
    liga(&mut g, grelha, juncao, 0);
    liga(&mut g, objecto, juncao, 1);
    liga(&mut g, juncao, saida, 0);
    assert!(
        !acusa(&g, grelha),
        "a arte entra pelo IRMAO: a corrente fundida desenha, logo a grelha esta' certa"
    );

    // (d) ⭐⭐⭐ **O DUPLICADOR SEM FORMA — a metade que a marca `veste_as_posicoes` existe para
    // comprar, e que uma MUTAÇÃO SOBREVIVENTE nomeou.** Sem a declaração do duplicador, o fecho
    // do (b) ainda funciona (a arte vem da `source.shape`, que é uma fonte declarada) e o gate
    // ficava verde com a marca apagada — *o corpus não continha o caso em que ela decide*.
    //
    // ⚠️ **Aqui não há fonte de arte nenhuma no grafo**, e é exactamente onde a pergunta muda:
    // sem a marca a grelha TAMBÉM acusaria, e o artista leria dois problemas onde há um. Quem
    // tem de receber a forma é o duplicador, e é ele que o diz.
    let mut g = Graph::new();
    let grelha = g.add_node("motion.grid");
    let dup = g.add_node("motion.duplicator");
    let saida = g.add_node("motion.output");
    liga(&mut g, grelha, dup, 1);
    liga(&mut g, dup, saida, 0);
    assert!(
        !acusa(&g, grelha),
        "com um duplicador a jusante a grelha cala-se, mesmo que a FORMA dele ainda falte"
    );
    assert!(
        diagnose(&g, &m.registry)
            .iter()
            .any(|d| d.node == dup && d.deficit == Deficit::MissingInput("shape")),
        "e quem acusa e' o duplicador, que e' onde a cura mora"
    );
}

/// ⛔⛔ **A LEI DO DONO É CONFIGURAÇÃO, E CARREGAR UM FICHEIRO NÃO LHE PODE TOCAR.**
///
/// ⚠️ **Este gate nasceu de uma mutação SOBREVIVENTE** (2026-09-19): apagar as três linhas que
/// levam a lei através do [`MotionState::install`] deixava os seis gates desta família VERDES,
/// porque **todos** constroem o estado com `MotionState::new()` e nunca carregam nada.
///
/// O mecanismo é o que torna isto caro de ver: o `install` **substitui a bomba de propósito**
/// (ela guarda os flocos que estão no ar), e a lei viaja dentro dela. ⇒ um `load` repunha o valor
/// de omissão — no produto isso é inofensivo enquanto a omissão for a lei do dono, e num ARNÊS
/// que a tivesse desligado **o programa medido muda a meio**, que é o defeito que não dá erro.
///
/// ⭐ **As duas metades são obrigatórias:** sem a segunda, um `install` que cravasse `true`
/// passaria — e é exactamente o `true` que a omissão devolve.
#[test]
fn a_lei_do_dono_sobrevive_a_um_load() {
    for lei in [false, true] {
        let mut m = MotionState::new();
        m.pump.define_a_lei(lei);
        let texto = m.doc.to_text();
        m.load_text(&texto).expect("o proprio texto volta");
        assert_eq!(
            m.pump.a_lei(),
            lei,
            "um `load` nao pode ligar nem desligar a lei do dono (esperado {lei})"
        );
    }
}

/// ⛔⛔⛔ **A LEI É DO MÓDULO MOTION, E A BOMBA NASCE NEUTRA — as duas metades, porque as curas
/// são OPOSTAS.**
///
/// ⚠️⚠️ **Este gate nasceu de ONZE vermelhos que só a varredura impactada viu** (2026-09-19): a
/// [`MotionCookPump`] nascia com a porta do ambiente, e o emissor de PARTÍCULAS
/// (`ph2d-particles`) constrói uma bomba **própria**. Uma partícula é uma posição pura — sem
/// `uv_rect` e sem `geometry_id` —, logo a lei cortava-a e **o emissor desaparecia do produto**,
/// com o oráculo do Godot a ler `len 0`. *Uma lei que viaja na peça partilhada alcança todo
/// consumidor dela, incluindo os que não são o módulo cujo dono a ordenou.*
///
/// ⭐ **A metade (2) é a que impede a cura barata:** desligar a lei em todo o lado põe os onze
/// verdes e apaga a ordem do dono do produto — *e o report dele — «o grid continua desenhando
/// quadrados» — reabre sem um único teste vermelho.*
#[test]
fn uma_bomba_nasce_neutra_e_o_motion_e_que_liga() {
    // (1) A peça PARTILHADA é neutra: um consumidor que não é o Motion não herda lei nenhuma.
    assert!(
        !ph2d_eval_motion::MotionCookPump::new().a_lei(),
        "a bomba e' a peca NEUTRA: quem a constroi sem ser o Motion nao pode ser cortado"
    );

    // (2) E o MÓDULO liga-a, senão a ordem do dono não chega ao produto.
    let m = MotionState::new();
    assert_eq!(
        m.pump.a_lei(),
        ph2d_eval_motion::so_com_forma_por_ordem(),
        "o MotionState e' o dono da ordem: ele tem de a ligar no arranque"
    );
}

/// ⭐⭐⭐ **SONDA: a bandeira estática concorda com a LEI, cozinhando?** — para cada candidato,
/// coze-se o nó sozinho e pergunta-se `tem_aparencia` à corrente que ele entrega.
///
/// `cargo test -p ph2d-app-motion --lib -- --ignored --nocapture a_bandeira_contra_a_lei`
#[test]
#[ignore = "sonda, nao um gate"]
fn a_bandeira_contra_a_lei() {
    use ph2d_nodegraph::graph::Graph;
    let mut m = MotionState::new();
    let inst = |p: &ph2d_nodegraph::node::PortSpec| p.ty.domain == Domain::Instances;
    let candidatos: Vec<(&'static str, bool)> = m
        .registry
        .manifests()
        .filter(|man| man.outputs.iter().any(inst) && !man.inputs.iter().any(inst))
        .map(|man| (man.name, m.registry.so_posicoes(man.id)))
        .collect();
    eprintln!("\n=== A BANDEIRA CONTRA A LEI ===\n");
    eprintln!("  no                          │ dim  │ marcado │ tem_aparencia");
    for (nome, marcado) in candidatos {
        let dim = m
            .registry
            .manifests()
            .find(|man| man.name == nome)
            .and_then(|man| man.outputs.first().map(|p| format!("{:?}", p.ty.dim)))
            .unwrap_or_default();
        let mut g = Graph::new();
        let n = g.add_node(nome);
        let saida = g.add_node("motion.output");
        let _ = g.connect(ph2d_nodegraph::graph::Edge {
            from: (n, 0),
            to: (saida, 0),
            delayed: false,
        });
        m.doc.graph = g;
        let aparencia = m
            .pump
            .cook
            .cook(&m.doc.graph, &m.registry, saida, 0.0)
            .ok()
            .map(|out| ph2d_eval_motion::tem_aparencia(out[0].as_stream()));
        eprintln!(
            "  {nome:<27} │ {dim:<4} │ {:<7} │ {}",
            if marcado { "SIM" } else { " - " },
            match aparencia {
                Some(true) => "SIM  ⛔ o aviso MENTIRIA",
                Some(false) => " -   ✅",
                None => "(nao coze sozinho)",
            }
        );
    }
    eprintln!();
}

/// ⭐⭐⭐ **SONDA: uma FORMA SOLTA no grafo dá aparência à corrente do grid?** — o report do dono
/// de 2026-09-19: *«Ao colocar uma shape no grafo, mesmo desconectado e sem Duplicator, esferas
/// aparecem nas posições do grid.»*
///
/// `cargo test -p ph2d-app-motion --lib -- --ignored --nocapture uma_forma_solta`
#[test]
#[ignore = "sonda, nao um gate"]
fn uma_forma_solta_no_grafo() {
    use ph2d_nodegraph::graph::{Edge, Graph};
    let mut m = MotionState::new();
    let mut g = Graph::new();
    let grelha = g.add_node("motion.grid");
    let saida = g.add_node("motion.output");
    g.connect(Edge {
        from: (grelha, 0),
        to: (saida, 0),
        delayed: false,
    })
    .expect("liga");
    let sinks = vec![saida];

    let mede = |m: &mut MotionState, g: &Graph, quando: &str| {
        m.doc.graph = g.clone();
        m.pump.mark_dirty();
        m.pump.pump(
            &m.doc.graph.clone(),
            &m.registry,
            &sinks,
            0,
            0.0,
            [0.0, 0.0, 1.0, 1.0],
            [1.0, 1.0],
        );
        let n = m.pump.instances.len();
        let tamanhos: Vec<[f32; 2]> = m.pump.instances.iter().take(2).map(|i| i.size).collect();
        let uvs: Vec<[f32; 4]> = m
            .pump
            .instances
            .iter()
            .take(2)
            .map(|i| i.atlas_uv)
            .collect();
        let vect = m.pump.vector_instances.len();
        eprintln!("  {quando:<28} sprites={n:<4} vector={vect:<4} size={tamanhos:?} uv={uvs:?}");
    };

    eprintln!("\n=== UMA FORMA SOLTA NO GRAFO ===\n");
    mede(&mut m, &g, "so' a grelha");

    // A forma, SOLTA: nenhuma aresta a liga a coisa nenhuma.
    let mut com_forma = g.clone();
    let forma = com_forma.add_node("source.shape");
    eprintln!("  (a forma solta e' o no' {})", forma.0);
    mede(&mut m, &com_forma, "grelha + forma SOLTA");

    // ⭐ E a hipótese seguinte: a forma LIGADA ao mesmo sink (o que a palette faz ao inserir
    // com um nó escolhido — o splice).
    let mut ligada = g.clone();
    let f2 = ligada.add_node("source.shape");
    match ligada.connect(Edge {
        from: (f2, 0),
        to: (saida, 0),
        delayed: false,
    }) {
        Ok(()) => mede(&mut m, &ligada, "grelha + forma NO MESMO SINK"),
        Err(e) => eprintln!("  a 2.a aresta no mesmo pino foi RECUSADA: {e:?}"),
    }

    // ⭐⭐⭐ **E a CENA QUE O DONO USOU** (`PH2D_GPU_COOK_DEMO=2`), que é o único sítio onde o
    // report pode ter nascido: a de laboratório acima não o reproduz.
    let mut cena = MotionState::new();
    let sinks2 =
        crate::motion_state::demo_router::build_level(Some("2"), &mut cena.doc, &cena.registry);
    if let Some(&s2) = sinks2.first() {
        let g2 = cena.doc.graph.clone();
        let conta = |c: &mut MotionState, gr: &Graph, rot: &str| {
            c.pump.mark_dirty();
            c.pump.pump(
                gr,
                &c.registry,
                &[s2],
                0,
                0.0,
                [0.0, 0.0, 1.0, 1.0],
                [1.0, 1.0],
            );
            let n = c.pump.instances.len();
            let sz = c.pump.instances.first().map(|i| i.size);
            let uv = c.pump.instances.first().map(|i| i.atlas_uv);
            eprintln!("  [cena=2] {rot:<24} sprites={n:<7} size={sz:?} uv={uv:?}");
        };
        conta(&mut cena, &g2, "como o dono a abre");
        let mut g3 = g2.clone();
        g3.add_node("source.shape");
        conta(&mut cena, &g3, "+ uma shape SOLTA");
    }

    // ⭐ E QUE CENA mostra um grid PEQUENO — a pergunta que o report do `360×360` levanta.
    eprintln!("\n  cena │ maior sink │ tem grid");
    for nivel in ["2", "90", "91", "97", "110", "121"] {
        let mut c = MotionState::new();
        let sinks =
            crate::motion_state::demo_router::build_level(Some(nivel), &mut c.doc, &c.registry);
        if sinks.is_empty() {
            continue;
        }
        let tem_grid = c
            .doc
            .graph
            .nodes()
            .iter()
            .any(|n| n.type_name == "motion.grid");
        let mut cook = ph2d_nodegraph::cook::Cook::new();
        let _ = cook.advance_tick(&c.doc.graph, &c.registry, 0.0);
        let mut maior = 0usize;
        for &sk in &sinks {
            if let Ok(o) = cook.cook(&c.doc.graph, &c.registry, sk, 0.0) {
                maior = maior.max(o[0].as_stream().count());
            }
        }
        eprintln!("  ={nivel:<4} │ {maior:>10} │ {tem_grid}");
    }

    // E o que a corrente do sink de facto carrega, lida pelo COOK directo.
    let mut cook = ph2d_nodegraph::cook::Cook::new();
    cook.advance_tick(&com_forma, &m.registry, 0.0)
        .expect("avanca");
    let out = cook
        .cook(&com_forma, &m.registry, saida, 0.0)
        .expect("coze");
    let st = out[0].as_stream();
    eprintln!(
        "  corrente do SINK: n={} uv_rect={} geometry_id={} tem_aparencia={}",
        st.count(),
        st.get("uv_rect").is_some(),
        st.get("geometry_id").is_some(),
        ph2d_eval_motion::tem_aparencia(st)
    );
}
