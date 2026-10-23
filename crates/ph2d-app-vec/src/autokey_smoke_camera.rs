//! **A cena do smoke «o Auto Key grava a câmera»** (F27 Bloco B1) — `PH2D_AUTOKEY_SMOKE=28`. A
//! primeira secção do Inspector que entra pela família COMPONENTE (o canal de qualquer campo do
//! catálogo): a câmera muda de altura, de enquadramento e de dolly no quadro 12 e volta no 24, o
//! Auto Key grava, e a cena pára no 18 — a meio — a medir que a curva conduz a câmera.

use ph2d_core::{Playhead, Vec2};
use ph2d_ecs::{Entity, GameCamera, Name, SimWorld, Transform};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_render::{Sprite, WHITE_TILE_KEY};
use ph2d_timeline::TimelineState;
use std::sync::Mutex;

use crate::autokey_smoke::avanca as go;
use crate::autokey_smoke_tudo::trilhas;

const QUADRO_12: f64 = 0.5;
const QUADRO_18: f64 = 0.75;
const QUADRO_24: f64 = 1.0;
const COR: [f32; 4] = [0.95, 0.6, 0.2, 1.0];
/// `(altura, offset, dolly)` da partida e do quadro 12.
const PARTIDA: (f32, [f32; 2], f32) = (10.0, [0.0, 0.0], 0.0);
const Q12: (f32, [f32; 2], f32) = (6.0, [2.0, 1.0], 0.5);

static CAMERA: Mutex<Option<u64>> = Mutex::new(None);

fn camera() -> Option<u64> {
    CAMERA.lock().ok().and_then(|c| *c)
}

pub(crate) fn step(
    nivel: u8,
    sim: &mut SimWorld,
    hero: &mut HeroScreen,
    tl: &mut TimelineState,
    ph: &mut Playhead,
) -> Option<String> {
    match nivel {
        1 => {
            let c = montar(sim);
            if let Ok(mut a) = CAMERA.lock() {
                *a = Some(c.to_bits());
            }
            hero.set_panel_visible("timeline", true);
            hero.set_panel_visible("inspector", true);
            hero.store.bump_panel_z(ph2d_editor_core::ids::INSP_PANEL);
            hero.store
                .set_collapsed(ph2d_editor_core::ids::INSP_LIVE_CAMERA_SECTION, false);
            go(2);
            Some(format!("[autokey-smoke] câmera montada · {}", medir(sim)))
        }
        s @ 2..=90 => {
            go(s + 1);
            let c = camera();
            match s {
                8 => {
                    tl.flags.auto_key = true;
                    ph.pause();
                    ph.seek(QUADRO_12);
                    hero.gizmo.replace_selection(c);
                    None
                }
                14 => {
                    mexer(sim, c, Q12);
                    None
                }
                22 => {
                    ph.seek(QUADRO_24);
                    None
                }
                28 => {
                    mexer(sim, c, PARTIDA);
                    None
                }
                32 => c.map(|b| format!("[autokey-smoke] trilhas da câmera: {}", trilhas(tl, b))),
                36 => {
                    tl.flags.auto_key = false;
                    ph.seek(QUADRO_18);
                    None
                }
                46 => {
                    let altura = c
                        .and_then(|b| sim.world().get::<GameCamera>(Entity::from_bits(b)))
                        .map(|g| g.height_world);
                    let meio = (PARTIDA.0 + Q12.0) * 0.5;
                    let ok = altura.is_some_and(|h| (h - meio).abs() < 0.5);
                    Some(format!(
                        "[autokey-smoke] q18 (a meio): {} — a altura tem de andar perto de {meio:.1}: {}",
                        medir(sim),
                        if ok {
                            "OK"
                        } else {
                            "PARADA — a curva não conduz a câmera"
                        }
                    ))
                }
                90 => Some(format!("[autokey-smoke] fim · {}", medir(sim))),
                _ => None,
            }
        }
        _ => None,
    }
}

fn montar(sim: &mut SimWorld) -> Entity {
    for (nome, x) in [("Box A", -3.0), ("Box B", 0.0), ("Box C", 3.0)] {
        sim.world_mut().spawn((
            Transform::from_translation(Vec2::new(x, 0.0)),
            Sprite::atlas(WHITE_TILE_KEY, [1.0, 1.0], COR),
            Name::new(nome.to_owned()),
        ));
    }
    sim.world_mut()
        .spawn((
            Name::new("Camera".to_owned()),
            Transform::default(),
            GameCamera::default(),
        ))
        .id()
}

fn mexer(sim: &mut SimWorld, c: Option<u64>, (altura, offset, dolly): (f32, [f32; 2], f32)) {
    let Some(mut g) = c.and_then(|b| sim.world_mut().get_mut::<GameCamera>(Entity::from_bits(b)))
    else {
        return;
    };
    g.height_world = altura;
    g.offset = offset;
    g.dolly = dolly;
}

fn medir(sim: &SimWorld) -> String {
    camera()
        .and_then(|b| sim.world().get::<GameCamera>(Entity::from_bits(b)))
        .map_or_else(
            || "sem câmera".to_owned(),
            |g| {
                format!(
                    "altura {:.2} offset [{:.2}, {:.2}] dolly {:.2}",
                    g.height_world, g.offset[0], g.offset[1], g.dolly
                )
            },
        )
}
