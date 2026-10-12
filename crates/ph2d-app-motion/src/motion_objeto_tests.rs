//! Os gates do Motion como objecto (doc ObjectsAndComponents 01 §13, F5).

use super::*;
use ph2d_ecs::SimWorld;
use ph2d_nodegraph::graph::NodeId;

const SIM: &str =
    include_str!("../../ph2d-motion-doc/tests/fixtures/motion_doc_sim_antes_do_dono.txt");

fn motion_com(text: &str) -> MotionState {
    let mut m = MotionState::new();
    m.doc = ph2d_motion_doc::MotionDoc::from_text(text).expect("a fixtura abre");
    m
}

fn objeto(sim: &mut SimWorld, nome: &str) -> (Entity, u64) {
    let e = sim
        .world_mut()
        .spawn((
            ph2d_ecs::Name::new(nome),
            ph2d_ecs::Transform::IDENTITY,
            MotionGenerator,
        ))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let sid = sim.world().get::<StableId>(e).unwrap().0;
    (e, sid)
}

/// ⭐⭐ **Zero perda:** o `MotionDoc` global de antes do F5 vira UM objecto Motion — nasce um, e ele é
/// dono de todos os nós; o grafo fica igual. (Mutação: `orfaos` sem `adopt_unowned` ⇒ RED.)
#[test]
fn o_documento_de_antes_vira_um_objecto_motion() {
    let mut m = motion_com(SIM);
    let antes = m.doc.graph.clone();
    let mut sim = SimWorld::new();
    let feito = upkeep(&mut m, sim.world_mut(), None, false);
    let sid = feito.nasceu.expect("nasce um objecto Motion");
    assert!(m.doc.unowned().is_empty(), "todo nó tem dono");
    assert_eq!(m.doc.owner_ids(), BTreeSet::from([sid]));
    assert_eq!(m.doc.graph, antes, "o grafo não muda");
    assert_eq!(m.objeto.editing, Some(sid));
    let again = upkeep(&mut m, sim.world_mut(), None, false);
    assert_eq!(again, Manutencao::default(), "estável: não nasce outro");
}

/// Sem nós, nada nasce.
#[test]
fn um_documento_vazio_nao_faz_nascer_nada() {
    let mut m = MotionState::new();
    let mut sim = SimWorld::new();
    assert_eq!(upkeep(&mut m, sim.world_mut(), None, false).nasceu, None);
}

/// ⛔ O objecto que MORRE leva os nós dele num passo do undo do Motion; Ctrl+Z devolve-os.
#[test]
fn apagar_o_objecto_tira_os_nos_e_o_undo_devolve() {
    let mut m = motion_com(SIM);
    let mut sim = SimWorld::new();
    let (e, sid) = objeto(&mut sim, "Fonte");
    m.doc.adopt_unowned(sid);
    let n = m.doc.graph.nodes().len();
    upkeep(&mut m, sim.world_mut(), None, false);
    sim.world_mut().despawn(e);
    let feito = upkeep(&mut m, sim.world_mut(), None, false);
    assert_eq!(feito.purgados, vec![sid]);
    assert!(m.doc.graph.nodes().is_empty());
    let back = m.history.undo(&m.doc).expect("um passo do undo");
    assert_eq!(back.graph.nodes().len(), n);
}

/// ⛔ Com o relógio a andar (o Play), um objecto destruído pelo jogo NÃO leva os nós.
#[test]
fn com_o_relogio_a_andar_nada_e_purgado() {
    let mut m = motion_com(SIM);
    let mut sim = SimWorld::new();
    let (e, sid) = objeto(&mut sim, "Fonte");
    m.doc.adopt_unowned(sid);
    upkeep(&mut m, sim.world_mut(), None, true);
    sim.world_mut().despawn(e);
    let feito = upkeep(&mut m, sim.world_mut(), None, true);
    assert!(feito.purgados.is_empty());
    assert!(!m.doc.graph.nodes().is_empty());
}

