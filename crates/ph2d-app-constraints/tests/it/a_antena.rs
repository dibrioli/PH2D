//! ⭐ **A cena 5 (Antena), sem a shell** — a sonda do smoke (plano §6): a curva escreve o corpo
//! (`apply_from_doc`), a pilha corre ([`quadro`] com o relógio) e mede-se a ponta. Tocar quadro a
//! quadro e arrastar o tempo dão o MESMO bit; a mola atrasa e assenta.

use bevy_ecs::entity::Entity;
use ph2d_app_constraints::quadro;
use ph2d_app_constraints::relogio::Relogio;
use ph2d_app_constraints::smoke::{self, BASE_NO_CORPO, CORPO, PONTA_NO_CORPO};
use ph2d_app_constraints::smoke_antena::chaves_do_corpo;
use ph2d_constraints_ecs::{Constraints, Estado, Estados, Laco};
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
    laco: Option<Laco>,
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
            laco: None,
        }
    }

    /// Um quadro do app no quadro `q`: devolve a ponta e o corpo no mundo.
    fn em(&mut self, q: i64) -> ([f32; 2], [f32; 2]) {
        #[expect(clippy::cast_precision_loss, reason = "quadros da cena")]
        self.em_t(q as f64 / self.tl.doc.fps_display)
    }

    /// O mesmo no instante `t` (segundos) — o ecrã desenha a 60 Hz, entre os quadros da timeline.
    fn em_t(&mut self, t: f64) -> ([f32; 2], [f32; 2]) {
        self.em_t_com(t, |_| {})
    }

    /// [`Self::em_t`] com uma mão que mexe na cena depois da curva (o gizmo a arrastar).
    fn em_t_com(&mut self, t: f64, mao: impl FnOnce(&mut SimWorld)) -> ([f32; 2], [f32; 2]) {
        apply_from_doc(self.sim.world_mut(), &mut self.tl.doc, t);
        mao(&mut self.sim);
        let relogio = Relogio {
            doc: &self.tl.doc,
            t,
            laco: self.laco,
        };
        quadro(
            &mut self.sim,
            &mut ph2d_vec_scene::VecScene::new(),
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
    // A antena dobra, não estica: a ponta fica sempre ao comprimento dela da base.
    let comprimento = PONTA_NO_CORPO[1] - BASE_NO_CORPO[1];
    for &(p, k) in &tocar {
        let d = (p[0] - k[0] - BASE_NO_CORPO[0]).hypot(p[1] - k[1] - BASE_NO_CORPO[1]);
        assert!((d - comprimento).abs() < 1e-4, "a antena esticou: {d}");
    }
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

/// ⭐ (W6) Um episódio AO VIVO com o tempo parado (o corpo arrastado no quadro 30, a mola a balançar)
/// não deixa rasto: tocar e arrastar o tempo depois dele dão o bit de quem nunca o teve.
#[test]
fn a_live_episode_leaves_play_and_scrub_bit_identical() {
    use ph2d_app_constraints::relogio::Parede;
    let mut limpo = Cena::nova();
    let tocar: Vec<[f32; 2]> = (0..=72).map(|q| limpo.em(q).0).collect();
    let mut c = Cena::nova();
    for q in 0..=30 {
        c.em(q);
    }
    let t30 = 30.0 / c.tl.doc.fps_display;
    let corpo = c.corpo;
    let (mut vivo, mut parede) = (false, 0.0);
    for i in 0..40 {
        parede += 1.0 / 60.0;
        c.sim.world_mut().insert_resource(Parede(parede));
        let dx = if i < 20 {
            0.03 * f32::from(u8::try_from(i).unwrap())
        } else {
            0.0
        };
        c.em_t_com(t30, |sim| {
            sim.world_mut()
                .get_mut::<ph2d_ecs::Transform>(corpo)
                .unwrap()
                .translation
                .x += dx;
        });
        vivo |= !c
            .sim
            .world()
            .resource::<ph2d_constraints_ecs::Molas>()
            .vivas
            .is_empty();
    }
    assert!(
        vivo,
        "o episódio não ligou a mola ao vivo — o gate não mede nada"
    );
    for q in (31..=40).chain([12, 72, 30, 5, 47]) {
        let got = c.em(q).0;
        assert_eq!(
            got.map(f32::to_bits),
            tocar[q as usize].map(f32::to_bits),
            "quadro {q}"
        );
    }
    assert!(
        c.sim
            .world()
            .resource::<ph2d_constraints_ecs::Molas>()
            .vivas
            .is_empty()
    );
}

/// ⭐ O ecrã desenha a 60 Hz e a timeline tem 24 quadros por segundo: entre dois quadros o corpo anda
/// (a curva lê o instante contínuo) e o atraso da mola tem de andar com ele — parado entre quadros, a
/// ponta treme colada ao corpo e salta no quadro seguinte (report do dono 25/10: «não é fluido»).
#[test]
fn a_ponta_anda_a_cada_imagem_do_ecra() {
    let mut c = Cena::nova();
    let atraso = |(p, k): ([f32; 2], [f32; 2])| [p[0] - k[0], p[1] - k[1]];
    let amostras: Vec<[f32; 2]> = (0..=120)
        .map(|i| atraso(c.em_t(f64::from(i) / 60.0)))
        .collect();
    let parado = amostras.windows(2).filter(|w| w[0] == w[1]).count();
    assert_eq!(parado, 0, "o atraso fica parado em {parado} de 120 imagens");
    // Entre quadros também: arrastar a um instante do meio dá o bit de tocar.
    for i in [97, 13, 50, 7, 120] {
        let got = atraso(c.em_t(f64::from(i) / 60.0));
        assert_eq!(
            got.map(f32::to_bits),
            amostras[i as usize].map(f32::to_bits),
            "imagem {i}"
        );
    }
}

/// A haste vai da base à ponta: a ponta DESENHADA da arte (centro + meio comprimento no rumo da
/// haste) cai na bola, a cada quadro.
#[test]
fn a_haste_chega_a_ponta() {
    let mut c = Cena::nova();
    let arte = achar(&mut c.sim, "Haste (arte)");
    let comprimento = PONTA_NO_CORPO[1] - BASE_NO_CORPO[1];
    for q in [0, 6, 12, 30, 60] {
        let (p, _) = c.em(q);
        let a = world_transform(c.sim.world(), arte).unwrap();
        let (s, co) = a.rotation.sin_cos();
        let fim = [
            a.translation.x + co * comprimento / 2.0,
            a.translation.y + s * comprimento / 2.0,
        ];
        assert!(
            (fim[0] - p[0]).hypot(fim[1] - p[1]) < 1e-3,
            "quadro {q}: a haste acaba em {fim:?}, a ponta está em {p:?} (arte {a:?})"
        );
    }
}

/// ⭐ Report do dono 25/10 — os extremos do laço. A cena 5 em VAIVÉM sobre o laço do smoke (0–72): a
/// ponta vira na ponta do laço sem salto, a volta não é a ida ao contrário, e em LAÇO o recomeço junta-se
/// ao fim. Tocar a 60 Hz e arrastar dão o mesmo bit.
#[test]
fn a_cena_5_nos_extremos_do_laco() {
    let laco = |vaivem, para_a_frente| {
        Some(Laco {
            de: 0.0,
            ate: 72.0,
            vaivem,
            para_a_frente,
        })
    };
    let mut c = Cena::nova();
    let atraso = |(p, k): ([f32; 2], [f32; 2])| [p[0] - k[0], p[1] - k[1]];
    let distancia = |a: [f32; 2], b: [f32; 2]| (a[0] - b[0]).hypot(a[1] - b[1]);
    // 60 Hz sobre 24 q/s: 2,5 imagens por quadro; 3 s de laço = 180 imagens.
    let tocar = |c: &mut Cena, vaivem, frente| -> Vec<[f32; 2]> {
        c.laco = laco(vaivem, frente);
        (0..=180)
            .map(|i| atraso(c.em_t(f64::from(i) / 60.0)))
            .collect()
    };
    let ida = tocar(&mut c, true, true);
    let volta = tocar(&mut c, true, false);
    let dentro = ida
        .windows(2)
        .map(|w| distancia(w[0], w[1]))
        .fold(0.0, f32::max);
    // 179: 0,4 quadros antes da ponta na ida, 0,4 depois na volta.
    let virada = distancia(ida[179], volta[179]);
    assert!(
        virada <= 1.5 * dentro,
        "a virada salta {virada} m (dentro {dentro})"
    );
    assert!(
        distancia(ida[20], volta[20]) > 1e-3,
        "a volta é a ida ao contrário"
    );
    let em_laco = tocar(&mut c, false, true);
    let costura = distancia(em_laco[179], em_laco[0]);
    assert!(
        costura <= 1.5 * dentro,
        "o recomeço salta {costura} m (dentro {dentro})"
    );
    c.laco = laco(false, true);
    for i in [133, 2, 179, 90] {
        let got = atraso(c.em_t(f64::from(i) / 60.0));
        assert_eq!(
            got.map(f32::to_bits),
            em_laco[i as usize].map(f32::to_bits),
            "imagem {i}"
        );
    }
}
