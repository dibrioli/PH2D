//! **A cena 1 conduzida** (`PH2D_CONSTRAINT_SMOKE=1`): grava a curva da mão, liga o Auto Key, pega a
//! espada no 24 e larga-a no 48 pela porta do cartão ([`crate::inspector::apply_all`]) — as chaves de
//! compensação nascem como nasceriam do clique —, e mede o salto e o passado arrastando o tempo.
//! No molde do `ph2d_app_components::autokey_smoke_componentes`: um passo por quadro.

use std::sync::Mutex;
use std::sync::atomic::{AtomicU8, Ordering};

use bevy_ecs::entity::Entity;
use ph2d_anim::RationalTime;
use ph2d_constraints_ecs::Constraints;
use ph2d_core::Playhead;
use ph2d_ecs::{Name, SimWorld, world_transform};
use ph2d_editor_core::constraint_edits::ConstraintFieldEdit as E;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_timeline::{AnimValue, Interp, PropKind, TimelineIntent, TimelineState};

use crate::smoke::{MAO, QUADRO_LARGA, QUADRO_PEGA};

/// Os quadros que a sonda visita depois dos gestos: antes, durante e depois de a mão a segurar.
pub const VISITAS: [i64; 6] = [10, 23, QUADRO_PEGA, 36, QUADRO_LARGA, 60];

static ESTAGIO: AtomicU8 = AtomicU8::new(0);
static MEDIDAS: Mutex<Medidas> = Mutex::new(Medidas::VAZIA);

#[derive(Clone, Copy, Debug)]
struct Medidas {
    espada: u64,
    mao: u64,
    item: u32,
    pega: [f32; 2],
    larga: [f32; 2],
    vistas: [[f32; 2]; VISITAS.len()],
}

impl Medidas {
    const VAZIA: Self = Self {
        espada: 0,
        mao: 0,
        item: 0,
        pega: [0.0; 2],
        larga: [0.0; 2],
        vistas: [[0.0; 2]; VISITAS.len()],
    };
}

/// As chaves da curva da mão ([`MAO`]), lineares.
#[must_use]
pub fn chaves_da_mao(mao: u64, fps: f64) -> Vec<TimelineIntent> {
    #[expect(clippy::cast_possible_truncation, reason = "fps inteiros")]
    let fps = fps.round() as u32;
    MAO.iter()
        .flat_map(|&(q, x, y, graus)| {
            [
                (PropKind::TranslationX, x),
                (PropKind::TranslationY, y),
                (PropKind::Rotation, graus.to_radians()),
            ]
            .map(|(prop, v)| TimelineIntent::AddKey {
                entity: mao,
                prop,
                t: RationalTime::from_frame(q, fps),
                value: AnimValue::Float(v),
                interp: Interp::Linear,
            })
        })
        .collect()
}

fn chamada(sim: &mut SimWorld, nome: &str) -> Option<Entity> {
    let w = sim.world_mut();
    let mut q = w.query::<(Entity, &Name)>();
    q.iter(w).find(|(_, n)| n.0 == nome).map(|(e, _)| e)
}

fn onde(sim: &SimWorld, bits: u64) -> [f32; 2] {
    Entity::try_from_bits(bits)
        .and_then(|e| world_transform(sim.world(), e))
        .map_or([f32::NAN; 2], |t| [t.translation.x, t.translation.y])
}

fn distancia(a: [f32; 2], b: [f32; 2]) -> f32 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

fn ir(tl: &TimelineState, ph: &mut Playhead, quadro: i64) {
    ph.seek_frame(quadro, tl.doc.fps_display);
}

