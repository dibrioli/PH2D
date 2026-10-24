//! **A cena do smoke «o Auto Key grava o alvo e o Visible»** (F27 Bloco B2) —
//! `PH2D_AUTOKEY_SMOKE=29`. Todo campo do Inspector é um canal, também os de TEXTO e o «Visible»: a
//! câmera segue a Box A do quadro 0 e a Box C do 12 (o nome do alvo é um canal de texto, por
//! degraus), e a Box B some no 12. A cena pára no 18 a medir as duas coisas.

use ph2d_core::{Playhead, Vec2};
use ph2d_ecs::{CameraFollow, Entity, GameCamera, Name, SimWorld, Transform, Visibility};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_render::{Sprite, WHITE_TILE_KEY};
use ph2d_timeline::TimelineState;
use std::sync::Mutex;

use crate::autokey_smoke::avanca as go;
use crate::autokey_smoke_tudo::trilhas;

const QUADRO_0: f64 = 0.0;
const QUADRO_6: f64 = 0.25;
const QUADRO_12: f64 = 0.5;
const QUADRO_18: f64 = 0.75;
const COR: [f32; 4] = [0.95, 0.6, 0.2, 1.0];

/// `(câmera, Box B)`.
static OBJETOS: Mutex<Option<(u64, u64)>> = Mutex::new(None);

fn objetos() -> Option<(u64, u64)> {
    OBJETOS.lock().ok().and_then(|c| *c)
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
            let o = montar(sim);
            if let Ok(mut a) = OBJETOS.lock() {
                *a = Some(o);
            }
            hero.set_panel_visible("timeline", true);
            hero.set_panel_visible("inspector", true);
            hero.store.bump_panel_z(ph2d_editor_core::ids::INSP_PANEL);
            hero.store
                .set_collapsed(ph2d_editor_core::ids::INSP_LIVE_CAMERA_SECTION, false);
            go(2);
            Some(format!("[autokey-smoke] cena montada · {}", medir(sim)))
        }
        s @ 2..=90 => {
            go(s + 1);
            let (c, b) = objetos().unzip();
            match s {
                8 => {
                    tl.flags.auto_key = true;
                    ph.pause();
                    ph.seek(QUADRO_0);
                    hero.gizmo.replace_selection(c);
                    None
                }
                12 => {
                    seguir(sim, c, "Box A");
                    None
                }
                16 => {
                    ph.seek(QUADRO_12);
                    None
                }
                20 => {
                    seguir(sim, c, "Box C");
                    None
                }
                24 => {
                    ph.seek(QUADRO_0);
                    hero.gizmo.replace_selection(b);
                    None
                }
                28 => {
                    esconder(sim, b, false);
                    None
                }
                32 => {
                    ph.seek(QUADRO_12);
                    None
                }
                36 => {
                    esconder(sim, b, true);
                    None
                }
                40 => {
                    tl.flags.auto_key = false;
                    ph.seek(QUADRO_6);
                    let t = |e: Option<u64>| e.map_or_else(String::new, |e| trilhas(tl, e));
                    Some(format!(
                        "[autokey-smoke] trilhas: câmera {} · Box B {}",
                        t(c),
                        t(b)
                    ))
                }
                // Cada medida segue-se a uma ida no tempo: o app só redesenha quando algo muda.
                46 => {
                    let v = veredito(sim, "q6 (antes)", "Box A", false);
                    ph.seek(QUADRO_18);
                    hero.gizmo.replace_selection(c);
                    Some(v)
                }
                52 => Some(veredito(sim, "q18 (depois)", "Box C", true)),
                90 => Some(format!("[autokey-smoke] fim · {}", medir(sim))),
                _ => None,
            }
        }
        _ => None,
    }
}

pub(crate) fn montar(sim: &mut SimWorld) -> (u64, u64) {
    let mut b = None;
    for (nome, x) in [("Box A", -3.0), ("Box B", 0.0), ("Box C", 3.0)] {
        let e = sim.world_mut().spawn((
            Transform::from_translation(Vec2::new(x, 0.0)),
            Sprite::atlas(WHITE_TILE_KEY, [1.0, 1.0], COR),
            Name::new(nome.to_owned()),
        ));
        if nome == "Box B" {
            b = Some(e.id());
        }
    }
    let b = b.expect("a Box B nasceu");
    sim.world_mut()
        .entity_mut(b)
        .insert(Visibility { hidden: true });
    let c = sim
        .world_mut()
        .spawn((
            Name::new("Camera".to_owned()),
            Transform::default(),
            GameCamera::default(),
            CameraFollow::default(),
        ))
        .id();
    (c.to_bits(), b.to_bits())
}

pub(crate) fn seguir(sim: &mut SimWorld, c: Option<u64>, alvo: &str) {
    if let Some(mut f) = c.and_then(|b| {
        sim.world_mut()
            .get_mut::<CameraFollow>(Entity::from_bits(b))
    }) {
        alvo.clone_into(&mut f.target);
    }
}

fn esconder(sim: &mut SimWorld, b: Option<u64>, hidden: bool) {
    if let Some(mut v) = b.and_then(|b| sim.world_mut().get_mut::<Visibility>(Entity::from_bits(b)))
    {
        v.hidden = hidden;
    }
}

fn estado(sim: &SimWorld) -> Option<(String, bool)> {
    let (c, b) = objetos()?;
    let w = sim.world();
    let alvo = w.get::<CameraFollow>(Entity::from_bits(c))?.target.clone();
    let escondida = w.get::<Visibility>(Entity::from_bits(b))?.hidden;
    Some((alvo, escondida))
}

fn medir(sim: &SimWorld) -> String {
    estado(sim).map_or_else(
        || "sem objectos".to_owned(),
        |(a, h)| {
            format!(
                "a câmera segue «{a}» · Box B {}",
                if h { "escondida" } else { "visível" }
            )
        },
    )
}

fn veredito(sim: &SimWorld, onde: &str, alvo: &str, escondida: bool) -> String {
    let ok = estado(sim).is_some_and(|(a, h)| a == alvo && h == escondida);
    format!(
        "[autokey-smoke] {onde}: {} — tem de seguir «{alvo}» com a Box B {}: {}",
        medir(sim),
        if escondida { "escondida" } else { "visível" },
        if ok {
            "OK"
        } else {
            "PARADA — a curva não conduz o campo"
        }
    )
}
