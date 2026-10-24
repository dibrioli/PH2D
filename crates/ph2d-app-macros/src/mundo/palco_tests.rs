//! Os gates da reprodução do mundo (`docs/Macros/17_W19_plano.md` §6, #7 #9 #10 #11) — uma trilha à mão:
//! a imagem, depois `RigidBody`+`Collider` num passo, depois `Simula` de 120 tiques sobre um chão Static
//! FORA do objecto Macro.

use super::*;
use crate::mundo::formato::{Componente, Inicial, Linha, Objeto, Simula};
use crate::mundo::palco_fisica::DT;
use ph2d_core::Vec2;
use ph2d_ecs::scene::{WorldSnapshot, register_ecs_components, world_to_snapshot};
use ph2d_ecs::{ChildOf, Transform, TransformPropagationState, WorklistBuf};
use ph2d_macro::{Event, MacroDoc, RefRect, Track};
use ph2d_physics_ecs::{Collider, ColliderShape, InputTape, PhysicsBridge};

const DONO: u64 = 10;
const IMAGEM: u64 = 11;
const CHAO: u64 = 1;
/// Um objecto que nasce com o passo do collider.
const SOMBRA: u64 = 12;
const T_COLLIDER: u64 = 1_000_000;
const T_SIMULA: u64 = 2_000_000;
const TIQUES: u32 = 120;

fn b<T: serde::Serialize>(v: &T) -> Vec<u8> {
    postcard::to_allocvec(v).expect("postcard")
}

fn corpo() -> RigidBody {
    RigidBody {
        kind: BodyKind::Dynamic,
    }
}

fn bola() -> Collider {
    Collider {
        shape: ColliderShape::Ball { radius: 0.25 },
        ..Collider::default()
    }
}

fn alto() -> Transform {
    Transform::from_translation(Vec2::new(0.0, 5.0))
}

fn registo() -> ComponentRegistry {
    let mut r = ComponentRegistry::new();
    register_ecs_components(&mut r);
    ph2d_physics_ecs::register_physics_components(&mut r);
    r
}

/// O mundo do FIM (o que o aplicar deixa): o chão fora, o objecto Macro e a imagem-corpo dentro.
fn mundo(macro_doc: Option<Vec<u8>>) -> (SimWorld, Entity, Entity) {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    w.spawn((
        StableId(CHAO),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 50.0,
                half_y: 0.1,
            },
            ..Collider::default()
        },
        Transform::default(),
    ));
    let dono = w.spawn((StableId(DONO), Transform::default())).id();
    if let Some(doc) = macro_doc {
        w.entity_mut(dono).insert(Macro {
            doc,
            ..Macro::default()
        });
    }
    let img = w
        .spawn((StableId(IMAGEM), alto(), corpo(), bola(), ChildOf(dono)))
        .id();
    w.spawn((StableId(SOMBRA), Transform::default(), ChildOf(dono)));
    (sim, dono, img)
}

/// A corrida da gravação: a ponte global, um tique por quadro, a partir do FIM.
fn o_que_o_artista_viu() -> [f32; 3] {
    let (mut sim, _, img) = mundo(None);
    let mut ponte = PhysicsBridge::new();
    for k in 1..=u64::from(TIQUES) {
        ponte.dispatch(&mut sim, true, k);
    }
    let t = sim.world().get::<Transform>(img).expect("pose");
    [t.translation.x, t.translation.y, t.rotation]
}

