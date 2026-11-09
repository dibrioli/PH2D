//! A lei pelo ponteiro: em ecrã, com a câmera real, nas poses que a ficha pede.

use super::*;
use crate::tinta::contorno;
use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, Transform};

const WIN: WindowSize = WindowSize {
    width: 1600,
    height: 1000,
};
const ZOOMS: [f32; 3] = [4.0, 15.0, 60.0];
const CAIXA: ColliderShape = ColliderShape::Cuboid {
    half_x: 0.5,
    half_y: 0.25,
};
const BOLA: ColliderShape = ColliderShape::Ball { radius: 0.4 };
const CAPSULA: ColliderShape = ColliderShape::Capsule {
    half_height: 0.3,
    radius: 0.2,
};

/// Nome, posição, rotação, escala, com pai.
type PoseDoTeste = (&'static str, [f32; 2], f32, [f32; 2], bool);

/// As poses da ficha. A última é filha de um pai rodado, fora da origem.
fn poses() -> Vec<PoseDoTeste> {
    vec![
        ("identidade", [0.0, 0.0], 0.0, [1.0, 1.0], false),
        (
            "escala 2×1 a 30°",
            [0.3, -0.2],
            30f32.to_radians(),
            [2.0, 1.0],
            false,
        ),
        (
            "espelho x",
            [0.3, -0.2],
            30f32.to_radians(),
            [-2.0, 1.0],
            false,
        ),
        (
            "espelho y",
            [-0.4, 0.1],
            -70f32.to_radians(),
            [1.5, -0.5],
            false,
        ),
        (
            "filho de pai fora da origem",
            [0.5, 0.25],
            15f32.to_radians(),
            [1.0, 1.0],
            true,
        ),
    ]
}

struct Cena {
    world: World,
    e: Entity,
    bus: ph2d_editor_core::action_bus::ActionBus,
    cam: Camera2d,
}

impl Cena {
    fn nova(pose: PoseDoTeste, shape: ColliderShape, zoom: f32) -> Self {
        let (_, t, rot, s, com_pai) = pose;
        let mut world = World::new();
        let mut tr = Transform::from_translation(Vec2::new(t[0], t[1]));
        tr.rotation = rot;
        tr.scale = Vec2::new(s[0], s[1]);
        let col = Collider {
            shape,
            offset: [0.15, -0.1],
            ..Collider::default()
        };
        let e = world.spawn((tr, col)).id();
        if com_pai {
            let mut p = Transform::from_translation(Vec2::new(1.5, 0.75));
            p.rotation = 40f32.to_radians();
            p.scale = Vec2::new(1.5, 1.5);
            let pai = world.spawn(p).id();
            world.entity_mut(e).insert(ChildOf(pai));
        }
        alterna(&mut world, e.to_bits());
        let centro = Pose::de(&world, e).expect("pose").t;
        Self {
            world,
            e,
            bus: Default::default(),
            cam: Camera2d::new(centro, zoom),
        }
    }

    fn medida(&self) -> Medida {
        Medida::de(self.world.get::<Collider>(self.e).expect("col"))
    }

    fn pose(&self) -> Pose {
        Pose::de(&self.world, self.e).expect("pose")
    }

    fn ecra(&self, a: Alca) -> (f32, f32) {
        no_ecra(&self.pose(), &self.medida(), a, &self.cam, WIN)
    }

    /// O funil do cartão: cada edição do barramento chega ao `Collider`, com o piso da lei.
    fn escoa(&mut self) {
        for a in self.bus.drain() {
            let ph2d_editor_core::action_bus::EditorAction::InspectorPhysicsEdit {
                edit: PhysicsFieldEdit::ShapeDrag { medidas, offset },
                ..
            } = a
            else {
                panic!("só a edição das alças: {a:?}");
            };
            let mut c = self.world.get_mut::<Collider>(self.e).expect("col");
            c.offset = offset;
            c.shape = match c.shape {
                ColliderShape::Cuboid { .. } => ColliderShape::Cuboid {
                    half_x: medidas[0],
                    half_y: medidas[1],
                },
                ColliderShape::Outline { .. } => ColliderShape::Outline {
                    half_x: medidas[0],
                    half_y: medidas[1],
                },
                ColliderShape::Ball { .. } => ColliderShape::Ball { radius: medidas[0] },
                ColliderShape::Capsule { .. } => ColliderShape::Capsule {
                    half_height: medidas[1],
                    radius: medidas[0],
                },
            };
            c.sanitize();
        }
    }

    fn porta(&mut self) -> Porta<'_> {
        Porta {
            camera: self.cam,
            window: WIN,
            world: &mut self.world,
            bus: &mut self.bus,
        }
    }

    fn desce(&mut self, s: (f32, f32)) -> bool {
        self.porta().desce(s)
    }

    fn move_(&mut self, s: (f32, f32), t: Teclas) {
        assert!(self.porta().move_(s, t));
        self.escoa();
    }
}

