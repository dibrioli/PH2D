//! A gravação do mundo e o fecho dela, sobre fotografias reais do `world_to_snapshot`.

use super::*;
use crate::mundo::formato::{Evento, Inicial, Mudanca, Quem};
use ph2d_ecs::scene::{ComponentRegistry, register_ecs_components, world_to_snapshot};
use ph2d_ecs::{
    ChildOf, Entity, Macro, Name, SimWorld, StableId, TransformPropagationState, WorklistBuf,
};
use ph2d_editor_core::macro_edits::FamiliaDaMacro;
use ph2d_macro::{MacroDoc, MacroTime, Nascimento, RefRect, Track};
use ph2d_physics_ecs::{Collider, RigidBody};

const DONO: u64 = 10;
const IMAGEM: u64 = 11;
const FORA: u64 = 20;

fn registo() -> ComponentRegistry {
    let mut r = ComponentRegistry::new();
    register_ecs_components(&mut r);
    ph2d_physics_ecs::register_physics_components(&mut r);
    r
}

fn foto(sim: &mut SimWorld, reg: &ComponentRegistry) -> WorldSnapshot {
    let mut st = TransformPropagationState::new(sim.world_mut());
    let mut wl = WorklistBuf::new();
    let mut out = WorldSnapshot::new();
    world_to_snapshot(sim.world_mut(), &mut st, &mut wl, reg, &mut out).expect("foto");
    out
}

/// O objecto Macro com uma imagem dentro, e um objecto fora; devolve os bits da imagem.
fn mundo() -> (SimWorld, u64) {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let d = w
        .spawn((
            ph2d_ecs::Transform::IDENTITY,
            Name::new("Macro"),
            Macro::default(),
            StableId(DONO),
        ))
        .id();
    let i = w
        .spawn((
            ph2d_ecs::Transform::IDENTITY,
            Name::new("Player"),
            StableId(IMAGEM),
            ChildOf(d),
        ))
        .id();
    w.spawn((ph2d_ecs::Transform::IDENTITY, Name::new("Chão"), StableId(FORA)));
    (sim, i.to_bits())
}

fn poe_o_collider(sim: &mut SimWorld, img: u64) {
    sim.world_mut()
        .entity_mut(Entity::from_bits(img))
        .insert((RigidBody::default(), Collider::default()));
}

fn trilha_do_painter() -> Track {
    Track {
        family: ph2d_tool_painter::macro_track::FAMILIA.into(),
        target: IMAGEM,
        initial: vec![],
        events: vec![],
        edicoes: vec![],
        nasce: Some(Nascimento {
            inicio: MacroTime(0),
            nome: "Player".into(),
            local: [0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0],
            pai: None,
        }),
        continua: None,
    }
}

fn familia(t: &Track) -> Option<FamiliaDaMacro> {
    crate::aplicar::familia(t, &[]).ok()
}

fn fecha_com(g: Gravacao, doc: &MacroDoc, reg: &ComponentRegistry) -> Option<super::super::Fecho> {
    super::super::fecha(g, doc, &familia, reg, &|_| None)
}

fn passos(f: &super::super::Fecho) -> Vec<Evento> {
    f.eventos
        .iter()
        .map(|(_, b)| Evento::le(b).expect("lê"))
        .collect()
}

#[test]
fn a_sub_arvore_e_o_objeto_macro_e_os_de_dentro() {
    let (mut sim, _) = mundo();
    let s = foto(&mut sim, &registo());
    let ids: Vec<u64> = sub_arvore(&s, DONO).keys().copied().collect();
    assert_eq!(ids, [DONO, IMAGEM]);
}

#[test]
fn o_collider_posto_a_gravar_entra_na_macro() {
    let reg = registo();
    let (mut sim, img) = mundo();
    let antes = foto(&mut sim, &reg);
    poe_o_collider(&mut sim, img);
    // Mexer FORA do objecto Macro não é desta gravação.
    let fora = ph2d_ecs::entity_with_stable_id(sim.world(), StableId(FORA)).expect("fora");
    sim.world_mut().entity_mut(fora).insert(RigidBody::default());
    let depois = foto(&mut sim, &reg);
    let mut g = Gravacao::new(DONO);
    g.mudou(&antes, &depois, 1_000);
    assert_eq!(g.passos.len(), 1, "um passo do undo = um passo da macro");
    let f = fecha_com(g, &MacroDoc::new(RefRect { w: 1.0, h: 1.0 }), &reg).expect("gravou");
    let Evento::Passo { mudancas, novos, .. } = &passos(&f)[0] else {
        panic!("um passo");
    };
    let Mudanca::Muda { id, componentes, .. } = &mudancas[0] else {
        panic!("muda: {mudancas:?}");
    };
    let mut nomes: Vec<&str> = componentes.iter().map(|c| c.0.as_str()).collect();
    nomes.sort_unstable();
    assert_eq!(nomes, ["ph2d::physics::Collider", "ph2d::physics::RigidBody"]);
    assert!(componentes.iter().all(|c| c.1.is_some()));
    let i = Inicial::le(&f.inicial).expect("lê");
    let o = i.objetos.iter().find(|o| o.id == *id).expect("o objecto do passo");
    assert_eq!((o.original, o.quem), (IMAGEM, Quem::Mundo));
    assert!(novos.is_empty(), "a imagem já estava no inicial");
    assert_eq!(mudancas.len(), 1, "o chão de fora não entrou");
}