/// ⭐ A cópia do editor de um objecto Motion leva um grafo PRÓPRIO. (Mutação: `copias` não ler o
/// leitor do Motion ⇒ RED.)
#[test]
fn duplicar_o_objecto_da_a_copia_um_grafo_proprio() {
    let mut m = motion_com(SIM);
    let mut sim = SimWorld::new();
    let (e, sid) = objeto(&mut sim, "Fonte");
    m.doc.adopt_unowned(sid);
    let n = m.doc.graph.nodes().len();
    upkeep(&mut m, sim.world_mut(), None, false);
    let mut reg = ph2d_ecs::scene::ComponentRegistry::new();
    ph2d_ecs::scene::register_ecs_components(&mut reg);
    let copy = ph2d_ecs::deep_copy_subtree(sim.world_mut(), &reg, e, None).expect("copia");
    ph2d_ecs::announce_copy(sim.world_mut(), &copy);
    let feito = upkeep(&mut m, sim.world_mut(), None, false);
    let (_, sid_c) = feito.copiados[0];
    assert_ne!(sid_c, sid);
    assert_eq!(m.doc.nodes_of(sid).len(), n);
    assert_eq!(m.doc.nodes_of(sid_c).len(), n, "a cópia tem os nós dela");
}

/// O dono em edição segue a selecção — também quando se selecciona um FILHO (o molde).
#[test]
fn o_dono_em_edicao_segue_a_seleccao_e_o_pai() {
    let mut m = MotionState::new();
    let mut sim = SimWorld::new();
    let (_, a) = objeto(&mut sim, "A");
    let (eb, b) = objeto(&mut sim, "B");
    let molde = sim.world_mut().spawn(ph2d_ecs::ChildOf(eb)).id();
    upkeep(&mut m, sim.world_mut(), None, false);
    assert_eq!(m.objeto.editing, Some(a.min(b)), "sem selecção: o 1.º vivo");
    upkeep(&mut m, sim.world_mut(), Some(molde.to_bits()), false);
    assert_eq!(m.objeto.editing, Some(b));
    let solto = sim.world_mut().spawn_empty().id();
    upkeep(&mut m, sim.world_mut(), Some(solto.to_bits()), false);
    assert_eq!(
        m.objeto.editing,
        Some(b),
        "seleccionar outra coisa não larga o grafo"
    );
}

/// ⭐ O olho de um objecto Motion esconde os sinks DELE (e só os dele).
#[test]
fn o_olho_esconde_os_sinks_do_dono() {
    let mut m = MotionState::new();
    let mut sim = SimWorld::new();
    let (ea, a) = objeto(&mut sim, "A");
    let (_, b) = objeto(&mut sim, "B");
    let sa = m.doc.graph.add_node("motion.output");
    let sb = m.doc.graph.add_node("motion.output");
    m.doc.set_owner(sa, a);
    m.doc.set_owner(sb, b);
    sim.world_mut()
        .entity_mut(ea)
        .insert(ph2d_ecs::Visibility { hidden: true });
    upkeep(&mut m, sim.world_mut(), None, false);
    assert_eq!(sinks_visiveis(&m, vec![sa, sb]), vec![sb]);
    assert!(
        m.objeto.desenha(),
        "o B continua a desenhar sem a ferramenta"
    );
}

/// ⭐⭐ **A forma que o Motion copia vira filha dele** na migração — e é molde (não se desenha).
#[test]
fn na_migracao_o_molde_vira_filho_e_molde() {
    let mut m = MotionState::new();
    let src = m.doc.graph.add_node("source.object");
    m.doc.graph.set_text_param(src, "object", "Bola");
    let mut sim = SimWorld::new();
    let bola = sim.world_mut().spawn(ph2d_ecs::Name::new("Bola")).id();
    let feito = upkeep(&mut m, sim.world_mut(), None, false);
    let sid = feito.nasceu.expect("a migração faz nascer o objecto");
    let pai = sim
        .world()
        .get::<ph2d_ecs::ChildOf>(bola)
        .map(|c| c.parent());
    assert_eq!(
        pai.and_then(|p| sim.world().get::<StableId>(p))
            .map(|s| s.0),
        Some(sid)
    );
    assert!(sim.world().get::<ph2d_ecs::MotionTemplate>(bola).is_some());
}