/// `a` e `b` são o mesmo ponto de ecrã, à folga de um `f32` que passou pela câmera.
fn perto(a: (f32, f32), b: (f32, f32), o_que: &str) {
    let d = (a.0 - b.0).hypot(a.1 - b.1);
    assert!(d < 0.05, "{o_que}: {a:?} × {b:?} ({d} px)");
}

/// O caminho do ponteiro: `px` píxeis de ecrã ao longo do eixo da alça, para fora da forma.
fn para_fora(c: &Cena, a: Alca, px: f32) -> (f32, f32) {
    let de = c.ecra(a);
    let centro = c.ecra(Alca::Miolo);
    let (dx, dy) = (de.0 - centro.0, de.1 - centro.1);
    let n = dx.hypot(dy);
    (de.0 + dx / n * px, de.1 + dy / n * px)
}

/// O lado oposto a `a` em ecrã: é o que tem de ficar parado.
fn oposta(a: Alca) -> Alca {
    match a {
        Alca::Miolo => Alca::Miolo,
        Alca::Lado(x, y) => Alca::Lado(-x, -y),
    }
}

/// Toda alça de caixa, em toda pose e zoom: o dedo acerta em ecrã, a alça vai para debaixo do ponteiro e a oposta
/// não se mexe. Corre com o Offset de partida não nulo: é aí que a compensação aparece.
#[test]
fn cada_alca_da_caixa_vai_para_debaixo_do_ponteiro_em_toda_pose_e_zoom() {
    for pose in poses() {
        for zoom in ZOOMS {
            for a in alcas(CAIXA) {
                if a == Alca::Miolo {
                    continue;
                }
                let mut c = Cena::nova(pose, CAIXA, zoom);
                let de = c.ecra(a);
                let parada = c.ecra(oposta(a));
                assert_eq!(
                    alca_em(&c.pose(), &c.medida(), &c.cam, WIN, de),
                    Some(a),
                    "{} zoom {zoom}: o dedo não acerta {a:?}",
                    pose.0
                );
                let alvo = para_fora(&c, a, 40.0);
                assert!(c.desce(de));
                c.move_(alvo, Teclas::default());
                let quem = format!("{} zoom {zoom} {a:?}", pose.0);
                perto(c.ecra(a), alvo, &quem);
                perto(
                    c.ecra(oposta(a)),
                    parada,
                    &format!("{quem}: a oposta andou"),
                );
                // A figura desenhada passa pela alça: nunca uma alça deslocada da forma.
                let bb = ph2d_vector::Shape::bounding_box(&contorno(
                    &c.pose(),
                    &c.medida(),
                    &c.cam,
                    WIN,
                ));
                let s = c.ecra(a);
                assert!(
                    bb.inflate(0.1, 0.1)
                        .contains((f64::from(s.0), f64::from(s.1))),
                    "{quem}: a alça fora do contorno"
                );
            }
        }
    }
}

