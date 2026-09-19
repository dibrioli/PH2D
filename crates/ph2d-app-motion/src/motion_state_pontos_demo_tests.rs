//! Gates da cena `=124` — **as posições e a marca**.
//!
//! ⚠️⚠️ **O que estes gates NÃO podem medir, e porquê:** a metade com forma passa por um
//! `source.shape`, e a geometria dele é assada pela SHELL — num arnês headless ele coze `n = 0`.
//! Medido sobre a cena `=121`, que funciona no app e dá exactamente o mesmo zero aqui. ⇒ *a
//! metade com forma prova-se pela ESTRUTURA do grafo; a corrente dela é do smoke do dono.*

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

/// ⭐⭐⭐ **A METADE DA ESQUERDA É SÓ POSIÇÕES — a metade que a cena existe para mostrar.**
///
/// ⚠️ **A régua é a mesma porta que o lowering e o gizmo usam** (`tem_aparencia`): um predicado
/// próprio aqui divergiria no dia em que uma origem de aparência nova nascesse, e o gate passaria
/// a afirmar uma coisa sobre uma cena que desenha outra.
#[test]
fn a_metade_da_esquerda_nao_traz_aparencia() {
    let (doc, reg, sinks) = cena();
    assert_eq!(sinks.len(), 2, "duas metades");
    let mut cook = ph2d_nodegraph::cook::Cook::new();
    cook.advance_tick(&doc.graph, &reg, 0.0).expect("avanca");
    let s = cook.cook(&doc.graph, &reg, sinks[0], 0.0).expect("coze");
    let c = s[0].as_stream();
    let n = (COLUNAS * LINHAS) as usize;
    assert_eq!(c.count(), n, "a metade da esquerda tem {n} posicoes");
    assert!(
        !ph2d_eval_motion::tem_aparencia(c),
        "ela nao pode trazer aparencia -- e' o que faz dela MARCAS e nao pecas"
    );
}

/// ⭐⭐ **A METADE DA DIREITA É O CONTROLO, e a estrutura dela é a prova que este arnês alcança.**
///
/// ⚠️ **Sem ela a cena não ensina nada:** *«não desenha»* e *«está partido»* têm o mesmo aspecto
/// no ecrã, e o que os separa é ver a MESMA nuvem a virar peças assim que uma forma chega. ⇒ o
/// gate exige o duplicador, a forma, e que a forma entre na porta que o manifesto declara.
#[test]
fn a_metade_da_direita_veste_a_mesma_nuvem() {
    let (doc, reg, _) = cena();
    let g = &doc.graph;
    let tipo = |n: &ph2d_nodegraph::graph::NodeInstance| n.type_name.clone();
    let acha = |t: &str| g.nodes().iter().find(|n| tipo(n) == t).map(|n| n.id);
    let dup = acha("motion.duplicator").expect("a cena tem um duplicador");
    let forma = acha("source.shape").expect("a cena tem uma forma");
    let entradas: Vec<(u16, ph2d_nodegraph::graph::NodeId)> = g
        .edges()
        .iter()
        .filter(|e| e.to.0 == dup)
        .map(|e| (e.to.1, e.from.0))
        .collect();
    assert!(
        entradas.contains(&(0, forma)),
        "a forma tem de entrar na porta 0 do duplicador, e as entradas sao {entradas:?}"
    );
    // ⚠️ E a porta `1` tem de vir de uma grelha (por um `motion.move`, que é como a metade se
    // afasta): sem esta metade, um duplicador ligado só à forma passaria.
    assert!(
        entradas.iter().any(|(p, _)| *p == 1),
        "os PONTOS tem de entrar na porta 1: {entradas:?}"
    );
    // O `reg` é o que valida o grafo no `cena()`; nomeá-lo aqui mantém a leitura honesta.
    let _ = reg;
}

/// ⭐⭐ **A GRELHA CABE INTEIRA NO TECTO DO GIZMO, e a folga é o assunto.**
///
/// ⚠️ A `=2` (o demo de PERFORMANCE que o report confundiu com esta) entrega `129 600` posições,
/// `31×` o tecto — ali a amostra ENGATA e o que se vê é uma nuvem escolhida. Aqui não engata,
/// logo **o artista vê a grelha inteira, posição a posição**, que é o que um smoke de marcas
/// precisa.
#[test]
fn a_grelha_cabe_inteira_no_tecto_do_gizmo() {
    let n = (COLUNAS * LINHAS) as usize;
    assert!(
        n <= crate::ponto_gizmo::MAX_PONTOS,
        "{n} posicoes contra um tecto de {} -- a amostra engataria e a cena mostraria uma \
         escolha em vez da grelha",
        crate::ponto_gizmo::MAX_PONTOS
    );
    // ⛔ E o CONTROLO: o lado DERIVADO é o que está MONTADO, não outro que por acaso também
    // caiba. *Sem esta metade, a cena podia derivar para `3 × 3` e o gate ficava verde.*
    //
    // ⚠️ **O PISO da derivação NÃO mora aqui, e o clippy foi quem o disse:** `COLUNAS` e `LINHAS`
    // são constantes, logo um `assert!` sobre elas é **dobrado pelo compilador** e este teste
    // afirmaria uma coisa que já é decidida na compilação. Ele está no `const _` da cena — ver
    // [`super::COLUNAS`] —, que é a mesma lei que o `ponto_gizmo_overlay` já escreve por extenso.
    let (doc, ..) = cena();
    let g = &doc.graph;
    let grelhas: Vec<f32> = g
        .nodes()
        .iter()
        .filter(|n| n.type_name == "motion.grid")
        .filter_map(|n| {
            g.node_params()
                .get(&n.id)
                .and_then(|m| m.get("rows"))
                .copied()
        })
        .collect();
    assert_eq!(
        grelhas,
        vec![LINHAS, LINHAS],
        "as duas metades tem {LINHAS} linhas"
    );
}

