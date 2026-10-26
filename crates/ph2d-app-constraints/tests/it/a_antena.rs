//! ⭐ **A cena 5 (Antena), sem a shell** — a sonda do smoke (plano §6): a curva escreve o corpo
//! (`apply_from_doc`), a pilha corre ([`quadro`] com o relógio) e mede-se a ponta. Tocar quadro a
//! quadro e arrastar o tempo dão o MESMO bit; a mola atrasa e assenta.

use bevy_ecs::entity::Entity;
use ph2d_app_constraints::quadro;
use ph2d_app_constraints::relogio::Relogio;
use ph2d_app_constraints::smoke::{self, CORPO, PONTA_NO_CORPO};
use ph2d_app_constraints::smoke_antena::chaves_do_corpo;
use ph2d_constraints_ecs::{Constraints, Estado, Estados};
use ph2d_core::Playhead;
use ph2d_ecs::{Name, SimWorld, world_transform};
use ph2d_preview_drive::PreviewDrive;
use ph2d_timeline::{TimelineState, apply_from_doc, apply_intent};

struct Cena {
    sim: SimWorld,
    drive: PreviewDrive,
    tl: TimelineState,
    corpo: Entity,
    ponta: Entity,
    haste: Entity,
}

fn achar(sim: &mut SimWorld, nome: &str) -> Entity {
    let w = sim.world_mut();
    let mut q = w.query::<(Entity, &Name)>();
    q.iter(w)
        .find(|(_, n)| n.0 == nome)
        .map(|(e, _)| e)
        .unwrap()
}

impl Cena {
    fn nova() -> Self {
        let mut sim = SimWorld::new();
        assert!(smoke::encenar(sim.world_mut(), 5));
        let (corpo, ponta, haste) = (
            achar(&mut sim, "Corpo"),
            achar(&mut sim, "Ponta"),
            achar(&mut sim, "Haste"),
        );
        let (mut tl, mut ph) = (TimelineState::default(), Playhead::default());
        for i in chaves_do_corpo(corpo.to_bits(), tl.doc.fps_display) {
            apply_intent(&mut tl, &mut ph, i);
        }
        Self {
            sim,
            drive: PreviewDrive::default(),
            tl,
            corpo,
            ponta,
            haste,
        }
    }

    /// Um quadro do app no quadro `q`: devolve a ponta e o corpo no mundo.
    fn em(&mut self, q: i64) -> ([f32; 2], [f32; 2]) {
        #[expect(clippy::cast_precision_loss, reason = "quadros da cena")]
        let t = q as f64 / self.tl.doc.fps_display;
        apply_from_doc(self.sim.world_mut(), &mut self.tl.doc, t);
        let relogio = Relogio {
            doc: &self.tl.doc,
            t,
        };
        quadro(
            &mut self.sim,
            &ph2d_vec_scene::VecScene::new(),
            &mut self.drive,
            Some(relogio),
        );
        self.drive.settle();
        let p = |e| {
            let t = world_transform(self.sim.world(), e).unwrap();
            [t.translation.x, t.translation.y]
        };
        (p(self.ponta), p(self.corpo))
    }
}

#[test]
fn a_cena_5_toca_e_arrasta_igual() {
    let mut c = Cena::nova();
    let tocar: Vec<([f32; 2], [f32; 2])> = (0..=72).map(|q| c.em(q)).collect();
    let atraso = |(p, k): ([f32; 2], [f32; 2])| {
        (p[0] - k[0] - PONTA_NO_CORPO[0]).hypot(p[1] - k[1] - PONTA_NO_CORPO[1])
    };
    let maior = tocar.iter().map(|&v| atraso(v)).fold(0.0, f32::max);
    assert!(
        maior > 0.2,
        "a ponta fica para trás: atraso máximo {maior} m"
    );
    // O corpo pára no 48: no 72 (1 s depois) a mola assentou.
    assert_eq!(tocar[72].1, [CORPO[4].1, CORPO[4].2]);
    assert!(
        atraso(tocar[72]) < 0.05,
        "a mola assenta: {}",
        atraso(tocar[72])
    );
    let estados = c.sim.world().resource::<Estados>();
    for e in [c.ponta, c.haste] {
        let id = c.sim.world().get::<Constraints>(e).unwrap().items[0].id;
        assert_eq!(estados.of(e, id), Some(Estado::Activa));
    }
    // Arrastar o tempo — para trás, para a frente, saltos — dá o bit de tocar.
    for q in [40, 12, 30, 5, 48, 47, 72, 0, 60] {
        let got = c.em(q);
        assert_eq!(
            got.0.map(f32::to_bits),
            tocar[q as usize].0.map(f32::to_bits),
            "quadro {q}: {got:?} ≠ {:?}",
            tocar[q as usize]
        );
    }
}