#[test]
fn a_alca_da_bola_e_da_capsula_vao_para_debaixo_do_ponteiro_e_o_oposto_fica() {
    for pose in poses() {
        for zoom in ZOOMS {
            for shape in [BOLA, CAPSULA] {
                for a in alcas(shape) {
                    if a == Alca::Miolo {
                        continue;
                    }
                    let mut c = Cena::nova(pose, shape, zoom);
                    let de = c.ecra(a);
                    // A alça lateral da cápsula tem curso até à soma: 8 px ficam dentro dela em qualquer zoom?
                    let alvo = para_fora(
                        &c,
                        a,
                        if matches!(a, Alca::Lado(0, _)) {
                            40.0
                        } else {
                            -3.0
                        },
                    );
                    assert!(c.desce(de));
                    c.move_(alvo, Teclas::default());
                    perto(
                        c.ecra(a),
                        alvo,
                        &format!("{} zoom {zoom} {shape:?} {a:?}", pose.0),
                    );
                }
            }
        }
    }
}

/// O topo mexe na altura e a base fica; a soma e o Radius continuam a valer.
#[test]
fn o_topo_da_capsula_mexe_so_na_altura_e_a_base_fica() {
    for pose in poses() {
        let mut c = Cena::nova(pose, CAPSULA, 15.0);
        let topo = Alca::Lado(0, 1);
        let base = c.ecra(Alca::Lado(0, -1));
        let alvo = para_fora(&c, topo, 30.0);
        assert!(c.desce(c.ecra(topo)));
        c.move_(alvo, Teclas::default());
        perto(c.ecra(Alca::Lado(0, -1)), base, "a base andou");
        let ColliderShape::Capsule {
            radius,
            half_height,
        } = c.medida().shape
        else {
            panic!("deixou de ser cápsula");
        };
        assert_eq!(radius, 0.2, "o Radius não é desta alça");
        assert!(half_height > 0.3);
    }
}

/// O Radius puxado deixa a Height como estava, e o piso é a bola: a soma nunca fica menor que 2 × Radius.
#[test]
fn o_radius_da_capsula_mantem_a_height_e_o_piso_e_a_bola() {
    let total = |m: &Medida| match m.shape {
        ColliderShape::Capsule {
            half_height,
            radius,
        } => half_height + radius,
        _ => panic!("deixou de ser cápsula"),
    };
    let mut c = Cena::nova(poses()[0], CAPSULA, 15.0);
    let a = Alca::Lado(1, 0);
    let antes = total(&c.medida());
    assert!(c.desce(c.ecra(a)));
    c.move_(para_fora(&c, a, 500.0), Teclas::default());
    assert!((total(&c.medida()) - antes).abs() < 1e-5);
    assert_eq!(
        c.medida().shape,
        ColliderShape::Capsule {
            half_height: 0.0,
            radius: antes
        },
        "o piso: uma bola"
    );
    // A base puxada para lá do topo pára na cápsula sem segmento.
    let mut c = Cena::nova(poses()[1], CAPSULA, 15.0);
    let a = Alca::Lado(0, -1);
    assert!(c.desce(c.ecra(a)));
    c.move_(para_fora(&c, a, -5000.0), Teclas::default());
    assert_eq!(
        c.medida().shape,
        ColliderShape::Capsule {
            half_height: 0.0,
            radius: 0.2
        }
    );
}