/// ⚠️ **AS DUAS METADES NÃO SE SOBREPÕEM** — a cena tem de se ler como DUAS coisas.
///
/// A régua mede a nuvem que a metade com forma produz ANTES do duplicador (que é a parte deste
/// grafo que um arnês headless alcança) contra a outra.
#[test]
fn as_duas_metades_ficam_separadas() {
    let (doc, reg, sinks) = cena();
    let g = &doc.graph;
    // A cabeça da metade com forma: o `motion.move` que a afasta para a direita.
    let afasta = g
        .nodes()
        .iter()
        .filter(|n| n.type_name == "motion.move")
        .filter(|n| {
            g.node_params()
                .get(&n.id)
                .and_then(|m| m.get("dx"))
                .copied()
                .unwrap_or(0.0)
                > 0.0
        })
        .map(|n| n.id)
        .next()
        .expect("a metade com forma afasta-se por um `motion.move` com `dx > 0`");
    let mut cook = ph2d_nodegraph::cook::Cook::new();
    cook.advance_tick(g, &reg, 0.0).expect("avanca");
    let faixa = |n: ph2d_nodegraph::graph::NodeId, cook: &mut ph2d_nodegraph::cook::Cook| {
        let o = cook.cook(g, &reg, n, 0.0).expect("coze");
        let Some(ph2d_nodegraph::attr::Column::Vec2(p)) = o[0].as_stream().get("P") else {
            panic!("sem P")
        };
        p.iter().fold(
            (f32::MAX, f32::MIN, f32::MAX, f32::MIN),
            |(xl, xh, yl, yh), q| (xl.min(q[0]), xh.max(q[0]), yl.min(q[1]), yh.max(q[1])),
        )
    };
    let (dir_lo, dir_hi, dir_ylo, dir_yhi) = faixa(afasta, &mut cook);
    let (esq_lo, esq_hi, ..) = faixa(sinks[0], &mut cook);
    assert!(
        esq_hi < dir_lo,
        "a da esquerda acaba em {esq_hi} e a da direita comeca em {dir_lo}"
    );

    // ⭐⭐⭐ **E A CENA TEM DE CABER NO QUE A CÂMERA DE ARRANQUE MOSTRA** — a metade que a FOTO
    // exigiu. ⛔ A 1.ª redacção desta cena empilhava as duas e a de baixo ficava três ecrãs
    // abaixo: *o gate «não se sobrepõem» ficava VERDE sobre uma cena em que o dono via metade.*
    // ⚠️ E o peso da peça conta: a forma é carimbada CENTRADA em `y` e pendurada em `x`, logo ela
    // transborda a nuvem por `2 × TAMANHO` à direita e por `TAMANHO / 3` (a esbelteza) em cima.
    let larg = (dir_hi + 2.0 * super::TAMANHO - esq_lo) / 2.0;
    let alt = (dir_yhi - dir_ylo) / 2.0 + super::TAMANHO / 3.0;
    assert!(
        larg <= super::VISTA_MEIA_LARGURA,
        "a cena mede {larg} de meia-largura contra os {} que a camera mostra",
        super::VISTA_MEIA_LARGURA
    );
    assert!(
        alt <= super::VISTA_MEIA_ALTURA,
        "a cena mede {alt} de meia-altura contra os {} que a camera mostra",
        super::VISTA_MEIA_ALTURA
    );
}

/// O roteiro do dono nomeia o que aparece na tela, e nada que não apareça.
#[test]
fn o_roteiro_nomeia_o_que_a_cena_tem() {
    let texto = include_str!("motion_state_pontos_demo.rs");
    for nome in ["Grid", "Duplicator", "Shape", "Gap X", "Gap Y"] {
        assert!(
            texto.contains(nome),
            "o roteiro tem de nomear {nome:?}, que e' o que o artista procura na tela"
        );
    }
}
