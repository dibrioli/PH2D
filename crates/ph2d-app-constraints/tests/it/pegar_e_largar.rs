//! ⭐ **Pegar, largar e cozer** (cena 1) — o laço do quadro sem a shell: a curva escreve
//! (`apply_from_doc`), a pilha corre ([`quadro`]), o cartão pede ([`apply_all`]) e a porta das chaves
//! ([`do_quadro`]) grava no documento. Depois arrasta-se o tempo e mede-se.

use bevy_ecs::entity::Entity;
use ph2d_app_constraints::chaves::do_quadro;
use ph2d_app_constraints::inspector::apply_all;
use ph2d_app_constraints::quadro;
use ph2d_app_constraints::smoke::{self, ESPADA_NO_CHAO, QUADRO_LARGA, QUADRO_PEGA};
use ph2d_app_constraints::smoke_pega::{VISITAS, chaves_da_mao};
use ph2d_constraints_ecs::{Constraints, Estado, Estados};
use ph2d_core::Playhead;
use ph2d_ecs::canal_constraint::{FORCA, chave};
use ph2d_ecs::{Name, SimWorld, Transform, world_transform};
use ph2d_editor_core::constraint_edits::ConstraintFieldEdit as E;
use ph2d_preview_drive::PreviewDrive;
use ph2d_timeline::{
    Instante, Interp, PropKind, TimelineIntent, TimelineState, apply_from_doc, apply_intent,
};

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

    /// Um quadro do app no quadro `q` da timeline; devolve a pose MUNDIAL da espada.
    fn em(&mut self, q: i64) -> Transform {
        #[expect(clippy::cast_precision_loss, reason = "quadros da cena")]
        self.em_t(q as f64 / self.tl.doc.fps_display)
    }

    /// O mesmo no tempo `t` (segundos) — entre quadros, onde a interpolação de uma chave se vê.
    fn em_t(&mut self, t: f64) -> Transform {
        self.ph.seek(t);
        apply_from_doc(self.sim.world_mut(), &mut self.tl.doc, self.ph.time());
        quadro(&mut self.sim, &mut self.drive);
        self.drive.settle();
        world_transform(self.sim.world(), self.espada).unwrap()
    }

    /// Um pedido do cartão e a porta das chaves no mesmo quadro (o Auto Key com o painel aberto).
    fn cartao(&mut self, edit: E) {
        apply_all(&mut self.sim, &[(self.espada.to_bits(), edit)]);
        let inst = Instante {
            t_cut: self.ph.time(),
            solo: false,
        };
        do_quadro(
            &mut self.tl,
            &mut self.sim,
            &self.drive,
            &self.ph,
            inst,
            true,
            Interp::Linear,
        );
    }

    /// Pega no 24 e larga no 48, com o Auto Key; devolve onde a espada estava em cada gesto.
    fn pega_e_larga(&mut self) -> ([f32; 2], [f32; 2]) {
        for q in 0..=QUADRO_PEGA {
            self.em(q);
        }
        let pega = xy(&self.em(QUADRO_PEGA));
        self.cartao(E::Forca(self.item, 1.0));
        for q in QUADRO_PEGA..=QUADRO_LARGA {
            self.em(q);
        }
        let larga = xy(&self.em(QUADRO_LARGA));
        self.cartao(E::Forca(self.item, 0.0));
        (pega, larga)
    }

    fn chaves_da_forca(&self) -> usize {
        let p = PropKind::Field(chave(FORCA, self.item));
        self.tl
            .doc
            .binding_for(self.espada.to_bits(), p)
            .and_then(|b| self.tl.doc.active_clip().track(b.target))
            .map_or(0, |t| t.keys().len())
    }
}

fn xy(t: &Transform) -> [f32; 2] {
    [t.translation.x, t.translation.y]
}

fn longe(a: [f32; 2], b: [f32; 2]) -> f32 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

#[test]
fn picking_and_dropping_with_auto_key_never_jumps_and_keeps_the_past() {
    let mut c = Cena::nova();
    let (pega, larga) = c.pega_e_larga();
    assert!(longe(pega, ESPADA_NO_CHAO) < 1e-6);
    assert!(
        longe(larga, ESPADA_NO_CHAO) > 1.0,
        "a mão levou a espada: {larga:?}"
    );
    assert_eq!(c.chaves_da_forca(), 4, "f−1 e f ao pegar e ao largar");
    // A chave de f−1 é um DEGRAU: a meio de 23 e 24 a espada ainda está no chão.
    let meio = (QUADRO_PEGA as f64 - 0.5) / c.tl.doc.fps_display;
    c.em_t(meio); // o canal da força chega um quadro depois de um salto de tempo
    assert!(
        longe(xy(&c.em_t(meio)), pega) < 1e-6,
        "a força já sobe antes do 24"
    );
    for _ in 0..2 {
        for &q in &VISITAS {
            let v = xy(&c.em(q));
            match q {
                q if q <= QUADRO_PEGA => {
                    assert!(longe(v, pega) < 1e-4, "no {q} a espada saiu do chão: {v:?}");
                }
                q if q >= QUADRO_LARGA => {
                    assert!(longe(v, larga) < 1e-4, "no {q} a espada saltou: {v:?}");
                }
                _ => assert!(longe(v, pega) > 0.5, "no {q} a mão já não a leva: {v:?}"),
            }
        }
    }
}

