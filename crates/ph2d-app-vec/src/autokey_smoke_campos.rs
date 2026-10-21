//! **A cena do smoke «o losango nos campos sem coluna»** (F24d) — `PH2D_AUTOKEY_SMOKE=8` monta-a para
//! o dono mexer à mão; `=9` mexe sozinha e a sonda mede os números da mensagem; `=10` é a `=9` com
//! o painel rolado até ao cartão do efeito no fim (a foto do olho e dos parâmetros dele).
//!
//! Uma estrela PARAMÉTRICA (os campos dela na fila ②), o traço amarelo como entrada da pilha, o
//! efeito Zig Zag, o Auto Key ligado e o relógio no quadro 12. O `=9`, no quadro 12: o olho da
//! entrada desligado, a mistura do objecto em Multiply, o 1.º parâmetro do Zig Zag a dobrar, a
//! estrela com 8 pontas; no quadro 24: tudo de volta; depois pára no quadro 18 e diz o que vê.

use ph2d_core::Playhead;
use ph2d_ecs::{Entity, SimWorld, VecShape};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_timeline::TimelineState;
use ph2d_vec_scene::paint_stack::{PaintEntry, PaintKind};
use ph2d_vec_scene::{BlendMode, Rgba8, ShapeKind, StrokeSpec, VecPath, VecScene};

use crate::autokey_smoke::avanca as go;
use crate::state::VecState;

const QUADRO_12: f64 = 0.5;
const QUADRO_24: f64 = 1.0;
const QUADRO_18: f64 = 0.75;
const AZUL: Rgba8 = Rgba8::new(40, 90, 230, 255);
const AMARELO: Rgba8 = Rgba8::new(250, 210, 40, 255);
/// O KIND do Zig Zag na tabela do motor (`PathEffect::from_kind`).
const ZIG_ZAG: usize = 1;
/// As pontas da estrela no quadro 12 (o campo 0 do catálogo, `POINTS`).
const PONTAS: f64 = 8.0;
/// Quanto o `=10` rola o painel do vetor: o cartão do Zig Zag no topo (medido na foto).
const ROLAGEM: f32 = 900.0;