/// Atravessar o lado oposto nunca dá medida negativa nem zero: pára no piso da lei do campo.
#[test]
fn puxar_para_la_do_oposto_para_no_piso_da_lei() {
    for pose in poses() {
        let mut c = Cena::nova(pose, CAIXA, 15.0);
        let a = Alca::Lado(1, 1);
        let parada = c.ecra(oposta(a));
        assert!(c.desce(c.ecra(a)));
        c.move_(para_fora(&c, a, -5000.0), Teclas::default());
        let ColliderShape::Cuboid { half_x, half_y } = c.medida().shape else {
            panic!();
        };
        assert!((half_x - MIN_EXTENT).abs() < 1e-5, "{half_x}");
        assert!((half_y - MIN_EXTENT).abs() < 1e-5, "{half_y}");
        perto(c.ecra(oposta(a)), parada, "o oposto andou ao bater no piso");
    }
    let mut c = Cena::nova(poses()[0], BOLA, 15.0);
    let a = Alca::Lado(1, 0);
    assert!(c.desce(c.ecra(a)));
    c.move_(para_fora(&c, a, -9000.0), Teclas::default());
    assert_eq!(c.medida().shape, ColliderShape::Ball { radius: MIN_EXTENT });
}

#[test]
fn o_miolo_arrasta_so_o_offset_e_a_forma_anda_com_o_ponteiro() {
    for pose in poses() {
        for shape in [CAIXA, BOLA, CAPSULA] {
            let mut c = Cena::nova(pose, shape, 15.0);
            let de = c.ecra(Alca::Miolo);
            assert_eq!(
                alca_em(&c.pose(), &c.medida(), &c.cam, WIN, de),
                Some(Alca::Miolo)
            );
            let alvo = (de.0 + 33.0, de.1 - 21.0);
            assert!(c.desce(de));
            c.move_(alvo, Teclas::default());
            assert_eq!(c.medida().shape, shape, "{}: a medida mudou", pose.0);
            perto(c.ecra(Alca::Miolo), alvo, pose.0);
        }
    }
}

/// Alt: simétrico a partir do centro, o Offset fica.
#[test]
fn alt_puxa_simetrico_e_o_offset_fica() {
    for pose in poses() {
        let mut c = Cena::nova(pose, CAIXA, 15.0);
        let a = Alca::Lado(1, 0);
        let centro = c.ecra(Alca::Miolo);
        let offset = c.medida().offset;
        let alvo = para_fora(&c, a, 25.0);
        assert!(c.desce(c.ecra(a)));
        c.move_(
            alvo,
            Teclas {
                alt: true,
                shift: false,
            },
        );
        assert_eq!(c.medida().offset, offset, "{}", pose.0);
        perto(c.ecra(a), alvo, pose.0);
        perto(c.ecra(Alca::Miolo), centro, pose.0);
        let d0 = (c.ecra(a).0 - centro.0).hypot(c.ecra(a).1 - centro.1);
        let e = c.ecra(oposta(a));
        let d1 = (e.0 - centro.0).hypot(e.1 - centro.1);
        assert!((d0 - d1).abs() < 0.05, "{}: {d0} × {d1}", pose.0);
    }
    let mut c = Cena::nova(poses()[0], CAPSULA, 15.0);
    let a = Alca::Lado(0, 1);
    let offset = c.medida().offset;
    let alvo = para_fora(&c, a, 25.0);
    assert!(c.desce(c.ecra(a)));
    c.move_(
        alvo,
        Teclas {
            alt: true,
            shift: false,
        },
    );
    assert_eq!(c.medida().offset, offset);
}

/// Shift no canto mantém a proporção de partida; o canto oposto fica. Com Alt, o centro fica.
#[test]
fn shift_no_canto_mantem_a_proporcao() {
    for pose in poses() {
        for alt in [false, true] {
            let mut c = Cena::nova(pose, CAIXA, 15.0);
            let a = Alca::Lado(1, 1);
            let parada = c.ecra(oposta(a));
            let centro = c.ecra(Alca::Miolo);
            // Só ao longo do eixo X local: sem a lei do Shift, a altura ficava e a proporção mudava.
            let (de, lado) = (c.ecra(a), para_fora(&c, Alca::Lado(1, 0), 60.0));
            let borda = c.ecra(Alca::Lado(1, 0));
            let alvo = (de.0 + lado.0 - borda.0, de.1 + lado.1 - borda.1);
            assert!(c.desce(de));
            c.move_(alvo, Teclas { alt, shift: true });
            let ColliderShape::Cuboid { half_x, half_y } = c.medida().shape else {
                panic!();
            };
            assert!(half_x > 0.5, "{}: cresceu", pose.0);
            assert!(
                (half_x / half_y - 2.0).abs() < 1e-4,
                "{}: {half_x}/{half_y}",
                pose.0
            );
            if alt {
                perto(c.ecra(Alca::Miolo), centro, "Alt+Shift: o centro");
            } else {
                perto(c.ecra(oposta(a)), parada, "Shift: o oposto");
            }
        }
    }
}

