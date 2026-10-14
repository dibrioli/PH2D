//! O palco de uma macro coze o mundo que a cena publicou para o editor: o caminho desenhado por
//! nome, as entradas do jogo, a visibilidade do dono.

use super::*;
use crate::macro_track::tocar::{Modo, TocadorDoMotion};
use ph2d_ecs::motion_inputs::{Escritor, Valor};
use ph2d_ecs::{MotionGenerator, Name, StableId, Transform, VecPathRef};
use ph2d_nodegraph::graph::{Edge, NodeId};

const DT: f64 = 1.0 / 60.0;
const TRILHO: &str = "Trilho";

struct Cena {
    sim: SimWorld,
    vec: ph2d_vec_scene::VecScene,
    mapa: ph2d_vec_entities::entities::VecEntityMap,
    dono: ph2d_ecs::Entity,
}

/// Um caminho aberto chamado [`TRILHO`] e um objecto Motion vivo.
fn cena() -> Cena {
    let mut sim = SimWorld::new();
    let mut vec = ph2d_vec_scene::VecScene::new();
    let id = vec.push_path(ph2d_vec_scene::VecPath {
        verts: vec![
            ph2d_vec_scene::VecVertex::corner([0.0, 0.0]),
            ph2d_vec_scene::VecVertex::corner([4.0, 0.0]),
            ph2d_vec_scene::VecVertex::corner([4.0, 3.0]),
        ],
        closed: false,
        ..ph2d_vec_scene::VecPath::default()
    });
    let e = sim
        .world_mut()
        .spawn((Transform::default(), Name::new(TRILHO), VecPathRef(id)))
        .id();
    let mut mapa = ph2d_vec_entities::entities::VecEntityMap::default();
    mapa.insert(id, e.to_bits());
    let dono = sim
        .world_mut()
        .spawn((Name::new("Motion"), Transform::IDENTITY, MotionGenerator))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    Cena {
        sim,
        vec,
        mapa,
        dono,
    }
}

/// O editor: `motion.path` sobre o [`TRILHO`] até à saída, do objecto Motion da cena; e uma entrada
/// do jogo que o grafo lê.
fn editor(c: &mut Cena) -> MotionState {
    let mut m = crate::macro_track::estado_de_teste();
    let sid = c.sim.world().get::<StableId>(c.dono).expect("stable id").0;
    let g = &mut m.doc.graph;
    let lfo = g.add_node("value.lfo");
    let path = g.add_node("motion.path");
    let forma = g.add_node("source.shape");
    let dup = g.add_node("motion.duplicator");
    let out = g.add_node("motion.output");
    let entrada = g.add_node("value.input");
    g.set_text_param(entrada, ph2d_node_value_input::NAME_KEY, "vida");
    for (a, b, porta) in [
        (lfo, path, 0),
        (forma, dup, 0),
        (path, dup, 1),
        (dup, out, 0),
    ] {
        g.connect(Edge {
            from: (a, 0),
            to: (b, porta),
            delayed: false,
        })
        .expect("a cadeia liga-se");
    }
    g.set_text_param(path, "path", TRILHO);
    g.set_param(path, "count", 8.0);
    for n in [lfo, path, forma, dup, out, entrada] {
        m.doc.set_owner(n, sid);
    }
    m
}

/// O quadro do editor: a manutenção, o mundo, e o cozimento.
fn publica(c: &mut Cena, m: &mut MotionState) {
    crate::motion_objeto::upkeep(m, c.sim.world_mut(), None, false);
    c.sim
        .world_mut()
        .insert_resource(ph2d_ecs::MotionInputs::default());
    c.sim
        .world_mut()
        .resource_mut::<ph2d_ecs::MotionInputs>()
        .set("vida", Escritor::Script, Valor::Numero(0.3));
    crate::motion_bridge::publish_shapes(
        m,
        &c.sim,
        &c.vec,
        &c.mapa,
        &ph2d_vec_scene::VecXforms::default(),
        None,
    );
    crate::motion_entradas::le_do_jogo(m, &c.sim);
    m.mundo = MundoDaCena::de(m);
}

fn coze(m: &mut MotionState) {
    let ph = ph2d_core::Playhead::new(DT);
    crate::motion_bridge::quadro::o_quadro(
        m,
        None,
        &ph,
        DT,
        crate::motion_bridge::quadro::MaoDoEditor::default(),
    );
}

/// O palco da gravação de um quadro do editor, com ou sem o mundo.
fn palco(c: &mut Cena, com_o_mundo: bool) -> (MotionState, TocadorDoMotion) {
    let mut m = editor(c);
    let ini = m.gravacao_arma().expect("arma");
    publica(c, &mut m);
    coze(&mut m);
    let cargas = m.gravacao_para();
    let mut t = TocadorDoMotion::novo(&ini, Modo::Fiel).expect("o inicial lê-se");
    if com_o_mundo {
        t.recebe_o_mundo(&m.mundo);
    }
    for k in &cargas {
        t.aplica(k, None).expect("o passo aplica-se");
    }
    (m, t)
}

fn retrato(m: &MotionState) -> Vec<u32> {
    crate::macro_track::gate_tests::retrato(m)
}

fn le_a_entrada(m: &MotionState) -> Option<Vec<f32>> {
    use ph2d_nodegraph::attr::Column;
    let e = m
        .pump
        .cook
        .externals()
        .get(&ph2d_nodegraph::external::input_of("vida"))?;
    match e.value.get(ph2d_nodegraph::external::INPUT_VALUE)? {
        Column::Scalar(v) => Some(v.clone()),
        _ => None,
    }
}

#[test]
fn o_palco_coze_o_caminho_e_as_entradas_que_a_cena_publicou() {
    let mut c = cena();
    let (m, t) = palco(&mut c, true);
    let n = m.pump.instances.len() + m.pump.vector_instances.len();
    assert_eq!(n, 8, "CONTROLO: o editor percorre o trilho com oito cópias");
    assert_eq!(retrato(t.palco()), retrato(&m), "o palco vê o mesmo trilho");
    assert_eq!(le_a_entrada(t.palco()), Some(vec![0.3]));
    let (_, sem) = palco(&mut cena(), false);
    assert_ne!(
        retrato(sem.palco()),
        retrato(&m),
        "CONTROLO: sem o mundo o palco não tem o trilho"
    );
    assert_eq!(le_a_entrada(sem.palco()), None);
}

#[test]
fn o_dono_escondido_na_cena_esconde_o_palco() {
    let mut c = cena();
    let dono = c.dono;
    c.sim
        .world_mut()
        .entity_mut(dono)
        .insert(ph2d_ecs::Visibility { hidden: true });
    let (m, t) = palco(&mut c, true);
    assert!(
        m.sinks.is_empty(),
        "CONTROLO: o editor não coze o dono escondido"
    );
    assert!(t.palco().sinks.is_empty(), "nem o palco");
    let mut c = cena();
    let dono = c.dono;
    c.sim
        .world_mut()
        .entity_mut(dono)
        .insert(ph2d_ecs::Visibility { hidden: true });
    let (_, sem) = palco(&mut c, false);
    assert_eq!(
        sem.palco().sinks,
        vec![NodeId(4)],
        "CONTROLO: sem os donos o palco desenharia"
    );
}
