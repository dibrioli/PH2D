//! Os gates da cerca das FORMAS na rota da placa (doc 121 W3) — SEM placa: a cerca é pura sobre o
//! que recebe, e a varredura das cenas que a achou é `#[ignore]` (o CI nunca a corre).

use super::{
    RECUSA_FORMA_COM_BRILHO, RECUSA_FORMA_COM_COLISOR, RECUSA_FORMA_COM_TRACO,
    RECUSA_FORMA_DO_VELLO, RECUSA_FORMAS_DESLIGADAS, formas_para_a_placa, handles_publicados,
    saida_com_mistura_em_formas,
};
use crate::motion_shape_gen::VecPathStore;
use crate::motion_shape_placa::GeometriasDaPlaca;
use crate::motion_state::MotionState;
use ph2d_node_source_lsystem as ls;
use ph2d_nodegraph::graph::Edge;
use ph2d_nodegraph::node::NodeTypeId;
use ph2d_vec_scene::{Rgba8, StrokeSpec};

/// O grafo que a `=108` tem e que um artista faz: um L-System seguido de um nó que a placa
/// despacha.
fn lsystem_move_output(geometry: Option<f32>) -> MotionState {
    let mut m = MotionState::new();
    let g = &mut m.doc.graph;
    let l = g.add_node("source.lsystem");
    if let Some(v) = geometry {
        g.set_param(l, ls::param::GEOMETRY, v);
    }
    let mv = g.add_node("motion.move");
    let out = g.add_node("motion.output");
    for (a, b) in [(l, mv), (mv, out)] {
        g.connect(Edge {
            from: (a, 0),
            to: (b, 0),
            delayed: false,
        })
        .unwrap();
    }
    m
}

/// ⭐⭐ **A cerca condicional ficou coberta pelo CONTEÚDO** — em `Branches` (o default do nó) o
/// L-System publica a fita (handles de forma), em `Segments` não publica nenhum. ⚠️ As duas metades:
/// sem a primeira a fita seria desenhada como quadrados, sem a segunda um L-System de posições
/// levaria a placa das formas por nada.
///
/// ⛔ **A premissa que este gate tinha MORREU** (doc 121 W3): ele afirmava que um L-System em
/// `Branches` fica na CPU (`desenha_forma_condicional`). Hoje a fita VAI à placa, pelo passe de
/// formas; o que se afirma é que a cerca a VÊ.
#[test]
fn a_fita_do_lsystem_publica_formas_e_os_segmentos_nao() {
    let publicadas = |g: Option<f32>| {
        let mut m = lsystem_move_output(g);
        crate::motion_externals::publish_all(&mut m, 0.0);
        handles_publicados(&m.pump.cook)
    };
    assert!(
        !publicadas(None).is_empty(),
        "o default do nó é `Branches` — a fita é uma forma viva"
    );
    assert!(!publicadas(Some(ls::GEOMETRY_BRANCHES as f32)).is_empty());
    assert!(
        publicadas(Some(ls::GEOMETRY_SEGMENTS as f32)).is_empty(),
        "em `Segments` o nó emite POSIÇÕES, e não publica forma nenhuma"
    );
}

/// ⚠️ **A bandeira de TIPO não mudou** — os outros dois leitores dela (a lei da aparência e a
/// das fontes de posições) continuam a ver o L-System como fonte de posições.
#[test]
fn a_bandeira_de_tipo_do_lsystem_nao_mudou() {
    let m = MotionState::new();
    let ty = NodeTypeId::of("source.lsystem");
    assert!(!m.registry.is_live_vector_source(ty));
    assert!(m.registry.has_live_vector_condition(ty));
    let em = |g: f32| move |n: &str| if n == ls::param::GEOMETRY { g } else { 0.0 };
    assert!(
        m.registry
            .emits_live_vector(ty, &em(ls::GEOMETRY_BRANCHES as f32))
    );
    assert!(
        !m.registry
            .emits_live_vector(ty, &em(ls::GEOMETRY_SEGMENTS as f32))
    );
    // O controlo: o `source.shape` é vectorial POR TIPO, com quaisquer params.
    assert!(
        m.registry
            .emits_live_vector(NodeTypeId::of("source.shape"), &|_| 0.0)
    );
}

