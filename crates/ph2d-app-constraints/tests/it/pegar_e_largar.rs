//! ⭐ **Pegar e largar com o Auto Key** (cena 1) — o laço do quadro sem a shell: a curva escreve
//! (`apply_from_doc`), a pilha corre ([`quadro`]), o cartão pega e larga ([`apply_all`]) e as chaves de
//! compensação ([`compensar`]) entram no documento. Depois arrasta-se o tempo e mede-se.

use bevy_ecs::entity::Entity;
use ph2d_app_constraints::cena_pega::{VISITAS, chaves_da_mao};
use ph2d_app_constraints::compensa::compensar;
use ph2d_app_constraints::inspector::apply_all;
use ph2d_app_constraints::quadro;
use ph2d_app_constraints::smoke::{self, ESPADA_NO_CHAO, QUADRO_LARGA, QUADRO_PEGA};
use ph2d_constraints_ecs::Constraints;
use ph2d_core::Playhead;
use ph2d_ecs::{Name, SimWorld, world_transform};
use ph2d_editor_core::constraint_edits::ConstraintFieldEdit as E;
use ph2d_preview_drive::PreviewDrive;
use ph2d_timeline::{Instante, Interp, TimelineState, apply_from_doc, apply_intent};

struct Cena {
    sim: SimWorld,
    drive: PreviewDrive,
    tl: TimelineState,
    ph: Playhead,
    espada: Entity,
    item: u32,
}

impl Cena {
    fn nova() -> Self {
        let mut sim = SimWorld::new();
        assert!(smoke::encenar(sim.world_mut(), 1));
        let achar = |sim: &mut SimWorld, nome: &str| {
            let w = sim.world_mut();
            let mut q = w.query::<(Entity, &Name)>();
            q.iter(w)
                .find(|(_, n)| n.0 == nome)
                .map(|(e, _)| e)
                .unwrap()
        };
        let (mao, espada) = (achar(&mut sim, "Mão"), achar(&mut sim, "Espada"));
        let item = sim.world().get::<Constraints>(espada).unwrap().items[0].id;
        let (mut tl, mut ph) = (TimelineState::default(), Playhead::default());
        for i in chaves_da_mao(mao.to_bits(), tl.doc.fps_display) {
            apply_intent(&mut tl, &mut ph, i);
        }
        tl.flags.auto_key = true;
        Self {
            sim,
            drive: PreviewDrive::default(),
            tl,
            ph,
            espada,
            item,
        }
    }

    /// Um quadro do app no quadro `q` da timeline; devolve onde a espada se vê.
    fn em(&mut self, q: i64) -> [f32; 2] {
        self.ph.seek_frame(q, self.tl.doc.fps_display);
        apply_from_doc(self.sim.world_mut(), &mut self.tl.doc, self.ph.time());
        quadro(&mut self.sim, &mut self.drive);
        self.drive.settle();
        let t = world_transform(self.sim.world(), self.espada).unwrap();
        [t.translation.x, t.translation.y]
    }

    /// O clique na força do cartão, e as chaves que o Auto Key do mesmo quadro grava.
    fn forca(&mut self, v: f32) {
        assert!(apply_all(
            &mut self.sim,
            &[(self.espada.to_bits(), E::Forca(self.item, v))]
        ));
        let inst = Instante {
            t_cut: self.ph.time(),
            solo: false,
        };
        let chaves = compensar(
            &self.tl,
            true,
            &mut self.sim,
            &self.drive,
            inst,
            Interp::Linear,
        );
        assert!(!chaves.is_empty(), "o Auto Key armado não deu chaves");
        for i in chaves {
            apply_intent(&mut self.tl, &mut self.ph, i);
        }
    }
}

fn longe(a: [f32; 2], b: [f32; 2]) -> f32 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

#[test]
fn picking_and_dropping_with_auto_key_never_jumps_and_keeps_the_past() {
    let mut c = Cena::nova();
    for q in 0..=QUADRO_PEGA {
        c.em(q);
    }
    let pega = c.em(QUADRO_PEGA);
    assert!(longe(pega, ESPADA_NO_CHAO) < 1e-6);
    c.forca(1.0);
    for q in QUADRO_PEGA..=QUADRO_LARGA {
        c.em(q);
    }
    let larga = c.em(QUADRO_LARGA);
    assert!(
        longe(larga, ESPADA_NO_CHAO) > 1.0,
        "a mão levou a espada: {larga:?}"
    );
    c.forca(0.0);
    let mut vistas = Vec::new();
    for _ in 0..2 {
        for &q in &VISITAS {
            vistas.push((q, c.em(q)));
        }
    }
    for (q, v) in vistas {
        let esperado = match q {
            q if q <= QUADRO_PEGA => Some(pega),
            q if q >= QUADRO_LARGA => Some(larga),
            _ => None,
        };
        match esperado {
            Some(e) => assert!(longe(v, e) < 1e-4, "no {q} a espada saltou: {v:?} × {e:?}"),
            None => assert!(longe(v, pega) > 0.5, "no {q} a mão já não a leva: {v:?}"),
        }
    }
}

/// Sem o Auto Key o gesto não grava nada — e o registo esvazia (não fica para o próximo quadro).
#[test]
fn without_auto_key_the_gesture_keys_nothing_and_is_forgotten() {
    let mut c = Cena::nova();
    c.tl.flags.auto_key = false;
    c.em(QUADRO_PEGA);
    assert!(apply_all(
        &mut c.sim,
        &[(c.espada.to_bits(), E::Forca(c.item, 1.0))]
    ));
    let inst = Instante {
        t_cut: c.ph.time(),
        solo: false,
    };
    assert!(compensar(&c.tl, true, &mut c.sim, &c.drive, inst, Interp::Linear).is_empty());
    c.tl.flags.auto_key = true;
    assert!(compensar(&c.tl, true, &mut c.sim, &c.drive, inst, Interp::Linear).is_empty());
}
