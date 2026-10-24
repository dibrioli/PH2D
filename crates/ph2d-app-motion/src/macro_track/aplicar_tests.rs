//! W18 (`docs/Macros/16_W18_plano.md` §2.7): uma macro do Motion GRAVADA aplicada pela porta da
//! família MACROS (`ph2d_app_macros::aplicar::aplica`) — novo e por cima — com o `MotionState` real;
//! e o aplicar inteiro é UM passo: o undo global (o mundo reposto pelo snapshot) tira o objecto Macro,
//! o clipe e os nós; o refazer devolve os três.

use super::roteiro::CENA;
use super::smoke::grava_gestos;
use super::transpor::{fim, tests::canon};
use crate::motion_objeto::{undo_e_do_global, upkeep};
use crate::motion_state::MotionState;
use ph2d_app_macros::aplicar::{self, Imagem, Portas};
use ph2d_app_macros::{Cena, Familias};
use ph2d_ecs::scene::{ComponentRegistry, WorldSnapshot, register_ecs_components};
use ph2d_ecs::{ChildOf, Entity, Macro, MotionGenerator, Name, SimWorld, StableId, Transform};
use ph2d_editor_core::macro_edits::OndeAplicar;
use ph2d_macro::ficheiro::Ficheiro;
use ph2d_macro::{MacroDoc, MacroTime, Nascimento, RefRect, Track};
use ph2d_motion_doc::MotionDoc;
use ph2d_nodegraph::graph::Edge;
use ph2d_timeline::{TimelineIntent, TimelineState};

/// O objecto que gravou.
const DONO: u64 = 7;

/// A macro de 3 nós (forma → duplicador → saída), gravada a partir do nada.
fn gravada() -> Track {
    let bytes = grava_gestos(DONO, &CENA[..6], &[]).expect("grava").0;
    ph2d_macro::codec::read(&bytes)
        .expect("lê")
        .tracks
        .remove(0)
}

fn ficheiro(t: Track) -> Ficheiro {
    let mut doc = MacroDoc::new(RefRect { w: 1.0, h: 1.0 });
    doc.tracks.push(t);
    Ficheiro {
        doc,
        modo: 0,
        nome: "Ondas".into(),
    }
}

/// A gravada com a ficha de nascimento: o objecto Motion «Ondas» dentro do objecto Macro.
fn com_ficha() -> Ficheiro {
    let mut t = gravada();
    t.nasce = Some(Nascimento {
        inicio: MacroTime(0),
        nome: "Ondas".into(),
        local: [1.0, 2.0, 0.0, 1.0, 1.0, 0.0, 0.0],
        pai: None,
    });
    ficheiro(t)
}

fn raiz(sim: &mut SimWorld, nome: &str) -> u64 {
    let b = sim
        .world_mut()
        .spawn((Name::new(nome), Transform::IDENTITY))
        .id()
        .to_bits();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    b
}

struct Palco {
    sim: SimWorld,
    motion: MotionState,
    tl: TimelineState,
    intents: Vec<TimelineIntent>,
    reg: ComponentRegistry,
}

impl Palco {
    fn novo() -> Self {
        let mut motion = MotionState::new();
        motion.sinks.clear();
        motion.gpu_enabled = false;
        let mut reg = ComponentRegistry::new();
        register_ecs_components(&mut reg);
        let mut p = Self {
            sim: SimWorld::new(),
            motion,
            tl: TimelineState::default(),
            intents: Vec::new(),
            reg,
        };
        p.quadro();
        p
    }

    /// A manutenção por quadro das duas famílias que guardam documento fora do undo global.
    fn quadro(&mut self) {
        upkeep(&mut self.motion, self.sim.world_mut(), None, false);
        ph2d_timeline_persist::upkeep(&mut self.tl, self.sim.world_mut());
    }

