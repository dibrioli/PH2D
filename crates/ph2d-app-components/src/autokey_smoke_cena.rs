//! **A cena do smoke «a Cena e os itens das listas gravam»** (F30, [plano](../../../docs/UI_New_and_Simple/F30_plano.md))
//! — `PH2D_AUTOKEY_SMOKE=35`. Só monta o palco; os gestos são do dono:
//!
//! - um chão estático e uma BOLA dinâmica no alto — ao tocar, ela cai; a gravidade do painel da
//!   física grava na Cena (§B1, §C);
//! - um objecto com DOIS temporizadores (1 s e 3 s) — animar a duração do 2.º, apagar o 1.º e ver a
//!   chave ficar no 2.º (§A1: a chave segue o ItemId, não a posição);
//! - o Auto Key e o «Physics» da Timeline armados, a Timeline e o painel da física abertos, NADA seleccionado (a vista da Cena,
//!   onde caem a gravidade e a mesa de som, §B2).
//!
//! O dispatcher `ph2d_app_vec::autokey_smoke` reserva o `"35"`; quem a chama é
//! [`crate::autokey_smoke_componentes::step`].

use std::sync::atomic::{AtomicU8, Ordering};

use ph2d_core::{Playhead, Vec2};
use ph2d_ecs::{Entity, ItemId, Name, SimWorld, Timer, Timers, Transform, Visibility};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, RigidBody};
use ph2d_render::{Sprite, WHITE_TILE_KEY};
use ph2d_timeline::TimelineState;

/// O número da cena no `PH2D_AUTOKEY_SMOKE`.
pub const CENA: &str = "35";

const RAIO: f32 = 0.5;
const BOLA_Y: f32 = 2.5;
const CHAO_Y: f32 = -1.5;
const CHAO_MEIO: [f32; 2] = [5.0, 0.25];
const RELOGIO_X: f32 = 4.0;
const RELOGIO_LADO: f32 = 1.2;
const DURACOES_US: [u64; 2] = [1_000_000, 3_000_000];
const BOLA_RGBA: [f32; 4] = [0.95, 0.55, 0.25, 1.0];
const CHAO_RGBA: [f32; 4] = [0.45, 0.48, 0.55, 1.0];
const RELOGIO_RGBA: [f32; 4] = [0.40, 0.75, 0.95, 1.0];
/// O estágio em que o Auto Key se arma — depois do arranque, que repõe as flags da Timeline.
const ARMA: u8 = 10;

static ESTAGIO: AtomicU8 = AtomicU8::new(0);

/// Um passo da cena por quadro; no-op sem `PH2D_AUTOKEY_SMOKE=35`.
pub fn step(
    sim: &mut SimWorld,
    hero: &mut HeroScreen,
    tl: &mut TimelineState,
    ph: &mut Playhead,
) -> Option<String> {
    let s = match ESTAGIO.load(Ordering::Relaxed) {
        0 => {
            let armada = std::env::var("PH2D_AUTOKEY_SMOKE").is_ok_and(|n| n == CENA);
            ESTAGIO.store(if armada { 1 } else { u8::MAX }, Ordering::Relaxed);
            if !armada {
                return None;
            }
            1
        }
        u8::MAX => return None,
        s => s,
    };
    if s <= ARMA {
        ESTAGIO.store(s + 1, Ordering::Relaxed);
    }
    match s {
        1 => {
            let (bola, chao, relogio) = montar(sim);
            hero.gizmo.replace_selection(None);
            hero.set_panel_visible("timeline", true);
            hero.set_panel_visible("physics", true);
            ph.pause();
            ph.seek(0.0);
            Some(format!(
                "[autokey-smoke] =35 montada · bola {bola:?} · chao {chao:?} · relogio {relogio:?} ({})",
                temporizadores(sim, relogio)
            ))
        }
        ARMA => {
            tl.flags.auto_key = true;
            // O Play corre a física (o interruptor «Physics» da Timeline): a bola cai ao tocar.
            tl.flags.simulate_physics = true;
            hero.gizmo.replace_selection(None);
            Some(format!(
                "[autokey-smoke] =35 Auto Key {} · Physics {} · seleccao {:?}",
                tl.flags.auto_key, tl.flags.simulate_physics, hero.gizmo.selection
            ))
        }
        _ => None,
    }
}

fn tr(chave: &str) -> &'static str {
    ph2d_i18n::tr(chave)
}

fn montar(sim: &mut SimWorld) -> (Entity, Entity, Entity) {
    let w = sim.world_mut();
    // O chão primeiro: a ordem das raízes é a de criação, e quem nasce primeiro desenha por baixo.
    let chao = w
        .spawn((
            Name::new(tr("app.components.autokey_smoke_cena.ground")),
            Transform::from_translation(Vec2::new(0.0, CHAO_Y)),
            Visibility::visible(),
            Sprite::atlas(
                WHITE_TILE_KEY,
                [2.0 * CHAO_MEIO[0], 2.0 * CHAO_MEIO[1]],
                CHAO_RGBA,
            ),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: CHAO_MEIO[0],
                    half_y: CHAO_MEIO[1],
                },
                ..Collider::default()
            },
        ))
        .id();
    let bola = w
        .spawn((
            Name::new(tr("app.components.autokey_smoke_cena.ball")),
            Transform::from_translation(Vec2::new(-1.5, BOLA_Y)),
            Visibility::visible(),
            crate::smoke_desenho::disco(RAIO, BOLA_RGBA),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: RAIO },
                ..Collider::default()
            },
        ))
        .id();
    let nomes = [
        "app.components.autokey_smoke_cena.timer_short",
        "app.components.autokey_smoke_cena.timer_long",
    ];
    let timers = nomes
        .iter()
        .zip(DURACOES_US)
        .map(|(n, d)| Timer {
            name: tr(n).to_owned(),
            duration_us: d,
            repeat: true,
            item: ItemId::NONE,
            ..Timer::default()
        })
        .collect();
    let relogio = w
        .spawn((
            Name::new(tr("app.components.autokey_smoke_cena.clock")),
            Transform::from_translation(Vec2::new(RELOGIO_X, CHAO_Y + CHAO_MEIO[1] + RELOGIO_LADO)),
            Visibility::visible(),
            Sprite::atlas(WHITE_TILE_KEY, [RELOGIO_LADO, RELOGIO_LADO], RELOGIO_RGBA),
            Timers(timers),
        ))
        .id();
    ph2d_ecs::assign_missing_stable_ids(w);
    ph2d_ecs::item_id::assign_missing_item_ids(w);
    ph2d_ecs::assign_missing_root_order(w);
    ph2d_ecs::assign_missing_sibling_order(w);
    (bola, chao, relogio)
}

fn temporizadores(sim: &SimWorld, relogio: Entity) -> String {
    sim.world().get::<Timers>(relogio).map_or_else(
        || "sem Timers".to_owned(),
        |t| {
            t.0.iter()
                .map(|x| format!("{} {} us item {:?}", x.name, x.duration_us, x.item))
                .collect::<Vec<_>>()
                .join(" · ")
        },
    )
}