/// Shift num lado não muda nada: só o canto tem proporção a guardar.
#[test]
fn shift_no_lado_e_o_lado_normal() {
    let mut c = Cena::nova(poses()[1], CAIXA, 15.0);
    let a = Alca::Lado(1, 0);
    let alvo = para_fora(&c, a, 25.0);
    assert!(c.desce(c.ecra(a)));
    c.move_(
        alvo,
        Teclas {
            alt: false,
            shift: true,
        },
    );
    perto(c.ecra(a), alvo, "o lado");
    let ColliderShape::Cuboid { half_y, .. } = c.medida().shape else {
        panic!();
    };
    assert_eq!(half_y, 0.25);
}

/// O dedo mede-se em ECRÃ: o mesmo raio em píxeis a qualquer zoom.
#[test]
fn o_dedo_tem_o_mesmo_tamanho_em_ecra_em_qualquer_zoom() {
    let a = Alca::Lado(1, 1);
    for zoom in ZOOMS {
        // Sem escala: o canto fica num sítio onde o fora do dedo é fora da forma.
        let c = Cena::nova(poses()[0], CAIXA, zoom);
        let (p, m) = (c.pose(), c.medida());
        let s = c.ecra(a);
        let (dentro_px, fora_px) = (AGARRE_PX - 0.5, AGARRE_PX + 0.5);
        assert_eq!(
            alca_em(&p, &m, &c.cam, WIN, (s.0 + dentro_px, s.1)),
            Some(a),
            "zoom {zoom}"
        );
        assert_eq!(
            alca_em(&p, &m, &c.cam, WIN, (s.0 + fora_px, s.1)),
            None,
            "zoom {zoom}"
        );
    }
}

/// Esc a meio devolve a forma do `Down` e deixa a edição acesa; o segundo Esc encerra. Sem edição não consome.
#[test]
fn esc_a_meio_devolve_a_forma_e_o_segundo_encerra() {
    let mut c = Cena::nova(poses()[1], CAIXA, 15.0);
    let antes = c.medida();
    let a = Alca::Lado(1, 1);
    assert!(c.desce(c.ecra(a)));
    c.move_(para_fora(&c, a, 40.0), Teclas::default());
    assert_ne!(c.medida(), antes);
    assert!(c.porta().esc());
    c.escoa();
    assert_eq!(c.medida(), antes, "voltou ao Down");
    assert!(ativo(&c.world), "a edição continua");
    assert!(!c.porta().sobe(), "o arrasto acabou no Esc");
    assert!(!c.porta().move_((1.0, 1.0), Teclas::default()));
    assert_eq!(c.bus.drain().count(), 0);
    assert!(c.porta().esc());
    assert!(!ativo(&c.world), "encerrou");
    assert!(!c.porta().esc(), "sem edição, o Esc segue para os outros");
}

/// Um clique parado não escreve nada: nem um passo de undo por engano.
#[test]
fn clicar_sem_mexer_nao_escreve() {
    let mut c = Cena::nova(poses()[0], CAIXA, 15.0);
    let s = c.ecra(Alca::Lado(1, 0));
    assert!(c.desce(s));
    assert!(c.porta().move_((s.0 + 0.5, s.1), Teclas::default()));
    assert!(c.porta().sobe());
    assert_eq!(c.bus.drain().count(), 0);
}

