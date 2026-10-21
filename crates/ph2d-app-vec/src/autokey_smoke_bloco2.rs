//! **As cenas do smoke «o Auto Key grava o bloco 2»** (F25) — `PH2D_AUTOKEY_SMOKE=11..13`, todas
//! a mexer sozinhas (a sonda mede) e a parar no quadro 18 para a foto.
//!
//! - `=11` VETOR: uma estrela com um FILTRO (Drop Shadow) e um perfil de largura (afina nas pontas),
//!   e um TEXTO ao lado; Auto Key ligado. No quadro 12: o raio e a cor da sombra, o perfil (engrossa
//!   no meio), o tamanho e a FAMÍLIA da fonte do texto e a quebra para Fixed; no 24 tudo volta; pára
//!   no 18 com a estrela seleccionada e o painel rolado até aos filtros.
//! - `=12` a mesma, a acabar com o TEXTO seleccionado e o painel rolado até ao texto.
//! - `=13` INSPECTOR: uma sprite; no quadro 12 o skew, o tint e a emissão; no 24 voltam; pára no 18
//!   com o Inspector à frente.
//! - `=14|15` as juntas — [`crate::autokey_smoke_juntas`].
//!
//! ⚠️ O texto nasce de um rectângulo cuja forma vira `VecShape::Text`: o desenho dos glifos coze-se
//! na shell (a fonte do sistema), e a porta que a cena alcança é a mesma da curva — um tamanho
//! escrito em `CanaisEscritos` ANTES de o Auto Key ligar, que o `canais::aplicar` devolve em
//! `textos` e a shell re-coze. As mudanças do quadro 12/24 vão ao componente, como o painel faz.

use ph2d_core::{Playhead, Vec2};
use ph2d_ecs::canal::familia::VETOR;
use ph2d_ecs::canal_vetor as cv;
use ph2d_ecs::{
    CanaisEscritos, Entity, FieldKey, FxOp, Name, SimWorld, SpriteEmissive, Transform, VecFilter,
    VecShape, VecStrokeProfile, VecTextParams,
};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_render::{Sprite, WHITE_TILE_KEY};
use ph2d_timeline::TimelineState;
use ph2d_vec_scene::{
    Paint, Rgba8, ShapeKind, StrokeSpec, VecPathId, VecScene, WidthStop, WidthStops,
};

use crate::autokey_smoke::avanca as go;
use crate::autokey_smoke_tudo::trilhas;
use crate::state::VecState;

const QUADRO_12: f64 = 0.5;
const QUADRO_24: f64 = 1.0;
const QUADRO_18: f64 = 0.75;
const LARANJA: Rgba8 = Rgba8::new(255, 140, 0, 255);
const AZUL: Rgba8 = Rgba8::new(40, 90, 230, 255);
const CREME: Rgba8 = Rgba8::new(250, 235, 200, 255);
/// A estrela à esquerda, o texto à direita (centro, meia largura).
const ESTRELA: ([f64; 2], f64) = ([-1.8, 0.0], 1.2);
const TEXTO: ([f64; 2], f64) = ([2.0, 0.0], 0.6);
const CONTEUDO: &str = "Auto Key texto";
/// O tamanho do texto no início e no quadro 12; a largura da quebra Fixed.
const TAMANHO: [f64; 2] = [0.45, 0.8];
const QUEBRA: f64 = 2.2;
/// O traço da estrela (largo, para o perfil se ver).
const TRACO: f64 = 0.16;
/// A sombra: raio e cor no início e no quadro 12.
const RAIO: [f32; 2] = [0.06, 0.35];
const SOMBRA: [[f32; 4]; 2] = [[0.0, 0.0, 0.0, 0.8], [0.95, 0.1, 0.1, 1.0]];
const DESLOCA: [f32; 2] = [0.15, -0.15];
/// O perfil: o multiplicador nas pontas e no meio, no início e no quadro 12.
const PERFIL: [[f64; 2]; 2] = [[0.15, 1.0], [1.0, 3.0]];
/// A sprite: o skew, o tint e a emissão do quadro 12.
const SKEW: f32 = 0.45;
const TINT: [f32; 3] = [1.0, 0.45, 0.1];
const EMISSAO: f32 = 0.8;
/// O tint de base (verde): o do quadro 12 é laranja, e o 18 fica a meio — mostarda.
const BASE: [f32; 3] = [0.25, 0.75, 0.45];
const LADO_SPRITE: f32 = 2.4;
/// Quanto o painel do vetor rola até aos filtros (`=11`) e ao texto (`=12`), e o Inspector (`=13`).
const ROLAGEM: [f32; 3] = [1400.0, 800.0, 0.0];

