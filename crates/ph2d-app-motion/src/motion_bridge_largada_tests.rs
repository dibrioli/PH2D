//! **OS GATES DAS DUAS LARGADAS** — ordem do dono (2026-09-19): *«Se arrastar um nó no grafo em
//! cima de outro nó, eles mudam de posição na cadeia. Se arrastar num nó em cima de uma conexão
//! (linha) […] ele passa a ser conectado naquela linha, contudo, sem quebrar a cadeia.»*
//!
//! ⚠️ **Irmão do [`super::tests`] por RESPONSABILIDADE:** ali mede-se *«mexer num FIO faz o que
//! se pede?»* (mover a ponta, inserir um tipo novo, recusar) e aqui *«largar uma CARTA faz o que
//! se pede?»* — o sujeito é o nó, e o que ele deixa para trás é metade da lei.
//!
//! `super` é `motion_bridge::rewire`.

use super::*;
use crate::motion_state::MotionState;
use ph2d_editor_core::ToastQueue;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// Uma cadeia de tipos, ligados `0 → 0` pela ordem. Devolve os ids.
fn cadeia(motion: &mut MotionState, tipos: &[&str]) -> Vec<NodeId> {
    let mut g = Graph::new();
    let ids: Vec<NodeId> = tipos.iter().map(|t| g.add_node(*t)).collect();
    for par in ids.windows(2) {
        g.connect(Edge {
            from: (par[0], 0),
            to: (par[1], 0),
            delayed: false,
        })
        .expect("a cadeia liga");
    }
    motion.doc.graph = g;
    ids
}

/// Quem alimenta a porta `0` de `n`.
fn fonte(motion: &MotionState, n: NodeId) -> Option<NodeId> {
    motion
        .doc
        .graph
        .edges()
        .iter()
        .find(|e| e.to.0 == n && e.to.1 == 0 && !e.delayed)
        .map(|e| e.from.0)
}

/// ⭐⭐⭐ **OS DOIS TROCAM DE LUGAR NA CADEIA** — a ordem do dono, medida na cadeia inteira.
///
/// `grid → move → scale → output`, e a troca de `scale` com `move` tem de dar
/// `grid → scale → move → output`. ⚠️ **A régua é a CADEIA INTEIRA e não «o `scale` mudou de
/// pai»:** meia troca (quem alimenta muda, quem é alimentado não) deixa o `scale` com dois pais e
/// o `output` órfão, e uma asserção sobre um elo só não vê isso.
#[test]
fn dois_nos_trocam_de_lugar_na_cadeia() {
    let mut motion = MotionState::new();
    let ids = cadeia(
        &mut motion,
        &[
            "motion.grid",
            "motion.move",
            "motion.scale",
            "motion.output",
        ],
    );
    let (grid, mv, sc, out) = (ids[0], ids[1], ids[2], ids[3]);
    let mut toasts = ToastQueue::default();
    swap_in_chain(&mut motion, &mut toasts, sc.0, mv.0, 0.0, 0.0);

    assert_eq!(
        fonte(&motion, sc),
        Some(grid),
        "o `scale` passa a vir do grid"
    );
    assert_eq!(fonte(&motion, mv), Some(sc), "e o `move` vem do `scale`");
    assert_eq!(fonte(&motion, out), Some(mv), "e a saida vem do `move`");
}

/// ⭐⭐ **E AS CARTAS TROCAM DE SÍTIO** — *«mudam de posição»* é as duas coisas. O arrastado fica
/// onde o alvo estava; o alvo vai para onde o arrastado COMEÇOU, que é o que o deslocamento
/// acumulado do arrasto (`back`) diz. ⛔ Sem isto a troca entrega um grafo certo com as duas
/// cartas empilhadas uma sobre a outra.
#[test]
fn as_cartas_trocam_de_sitio() {
    use ph2d_nodegraph::graph::Pos;
    let mut motion = MotionState::new();
    // ⚠️ **Os dois trocados têm de ter ENTRADA:** trocar uma FONTE (`motion.grid`, zero entradas)
    // com um filtro é recusado pela máquina de sempre — uma fonte não pode ficar no meio —, e a
    // 1.ª redacção deste gate media as posições sobre uma troca que nunca aconteceu. *Um gate
    // cujo sujeito é recusado afirma sobre o nada.*
    let ids = cadeia(
        &mut motion,
        &[
            "motion.grid",
            "motion.move",
            "motion.scale",
            "motion.output",
        ],
    );
    let (mv, sc) = (ids[1], ids[2]);
    motion.doc.graph.set_pos(sc, Pos { x: 300.0, y: 40.0 });
    motion.doc.graph.set_pos(mv, Pos { x: 100.0, y: 10.0 });
    let mut toasts = ToastQueue::default();
    // O `scale` foi arrastado de (100, 0) até (300, 40) — logo o deslocamento é (200, 40).
    swap_in_chain(&mut motion, &mut toasts, sc.0, mv.0, 200.0, 40.0);

    assert_eq!(
        motion.doc.graph.pos(sc),
        Some(Pos { x: 100.0, y: 10.0 }),
        "o arrastado fica onde o alvo estava"
    );
    assert_eq!(
        motion.doc.graph.pos(mv),
        Some(Pos { x: 100.0, y: 0.0 }),
        "e o alvo vai para onde o arrastado comecou"
    );
}