/// O miolo sem mexer é um clique; arrastado, ou numa alça, não.
#[test]
fn so_o_miolo_parado_e_um_clique() {
    let mut c = Cena::nova(poses()[0], CAIXA, 15.0);
    let s = c.ecra(Alca::Miolo);
    assert!(c.desce(s));
    assert!(c.porta().move_((s.0 + 0.5, s.1), Teclas::default()));
    assert_eq!(crate::clique_parado(&c.world), Some(s));
    assert!(!crate::a_meio(&c.world));
    c.move_((s.0 + 10.0, s.1), Teclas::default());
    assert_eq!(crate::clique_parado(&c.world), None, "arrastou");
    assert!(crate::a_meio(&c.world));
    assert!(c.porta().sobe());
    let lado = c.ecra(Alca::Lado(1, 0));
    assert!(c.desce(lado));
    assert_eq!(crate::clique_parado(&c.world), None, "numa alça fica");
}

/// O cursor segue a alça no ecrã: com a forma a 90°, o lado direito sobe e o gesto é vertical.
#[test]
fn o_cursor_da_alca_segue_a_rodacao() {
    use crate::CursorDaAlca as K;
    let reta = Cena::nova(("reta", [0.0, 0.0], 0.0, [1.0, 1.0], false), CAIXA, 15.0);
    let de = |c: &Cena, a| crate::cursor_em(&c.world, &c.cam, WIN, c.ecra(a));
    assert_eq!(de(&reta, Alca::Lado(1, 0)), Some(K::Horizontal));
    assert_eq!(de(&reta, Alca::Lado(0, 1)), Some(K::Vertical));
    assert_eq!(de(&reta, Alca::Lado(1, 1)), Some(K::Sobe));
    assert_eq!(de(&reta, Alca::Lado(-1, 1)), Some(K::Desce));
    let longe = reta.ecra(Alca::Lado(1, 1));
    let fora = crate::cursor_em(&reta.world, &reta.cam, WIN, (longe.0 + 200.0, longe.1));
    assert_eq!(fora, None, "fora da forma, o cursor é de quem o pedir");
    let rodada = Cena::nova(
        (
            "rodada",
            [0.0, 0.0],
            std::f32::consts::FRAC_PI_2,
            [1.0, 1.0],
            false,
        ),
        CAIXA,
        15.0,
    );
    assert_eq!(de(&rodada, Alca::Lado(1, 0)), Some(K::Vertical));
    let s = reta.ecra(Alca::Miolo);
    assert_eq!(
        crate::cursor_em(&reta.world, &reta.cam, WIN, s),
        Some(K::Mover)
    );
}

/// Fora da forma e das alças o `Down` segue para o canvas.
#[test]
fn fora_das_alcas_o_canvas_continua_a_funcionar() {
    let mut c = Cena::nova(poses()[0], CAIXA, 15.0);
    let s = c.ecra(Alca::Lado(1, 1));
    assert!(!c.desce((s.0 + 200.0, s.1 + 200.0)));
    let mut sem = Cena::nova(poses()[0], CAIXA, 15.0);
    alterna(&mut sem.world, sem.e.to_bits());
    assert!(!ativo(&sem.world), "clicar de novo encerra");
    assert!(!sem.desce(s), "sem edição não há alças");
}

/// Trocar de objecto, dar Play ou perder a forma encerram.
#[test]
fn trocar_de_objeto_dar_play_ou_perder_a_forma_encerram() {
    let c = Cena::nova(poses()[0], CAIXA, 15.0);
    let (mut w, e) = (c.world, c.e);
    assert!(quadro(&mut w, Some(e.to_bits()), false));
    assert!(!quadro(&mut w, Some(e.to_bits()), true), "Play");
    assert!(!ativo(&w));
    alterna(&mut w, e.to_bits());
    assert!(!quadro(&mut w, None, false), "outra seleção");
    alterna(&mut w, e.to_bits());
    w.entity_mut(e).remove::<Collider>();
    assert!(!quadro(&mut w, Some(e.to_bits()), false), "sem forma");
}

