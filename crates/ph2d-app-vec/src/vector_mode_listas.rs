//! ⭐ **As cenas 43 e 44** (`PH2D_OBJECT_MODE_SMOKE=43|44`, F43 — Effects e Filters no layout do
//! Figma). Um rectângulo com DOIS efeitos (Zig Zag, Wave) e DOIS filtros (Drop Shadow, Outer Glow),
//! selecionado, com o Inspector rolado até à secção Effects. A 43 abre a linha do Zig Zag (o painel
//! de propriedades à ESQUERDA do Inspector); a 44 abre a lista do «+» de Effects.

use crate::state::VecState;
use ph2d_ecs::{FxOp, SimWorld, VecFilter};
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_vec_scene::effect::{FxEntry, PathEffect};
use ph2d_vec_scene::{Paint, Rgba8, ShapeKind, VecPathId, VecScene};
use std::sync::atomic::{AtomicU8, Ordering};

/// O estágio — UM só, a seguir à cena 41 (`46`) e antes da 12 (`50..`).
pub(crate) const FIRST: u8 = crate::vector_mode_filtro_zoom::LAST + 1;
pub(crate) const LAST: u8 = FIRST;
const _: () = assert!(LAST < 50);

const AZUL: Rgba8 = Rgba8::new(90, 150, 220, 255);

static PASSO: AtomicU8 = AtomicU8::new(0);
/// Quantas vezes já rolou: a secção pode morar no fim do painel, onde a rolagem pára antes do alvo.
static ROLAGENS: AtomicU8 = AtomicU8::new(0);
const MAX_ROLAGENS: u8 = 40;

/// O efeito com números que SE VEEM na forma (o default do Zig Zag tem tamanho 0).
fn ajustado(mut e: PathEffect) -> PathEffect {
    let nomes: Vec<&str> = e.params().iter().map(|p| p.name).collect();
    for (i, n) in nomes.iter().enumerate() {
        let v = match *n {
            "Size" => 6.0,
            "Ridges" => 10.0,
            "Bend" => 20.0,
            _ => continue,
        };
        e.set(i, v);
    }
    e
}

/// Um quadro da cena; `Some(9)` = acabou. `menu` = a 44 (a lista do «+» em vez do painel).
pub fn step(
    vec: &mut VecState,
    scene: &mut VecScene,
    sim: &mut SimWorld,
    hero: &mut HeroScreen,
    menu: bool,
) -> Option<u8> {
    let passo = PASSO.load(Ordering::Relaxed);
    let ids: Vec<VecPathId> = vec.entities.keys().copied().collect();
    match passo {
        0 => {
            crate::vector_mode::draw_shape(vec, scene, ShapeKind::Rectangle, [0.0, 0.0], 1.2);
            PASSO.store(1, Ordering::Relaxed);
        }
        1 if ids.len() == 1 => {
            let p = scene.path_mut(ids[0])?;
            p.fill = Some(Paint::solid(AZUL));
            p.effects = vec![
                FxEntry::new(ajustado(PathEffect::ZigZag(
                    ph2d_vec_scene::fx_zigzag::ZigZagSpec::default(),
                ))),
                FxEntry::new(ajustado(PathEffect::Warp(
                    ph2d_vec_scene::fx_warp_presets::WarpSpec::new(
                        ph2d_vec_scene::fx_warp_presets::WarpStyle::Wave,
                    ),
                ))),
            ];
            let sombra = FxOp {
                radius: 0.06,
                offset: [0.08, -0.08],
                opacity: 0.6,
                ..FxOp::new(FxOp::DROP_SHADOW)
            };
            let brilho = FxOp {
                radius: 0.08,
                ..FxOp::new(FxOp::GLOW)
            };
            let filtro = VecFilter::from_ops(vec![sombra, brilho]);
            let _ = ph2d_vec_entities::filter::set_filter(sim, &vec.entities, &ids, Some(filtro));
            eprintln!("[cena 43] rectângulo com Zig Zag + Wave e Drop Shadow + Outer Glow");
            PASSO.store(2, Ordering::Relaxed);
        }
        // A forma entra em Edit um quadro depois do desenho; só então a selecção assenta nela.
        2 if vec.edit.objects.first().copied() == vec.entities.get(ids.first()?).copied() => {
            hero.gizmo
                .replace_selection(vec.entities.get(ids.first()?).copied());
            PASSO.store(3, Ordering::Relaxed);
        }
        3 => return abre(hero, menu),
        _ => {}
    }
    None
}

/// Rola o Inspector até a secção Effects ficar no alto e abre o que a cena mostra.
fn abre(hero: &mut HeroScreen, menu: bool) -> Option<u8> {
    let painel = ph2d_editor_core::ids::VECTOR_PANEL;
    let area = hero
        .escala()
        .rect_to_logical(hero.panel_rect_fisico(painel)?);
    let cabeca = hero
        .hit_index
        .rect_for(ph2d_tool_vector::ids::VECTOR_SECTION_EFFECTS)
        .filter(|r| r.h > 0.0);
    let alvo = area.y + area.h * 0.15;
    let Some(c) = cabeca else {
        // Ainda fora da vista (ou recortada): desce meio painel e volta a olhar.
        let s = hero.store.panel_scroll(painel);
        hero.store.set_panel_scroll(painel, s + area.h * 0.5);
        return None;
    };
    if (c.y - alvo).abs() > area.h * 0.05 && ROLAGENS.fetch_add(1, Ordering::Relaxed) < MAX_ROLAGENS
    {
        let s = hero.store.panel_scroll(painel);
        hero.store
            .set_panel_scroll(painel, (s + c.y - alvo).max(0.0));
        return None;
    }
    let id = if menu {
        ph2d_tool_vector::ids::VECTOR_FX_ADD_MENU
    } else {
        ph2d_tool_vector::ids::vector_fx_row_id(0)
    };
    hero.hit_index.rect_for(id)?;
    if let Some(InteractiveState::Dropdown { open, .. }) = hero.store.get_mut(id) {
        *open = true;
    }
    Some(9)
}