/// ⛔⛔ **UMA TROCA QUE NÃO CABE RECUSA, E O GRAFO FICA INTACTO** — os manifestos podem ter
/// contagens de porta diferentes, e as portas trocam pelo ÍNDICE. *Uma troca meia-feita é pior do
/// que nenhuma*, e é a mesma máquina (`connect` + `validate` num clone) que toda esta família usa.
///
/// ⭐ **A segunda espécie de recusa apareceu ao escrever o gate das POSIÇÕES:** trocar uma FONTE
/// (zero entradas) com um filtro é recusado pela mesma máquina — e isso é a lei, não uma
/// limitação: *uma fonte não tem por onde ser alimentada, logo não pode ficar no meio da cadeia.*
#[test]
fn uma_troca_que_nao_cabe_recusa_e_nao_mexe_no_grafo() {
    let mut motion = MotionState::new();
    // O `duplicator` tem DUAS entradas; a `motion.output` tem uma. Trocar as ligações de uma
    // `motion.grid` (nenhuma entrada) com o duplicador deixa a porta 1 dele sem onde ir.
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let forma = g.add_node("source.shape");
    let dup = g.add_node("motion.duplicator");
    let out = g.add_node("motion.output");
    for (from, to, port) in [(forma, dup, 0u16), (grid, dup, 1), (dup, out, 0)] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, port),
            delayed: false,
        })
        .expect("a cena liga");
    }
    motion.doc.graph = g;
    let antes = motion.doc.graph.edges().to_vec();
    let mut toasts = ToastQueue::default();
    swap_in_chain(&mut motion, &mut toasts, grid.0, dup.0, 0.0, 0.0);
    assert_eq!(
        motion.doc.graph.edges(),
        antes.as_slice(),
        "a troca recusada nao pode deixar o grafo meio rewired"
    );

    // A segunda espécie: uma FONTE não pode ir para o meio da cadeia.
    let mut motion = MotionState::new();
    let ids = cadeia(
        &mut motion,
        &["motion.grid", "motion.move", "motion.output"],
    );
    let antes = motion.doc.graph.edges().to_vec();
    swap_in_chain(&mut motion, &mut toasts, ids[0].0, ids[1].0, 0.0, 0.0);
    assert_eq!(
        motion.doc.graph.edges(),
        antes.as_slice(),
        "uma fonte nao tem por onde ser alimentada, logo nao pode ficar no meio"
    );
}

/// ⭐⭐⭐ **O NÓ ENTRA NO FIO E FECHA A CADEIA DE ONDE SAIU** — as DUAS metades do *«sem quebrar a
/// cadeia»*, na mesma corrida.
///
/// `a → b → c` e `d → e`; enfiar o `b` no fio `d → e` tem de dar `a → c` **e** `d → b → e`.
/// ⛔ **Sem a primeira metade** o artista fica com um buraco onde o nó estava; **sem a segunda**,
/// com o nó ligado em dois sítios ao mesmo tempo.
#[test]
fn um_no_enfiado_num_fio_fecha_a_cadeia_de_onde_saiu() {
    let mut motion = MotionState::new();
    let mut g = Graph::new();
    let a = g.add_node("motion.grid");
    let b = g.add_node("motion.move");
    let c = g.add_node("motion.output");
    let d = g.add_node("motion.grid");
    let e = g.add_node("motion.output");
    for (from, to) in [(a, b), (b, c), (d, e)] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, 0),
            delayed: false,
        })
        .expect("a cena liga");
    }
    motion.doc.graph = g;
    let mut toasts = ToastQueue::default();
    splice_existing_into_wire(&mut motion, &mut toasts, b.0, e.0, 0);

    assert_eq!(
        fonte(&motion, c),
        Some(a),
        "a cadeia de onde ele saiu FECHOU"
    );
    assert_eq!(fonte(&motion, b), Some(d), "e ele entrou no fio");
    assert_eq!(
        fonte(&motion, e),
        Some(b),
        "sem deixar a outra ponta a pairar"
    );
}