fn doc(conferencia: [f32; 3]) -> Vec<u8> {
    let inicial = Inicial {
        objetos: vec![
            Objeto {
                id: 0,
                original: DONO,
                quem: Quem::Macro,
            },
            Objeto {
                id: 1,
                original: IMAGEM,
                quem: Quem::Mundo,
            },
        ],
        linhas: vec![
            Linha {
                id: 0,
                pai: None,
                componentes: vec![],
            },
            Linha {
                id: 1,
                pai: None,
                componentes: vec![Componente {
                    nome: TRANSFORM.into(),
                    bytes: b(&alto()),
                }],
            },
        ],
        recursos: vec![],
    };
    let collider = Evento::Passo {
        novos: vec![Objeto {
            id: 2,
            original: SOMBRA,
            quem: Quem::Mundo,
        }],
        mudancas: vec![
            Mudanca::Nasce(Linha {
                id: 2,
                pai: None,
                componentes: vec![Componente {
                    nome: TRANSFORM.into(),
                    bytes: b(&Transform::default()),
                }],
            }),
            Mudanca::Muda {
                id: 1,
                pai: None,
                componentes: vec![
                    (CORPO.into(), Some(b(&corpo()))),
                    ("ph2d::physics::Collider".into(), Some(b(&bola()))),
                ],
            },
        ],
        recursos: vec![],
    };
    let simula = Evento::Simula(Simula {
        de: 0,
        ate: TIQUES,
        fita: InputTape::new().to_wire(),
        conferencia: vec![(1, conferencia)],
    });
    let track = Track {
        family: super::super::formato::FAMILIA.into(),
        target: DONO,
        initial: inicial.escreve(),
        events: vec![
            Event {
                t: MacroTime(T_COLLIDER),
                payload: collider.escreve(),
            },
            Event {
                t: MacroTime(T_SIMULA),
                payload: simula.escreve(),
            },
        ],
        edicoes: vec![],
        nasce: None,
        continua: None,
    };
    ph2d_macro::codec::write(&MacroDoc {
        reference: RefRect { w: 1.0, h: 1.0 },
        tracks: vec![track],
    })
}

struct Cena {
    sim: SimWorld,
    reg: ComponentRegistry,
    dono: Entity,
    img: Entity,
    drive: PreviewDrive,
    palcos: PalcosDoMundo,
    global: PhysicsBridge,
}

impl Cena {
    fn nova() -> Self {
        let (sim, dono, img) = mundo(Some(doc(o_que_o_artista_viu())));
        Self {
            sim,
            reg: registo(),
            dono,
            img,
            drive: PreviewDrive::default(),
            palcos: PalcosDoMundo::default(),
            global: PhysicsBridge::new(),
        }
    }

    /// Um quadro com o clipe em `us`: a Timeline, a física global (se `fisica`, no tique `tique`), o palco.
    fn quadro(&mut self, us: u64, fisica: Option<u64>) {
        self.sim
            .world_mut()
            .entity_mut(self.dono)
            .insert(MacroTempo {
                segundos: us as f64 / 1e6,
            });
        if let Some(k) = fisica {
            let antes = *self.sim.world().get::<Transform>(self.img).expect("pose");
            self.global.dispatch(&mut self.sim, true, k);
            let agora = *self.sim.world().get::<Transform>(self.img).expect("pose");
            if agora != antes {
                self.drive.driven(
                    self.img,
                    ph2d_preview_drive::Driven::SolverPose(antes),
                    ph2d_preview_drive::Driven::SolverPose(agora),
                );
            }
        }
        let cx = Contexto::novo(&self.reg, PhysicsSettings::default());
        self.palcos.segue(&mut self.sim, &cx, &mut self.drive);
        self.drive.settle();
    }

    fn escondida(&mut self) -> bool {
        let mut q = self
            .sim
            .world_mut()
            .query::<(&StableId, Option<&Visibility>)>();
        q.iter(self.sim.world())
            .find(|(s, _)| s.0 == SOMBRA)
            .is_some_and(|(_, v)| v.is_some_and(|v| v.hidden))
    }

    fn pose(&self) -> [u32; 3] {
        let t = self.sim.world().get::<Transform>(self.img).expect("pose");
        [
            t.translation.x.to_bits(),
            t.translation.y.to_bits(),
            t.rotation.to_bits(),
        ]
    }

    fn captura(&mut self) -> WorldSnapshot {
        let mut prop = TransformPropagationState::new(self.sim.world_mut());
        let mut out = WorldSnapshot::new();
        world_to_snapshot(
            self.sim.world_mut(),
            &mut prop,
            &mut WorklistBuf::default(),
            &self.reg,
            &mut out,
        )
        .expect("captura");
        out
    }
}