    fn aplica(&mut self, fi: &Ficheiro, onde: OndeAplicar) -> u64 {
        let mut nasce = |_: &mut SimWorld, _: Imagem| Err(String::new());
        let mut pixeis = |_: &SimWorld, _: u64| None;
        let e_imagem = |_: &SimWorld, _: u64| false;
        let mut portas = Portas {
            raiz: &raiz,
            imagem: &mut nasce,
            pixeis: &mut pixeis,
            e_imagem: &e_imagem,
            centro: [0.0, 0.0],
        };
        let mut f = Familias {
            painter: None,
            painter_activo: false,
            motion: &mut self.motion,
            motion_em_edicao: None,
            motion_na_mao: false,
            etapas: &[],
            escolhido: None,
            portas: None,
        };
        let mut c = Cena {
            sim: &mut self.sim,
            timeline: &mut self.tl,
            intents: &mut self.intents,
            partidas: None,
        };
        let r = aplicar::aplica(fi, onde, &mut f, &mut c, &mut portas).expect("aplica");
        self.quadro();
        r
    }

    /// O que o undo global fotografa (o `WorldSnapshot` do `ProjectState`).
    fn foto(&mut self) -> WorldSnapshot {
        let mut prop = ph2d_ecs::TransformPropagationState::new(self.sim.world_mut());
        let mut work = ph2d_ecs::WorklistBuf::default();
        let mut s = WorldSnapshot::new();
        ph2d_ecs::scene::world_to_snapshot(
            self.sim.world_mut(),
            &mut prop,
            &mut work,
            &self.reg,
            &mut s,
        )
        .expect("fotografa");
        s
    }

    /// O undo global: o mundo volta ao `s` (bits novos), e o quadro seguinte corre.
    fn repoe(&mut self, s: &WorldSnapshot) {
        let w = self.sim.world_mut();
        // Como o `ProjectState::restore`: as editáveis (o despawn cascateia pelos filhos).
        let todos: Vec<Entity> = w
            .query_filtered::<Entity, ph2d_ecs::With<Transform>>()
            .iter(w)
            .collect();
        for e in todos {
            let _ = w.despawn(e);
        }
        ph2d_ecs::scene::snapshot_to_world(w, s, &self.reg).expect("repõe");
        self.quadro();
    }

    fn bits_de(&mut self, sid: u64) -> Option<u64> {
        ph2d_ecs::entity_of_stable_id(self.sim.world_mut(), StableId(sid)).map(Entity::to_bits)
    }

    fn sid(&self, bits: u64) -> u64 {
        self.sim
            .world()
            .get::<StableId>(Entity::from_bits(bits))
            .expect("tem identidade")
            .0
    }

    fn trilha(&self, bits: u64) -> Track {
        let m = self
            .sim
            .world()
            .get::<Macro>(Entity::from_bits(bits))
            .expect("tem a Macro");
        ph2d_macro::codec::read(&m.doc)
            .expect("lê")
            .tracks
            .remove(0)
    }

    fn clipes(&self) -> usize {
        self.tl
            .doc
            .clips()
            .iter()
            .filter(|c| c.owner.is_some())
            .count()
    }
}