/// O pai andou entre o `Down` e o `Move`: a alça continua a ir para debaixo do ponteiro.
#[test]
fn pai_que_anda_a_meio_do_arrasto_nao_tira_a_alca_do_ponteiro() {
    let mut c = Cena::nova(poses()[4], CAIXA, 15.0);
    let a = Alca::Lado(1, 1);
    assert!(c.desce(c.ecra(a)));
    let pai = c.world.get::<ChildOf>(c.e).expect("pai").0;
    c.world
        .get_mut::<Transform>(pai)
        .expect("pai")
        .translation
        .x += 0.5;
    // O ponteiro segue a alça de onde ela ficou com o pai novo, e puxa o resto ao longo do eixo.
    let alvo = para_fora(&c, a, 40.0);
    c.move_(alvo, Teclas::default());
    perto(c.ecra(a), alvo, "com o pai noutro sítio");
}

/// Escala nula não dá alça: não há ponto do ecrã que volte à forma.
#[test]
fn escala_nula_nao_oferece_alcas() {
    let mut c = Cena::nova(poses()[0], CAIXA, 15.0);
    c.world.get_mut::<Transform>(c.e).expect("t").scale = Vec2::new(0.0, 1.0);
    assert!(!c.desce((800.0, 500.0)));
}

/// Ctrl+Z que refaz o objecto com outros bits: a mesma identidade durável, a edição continua; outro objecto encerra.
#[test]
fn o_mesmo_objeto_refeito_pelo_undo_continua_em_edicao() {
    let mut w = World::new();
    let id = ph2d_ecs::StableId(7);
    let col = Collider::default();
    let e = w.spawn((Transform::default(), col, id)).id();
    alterna(&mut w, e.to_bits());
    let refeito = w.spawn((Transform::default(), col, id)).id();
    assert!(quadro(&mut w, Some(refeito.to_bits()), false));
    assert_eq!(editando(&w), Some(refeito.to_bits()));
    let outro = w
        .spawn((Transform::default(), col, ph2d_ecs::StableId(8)))
        .id();
    assert!(!quadro(&mut w, Some(outro.to_bits()), false));
}

/// Durante o arrasto a mão da família segura o objecto (a timeline não o conduz, o Auto Key espera o soltar).
#[test]
fn a_mao_da_familia_segura_so_durante_o_arrasto() {
    let segura = |c: &Cena| {
        c.world
            .get_resource::<MaosDasFamilias>()
            .is_some_and(MaosDasFamilias::alguma)
    };
    use ph2d_ecs::MaosDasFamilias;
    for esc in [false, true] {
        let mut c = Cena::nova(poses()[1], CAIXA, 15.0);
        let a = Alca::Lado(1, 1);
        assert!(c.desce(c.ecra(a)));
        assert!(!segura(&c), "um clique parado não é gesto");
        c.move_(para_fora(&c, a, 40.0), Teclas::default());
        assert!(segura(&c), "o arrasto segura");
        if esc {
            assert!(c.porta().esc());
        } else {
            assert!(c.porta().sobe());
        }
        assert!(!segura(&c), "acabou");
    }
}

/// ⭐ (12) **Com os pontos em edição a caixa da imagem recua**: tracejada em `Text3`, nunca o realce do caminho;
/// sem pontos volta à `Selection` cheia. *Mutação: a caixa sempre `Selection` ⇒ vermelho.*
#[test]
fn com_os_pontos_a_caixa_da_imagem_recua() {
    use crate::tinta::tinta_da_caixa;
    use ph2d_tokens::ColorToken as T;
    assert_eq!(tinta_da_caixa(true), (T::Text3, true));
    assert_eq!(tinta_da_caixa(false), (T::Selection, false));
    assert_ne!(tinta_da_caixa(true).0, T::Accent, "o realce é do caminho");
}