/// ⭐ **E um nó DESCONECTADO também entra** — *«mesmo se estiver desconectado»*. Aqui não há
/// cadeia para fechar, e a ausência dela não pode ser lida como um motivo para recusar.
#[test]
fn um_no_desconectado_tambem_entra_no_fio() {
    let mut motion = MotionState::new();
    let mut g = Graph::new();
    let d = g.add_node("motion.grid");
    let e = g.add_node("motion.output");
    let solto = g.add_node("motion.move");
    g.connect(Edge {
        from: (d, 0),
        to: (e, 0),
        delayed: false,
    })
    .expect("o fio");
    motion.doc.graph = g;
    let mut toasts = ToastQueue::default();
    splice_existing_into_wire(&mut motion, &mut toasts, solto.0, e.0, 0);
    assert_eq!(fonte(&motion, solto), Some(d));
    assert_eq!(fonte(&motion, e), Some(solto));
}

/// ⛔ **Largar um nó no PRÓPRIO fio é inerte** — seria um laço. O painel já não o oferece; esta é
/// a segunda porta, porque a intenção chega por outros caminhos.
#[test]
fn enfiar_um_no_no_proprio_fio_e_inerte() {
    let mut motion = MotionState::new();
    let ids = cadeia(
        &mut motion,
        &["motion.grid", "motion.move", "motion.output"],
    );
    let (mv, out) = (ids[1], ids[2]);
    let antes = motion.doc.graph.edges().to_vec();
    let mut toasts = ToastQueue::default();
    splice_existing_into_wire(&mut motion, &mut toasts, mv.0, out.0, 0);
    assert_eq!(motion.doc.graph.edges(), antes.as_slice());
}

/// ⛔⛔ **O HEAL DE UM NÓ APAGADO FAZ A PONTE PELA PORTA PRINCIPAL** — o mesmo defeito do report
/// do dono, um gesto mais atrás: o `heal_deleted_node` dizia *«PRIMARY input (port 0)»* e usava a
/// `0`, e as duas coisas deixaram de ser a mesma no dia em que o `motion.duplicator` declarou
/// `primary_input = 1`.
///
/// `grid → duplicator(points)` com uma `source.shape` na `shape`: apagar o duplicador tem de
/// ligar o **grid** à saída, nunca a forma. ⚠️ **Nem o tipo nem o `validate` acusam** — as duas
/// entradas dele são `INST_VEC2`, logo a cadeia «curada» pela porta errada é um grafo VÁLIDO que
/// carrega a aparência no lugar das posições.
#[test]
fn o_heal_de_um_duplicador_apagado_faz_a_ponte_pela_principal() {
    let mut motion = MotionState::new();
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let forma = g.add_node("source.shape");
    let dup = g.add_node("motion.duplicator");
    let out = g.add_node("motion.output");
    for (from, to, port) in [(forma, dup, 0u16), (grid, dup, 1), (dup, out, 0)] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, port),
            delayed: false,
        })
        .expect("a cena liga");
    }
    motion.doc.graph = g;
    assert!(heal_deleted_node(&mut motion, dup), "a cadeia cura");
    assert_eq!(
        fonte(&motion, out),
        Some(grid),
        "a ponte e' pelos POINTS (a porta principal), nunca pela shape"
    );
}