/// ⚠️ Abrir um projecto NÃO desfaz um molde arrastado para fora: o 1.º quadro só semeia.
#[test]
fn abrir_o_projecto_nao_reprende_um_molde_solto() {
    let mut m = MotionState::new();
    let mut sim = SimWorld::new();
    let (_, sid) = objeto(&mut sim, "Fonte");
    let src = m.doc.graph.add_node("source.object");
    m.doc.graph.set_text_param(src, "object", "Bola");
    m.doc.set_owner(src, sid);
    let bola = sim.world_mut().spawn(ph2d_ecs::Name::new("Bola")).id();
    upkeep(&mut m, sim.world_mut(), None, false);
    assert!(sim.world().get::<ph2d_ecs::ChildOf>(bola).is_none());
    // E uma ligação NOVA (o nó passa a nomear outro objecto) prende esse.
    let disco = sim.world_mut().spawn(ph2d_ecs::Name::new("Disco")).id();
    m.doc.graph.set_text_param(src, "object", "Disco");
    upkeep(&mut m, sim.world_mut(), None, false);
    assert!(sim.world().get::<ph2d_ecs::ChildOf>(disco).is_some());
}

fn inst(x: f32, y: f32) -> ph2d_render::RenderInstance {
    ph2d_render::RenderInstance {
        world_pos: [x, y],
        basis: [1.0, 0.0, 0.0, 1.0],
        ..bytemuck::Zeroable::zeroed()
    }
}

/// ⭐⭐ **A origem**: a faixa do sink de um objecto deslocado e rodado segue a afim dele; a do outro
/// não; mudar só a afim (bomba parada) reaplica sobre o LOCAL — nunca duas vezes. (Mutação: reaplicar
/// sobre as listas já transformadas ⇒ RED.)
#[test]
fn a_origem_leva_a_faixa_do_dono_e_nunca_duas_vezes() {
    let mut m = MotionState::new();
    let sa = m.doc.graph.add_node("motion.output");
    let sb = m.doc.graph.add_node("motion.output");
    m.doc.set_owner(sa, 1);
    m.doc.set_owner(sb, 2);
    m.pump.instances = vec![inst(1.0, 0.0), inst(5.0, 5.0)];
    m.pump.faixas = vec![
        ph2d_eval_motion::FaixaDoSink {
            sink: sa,
            instancias: 0..1,
            vetores: 0..0,
            ladrilhos: 0..0,
        },
        ph2d_eval_motion::FaixaDoSink {
            sink: sb,
            instancias: 1..2,
            vetores: 0..0,
            ladrilhos: 0..0,
        },
    ];
    m.pump.composicoes = 1;
    // A: rodado 90° e deslocado (10, 0). B: identidade.
    let rot = [0.0, 1.0, -1.0, 0.0, 10.0, 0.0];
    m.objeto.donos.insert(
        1,
        Dono {
            entity: 0,
            visible: true,
            afim: rot,
        },
    );
    m.objeto.donos.insert(
        2,
        Dono {
            entity: 0,
            visible: true,
            afim: IDENTIDADE,
        },
    );
    origem::aplica_cpu(&mut m);
    assert_eq!(m.pump.instances[0].world_pos, [10.0, 1.0]);
    assert_eq!(m.pump.instances[0].basis, [0.0, 1.0, -1.0, 0.0]);
    assert_eq!(
        m.pump.instances[1].world_pos,
        [5.0, 5.0],
        "o outro dono não mexe"
    );
    // Mesmo quadro de novo (nada mudou): nada se reaplica.
    origem::aplica_cpu(&mut m);
    assert_eq!(m.pump.instances[0].world_pos, [10.0, 1.0]);
    // Só a afim muda (a bomba não recompôs): parte do LOCAL.
    m.objeto.donos.get_mut(&1).unwrap().afim = [1.0, 0.0, 0.0, 1.0, 0.0, 3.0];
    origem::aplica_cpu(&mut m);
    assert_eq!(m.pump.instances[0].world_pos, [1.0, 3.0]);
    // De volta à identidade: as listas voltam ao local.
    m.objeto.donos.get_mut(&1).unwrap().afim = IDENTIDADE;
    origem::aplica_cpu(&mut m);
    assert_eq!(m.pump.instances[0].world_pos, [1.0, 0.0]);
}