/// Um passo da cena por quadro; inerte sem `PH2D_CONSTRAINT_SMOKE=1`.
pub fn step(
    sim: &mut SimWorld,
    hero: &mut HeroScreen,
    tl: &mut TimelineState,
    ph: &mut Playhead,
) -> Option<String> {
    let s = match ESTAGIO.load(Ordering::Relaxed) {
        0 => {
            let armada = crate::smoke::nivel_armado() == Some(1);
            ESTAGIO.store(if armada { 1 } else { u8::MAX }, Ordering::Relaxed);
            if !armada {
                return None;
            }
            1
        }
        u8::MAX => return None,
        s => s,
    };
    if s < u8::MAX - 1 {
        ESTAGIO.store(s + 1, Ordering::Relaxed);
    }
    let mut m = MEDIDAS.lock().ok()?;
    match s {
        1 => {
            let (mao, espada) = (chamada(sim, "Mão")?, chamada(sim, "Espada")?);
            m.mao = mao.to_bits();
            m.espada = espada.to_bits();
            m.item = sim.world().get::<Constraints>(espada)?.items.first()?.id;
            for i in chaves_da_mao(m.mao, tl.doc.fps_display) {
                ph2d_timeline::apply_intent(tl, ph, i);
            }
            hero.set_panel_visible("timeline", true);
            hero.set_panel_visible("inspector", true);
            let cartao = ph2d_editor_core::ids::INSP_LIVE_CONSTRAINTS_SECTION;
            hero.store.set_collapsed(cartao, false);
            tl.flags.auto_key = true;
            ph.pause();
            ir(tl, ph, QUADRO_PEGA);
            None
        }
        8 => {
            m.pega = onde(sim, m.espada);
            crate::inspector::apply_all(sim, &[(m.espada, E::Forca(m.item, 1.0))]);
            Some(format!(
                "[constraint-smoke] =1 pegou no {QUADRO_PEGA} em {:?}",
                m.pega
            ))
        }
        14 => {
            ir(tl, ph, QUADRO_LARGA);
            None
        }
        20 => {
            m.larga = onde(sim, m.espada);
            crate::inspector::apply_all(sim, &[(m.espada, E::Forca(m.item, 0.0))]);
            Some(format!(
                "[constraint-smoke] =1 largou no {QUADRO_LARGA} em {:?}",
                m.larga
            ))
        }
        26 => {
            tl.flags.auto_key = false;
            None
        }
        s if (30..30 + 6 * VISITAS.len()).contains(&usize::from(s)) => {
            let k = (usize::from(s) - 30) / 6;
            match (usize::from(s) - 30) % 6 {
                0 => ir(tl, ph, VISITAS[k]),
                5 => m.vistas[k] = onde(sim, m.espada),
                _ => {}
            }
            None
        }
        70 => {
            tl.flags.auto_key = true;
            ir(tl, ph, 0);
            Some(veredito(&m, tl))
        }
        _ => None,
    }
}

fn ok(cond: bool, falha: &str) -> String {
    if cond {
        "OK".to_owned()
    } else {
        format!("FAIL — {falha}")
    }
}

fn vista(m: &Medidas, quadro: i64) -> [f32; 2] {
    VISITAS
        .iter()
        .position(|&q| q == quadro)
        .map_or([f32::NAN; 2], |k| m.vistas[k])
}

fn veredito(m: &Medidas, tl: &TimelineState) -> String {
    let forca = PropKind::Field(ph2d_ecs::canal_constraint::chave(
        ph2d_ecs::canal_constraint::FORCA,
        m.item,
    ));
    let quadros: Vec<i64> = tl
        .doc
        .binding_for(m.espada, forca)
        .and_then(|b| tl.doc.active_clip().track(b.target))
        .map(|t| {
            t.keys()
                .iter()
                .map(|k| (k.t.to_seconds() * tl.doc.fps_display).round() as i64)
                .collect()
        })
        .unwrap_or_default();
    let (s24, s48) = (
        distancia(vista(m, QUADRO_PEGA), m.pega),
        distancia(vista(m, QUADRO_LARGA), m.larga),
    );
    [
        format!(
            "[constraint-smoke] =1 salto ao pegar {s24:.6} m · ao largar {s48:.6} m: {}",
            ok(s24 < 1e-4 && s48 < 1e-4, "a espada salta")
        ),
        format!(
            "[constraint-smoke] =1 no 10 e no 23 a espada no chão ({:?} · {:?}): {}",
            vista(m, 10),
            vista(m, 23),
            ok(
                distancia(vista(m, 10), m.pega) < 1e-4 && distancia(vista(m, 23), m.pega) < 1e-4,
                "o passado mudou"
            )
        ),
        format!(
            "[constraint-smoke] =1 no 36 a espada no ar (longe do chão {:.2} m): {}",
            distancia(vista(m, 36), m.pega),
            ok(distancia(vista(m, 36), m.pega) > 0.5, "a mão não a levou")
        ),
        format!(
            "[constraint-smoke] =1 no 60 a espada onde foi largada: {}",
            ok(distancia(vista(m, 60), m.larga) < 1e-4, "o depois mudou")
        ),
        format!(
            "[constraint-smoke] =1 chaves da força nos quadros {quadros:?}: {}",
            ok(
                quadros == [QUADRO_PEGA - 1, QUADRO_PEGA, QUADRO_LARGA - 1, QUADRO_LARGA],
                "faltam as chaves de compensação"
            )
        ),
    ]
    .join("\n")
}