/// Um store com uma estrela LISA, uma com TRAÇO e uma com TINTA própria.
fn store() -> (VecPathStore, u32, u32, u32) {
    let mut s = VecPathStore::default();
    let lisa = s.push(ph2d_vec_scene::star([0.0, 0.0], 0.5, 0.5, 5, 0.4));
    let mut t = ph2d_vec_scene::ellipse([0.0, 0.0], 0.5, 0.5);
    t.stroke = Some(StrokeSpec::new(Rgba8::new(0, 0, 0, 255), 0.05));
    let tracada = s.push(t);
    let mut p = ph2d_vec_scene::ellipse([0.0, 0.0], 0.5, 0.5);
    p.fill = Some(ph2d_vec_scene::Paint::Solid(Rgba8::new(1, 2, 3, 255)));
    let pintada = s.push(p);
    (s, lisa, tracada, pintada)
}

/// ⭐⭐⭐ **A cerca por conteúdo, motivo a motivo** — cada recusa com o SEU nome, e o CONTROLO
/// (formas lisas, a placa ligada) a passar. ⚠️ Sem o controlo, uma cerca incondicional passaria
/// em todas as linhas de recusa.
#[test]
fn a_cerca_das_formas_nomeia_cada_recusa() {
    let (s, lisa, tracada, pintada) = store();
    let mut g = GeometriasDaPlaca::default();
    let mut cerca = |ligada, vivas: &[u32], brilho, colisor| {
        formas_para_a_placa(ligada, vivas, brilho, colisor, &s, &mut g)
    };
    assert_eq!(
        cerca(true, &[lisa], false, false),
        Ok(()),
        "o CONTROLO passa"
    );
    assert_eq!(
        cerca(false, &[], false, false),
        Ok(()),
        "sem formas não há o que recusar"
    );
    assert_eq!(
        cerca(false, &[lisa], false, false),
        Err(RECUSA_FORMAS_DESLIGADAS)
    );
    assert_eq!(
        cerca(true, &[lisa], true, false),
        Err(RECUSA_FORMA_COM_BRILHO)
    );
    assert_eq!(
        cerca(true, &[lisa], false, true),
        Err(RECUSA_FORMA_COM_COLISOR)
    );
    assert_eq!(
        cerca(true, &[lisa, tracada], false, false),
        Err(RECUSA_FORMA_COM_TRACO)
    );
    assert_eq!(
        cerca(true, &[pintada, lisa], false, false),
        Err(RECUSA_FORMA_DO_VELLO)
    );
    // ⚠️ Um handle AUSENTE do store não recusa — não há o que desenhar.
    assert_eq!(cerca(true, &[lisa, 9_999], false, false), Ok(()));
}

/// ⭐ **A cerca e a rota da CPU perguntam ao MESMO cache** — depois da cerca, a geometria está
/// preparada, e a placa desenha-a sem a preparar outra vez.
#[test]
fn a_cerca_prepara_as_geometrias_que_a_placa_liga() {
    let (s, lisa, _, _) = store();
    let mut g = GeometriasDaPlaca::default();
    assert!(g.is_empty());
    formas_para_a_placa(true, &[lisa], false, false, &s, &mut g).expect("passa");
    assert_eq!(g.len(), 1, "a lisa ficou preparada no cache partilhado");
}