/// A afim de um objecto compõe a dos pais (a mesma lei da propagação).
#[test]
fn a_afim_compoe_os_pais() {
    let mut sim = SimWorld::new();
    let em = |x: f32, y: f32| {
        let mut t = ph2d_ecs::Transform::IDENTITY;
        t.translation.x = x;
        t.translation.y = y;
        t
    };
    let pai = sim.world_mut().spawn(em(10.0, 0.0)).id();
    let filho = sim
        .world_mut()
        .spawn((em(0.0, 2.0), ph2d_ecs::ChildOf(pai)))
        .id();
    let a = afim_de(sim.world(), filho);
    assert_eq!([a[4], a[5]], [10.0, 2.0]);
    let _ = NodeId(0);
}

/// ⭐⭐ **UM Ctrl+Z** (doc ObjectsAndComponents 01 §14, item 3): o undo global devolve o objecto
/// Motion (o MESMO `StableId`, bits novos) e o cemitério devolve os nós dele no mesmo quadro; o passo
/// da purga sai da fila do Motion. Editado entretanto ⇒ fica o caminho de sempre (dois passos).
/// (Mutação: `ressuscita` fora do `upkeep` ⇒ RED.)
#[test]
fn um_ctrl_z_global_devolve_o_objecto_com_os_nos() {
    let mut m = motion_com(SIM);
    let mut sim = SimWorld::new();
    let (e, sid) = objeto(&mut sim, "Fonte");
    m.doc.adopt_unowned(sid);
    let grafo = m.doc.graph.clone();
    upkeep(&mut m, sim.world_mut(), None, false);
    sim.world_mut().despawn(e);
    upkeep(&mut m, sim.world_mut(), None, false);
    assert!(m.doc.graph.nodes().is_empty());
    assert!(
        undo_e_do_global(&m),
        "o Ctrl+Z da ferramenta Motion passa ao undo global (não dá os nós a outro objecto)"
    );

    sim.world_mut().spawn((
        ph2d_ecs::Name::new("Fonte"),
        ph2d_ecs::Transform::IDENTITY,
        MotionGenerator,
        StableId(sid),
    ));
    let feito = upkeep(&mut m, sim.world_mut(), None, false);
    assert!(feito.ressuscitados);
    assert!(!undo_e_do_global(&m));
    assert_eq!(m.doc.graph, grafo, "os nós voltaram com o objecto");
    assert_eq!(m.doc.owner_ids(), BTreeSet::from([sid]));
    assert!(
        m.history.undo(&m.doc).is_none(),
        "o passo da purga saiu da fila"
    );
}

#[test]
fn editado_depois_de_apagar_o_motion_volta_em_dois_passos() {
    let mut m = motion_com(SIM);
    let mut sim = SimWorld::new();
    let (e, sid) = objeto(&mut sim, "Fonte");
    m.doc.adopt_unowned(sid);
    upkeep(&mut m, sim.world_mut(), None, false);
    sim.world_mut().despawn(e);
    upkeep(&mut m, sim.world_mut(), None, false);
    m.history.push_undo(m.doc.clone()); // uma edição depois da purga
    sim.world_mut().spawn((
        ph2d_ecs::Name::new("Fonte"),
        ph2d_ecs::Transform::IDENTITY,
        MotionGenerator,
        StableId(sid),
    ));
    let feito = upkeep(&mut m, sim.world_mut(), None, false);
    assert!(!feito.ressuscitados);
    assert!(m.doc.graph.nodes().is_empty(), "nada sobrescreve a edição");
}