#[allow(clippy::too_many_arguments)]
pub(crate) fn step(
    nivel: u8,
    // `0` = `=11` · `1` = `=12` · `2` = `=13` · `3|4` = `=14|15` (as juntas).
    modo: u8,
    vec: &mut VecState,
    scene: &mut VecScene,
    sim: &mut SimWorld,
    hero: &mut HeroScreen,
    tl: &mut TimelineState,
    ph: &mut Playhead,
) -> Option<String> {
    if modo >= 3 {
        return crate::autokey_smoke_juntas::step(nivel, modo - 3, sim, hero, tl, ph);
    }
    if modo == 2 {
        return sprite(nivel, sim, hero, tl, ph);
    }
    match nivel {
        1 => {
            hero.store.set_command_pick(crate::object_add::VECTOR.id());
            go(2);
            None
        }
        2 if hero.image_edit.active_tool_id == Some(crate::vector_mode::VECTOR) => {
            crate::vector_mode::draw_shape(vec, scene, ShapeKind::Star, ESTRELA.0, ESTRELA.1);
            crate::vector_mode::draw_shape(vec, scene, ShapeKind::Rectangle, TEXTO.0, TEXTO.1);
            go(3);
            None
        }
        3 => {
            let [(estrela, eb), (texto, tb)] = dois(vec)?;
            let p = scene.path_mut(estrela)?;
            p.fill = Some(Paint::Solid(LARANJA));
            p.stroke = Some(StrokeSpec::new(AZUL, TRACO));
            let p = scene.path_mut(texto)?;
            p.fill = Some(Paint::Solid(CREME));
            p.stroke = None;
            crate::fx_live::set_filter(
                sim,
                &vec.entities,
                &[estrela],
                Some(VecFilter {
                    ops: vec![sombra(0)],
                }),
            );
            crate::profile_live::arm(sim, &vec.entities, &[estrela], &perfil(0));
            let te = Entity::from_bits(tb);
            sim.world_mut()
                .entity_mut(te)
                .insert(VecShape::Text(VecTextParams {
                    text: CONTEUDO.to_owned(),
                    origin: TEXTO.0,
                    family: None,
                    // Diferente do que se escreve abaixo: é a diferença que faz a shell re-cozer.
                    size: TAMANHO[0] * 0.5,
                    weight: 400.0,
                    line_height: 1.2,
                    tracking: 0.0,
                    align: 0,
                    axes: Vec::new(),
                    wrap_width: None,
                }));
            sim.world_mut()
                .get_resource_or_insert_with(CanaisEscritos::default)
                .escrever(
                    tb,
                    FieldKey::new(VETOR, cv::TEXTO_TAMANHO, 0, 0),
                    TAMANHO[0] as f32,
                );
            hero.set_panel_visible("timeline", true);
            go(4);
            Some(format!(
                "[autokey-smoke] estrela {estrela} ({eb:#x}): Drop Shadow raio {}, perfil pontas {} \
                 meio {}; texto {texto} ({tb:#x}) «{CONTEUDO}» tamanho {}, fonte embutida, quebra Auto",
                RAIO[0], PERFIL[0][0], PERFIL[0][1], TAMANHO[0]
            ))
        }
        s @ 4..=90 => {
            go(s + 1);
            let [(estrela, eb), (texto, tb)] = dois(vec)?;
            let te = Entity::from_bits(tb);
            match s {
                8 => {
                    tl.flags.auto_key = true;
                    ph.pause();
                    ph.seek(QUADRO_12);
                    seleciona(vec, hero, texto, tb);
                    None
                }
                14 => {
                    let familia = familia_serifa();
                    let f = familia.clone();
                    receita(sim, te, |t| {
                        t.size = TAMANHO[1];
                        t.family = f;
                        t.wrap_width = Some(QUEBRA);
                    })?;
                    Some(format!(
                        "[autokey-smoke] q12: texto ← tamanho {} · família {familia:?} · quebra Fixed {QUEBRA}",
                        TAMANHO[1]
                    ))
                }
                18 => {
                    seleciona(vec, hero, estrela, eb);
                    None
                }
                22 => {
                    sombra_em(sim, Entity::from_bits(eb), 1)?;
                    crate::profile_live::arm(sim, &vec.entities, &[estrela], &perfil(1));
                    Some(format!(
                        "[autokey-smoke] q12: sombra ← raio {} cor {:?} · perfil ← pontas {} meio {} · {}",
                        RAIO[1],
                        SOMBRA[1],
                        PERFIL[1][0],
                        PERFIL[1][1],
                        trilhas(tl, eb)
                    ))
                }
                28 => {
                    ph.seek(QUADRO_24);
                    seleciona(vec, hero, texto, tb);
                    None
                }
                33 => {
                    receita(sim, te, |t| {
                        t.size = TAMANHO[0];
                        t.family = None;
                        t.wrap_width = None;
                    })?;
                    Some(format!(
                        "[autokey-smoke] q24: texto de volta · {}",
                        trilhas(tl, tb)
                    ))
                }
                37 => {
                    seleciona(vec, hero, estrela, eb);
                    None
                }
                41 => {
                    sombra_em(sim, Entity::from_bits(eb), 0)?;
                    crate::profile_live::arm(sim, &vec.entities, &[estrela], &perfil(0));
                    Some(format!(
                        "[autokey-smoke] q24: estrela de volta · {}",
                        trilhas(tl, eb)
                    ))
                }
                // `=12`: o texto fica seleccionado ANTES do seek — o painel tem de seguir o
                // documento que a timeline escreve sem a selecção mudar (`seed_text_sliders`).
                44 if modo == 1 => {
                    seleciona(vec, hero, texto, tb);
                    None
                }
                46 => {
                    ph.seek(QUADRO_18);
                    None
                }
                56 => Some(format!(
                    "[autokey-smoke] q18 (a meio): {} · estrela {} · texto {}",
                    medir(sim, eb, tb),
                    trilhas(tl, eb),
                    trilhas(tl, tb)
                )),
                64 => {
                    hero.store.set_panel_scroll(
                        ph2d_editor_core::ids::VECTOR_PANEL,
                        ROLAGEM[usize::from(modo)],
                    );
                    None
                }
                90 => Some(format!("[autokey-smoke] fim · {}", medir(sim, eb, tb))),
                _ => None,
            }
        }
        _ => None,
    }
}