#[test]
fn o_que_o_painter_grava_nao_se_grava_duas_vezes() {
    let reg = registo();
    let (mut sim, img) = mundo();
    let antes = foto(&mut sim, &reg);
    let px: ph2d_ecs::SpritePixels = postcard::from_bytes(&[7u8; 32]).expect("um AssetId");
    sim.world_mut()
        .entity_mut(Entity::from_bits(img))
        .insert(px);
    poe_o_collider(&mut sim, img);
    let depois = foto(&mut sim, &reg);
    let mut g = Gravacao::new(DONO);
    g.mudou(&antes, &depois, 1_000);
    let mut doc = MacroDoc::new(RefRect { w: 1.0, h: 1.0 });
    doc.tracks.push(trilha_do_painter());
    let f = fecha_com(g, &doc, &reg).expect("gravou");
    let Evento::Passo { mudancas, .. } = &passos(&f)[0] else {
        panic!("um passo");
    };
    let Mudanca::Muda { componentes, .. } = &mudancas[0] else {
        panic!("muda");
    };
    assert!(
        componentes.iter().all(|c| c.0 != "ph2d::ecs::SpritePixels"),
        "os píxeis são da trilha do Painter"
    );
    assert!(componentes.iter().any(|c| c.0 == "ph2d::physics::Collider"));
    let i = Inicial::le(&f.inicial).expect("lê");
    let o = i.objetos.iter().find(|o| o.original == IMAGEM).expect("a imagem");
    assert_eq!(o.quem, Quem::Trilha(0), "quem faz nascer a imagem é a trilha 0");
    assert!(f.recusados.is_empty());
}

#[test]
fn um_componente_recusado_diz_se_pelo_nome() {
    let reg = registo();
    let (mut sim, img) = mundo();
    let antes = foto(&mut sim, &reg);
    sim.world_mut()
        .entity_mut(Entity::from_bits(img))
        .insert(postcard::from_bytes::<ph2d_ecs::Tags>(&[1, 3]).expect("uma tag"));
    let depois = foto(&mut sim, &reg);
    let mut g = Gravacao::new(DONO);
    g.mudou(&antes, &depois, 1_000);
    let f = fecha_com(g, &MacroDoc::new(RefRect { w: 1.0, h: 1.0 }), &reg);
    let f = f.expect("o inicial grava");
    assert!(f.recusados.contains("ph2d::ecs::Tags"), "{:?}", f.recusados);
    let aviso = super::super::aviso(&f.recusados).expect("o painel di-lo");
    assert_eq!(aviso.detalhe.as_deref(), Some("Tags"));
}

#[test]
fn simular_grava_um_passo_e_nenhuma_pose() {
    let reg = registo();
    let (mut sim, img) = mundo();
    let antes = foto(&mut sim, &reg);
    poe_o_collider(&mut sim, img);
    let depois = foto(&mut sim, &reg);
    let mut g = Gravacao::new(DONO);
    g.mudou(&antes, &depois, 1_000);
    let fita = ph2d_physics_ecs::TapeWire::default();
    // Três quadros de corrida contígua (0→2→4→6), depois um quadro parado.
    for (k, de) in [0u64, 2, 4].into_iter().enumerate() {
        let y = -(de as f32);
        g.correu(de, de + 2, vec![(IMAGEM, [0.0, y, 0.0])], &fita, 2_000 + k as u64);
        g.quadro();
    }
    g.quadro();
    let f = fecha_com(g, &MacroDoc::new(RefRect { w: 1.0, h: 1.0 }), &reg).expect("gravou");
    let ps = passos(&f);
    assert_eq!(ps.len(), 2, "o collider e UMA corrida: {ps:?}");
    let Evento::Simula(s) = &ps[1] else {
        panic!("a corrida");
    };
    assert_eq!((s.de, s.ate), (0, 6));
    assert_eq!(s.conferencia.len(), 1, "uma pose por corpo, a do fim — nenhuma por tique");
    assert_eq!(s.conferencia[0].1, [0.0, -4.0, 0.0]);
}