/// ⭐⭐⭐ **O GESTO INTEIRO É *UM* PASSO DE UNDO — E O REDO DEVOLVE-O** — ordem do dono
/// (2026-09-19): *«Undo/redo implementado para essas ações.»*
///
/// ⚠️ **A régua entra pela ROTA e não pela operação:** ela empurra as intenções que o painel de
/// facto empurra (`BeginDrag` · `MoveNodes` · `SwapInChain` · `EndDrag`) e mede a PILHA. *Chamar
/// `swap_in_chain` à mão não afirma nada sobre o parênteses do arrasto, que é onde o número de
/// Ctrl+Z é decidido.*
///
/// As três metades: a acção acontece · **UM** `Ctrl+Z` devolve a fiação E as posições · o redo
/// volta a aplicá-la.
#[test]
fn o_gesto_inteiro_e_um_passo_de_undo_e_o_redo_devolve() {
    use ph2d_nodegraph::graph::Pos;
    use ph2d_panel_motion_graph::{GraphIntent, push_intent};

    let mut motion = MotionState::new();
    let ids = cadeia(
        &mut motion,
        &[
            "motion.grid",
            "motion.move",
            "motion.scale",
            "motion.output",
        ],
    );
    let (grid, mv, sc) = (ids[0], ids[1], ids[2]);
    motion.doc.graph.set_pos(mv, Pos { x: 100.0, y: 0.0 });
    motion.doc.graph.set_pos(sc, Pos { x: 300.0, y: 0.0 });
    let _ = ph2d_panel_motion_graph::drain_intents();

    // O arrasto REAL, como o painel o empurra: o `scale` sobre o `move`.
    push_intent(GraphIntent::BeginDrag);
    push_intent(GraphIntent::MoveNodes {
        nodes: vec![sc.0],
        dx: -200.0,
        dy: 0.0,
    });
    push_intent(GraphIntent::SwapInChain {
        a: sc.0,
        b: mv.0,
        back_dx: -200.0,
        back_dy: 0.0,
    });
    push_intent(GraphIntent::EndDrag);
    let aplica = |motion: &mut MotionState| {
        super::super::apply_graph_intents(
            motion,
            &mut ph2d_core::Playhead::default(),
            &mut ToastQueue::default(),
            &mut ph2d_editor_core::screens::layout::CenterSplit::None,
        );
    };
    aplica(&mut motion);

    assert_eq!(fonte(&motion, sc), Some(grid), "a troca aconteceu");
    assert_eq!(
        motion.doc.graph.pos(sc),
        Some(Pos { x: 100.0, y: 0.0 }),
        "e as cartas trocaram"
    );

    // ⭐ UM passo, e ele devolve as DUAS coisas.
    motion.doc = motion.history.undo(&motion.doc).expect("um passo de undo");
    assert_eq!(fonte(&motion, mv), Some(grid), "a fiacao voltou");
    assert_eq!(
        motion.doc.graph.pos(sc),
        Some(Pos { x: 300.0, y: 0.0 }),
        "e as posicoes tambem"
    );
    assert!(
        !motion.history.can_undo(),
        "o gesto inteiro e' UM passo: mover e trocar nao podem pedir dois Ctrl+Z"
    );

    // ⭐ E o redo devolve a troca.
    motion.doc = motion.history.redo(&motion.doc).expect("o redo existe");
    assert_eq!(fonte(&motion, sc), Some(grid), "o redo repoe a troca");
    assert_eq!(motion.doc.graph.pos(sc), Some(Pos { x: 100.0, y: 0.0 }));
}