/// Um gesto do cartão é UM passo de undo da timeline: o Ctrl+Z tira as duas chaves de uma vez.
#[test]
fn one_gesture_is_one_undo_step() {
    let mut c = Cena::nova();
    c.em(QUADRO_PEGA);
    c.cartao(E::Forca(c.item, 1.0));
    assert_eq!(c.chaves_da_forca(), 2);
    apply_intent(&mut c.tl, &mut c.ph, TimelineIntent::Undo);
    assert_eq!(c.chaves_da_forca(), 0);
}

/// Sem o Auto Key o gesto não grava nada — e o registo esvazia (não fica para o próximo quadro).
#[test]
fn without_auto_key_the_gesture_keys_nothing_and_is_forgotten() {
    let mut c = Cena::nova();
    c.tl.flags.auto_key = false;
    c.em(QUADRO_PEGA);
    c.cartao(E::Forca(c.item, 1.0));
    assert_eq!(c.chaves_da_forca(), 0);
    c.tl.flags.auto_key = true;
    c.cartao(E::Forca(c.item, 1.0));
    assert_eq!(c.chaves_da_forca(), 0);
}

/// ⭐ Cozer a timeline inteira depois de pegar e largar: em cada quadro a espada fica onde a pilha a
/// punha, AO BIT, e a pilha não faz nada (força 0).
#[test]
fn the_bake_equals_the_live_stack_at_every_frame() {
    let mut c = Cena::nova();
    c.pega_e_larga();
    let fim = 72;
    let vivas: Vec<Transform> = (0..=fim).map(|q| c.em(q)).collect();
    c.em(36);
    c.cartao(E::Cozer);
    for (q, viva) in (0..=fim).zip(&vivas) {
        let cozida = c.em(q);
        assert_eq!(
            (
                cozida.translation.x.to_bits(),
                cozida.translation.y.to_bits(),
                cozida.rotation.to_bits(),
                cozida.scale.x.to_bits(),
                cozida.scale.y.to_bits(),
            ),
            (
                viva.translation.x.to_bits(),
                viva.translation.y.to_bits(),
                viva.rotation.to_bits(),
                viva.scale.x.to_bits(),
                viva.scale.y.to_bits(),
            ),
            "no quadro {q} o bake deu {cozida:?} e a pilha {viva:?}"
        );
        let estado = c
            .sim
            .world()
            .get_resource::<Estados>()
            .and_then(|s| s.of(c.espada, c.item));
        assert_eq!(
            estado,
            Some(Estado::SemForca),
            "no quadro {q} a pilha ainda age"
        );
    }
}

fn bits(t: &Transform) -> [u32; 5] {
    [
        t.translation.x.to_bits(),
        t.translation.y.to_bits(),
        t.rotation.to_bits(),
        t.scale.x.to_bits(),
        t.scale.y.to_bits(),
    ]
}

/// ⭐ Cozer um TROÇO do meio com a pilha ACTIVA e a meia força (a espada presa à mão a 50 %, sem curva
/// de pose): a pilha parte da pose AUTORADA (não da viva, que já a leva), a força é 0 só lá dentro,
/// fora nada muda ao bit, e o último quadro cozido segura até à borda (sem rampa).
#[test]
fn baking_a_stretch_with_the_stack_active_keeps_every_frame_and_the_outside() {
    let mut c = Cena::nova();
    for q in 0..=QUADRO_PEGA {
        c.em(q);
    }
    c.cartao(E::Forca(c.item, 0.5));
    let (de, ate, fps) = (30_i32, 40_i32, c.tl.doc.fps_display);
    let vivas: Vec<[u32; 5]> = (20..=50).map(|q| bits(&c.em(q))).collect();
    c.em(36);
    c.ph.set_loop(f64::from(de) / fps, f64::from(ate) / fps);
    c.cartao(E::Cozer);
    for (q, viva) in (20..=50).zip(&vivas) {
        assert_eq!(&bits(&c.em(q)), viva, "no quadro {q} o bake mudou a pose");
    }
    let ultima = bits(&c.em(i64::from(ate)));
    let entre = bits(&c.em_t((f64::from(ate) + 0.5) / fps));
    assert_eq!(
        entre, ultima,
        "entre o {ate} e a borda a pose rampa para a autorada"
    );
}