fn bits(p: [f32; 3]) -> [u32; 3] {
    p.map(f32::to_bits)
}

/// O instante do clipe `k` tiques dentro do `Simula`.
fn no_tique(k: u32) -> u64 {
    T_SIMULA + (f64::from(k) * DT * 1e6).round() as u64
}

#[test]
fn antes_do_collider_o_corpo_nao_cai() {
    let mut c = Cena::nova();
    let doc_pose = c.pose();
    for (q, us) in [100_000u64, 300_000, 500_000, 900_000]
        .into_iter()
        .enumerate()
    {
        c.quadro(us, Some(q as u64 + 1));
        assert!(
            c.sim.world().get::<RigidBody>(c.img).is_none(),
            "em t={us} µs ainda não há corpo"
        );
        assert!(c.sim.world().get::<Collider>(c.img).is_none());
        assert_eq!(c.pose(), doc_pose, "em t={us} µs a pose não muda");
        assert!(
            c.escondida(),
            "em t={us} µs o objecto que nasce depois está escondido"
        );
    }
    // Entre o passo do collider e o Simula: corpo, mas da cena (cinemático) — não cai.
    c.quadro(1_500_000, Some(5));
    let k = c.sim.world().get::<RigidBody>(c.img).map(|r| r.kind);
    assert_eq!(k, Some(BodyKind::Kinematic));
    assert_eq!(c.pose(), doc_pose);
    assert!(!c.escondida(), "nasceu");
}

#[test]
fn recozer_o_simula_da_o_bit_que_o_artista_viu() {
    let viu = o_que_o_artista_viu();
    assert!(viu[1] < 4.0, "controlo: a gravação caiu ({viu:?})");
    let mut c = Cena::nova();
    c.quadro(no_tique(TIQUES), None);
    assert_eq!(
        c.pose(),
        bits(viu),
        "o fim do Simula recozido é o bit da conferência"
    );
    // E um quadro por tique (só os tiques novos) dá o mesmo.
    let mut d = Cena::nova();
    for k in 0..=TIQUES {
        d.quadro(no_tique(k), None);
    }
    assert_eq!(d.pose(), bits(viu));
    assert_eq!(d.palcos.recozido(d.dono, 0).map(|r| r.do_zero), Some(1));
    // Depois do evento a pose fica no `ate`.
    d.quadro(no_tique(TIQUES) + 3_000_000, None);
    assert_eq!(d.pose(), bits(viu));
}

#[test]
fn a_tocar_nada_entra_no_undo_nem_no_save() {
    let mut c = Cena::nova();
    let fim = c.captura();
    for us in [500_000, 1_500_000, no_tique(30), no_tique(60)] {
        c.quadro(us, None);
        let vivo_pose = c.pose();
        let vivo = c.drive.substitute(&mut c.sim);
        assert_eq!(c.captura(), fim, "a captura em t={us} µs é o FIM");
        PreviewDrive::restore(&mut c.sim, &vivo);
        assert_eq!(c.pose(), vivo_pose, "o restore devolve o vivo");
    }
    // O clipe parou: o mundo volta a ser o documento.
    c.sim.world_mut().entity_mut(c.dono).remove::<MacroTempo>();
    c.quadro_sem_tempo();
    assert_eq!(c.captura(), fim);
    assert!(c.drive.blobs.is_empty());
}

impl Cena {
    fn quadro_sem_tempo(&mut self) {
        let cx = Contexto::novo(&self.reg, PhysicsSettings::default());
        self.palcos.segue(&mut self.sim, &cx, &mut self.drive);
        self.drive.settle();
    }
}