/// ⭐⭐ **O ECO NASCE DA ACÇÃO, E MORRE SOZINHO** — *«Após a troca o conjunto linha e nó piscam e
/// se acentam»*.
///
/// ⚠️ **As três metades:** ele acende com os nós E os fios certos · a intensidade DESCE com o
/// relógio de PAREDE (não com o playhead — a cena pode estar pausada) · e ele **apaga-se**.
/// ⛔ Sem a última, o canal fica com o último valor publicado para sempre, que é o modo de falha
/// que todo canal lateral deste painel tem de evitar.
#[test]
fn o_eco_da_largada_nasce_da_accao_e_morre_sozinho() {
    let mut motion = MotionState::new();
    let ids = cadeia(
        &mut motion,
        &[
            "motion.grid",
            "motion.move",
            "motion.scale",
            "motion.output",
        ],
    );
    let (mv, sc) = (ids[1], ids[2]);
    motion.ui_now = 10.0;
    let mut toasts = ToastQueue::default();
    swap_in_chain(&mut motion, &mut toasts, sc.0, mv.0, 0.0, 0.0);

    let crate::motion_state::PiscadaPendente { nos, fios, inicio } =
        motion.piscada.clone().expect("a troca acende o eco");
    assert_eq!(inicio, 10.0, "o eco parte do relogio de PAREDE da shell");
    assert!(
        nos.contains(&sc.0) && nos.contains(&mv.0),
        "os DOIS nos da troca piscam: {nos:?}"
    );
    assert!(
        !fios.is_empty(),
        "e os fios NOVOS deles tambem — senao so' metade do «conjunto linha e no» pisca"
    );

    // ⚠️ **As leituras são nos PICOS**, e não em dois instantes quaisquer: desde que o eco PISCA,
    // duas amostras arbitrárias podem cair em fases diferentes da onda e comparar o pulso em vez
    // do envelope. *Uma régua que amostra uma onda em pontos escolhidos à mão mede a fase.*
    let t_de = |motion: &mut MotionState, agora: f32| {
        motion.ui_now = agora;
        super::super::publicar_piscada(motion);
        ph2d_panel_motion_graph::current_graph_flash().map(|p| p.t)
    };
    let meio = super::super::PERIODO_S * 0.5;
    let primeiro = t_de(&mut motion, 10.0 + meio).expect("o 1.o pico");
    let ultimo = t_de(&mut motion, 10.0 + super::super::PISCADA_S - meio).expect("o ultimo pico");
    assert!(
        primeiro > ultimo,
        "o envelope tem de DESCER de pico a pico: {primeiro} depois {ultimo}"
    );
    assert_eq!(
        t_de(&mut motion, 99.0),
        None,
        "e no fim ele APAGA-SE — um canal lateral que fica aceso e' o defeito que ele evita"
    );
    assert!(motion.piscada.is_none(), "e o pendente sai do estado");
}

/// ⭐⭐⭐ **O ECO PISCA TRÊS VEZES E ASSENTA** — ordem do dono (2026-09-19, depois do smoke): *«faça
/// ambos piscarem mais vezes depois da troca»*.
///
/// ⚠️ **A régua conta os PICOS pela forma da curva**, varrendo-a fino — e não pelos instantes em
/// que a lei os põe. *Um gate que amostra nos picos que ele próprio calculou não afirma que eles
/// existem: afirma que a aritmética dele é a mesma da lei.*
///
/// As três metades: **quantas** passagens · o envelope a **descer** de pico a pico · e o eco a
/// chegar a ZERO no fim (o *«se acentam»*, que uma onda sem envelope não dá).
#[test]
fn o_eco_pisca_tres_vezes_e_assenta() {
    use super::super::{PISCADA_S, PISCADAS, intensidade_do_eco};

    const AMOSTRAS: usize = 2000;
    let v: Vec<f32> = (0..=AMOSTRAS)
        .map(|i| {
            #[expect(clippy::cast_precision_loss, reason = "2000 amostras cabem num f32")]
            let t = PISCADA_S * (i as f32 / AMOSTRAS as f32);
            intensidade_do_eco(t)
        })
        .collect();

    let picos: Vec<f32> = v
        .windows(3)
        .filter(|w| w[1] > w[0] && w[1] >= w[2])
        .map(|w| w[1])
        .collect();
    assert_eq!(
        picos.len(),
        PISCADAS as usize,
        "o eco tem de piscar {PISCADAS} vezes: {} picos",
        picos.len()
    );
    assert!(
        picos.windows(2).all(|p| p[1] < p[0]),
        "e cada passagem chega MENOS alto que a anterior: {picos:?}"
    );
    assert!(
        v.last().is_some_and(|&x| x < 1e-3),
        "e no fim ele assenta em ZERO — sem isto sao piscadelas, nunca um eco"
    );
    // ⛔ O CONTROLO: sem o pulso a curva teria UM pico só. Sem esta metade, uma lei que
    // devolvesse o envelope nu passaria nas outras duas.
    assert!(
        picos.len() > 1,
        "o controlo: um envelope sem pulso tem um pico so'"
    );
}