/// A estrela e o rectângulo-texto, por ordem de criação.
fn dois(vec: &VecState) -> Option<[(VecPathId, u64); 2]> {
    let mut it = vec.entities.iter().map(|(&id, &b)| (id, b));
    Some([it.next()?, it.next()?])
}

fn seleciona(vec: &mut VecState, hero: &mut HeroScreen, id: VecPathId, bits: u64) {
    vec.pen.select_many(&[id]);
    hero.gizmo.replace_selection(Some(bits));
}

/// Uma família do catálogo que o menu da fonte usa, visivelmente diferente da embutida.
fn familia_serifa() -> Option<String> {
    let todas = ph2d_system_fonts::library::pickable_families();
    ["DejaVu Serif", "Liberation Serif", "Noto Serif"]
        .iter()
        .find_map(|q| todas.iter().flatten().find(|f| f.as_str() == *q).cloned())
        .or_else(|| todas.iter().flatten().next().cloned())
}

fn receita(sim: &mut SimWorld, e: Entity, f: impl FnOnce(&mut VecTextParams)) -> Option<()> {
    match &mut *sim.world_mut().get_mut::<VecShape>(e)? {
        VecShape::Text(t) => f(t),
        VecShape::Param { .. } => return None,
    }
    Some(())
}

fn sombra(n: usize) -> FxOp {
    FxOp {
        radius: RAIO[n],
        offset: DESLOCA,
        color: SOMBRA[n],
        opacity: 1.0,
        ..FxOp::new(FxOp::DROP_SHADOW)
    }
}

fn sombra_em(sim: &mut SimWorld, e: Entity, n: usize) -> Option<()> {
    let mut f = sim.world_mut().get_mut::<VecFilter>(e)?;
    let op = f.ops.first_mut()?;
    op.radius = RAIO[n];
    op.color = SOMBRA[n];
    Some(())
}

fn perfil(n: usize) -> WidthStops {
    let [ponta, meio] = PERFIL[n];
    WidthStops::new(vec![
        WidthStop {
            pos: 0.0,
            mult: ponta,
        },
        WidthStop {
            pos: 0.5,
            mult: meio,
        },
        WidthStop {
            pos: 1.0,
            mult: ponta,
        },
    ])
}

/// O que a tela desenha: a sombra, o perfil e a receita do texto.
fn medir(sim: &SimWorld, eb: u64, tb: u64) -> String {
    let w = sim.world();
    let (e, t) = (Entity::from_bits(eb), Entity::from_bits(tb));
    let fx = w
        .get::<VecFilter>(e)
        .and_then(|f| f.ops.first().copied())
        .map_or_else(
            || "sem filtro".to_owned(),
            |o| format!("sombra raio {:.3} cor {:.2?}", o.radius, o.color),
        );
    let pf = w.get::<VecStrokeProfile>(e).map_or_else(
        || "sem perfil".to_owned(),
        |p| {
            let s: Vec<String> = p
                .stops
                .as_slice()
                .iter()
                .map(|s| format!("{:.2}@{:.2}", s.mult, s.pos))
                .collect();
            format!("perfil [{}]", s.join(" "))
        },
    );
    let tx = match w.get::<VecShape>(t) {
        Some(VecShape::Text(p)) => format!(
            "texto tamanho {:.3} família {:?} quebra {:?}",
            p.size, p.family, p.wrap_width
        ),
        _ => "sem texto".to_owned(),
    };
    format!("{fx} · {pf} · {tx}")
}

