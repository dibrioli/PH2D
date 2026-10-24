//! W18 (`docs/Macros/16_W18_plano.md` §2.7): o lado MACROS de aplicar uma macro do Motion — novo e por
//! cima. Nasce o mundo (o objecto Macro, o objecto Motion dentro dele), a porta do Motion
//! (`TranspoeDaFamilia::aplica`) recebe o dono e a vida certos, a trilha que ela devolve é a instalada,
//! e o clipe vive com o objecto Macro: o undo global tira-o, o refazer devolve-o. O lado do Motion (os
//! nós no documento vivo, presos à vida) é de `ph2d-app-motion` (`macro_track/aplicar_tests.rs`).

use ph2d_app_macros::aplicar::{self, Imagem, Portas};
use ph2d_app_macros::{Cena, Familias};
use ph2d_ecs::scene::{ComponentRegistry, WorldSnapshot, register_ecs_components};
use ph2d_ecs::{ChildOf, Entity, Macro, MotionGenerator, Name, SimWorld, StableId, Transform};
use ph2d_editor_core::macro_edits::{self as m, ErroDaMacroView, OndeAplicar};
use ph2d_macro::ficheiro::Ficheiro;
use ph2d_macro::{Event, MacroDoc, MacroTime, Nascimento, RefRect, Track};
use ph2d_timeline::{TimelineIntent, TimelineState};

/// O Motion de brinquedo: regista cada `(dono, vida)` e devolve a trilha a apontar o dono; e cada
/// continuação (`(dono, vida, a cadeia que recebeu: (alvo como veio, alvo aplicado))`).
#[derive(Default)]
struct MotionQueRegista(Vec<(u64, u64)>, Vec<Continuada>);

/// Uma continuação recebida: o dono, a vida e a cadeia.
type Continuada = (u64, u64, Vec<(u64, u64)>);

impl ph2d_macro::Gravadora for MotionQueRegista {
    fn abre(
        &mut self,
        rec: &mut ph2d_macro::MacroRecorder,
    ) -> Result<ph2d_macro::TrackId, &'static str> {
        Ok(rec.open_track("motion", 0, Vec::new()))
    }
    fn passos(&mut self) -> Vec<Vec<u8>> {
        Vec::new()
    }
    fn para(&mut self) -> Vec<Vec<u8>> {
        Vec::new()
    }
}

impl ph2d_editor_core::macro_tutorial::TutorialDaFamilia for MotionQueRegista {}

impl ph2d_editor_core::macro_tutorial::TranspoeDaFamilia for MotionQueRegista {
    fn aplica(&mut self, t: &Track, dono: u64, vida: u64) -> Result<Track, ErroDaMacroView> {
        self.0.push((dono, vida));
        let mut n = t.clone();
        n.target = dono;
        n.initial = vec![0xA7];
        Ok(n)
    }

    fn continua(
        &mut self,
        t: &Track,
        anteriores: &[(&Track, &Track)],
        dono: u64,
        vida: u64,
    ) -> Result<Track, ErroDaMacroView> {
        let cadeia = anteriores
            .iter()
            .map(|(a, b)| (a.target, b.target))
            .collect();
        self.1.push((dono, vida, cadeia));
        let mut n = t.clone();
        n.target = dono;
        n.initial = vec![0xC0];
        Ok(n)
    }
}