#[allow(clippy::too_many_arguments)]
pub(crate) fn step(
    nivel: u8,
    // `0` = o dono mexe · `1` = mexe sozinha · `2` = mexe sozinha e rola até ao efeito.
    modo: u8,
    vec: &mut VecState,
    scene: &mut VecScene,
    sim: &mut SimWorld,
    hero: &mut HeroScreen,
    tl: &mut TimelineState,
    ph: &mut Playhead,
) -> Option<String> {
    match nivel {
        1 => {
            hero.store.set_command_pick(crate::object_add::VECTOR.id());
            go(2);
            None
        }
        2 if hero.image_edit.active_tool_id == Some(crate::vector_mode::VECTOR) => {
            crate::vector_mode::draw_shape(vec, scene, ShapeKind::Star, [0.0, 0.0], 1.8);
            go(3);
            None
        }
        3 => {
            let (&id, &bits) = vec.entities.iter().next()?;
            let p = scene.path_mut(id)?;
            p.stroke = Some(StrokeSpec::new(AZUL, 0.04));
            p.paints = vec![PaintEntry::new(PaintKind::Stroke(StrokeSpec::new(
                AMARELO, 0.1,
            )))];
            crate::fx_bridge::add(scene, id, ZIG_ZAG);
            vec.pen.select_many(&[id]);
            hero.gizmo.replace_selection(Some(bits));
            hero.set_panel_visible("timeline", true);
            tl.flags.auto_key = true;
            ph.pause();
            ph.seek(QUADRO_12);
            go(4);
            Some(format!(
                "[autokey-smoke] estrela paramétrica {id} (entidade {bits:#x}): traço azul, entrada 1 \
                 da pilha = traço amarelo, Zig Zag; quadro 12; Auto Key LIGADO"
            ))
        }
        s @ 4..=80 if modo > 0 => {
            go(s + 1);
            let (&id, &bits) = vec.entities.iter().next()?;
            let e = Entity::from_bits(bits);
            match s {
                20 => {
                    scene.path_mut(id)?.paints.first_mut()?.enabled = false;
                    Some("[autokey-smoke] q12: olho da entrada 1 ← desligado".to_string())
                }
                25 => {
                    scene.path_mut(id)?.blend = BlendMode::Multiply;
                    Some("[autokey-smoke] q12: mistura ← Multiply".to_string())
                }
                30 => {
                    let fx = scene.path_mut(id)?.effects.first_mut()?;
                    let v = na_faixa(&mut fx.effect, 0.5)?;
                    Some(format!("[autokey-smoke] q12: Zig Zag p0 ← {v:.3}"))
                }
                35 => {
                    pontas(sim, e, PONTAS)?;
                    Some(format!(
                        "[autokey-smoke] q12: estrela ← {PONTAS} pontas · {}",
                        crate::autokey_smoke_tudo::trilhas(tl, bits)
                    ))
                }
                40 => {
                    ph.seek(QUADRO_24);
                    None
                }
                50 => {
                    let p = scene.path_mut(id)?;
                    p.paints.first_mut()?.enabled = true;
                    p.blend = BlendMode::Normal;
                    na_faixa(&mut p.effects.first_mut()?.effect, 0.1)?;
                    pontas(sim, e, 5.0)?;
                    Some(format!(
                        "[autokey-smoke] q24: tudo de volta · {}",
                        crate::autokey_smoke_tudo::trilhas(tl, bits)
                    ))
                }
                55 => {
                    ph.seek(QUADRO_18);
                    None
                }
                65 => Some(format!(
                    "[autokey-smoke] q18 (a meio): {} · {}",
                    medir(scene.path(id)?, sim, e),
                    crate::autokey_smoke_tudo::trilhas(tl, bits)
                )),
                // O cartão do efeito à vista (o painel rola até ele) — a foto do olho e dos
                // parâmetros do efeito.
                70 if modo == 2 => {
                    hero.store
                        .set_panel_scroll(ph2d_editor_core::ids::VECTOR_PANEL, ROLAGEM);
                    None
                }
                80 => Some(format!(
                    "[autokey-smoke] fim · {}",
                    crate::autokey_smoke_tudo::trilhas(tl, bits)
                )),
                _ => None,
            }
        }
        _ => None,
    }
}

/// O 1.º parâmetro do efeito na fracção `t` da faixa dele; devolve o valor.
fn na_faixa(fx: &mut ph2d_vec_scene::effect::PathEffect, t: f64) -> Option<f64> {
    let p = fx.params().first()?;
    let v = p.min + (p.max - p.min) * t;
    fx.set(0, v);
    Some(v)
}

/// As pontas da estrela (o campo 0 da forma paramétrica).
fn pontas(sim: &mut SimWorld, e: Entity, n: f64) -> Option<()> {
    match &mut *sim.world_mut().get_mut::<VecShape>(e)? {
        VecShape::Param { values, .. } => values[0] = n,
        VecShape::Text(_) => return None,
    }
    Some(())
}

/// O que a tela desenha: o olho da entrada, a mistura, o parâmetro do efeito e as pontas.
fn medir(p: &VecPath, sim: &SimWorld, e: Entity) -> String {
    let olho = p.paints.first().is_some_and(|x| x.enabled);
    let fx = p.effects.first().map_or(-1.0, |x| x.effect.get(0));
    let n = match sim.world().get::<VecShape>(e) {
        Some(VecShape::Param { values, .. }) => values[0],
        _ => -1.0,
    };
    format!(
        "olho da entrada 1 {olho} · mistura {:?} · Zig Zag p0 {fx:.3} · pontas {n:.2}",
        p.blend
    )
}