/// ⭐ **Os handles publicados** — os de uma `source.shape` real, pela membrana do produto; o
/// CONTROLO é uma grelha, que não publica nenhum.
#[test]
fn a_forma_publica_o_handle_dela() {
    let build = |ty: &str| {
        let mut m = MotionState::new();
        let src = m.doc.graph.add_node(ty);
        let out = m.doc.graph.add_node("motion.output");
        m.doc
            .graph
            .connect(Edge {
                from: (src, 0),
                to: (out, 0),
                delayed: false,
            })
            .expect("fio");
        crate::motion_externals::publish_all(&mut m, 0.0);
        handles_publicados(&m.pump.cook)
    };
    assert_eq!(build("source.shape").len(), 1, "uma forma, um handle");
    assert!(
        build("motion.grid").is_empty(),
        "o CONTROLO não publica forma"
    );
}

/// Uma `source.shape` pelo `motion.strobe` com o OPERADOR DO FLASH `flash_blend` (`0` = o modo do
/// sink ⇒ nenhuma coluna `blend`), cozida pela bomba da CPU no tique `0`.
fn forma_com_flash(flash_blend: f32) -> MotionState {
    let mut m = MotionState::new();
    let g = &mut m.doc.graph;
    let src = g.add_node("source.shape");
    let st = g.add_node("motion.strobe");
    g.set_param(st, ph2d_node_motion_strobe::FLASH_BLEND, flash_blend);
    let out = g.add_node("motion.output");
    for (a, b) in [(src, st), (st, out)] {
        g.connect(Edge {
            from: (a, 0),
            to: (b, 0),
            delayed: false,
        })
        .expect("fio");
    }
    m.sinks = vec![out];
    crate::motion_externals::publish_all(&mut m, 0.0);
    let scopes = ph2d_node_motion_time_remap::time_scopes(&m.doc.graph, &m.registry);
    let sinks = m.sinks.clone();
    m.pump.set_taps(&sinks);
    m.pump.advance_or_scrub_scoped(
        &m.doc.graph,
        &m.registry,
        &m.sinks,
        0,
        |t| t as f64 / 60.0,
        m.default_uv_rect,
        m.default_size,
        &scopes,
    );
    m
}

/// ⭐⭐ **A MISTURA POR LINHA de uma saída com formas manda o quadro à CPU ANTES do plano** (doc 121
/// W3) — o passe de formas não tem a camada que ela pede, e uma recusa DENTRO do cozimento
/// deixaria a rota híbrida sem desenhar nada.
///
/// As três metades, e cada uma é um defeito diferente:
/// - **a memória da CPU**: uma saída cozida com forma E com a coluna `blend` pede o Vello;
/// - **o CONTROLO**: a mesma forma com o operador no modo do sink (nenhuma coluna) NÃO pede — sem
///   ele, uma cerca que respondesse sempre `true` devolvia toda forma à CPU e ficava verde;
/// - **a bandeira do dispositivo é CONSUMIDA**: ela cobre o 1.º quadro (a CPU ainda não coseu a
///   saída) e depois a memória manda — uma bandeira que não se gasta prendia o documento na CPU
///   depois de o artista tirar a mistura.
#[test]
fn a_mistura_por_linha_das_formas_manda_o_quadro_a_cpu() {
    let mut com = forma_com_flash(1.0);
    assert!(
        !handles_publicados(&com.pump.cook).is_empty(),
        "a fixtura tem de conter uma forma viva"
    );
    assert!(
        saida_com_mistura_em_formas(&mut com),
        "forma + coluna `blend` na saida cozida: a placa nao tem a camada que ela pede"
    );

    let mut sem = forma_com_flash(0.0);
    assert!(
        !saida_com_mistura_em_formas(&mut sem),
        "sem operador (o modo do sink) nao ha' coluna `blend`: a forma vai a' placa"
    );

    sem.formas_pedem_o_vello = true;
    assert!(
        saida_com_mistura_em_formas(&mut sem),
        "a recusa do dispositivo cobre o 1.º quadro"
    );
    assert!(
        !saida_com_mistura_em_formas(&mut sem),
        "a bandeira e' CONSUMIDA: depois dela manda a memoria da CPU"
    );
}
