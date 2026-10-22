//! `PH2D_AUTOKEY_SMOKE` — **o Auto Key sobre uma forma vetorial** (plano 34; report do dono 19/10:
//! *«opacity ainda não funciona com auto key, não grava quadros, assim como as outras propriedades
//! dos desenhos vetoriais»*).
//!
//! Um retângulo com a opacidade animada `20 → 100 %` e a posição animada `X −1,5 → 1,5` em `0..1 s`,
//! a Timeline aberta, o **Auto Key ligado** e o relógio PARADO a meio (`0,5 s`), a forma
//! seleccionada com a ferramenta do vetor na mão (o Appearance à vista).
//!
//! - `=1` — a cena, para o dono mexer.
//! - `=2` — a mesma, e a cena mexe sozinha (para medir sem rato): a meio, digita `30` na Opacity e
//!   empurra a forma `+1` em X; com `PH2D_AUTOKEY_LOG=1` o terminal diz o que o Auto Key viu.

use std::sync::atomic::{AtomicU8, Ordering};

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Playhead;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_timeline::{PropKind, TimelineState};
use ph2d_vec_scene::{ShapeKind, VecScene};

use crate::state::VecState;

static ESTAGIO: AtomicU8 = AtomicU8::new(0);
static ESPERA: AtomicU8 = AtomicU8::new(0);
/// `0` = as cenas `=1|2` · `1` = `=3` (o dono) · `2` = `=4` (mexe sozinha) · `3` = `=5` (o dono,
/// o losango em todo campo — `autokey_smoke_tudo`) · `4` = `=6` (mexe sozinha) · `5` = `=7` (a `=6`
/// com o Inspector à frente no fim, para a foto) · `6` = `=8` (o dono, o losango nos campos sem
/// coluna — `autokey_smoke_campos`) · `7` = `=9` (mexe sozinha) · `8` = `=10` (a `=9` rolada até
/// ao efeito) · `9..=13` = `=11..15` (o bloco 2 — `autokey_smoke_bloco2` e `autokey_smoke_juntas`)
/// · `14..=24` = `=16..26` (uma bancada por tipo de junta — `autokey_smoke_tipos`).
static DESENHO: AtomicU8 = AtomicU8::new(0);
/// O estágio da cena — a do desenho avança-o daqui.
pub(crate) fn avanca(n: u8) {
    ESTAGIO.store(n, Ordering::Relaxed);
}

/// O instante onde o relógio pára — o meio da rampa.
const MEIO: f64 = 0.5;