/// `=13`: a sprite e o Inspector.
fn sprite(
    nivel: u8,
    sim: &mut SimWorld,
    hero: &mut HeroScreen,
    tl: &mut TimelineState,
    ph: &mut Playhead,
) -> Option<String> {
    static SPRITE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    use std::sync::atomic::Ordering::Relaxed;
    match nivel {
        1 => {
            let e = sim
                .world_mut()
                .spawn((
                    Transform::from_translation(Vec2::new(0.0, 0.0)),
                    Sprite::atlas(
                        WHITE_TILE_KEY,
                        [LADO_SPRITE, LADO_SPRITE],
                        [BASE[0], BASE[1], BASE[2], 1.0],
                    ),
                    Name::new("Sprite".to_owned()),
                ))
                .id();
            SPRITE.store(e.to_bits(), Relaxed);
            hero.gizmo.replace_selection(Some(e.to_bits()));
            hero.set_panel_visible("timeline", true);
            hero.set_panel_visible("inspector", true);
            hero.store.bump_panel_z(ph2d_editor_core::ids::INSP_PANEL);
            hero.store
                .set_collapsed(ph2d_editor_core::ids::INSP_LIVE_COLOR_SECTION, false);
            // A emissão mora na Render Source.
            hero.store
                .set_collapsed(ph2d_editor_core::ids::INSP_LIVE_RENDER_SECTION, false);
            go(2);
            Some(format!(
                "[autokey-smoke] sprite verde {:#x} {LADO_SPRITE}×{LADO_SPRITE}, sem skew, sem emissão",
                e.to_bits()
            ))
        }
        s @ 2..=90 => {
            go(s + 1);
            let bits = SPRITE.load(Relaxed);
            let e = Entity::from_bits(bits);
            match s {
                8 => {
                    tl.flags.auto_key = true;
                    ph.pause();
                    ph.seek(QUADRO_12);
                    None
                }
                14 => {
                    sim.world_mut().get_mut::<Transform>(e)?.skew_x = SKEW;
                    let mut sp = sim.world_mut().get_mut::<Sprite>(e)?;
                    sp.tint[..3].copy_from_slice(&TINT);
                    sim.world_mut()
                        .entity_mut(e)
                        .insert(SpriteEmissive(EMISSAO));
                    Some(format!(
                        "[autokey-smoke] q12: skew ← {SKEW} · tint ← {TINT:?} · emissão ← {EMISSAO}"
                    ))
                }
                20 => Some(format!("[autokey-smoke] q12 · {}", trilhas(tl, bits))),
                24 => {
                    ph.seek(QUADRO_24);
                    None
                }
                30 => {
                    sim.world_mut().get_mut::<Transform>(e)?.skew_x = 0.0;
                    let mut sp = sim.world_mut().get_mut::<Sprite>(e)?;
                    sp.tint[..3].copy_from_slice(&BASE);
                    sim.world_mut().entity_mut(e).remove::<SpriteEmissive>();
                    Some("[autokey-smoke] q24: tudo de volta".to_owned())
                }
                36 => Some(format!("[autokey-smoke] q24 · {}", trilhas(tl, bits))),
                40 => {
                    ph.seek(QUADRO_18);
                    None
                }
                50 => Some(format!(
                    "[autokey-smoke] q18 (a meio): {} · {}",
                    medir_sprite(sim, e),
                    trilhas(tl, bits)
                )),
                54 => {
                    hero.store
                        .set_panel_scroll(ph2d_editor_core::ids::INSP_PANEL, ROLAGEM[2]);
                    None
                }
                90 => Some(format!("[autokey-smoke] fim · {}", medir_sprite(sim, e))),
                _ => None,
            }
        }
        _ => None,
    }
}

fn medir_sprite(sim: &SimWorld, e: Entity) -> String {
    let w = sim.world();
    let skew = w.get::<Transform>(e).map_or(f32::NAN, |t| t.skew_x);
    let tint = w.get::<Sprite>(e).map(|s| s.tint);
    let em = w.get::<SpriteEmissive>(e).map(|m| m.0);
    format!("skew {skew:.4} · tint {tint:.3?} · emissão {em:?}")
}