/// ⭐⭐⭐ **UM VÃO APERTADO ABRE-SE, E UM VÃO FOLGADO NÃO SE MEXE** — ordem do dono (2026-09-19,
/// depois do smoke): *«se o espaço onde o nó entrou for muito apertado, crie espaço»*.
///
/// ⚠️ **As três metades:** o `v` e a JUSANTE dele afastam-se o bastante para três cartas caberem ·
/// o nó entra na coluna do MEIO (senão o vão abre e a sobreposição fica) · e com espaço a sobrar
/// **nada se mexe** — *uma arrumação que corre sempre tira o desenho das mãos do artista*.
#[test]
fn um_vao_apertado_abre_se_e_um_folgado_fica_quieto() {
    use ph2d_nodegraph::graph::Pos;
    use ph2d_nodegraph::layout::DX;

    // `u → v → w`, com o `solto` a ser enfiado no fio `u → v`.
    let cena = |vao: f32| {
        let mut motion = MotionState::new();
        let mut g = Graph::new();
        let u = g.add_node("motion.grid");
        let v = g.add_node("motion.move");
        let w = g.add_node("motion.output");
        let solto = g.add_node("motion.scale");
        for (a, b) in [(u, v), (v, w)] {
            g.connect(Edge {
                from: (a, 0),
                to: (b, 0),
                delayed: false,
            })
            .expect("a cadeia liga");
        }
        g.set_pos(u, Pos { x: 0.0, y: 0.0 });
        g.set_pos(v, Pos { x: vao, y: 0.0 });
        g.set_pos(
            w,
            Pos {
                x: vao + 300.0,
                y: 0.0,
            },
        );
        g.set_pos(solto, Pos { x: 20.0, y: 400.0 });
        motion.doc.graph = g;
        (motion, u, v, w, solto)
    };

    // (A) APERTADO: o vão mede menos do que as três cartas precisam.
    let (mut motion, u, v, w, solto) = cena(DX);
    let antes_w = motion.doc.graph.pos(w).expect("w");
    let mut toasts = ToastQueue::default();
    splice_existing_into_wire(&mut motion, &mut toasts, solto.0, v.0, 0);
    let (pu, pv, pn) = (
        motion.doc.graph.pos(u).expect("u"),
        motion.doc.graph.pos(v).expect("v"),
        motion.doc.graph.pos(solto).expect("no'"),
    );
    assert!(
        pv.x - pu.x >= 2.0 * DX,
        "o vao tem de abrir para DOIS passos de coluna: {}",
        pv.x - pu.x
    );
    assert_eq!(pn.x, pu.x + DX, "e o no' entra na coluna do MEIO");
    assert_eq!(pn.y, 400.0, "mantendo o `y` que a mao escolheu");
    assert!(
        motion.doc.graph.pos(w).expect("w").x > antes_w.x,
        "e a JUSANTE anda com ele — senao o `v` empurrado sobrepoe-se a quem ele alimenta"
    );

    // (B) FOLGADO: já cabe, logo nada se mexe.
    let (mut motion, _u, v, w, solto) = cena(3.0 * DX);
    let antes: Vec<_> = [v, w, solto]
        .map(|n| motion.doc.graph.pos(n).expect("pos"))
        .to_vec();
    splice_existing_into_wire(&mut motion, &mut toasts, solto.0, v.0, 0);
    let depois: Vec<_> = [v, w, solto]
        .map(|n| motion.doc.graph.pos(n).expect("pos"))
        .to_vec();
    assert_eq!(
        antes, depois,
        "com espaco a sobrar, o desenho do artista fica como ele o deixou"
    );
}

/// ⭐⭐ **E A MESMA LEI VALE PARA O SPLICE DA PALETA** — a rota que CRIA o nó no fio (o `R` sobre
/// uma ligação, ou a paleta aberta por ela). *Um vão apertado aperta igual, venha o nó de onde
/// vier* — e é isso que faz de [`super::espaco::abre_espaco`] uma porta com dois chamadores em vez
/// de uma cura num deles.
#[test]
fn o_splice_da_paleta_abre_o_mesmo_espaco() {
    use ph2d_nodegraph::graph::Pos;
    use ph2d_nodegraph::layout::DX;

    let mut motion = MotionState::new();
    let mut g = Graph::new();
    let u = g.add_node("motion.grid");
    let v = g.add_node("motion.output");
    g.connect(Edge {
        from: (u, 0),
        to: (v, 0),
        delayed: false,
    })
    .expect("o fio");
    g.set_pos(u, Pos { x: 0.0, y: 0.0 });
    g.set_pos(v, Pos { x: DX, y: 0.0 });
    motion.doc.graph = g;

    let mut toasts = ToastQueue::default();
    splice_node(&mut motion, &mut toasts, v.0, 0, "motion.move", 30.0, 10.0);

    let pv = motion.doc.graph.pos(v).expect("v");
    assert!(
        pv.x >= 2.0 * DX,
        "o vao tem de abrir tambem por esta porta: {}",
        pv.x
    );
}