/// Um passo da cena por quadro. Devolve uma linha para o terminal quando algo acontece.
pub fn step(
    vec: &mut VecState,
    scene: &mut VecScene,
    sim: &mut SimWorld,
    hero: &mut HeroScreen,
    tl: &mut TimelineState,
    ph: &mut Playhead,
) -> Option<String> {
    let nivel = match ESTAGIO.load(Ordering::Relaxed) {
        0 => {
            let n = std::env::var("PH2D_AUTOKEY_SMOKE").ok()?;
            ESTAGIO.store(1, Ordering::Relaxed);
            ESPERA.store(if n == "2" { 1 } else { 0 }, Ordering::Relaxed);
            // `=3` (o dono) e `=4` (mexe sozinha): o DESENHO grava (plano 35) — `autokey_smoke_desenho`.
            let desenho = match n.as_str() {
                "3" => 1,
                "4" => 2,
                "5" => 3,
                "6" => 4,
                "7" => 5,
                "8" => 6,
                "9" => 7,
                "10" => 8,
                "11" => 9,
                "12" => 10,
                "13" => 11,
                "14" => 12,
                "15" => 13,
                // `=16..26`: uma bancada por tipo de junta — `autokey_smoke_tipos`.
                x => x
                    .parse::<u8>()
                    .ok()
                    .filter(|v| (16..=26).contains(v))
                    .map_or(0, |v| v - 2),
            };
            DESENHO.store(desenho, Ordering::Relaxed);
            1
        }
        s => s,
    };
    if DESENHO.load(Ordering::Relaxed) >= 14 {
        let modo = DESENHO.load(Ordering::Relaxed) - 14;
        return crate::autokey_smoke_tipos::step(nivel, modo, sim, hero, tl, ph);
    }
    if DESENHO.load(Ordering::Relaxed) >= 9 {
        return crate::autokey_smoke_bloco2::step(
            nivel,
            DESENHO.load(Ordering::Relaxed) - 9,
            vec,
            scene,
            sim,
            hero,
            tl,
            ph,
        );
    }
    if DESENHO.load(Ordering::Relaxed) >= 6 {
        return crate::autokey_smoke_campos::step(
            nivel,
            DESENHO.load(Ordering::Relaxed) - 6,
            vec,
            scene,
            sim,
            hero,
            tl,
            ph,
        );
    }
    if DESENHO.load(Ordering::Relaxed) >= 3 {
        return crate::autokey_smoke_tudo::step(
            nivel,
            DESENHO.load(Ordering::Relaxed) - 3,
            vec,
            scene,
            sim,
            hero,
            tl,
            ph,
        );
    }
    if DESENHO.load(Ordering::Relaxed) > 0 {
        return crate::autokey_smoke_desenho::step(
            nivel,
            DESENHO.load(Ordering::Relaxed) == 2,
            vec,
            scene,
            sim,
            hero,
            tl,
            ph,
        );
    }
    let go = |n| ESTAGIO.store(n, Ordering::Relaxed);
    match nivel {
        1 => {
            hero.store.set_command_pick(crate::object_add::VECTOR.id());
            go(2);
            None
        }
        2 if hero.image_edit.active_tool_id == Some(crate::vector_mode::VECTOR) => {
            crate::vector_mode::draw_shape(vec, scene, ShapeKind::Rectangle, [0.0, 0.0], 0.6);
            go(3);
            None
        }
        3 => {
            let (&id, &bits) = vec.entities.iter().next()?;
            for (t, v) in [(0.0, 0.2_f32), (1.0, 1.0)] {
                key(tl, bits, PropKind::Opacity, t, v);
            }
            for (t, v) in [(0.0, -1.5_f32), (1.0, 1.5)] {
                key(tl, bits, PropKind::TranslationX, t, v);
            }
            vec.pen.select_many(&[id]);
            hero.gizmo.replace_selection(Some(bits));
            hero.set_panel_visible("timeline", true);
            tl.flags.auto_key = true;
            ph.pause();
            ph.seek(MEIO);
            go(4);
            Some(format!(
                "[autokey-smoke] forma {id} (entidade {bits:#x}): Opacity 20→100 %, X −1,5→1,5 em 0..1 s; \
                 relógio parado em {MEIO} s; Auto Key LIGADO"
            ))
        }
        // `=2`: espera uns quadros parados e mexe como o artista mexeria.
        s @ 4..=60 if ESPERA.load(Ordering::Relaxed) == 1 => {
            go(s + 1);
            let (&id, &bits) = vec.entities.iter().next()?;
            match s {
                30 => {
                    crate::appearance::set_opacity(scene, &[id], 0.3);
                    crate::driven_style::follow_the_hand(sim, &vec.entities, &[id], 0.3);
                    Some(format!(
                        "[autokey-smoke] Opacity ← 30 · {}",
                        chaves(tl, bits)
                    ))
                }
                45 => {
                    if let Some(mut xf) = sim
                        .world_mut()
                        .get_mut::<Transform>(Entity::from_bits(bits))
                    {
                        xf.translation.x += 1.0;
                    }
                    Some(format!("[autokey-smoke] X ← X + 1 · {}", chaves(tl, bits)))
                }
                60 => Some(format!("[autokey-smoke] fim · {}", chaves(tl, bits))),
                _ => None,
            }
        }
        _ => None,
    }
}

fn key(tl: &mut TimelineState, bits: u64, prop: PropKind, t: f64, v: f32) {
    tl.doc.upsert_key(
        bits,
        prop,
        RationalTime::from_seconds(t),
        AnimValue::Float(v),
        Interp::Linear,
    );
}

/// «Opacity: N chaves · X: M chaves» — o que a dope-sheet mostraria.
fn chaves(tl: &TimelineState, bits: u64) -> String {
    let n = |p| {
        tl.doc
            .binding_for(bits, p)
            .and_then(|b| tl.doc.active_clip().track(b.target))
            .map_or(0, |t| t.keys().len())
    };
    format!(
        "chaves: Opacity {} · X {}",
        n(PropKind::Opacity),
        n(PropKind::TranslationX)
    )
}