/// Uma trilha do Motion de 2 s (o conteúdo é do Motion; aqui só a família e o relógio contam).
fn trilha(nasce: bool) -> Ficheiro {
    let mut doc = MacroDoc::new(RefRect { w: 1.0, h: 1.0 });
    doc.tracks.push(Track {
        family: "motion".into(),
        target: 3,
        initial: Vec::new(),
        events: vec![Event {
            t: MacroTime(2_000_000),
            payload: vec![1],
        }],
        edicoes: Vec::new(),
        nasce: nasce.then(|| Nascimento {
            inicio: MacroTime(0),
            nome: "Ondas".into(),
            local: [1.0, 2.0, 0.0, 1.0, 1.0, 0.0, 0.0],
            pai: None,
        }),
        continua: None,
    });
    Ficheiro {
        doc,
        modo: 0,
        nome: "Ondas".into(),
    }
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

#[derive(Default)]
struct Palco {
    sim: SimWorld,
    motion: MotionQueRegista,
    tl: TimelineState,
    intents: Vec<TimelineIntent>,
}

impl Palco {
    fn aplica(&mut self, fi: &Ficheiro, onde: OndeAplicar) -> Result<u64, ErroDaMacroView> {
        let mut nasce = |_: &mut SimWorld, _: Imagem| Err(String::new());
        let mut pixeis = |_: &SimWorld, _: u64| None;
        let e_imagem = |_: &SimWorld, _: u64| false;
        let mut portas = Portas {
            raiz: &raiz,
            imagem: &mut nasce,
            pixeis: &mut pixeis,
            e_imagem: &e_imagem,
            centro: [0.0, 0.0],
            registo: &crate::portas_de_teste::registo(),
            recurso: &crate::portas_de_teste::sem_recurso,
            remapeia: &crate::portas_de_teste::remapeia,
        };
        let mut f = Familias {
            painter: None,
            painter_activo: false,
            motion: &mut self.motion,
            motion_em_edicao: None,
            motion_na_mao: false,
            etapas: crate::portas_de_teste::COM_MOTION,
            escolhido: None,
            portas: None,
        };
        let mut c = Cena {
            sim: &mut self.sim,
            timeline: &mut self.tl,
            intents: &mut self.intents,
            partidas: None,
        };
        let r = aplicar::aplica(fi, onde, &mut f, &mut c, &mut portas);
        ph2d_timeline_persist::upkeep(&mut self.tl, self.sim.world_mut());
        r
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

    fn foto(&mut self, reg: &ComponentRegistry) -> WorldSnapshot {
        let mut prop = ph2d_ecs::TransformPropagationState::new(self.sim.world_mut());
        let mut work = ph2d_ecs::WorklistBuf::default();
        let mut s = WorldSnapshot::new();
        ph2d_ecs::scene::world_to_snapshot(self.sim.world_mut(), &mut prop, &mut work, reg, &mut s)
            .expect("fotografa");
        s
    }

    /// O undo global: o mundo volta ao `s` (como o `ProjectState::restore`), e o quadro seguinte corre.
    fn repoe(&mut self, s: &WorldSnapshot, reg: &ComponentRegistry) {
        let w = self.sim.world_mut();
        let todos: Vec<Entity> = w
            .query_filtered::<Entity, ph2d_ecs::With<Transform>>()
            .iter(w)
            .collect();
        for e in todos {
            let _ = w.despawn(e);
        }
        ph2d_ecs::scene::snapshot_to_world(w, s, reg).expect("repõe");
        ph2d_timeline_persist::upkeep(&mut self.tl, self.sim.world_mut());
    }
}

fn registo() -> ComponentRegistry {
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    reg
}

/// ⭐ Novo: o objecto Motion «Ondas» nasce dentro do objecto Macro, na pose da ficha; a porta do Motion
/// recebe-o como dono e o objecto Macro como vida; a trilha dela é a instalada, e o clipe nasce sem
/// passo próprio na Timeline. Um Ctrl+Z tira o clipe com o objecto, um Ctrl+Y devolve-o.
#[test]
fn aplicar_novo_um_motion_faz_nascer_o_objecto_dentro_e_o_clipe_vive_com_ele() {
    let reg = registo();
    let mut p = Palco::default();
    let antes = p.foto(&reg);
    let r = p
        .aplica(&trilha(true), OndeAplicar::Novo { em: None })
        .expect("aplica");
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
    assert_eq!(
        w.get::<Transform>(filho)
            .map(|t| (t.translation.x, t.translation.y)),
        Some((1.0, 2.0)),
        "na pose local da ficha"
    );
    let (dono, vida) = (p.sid(filho.to_bits()), p.sid(r));
    assert_eq!(
        p.motion.0,
        [(dono, vida)],
        "o Motion recebe o dono e a vida"
    );
    let t = p.trilha(r);
    assert_eq!(
        (t.target, t.initial),
        (dono, vec![0xA7]),
        "a trilha do Motion é a instalada"
    );
    assert!(t.nasce.is_some(), "com a ficha, para a próxima exportação");
    assert_eq!(p.clipes(), 1, "o clipe da macro");
    assert!(
        !p.tl.history.can_undo(),
        "o clipe não é um passo próprio da Timeline"
    );
    let depois = p.foto(&reg);
    p.repoe(&antes, &reg);
    assert_eq!(p.clipes(), 0, "um Ctrl+Z tira o clipe com o objecto");
    p.repoe(&depois, &reg);
    assert_eq!(p.clipes(), 1, "um Ctrl+Y devolve-o");
}

/// ⭐ Por cima de um objecto Motion: o objecto Macro é próprio, a porta recebe o ALVO como dono, a trilha
/// toca nele; e um alvo que não é Motion recusa sem deixar nada no mundo.
#[test]
fn aplicar_por_cima_um_motion_da_o_alvo_como_dono() {
    let mut p = Palco::default();
    let alvo = p
        .sim
        .world_mut()
        .spawn((Name::new("Campo"), Transform::IDENTITY, MotionGenerator))
        .id()
        .to_bits();
    ph2d_ecs::assign_missing_stable_ids(p.sim.world_mut());
    let r = p
        .aplica(&trilha(false), OndeAplicar::PorCima { alvo })
        .expect("aplica");
    assert_ne!(r, alvo, "o objecto Macro é próprio");
    assert_eq!(
        p.motion.0,
        [(p.sid(alvo), p.sid(r))],
        "dono = o alvo, vida = o objecto Macro"
    );
    assert_eq!(p.trilha(r).target, p.sid(alvo), "a trilha toca no alvo");
    assert_eq!(p.clipes(), 1);

    let solto = p
        .sim
        .world_mut()
        .spawn((Name::new("Solto"), Transform::IDENTITY))
        .id()
        .to_bits();
    let n = p.sim.world().iter_entities().count();
    let e = p
        .aplica(&trilha(false), OndeAplicar::PorCima { alvo: solto })
        .expect_err("não é um objecto Motion");
    assert_eq!(e.chave, m::MACRO_APLICAR_INCOMPATIVEL);
    assert_eq!(p.sim.world().iter_entities().count(), n, "nada nasceu");
    assert_eq!(p.motion.0.len(), 1, "a porta do Motion não foi chamada");
}

/// ⭐ W19: uma continuação do Motion (`continua: Some(0)`) não passa outra vez pela porta de aplicar
/// (que juntaria os nós de novo): vai à porta da continuação com o mesmo dono e a cadeia — a 1.ª
/// trilha como veio e como ficou. Novo e por cima.
#[test]
fn a_continuacao_do_motion_vai_a_porta_da_continuacao() {
    let mut fi = trilha(true);
    let mut volta = fi.doc.tracks[0].clone();
    (volta.nasce, volta.continua) = (None, Some(0));
    fi.doc.tracks.push(volta);
    let mut p = Palco::default();
    let r = p
        .aplica(&fi, OndeAplicar::Novo { em: None })
        .expect("aplica");
    let filho = p
        .sim
        .world()
        .iter_entities()
        .find(|e| e.get::<ChildOf>().map(ChildOf::parent) == Some(Entity::from_bits(r)))
        .map(|e| e.id().to_bits())
        .expect("o objecto Motion");
    let (dono, vida) = (p.sid(filho), p.sid(r));
    assert_eq!(p.motion.0, [(dono, vida)], "aplicar só a 1.ª");
    assert_eq!(
        p.motion.1,
        [(dono, vida, vec![(3, dono)])],
        "a continuação, com a cadeia"
    );
    let d = ph2d_macro::codec::read(
        &p.sim
            .world()
            .get::<Macro>(Entity::from_bits(r))
            .expect("a Macro")
            .doc,
    )
    .expect("lê");
    assert_eq!(
        (
            d.tracks[1].target,
            d.tracks[1].initial.as_slice(),
            d.tracks[1].continua
        ),
        (dono, [0xC0].as_slice(), Some(0)),
        "a trilha da continuação é a que a porta devolveu"
    );

    let mut q = Palco::default();
    let alvo = q
        .sim
        .world_mut()
        .spawn((Name::new("Campo"), Transform::IDENTITY, MotionGenerator))
        .id()
        .to_bits();
    ph2d_ecs::assign_missing_stable_ids(q.sim.world_mut());
    let r = q
        .aplica(&fi, OndeAplicar::PorCima { alvo })
        .expect("aplica");
    let (dono, vida) = (q.sid(alvo), q.sid(r));
    assert_eq!(q.motion.0, [(dono, vida)]);
    assert_eq!(q.motion.1, [(dono, vida, vec![(3, dono)])]);
}