#[test]
fn a_fisica_global_nao_escreve_por_cima_da_macro() {
    let mut so = Cena::nova();
    let mut com = Cena::nova();
    let fim = com.captura();
    for k in 0..=TIQUES {
        so.quadro(no_tique(k), None);
        com.quadro(no_tique(k), Some(u64::from(k) + 1));
        assert_eq!(com.pose(), so.pose(), "no tique {k} a pose é a do palco");
        let kind = com.sim.world().get::<RigidBody>(com.img).map(|r| r.kind);
        assert_eq!(
            kind,
            Some(BodyKind::Kinematic),
            "o corpo da macro é da cena enquanto toca"
        );
        let vivo = com.drive.substitute(&mut com.sim);
        assert_eq!(
            com.captura(),
            fim,
            "no tique {k} o que o solver global escreveu não entra no undo"
        );
        PreviewDrive::restore(&mut com.sim, &vivo);
    }
}

#[test]
fn o_scrub_para_tras_recoze_e_da_o_mesmo() {
    let mut c = Cena::nova();
    let um = T_SIMULA + 1_000_000;
    c.quadro(um, None);
    let a = c.pose();
    c.quadro(T_SIMULA + 500_000, None);
    let meio = c.pose();
    assert_ne!(meio, a, "controlo: a meio o corpo está noutro sítio");
    c.quadro(um, None);
    assert_eq!(c.pose(), a);
    let r = c.palcos.recozido(c.dono, 0).expect("palco");
    assert_eq!(
        (r.do_zero, r.retomas),
        (1, 1),
        "o scrub para trás retoma de um ponto"
    );
}

impl Cena {
    fn com_doc(doc: Vec<u8>) -> Self {
        let mut c = Self::nova();
        c.sim.world_mut().entity_mut(c.dono).insert(Macro {
            doc,
            ..Macro::default()
        });
        c
    }
}

#[test]
fn a_queda_que_deu_outro_resultado_diz_se() {
    let viu = o_que_o_artista_viu();
    let mut certo = Cena::nova();
    certo.quadro(no_tique(TIQUES), None);
    assert_eq!(
        certo.palcos.erro_de(certo.dono.to_bits()),
        None,
        "controlo: o bit confere"
    );
    let mut adulterada = viu;
    adulterada[1] += 0.01;
    let mut c = Cena::com_doc(doc(adulterada));
    c.quadro(no_tique(TIQUES / 2), None);
    assert_eq!(
        c.palcos.erro_de(c.dono.to_bits()),
        None,
        "a meio ainda não se confere"
    );
    c.quadro(no_tique(TIQUES), None);
    let e = c
        .palcos
        .erro_de(c.dono.to_bits())
        .expect("a conferência diz-se");
    assert_eq!(e.chave, ph2d_editor_core::macro_etapas::erros::MUNDO_DIFERE);
    let d = e.detalhe.expect("o objecto e a diferença");
    assert!(d.contains("0.0100 m"), "{d}");
    // E continua dito com o clipe a voltar atrás.
    c.quadro(no_tique(10), None);
    assert!(c.palcos.erro_de(c.dono.to_bits()).is_some());
}

#[test]
fn esperar_numa_etapa_do_mundo_atrasa_a_queda() {
    use ph2d_editor_core::macro_edits::CampoDaEtapa;
    use ph2d_editor_core::macro_etapas::EtapasDaFamilia;
    let viu = o_que_o_artista_viu();
    let mut d = ph2d_macro::codec::read(&doc(viu)).expect("doc");
    crate::etapas_mundo::EtapasDoMundo
        .edita(&mut d.tracks[0], 1, &CampoDaEtapa::Espera(1.0))
        .expect("a espera cabe");
    let mut c = Cena::com_doc(ph2d_macro::codec::write(&d));
    let mut sem = Cena::nova();
    let inicio = c.pose();
    c.quadro(no_tique(TIQUES / 2), None);
    sem.quadro(no_tique(TIQUES / 2), None);
    assert_ne!(sem.pose(), inicio, "controlo: sem a espera já está a cair");
    assert_eq!(c.pose(), inicio, "com 1 s de espera ainda não caiu");
    c.quadro(no_tique(TIQUES) + 1_000_000, None);
    assert_eq!(c.pose(), bits(viu), "a queda acontece inteira, 1 s depois");
}