/// ⭐ Novo: nasce o objecto Macro com o objecto Motion dentro, o documento vivo ganha os nós da
/// gravação no `StableId` dele, e a trilha instalada aponta-o. Depois o aplicar inteiro desfaz-se num
/// Ctrl+Z (o mundo, o clipe e os nós) e refaz-se num Ctrl+Y.
#[test]
fn aplicar_novo_um_motion_gravado_e_desfaze_lo_num_passo() {
    let mut p = Palco::novo();
    let fi = com_ficha();
    let gravado = canon(&fim(&fi.doc.tracks[0]).expect("toca"), DONO);
    assert_eq!(gravado.0.len(), 3, "a fixtura cria 3 nós");
    let doc_antes = p.motion.doc.clone();
    let mundo_antes = p.foto();

    let r = p.aplica(&fi, OndeAplicar::Novo { em: None });
    let w = p.sim.world();
    let filhos: Vec<Entity> = w
        .iter_entities()
        .filter(|e| e.get::<ChildOf>().map(ChildOf::parent) == Some(Entity::from_bits(r)))
        .map(|e| e.id())
        .collect();
    assert_eq!(
        filhos.len(),
        1,
        "o objecto Motion nasceu dentro do objecto Macro"
    );
    let filho = filhos[0];
    assert!(w.get::<MotionGenerator>(filho).is_some());
    assert_eq!(w.get::<Name>(filho).map(|n| n.0.as_str()), Some("Ondas"));
    let sid = p.sid(filho.to_bits());
    assert_eq!(
        canon(&p.motion.doc, sid),
        gravado,
        "o grafo do fim da gravação, do dono novo"
    );
    assert_eq!(
        p.trilha(r).target,
        sid,
        "a trilha instalada aponta o objecto novo"
    );
    assert_eq!(p.clipes(), 1, "o clipe da macro");
    assert!(
        !p.tl.history.can_undo(),
        "o clipe não é um passo próprio da Timeline"
    );
    let doc_depois = p.motion.doc.clone();
    let mundo_depois = p.foto();
    assert!(
        undo_e_do_global(&p.motion, false),
        "o Ctrl+Z da ferramenta Motion vai ao undo global"
    );

    p.repoe(&mundo_antes);
    assert_eq!(p.motion.doc, doc_antes, "um Ctrl+Z tira os nós");
    assert_eq!(p.clipes(), 0, "e o clipe");
    assert!(
        undo_e_do_global(&p.motion, true),
        "o Ctrl+Y também é do global"
    );

    p.repoe(&mundo_depois);
    assert_eq!(p.motion.doc, doc_depois, "um Ctrl+Y devolve os nós");
    assert_eq!(p.clipes(), 1, "e o clipe");
    let filho = p.bits_de(sid).expect("o objecto Motion voltou");
    assert_eq!(canon(&p.motion.doc, p.sid(filho)), gravado);
}

/// ⭐ Por cima de um objecto Motion com 2 nós: os dele ficam intactos, os 3 da macro juntam-se, a
/// trilha toca nele; um Ctrl+Z devolve-lhe só os seus, um Ctrl+Y os cinco.
#[test]
fn aplicar_por_cima_um_motion_gravado_e_desfaze_lo_num_passo() {
    let mut p = Palco::novo();
    let alvo = p
        .sim
        .world_mut()
        .spawn((Name::new("Campo"), Transform::IDENTITY, MotionGenerator))
        .id()
        .to_bits();
    ph2d_ecs::assign_missing_stable_ids(p.sim.world_mut());
    let t_sid = p.sid(alvo);
    let (a, b) = {
        let d = &mut p.motion.doc;
        let a = d.graph.add_node("field.box");
        let b = d.graph.add_node("force.wind");
        d.set_owner(a, t_sid);
        d.set_owner(b, t_sid);
        d.graph.set_param(a, "width", 2.0);
        d.graph
            .connect(Edge {
                from: (a, 0),
                to: (b, 0),
                delayed: false,
            })
            .expect("liga");
        (a, b)
    };
    p.quadro();
    let doc_antes = p.motion.doc.clone();
    let deles: MotionDoc = doc_antes.clone();
    let mundo_antes = p.foto();

    let r = p.aplica(&ficheiro(gravada()), OndeAplicar::PorCima { alvo });
    assert_ne!(r, alvo, "o objecto Macro é próprio");
    assert_eq!(p.trilha(r).target, t_sid, "a trilha toca no alvo");
    let nos = p.motion.doc.nodes_of(t_sid);
    assert_eq!(nos.len(), 5, "{nos:?}");
    assert_eq!(&nos[..2], &[a, b], "os de antes ficam com os seus ids");
    assert_eq!(
        p.motion.doc.graph.node_param_overrides(a),
        deles.graph.node_param_overrides(a),
        "e com os seus params"
    );
    let tipos: Vec<_> = nos[2..]
        .iter()
        .map(|n| p.motion.doc.graph.node(*n).expect("nó").type_name.as_str())
        .collect();
    assert_eq!(
        tipos,
        ["source.shape", "motion.duplicator", "motion.output"]
    );
    let doc_depois = p.motion.doc.clone();
    let mundo_depois = p.foto();

    p.repoe(&mundo_antes);
    assert_eq!(
        p.motion.doc, doc_antes,
        "um Ctrl+Z devolve ao alvo só os seus nós"
    );
    assert_eq!(p.clipes(), 0);
    p.repoe(&mundo_depois);
    assert_eq!(p.motion.doc, doc_depois, "um Ctrl+Y os cinco");
    assert_eq!(p.clipes(), 1);
}
