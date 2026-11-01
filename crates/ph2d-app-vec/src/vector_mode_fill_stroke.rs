//! ⭐ **As cenas 45 e 46** (`PH2D_OBJECT_MODE_SMOKE=45|46`, F44 — Fill e Stroke no layout do Figma).
//! Um rectângulo com degradê linear (azul → roxo), traço escuro, uma SOMBRA (camada de
//! preenchimento deslocada) e um segundo traço amarelo por fora, selecionado, com o Inspector rolado
//! até à secção Fill. A 45 abre o painel do preenchimento de base (o tipo e o degradê, à ESQUERDA do
//! Inspector); a 46 abre o «Stroke settings» pelo ⚙ (Cap e Join em glifos).

use crate::state::VecState;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_vec_scene::{
    GradientStop, Paint, PaintEntry, Rgba8, ShapeKind, StrokeAlign, StrokeSpec, VecPathId, VecScene,
};
use std::sync::atomic::{AtomicU8, Ordering};

/// O estágio — UM só, a seguir às cenas 43/44 e antes da 12 (`50..`).
pub(crate) const FIRST: u8 = crate::vector_mode_listas::LAST + 1;
pub(crate) const LAST: u8 = FIRST;
const _: () = assert!(LAST < 50);

const AZUL: Rgba8 = Rgba8::new(70, 130, 230, 255);
const ROXO: Rgba8 = Rgba8::new(150, 80, 210, 255);
const ESCURO: Rgba8 = Rgba8::new(30, 30, 40, 255);
const SOMBRA: Rgba8 = Rgba8::new(20, 20, 30, 140);
const AMARELO: Rgba8 = Rgba8::new(245, 200, 60, 255);

static PASSO: AtomicU8 = AtomicU8::new(0);
static ROLAGENS: AtomicU8 = AtomicU8::new(0);
const MAX_ROLAGENS: u8 = 40;

/// Um quadro da cena; `Some(9)` = acabou. `traco` = a 46 (o «Stroke settings» em vez do Fill).
pub fn step(
    vec: &mut VecState,
    scene: &mut VecScene,
    hero: &mut HeroScreen,
    traco: bool,
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
            p.fill = Some(Paint::Linear {
                stops: vec![GradientStop::new(0.0, AZUL), GradientStop::new(1.0, ROXO)],
                start: [-0.6, 0.0],
                end: [0.6, 0.0],
            });
            let mut base = StrokeSpec::new(ESCURO, 0.04);
            base.align = StrokeAlign::Centre;
            p.stroke = Some(base);
            let mut sombra = PaintEntry::fill(Paint::solid(SOMBRA));
            sombra.offset = [0.15, -0.15];
            let mut fora = StrokeSpec::new(AMARELO, 0.05);
            fora.align = StrokeAlign::Outer;
            // A sombra por BAIXO de tudo não existe na pilha (ela pinta por cima do chão): fica
            // por cima, deslocada — o «offset print» que motivou a camada.
            p.paints = vec![sombra, PaintEntry::stroke(fora)];
            eprintln!(
                "[cena {}] rectângulo: degradê azul→roxo, traço escuro, sombra deslocada e traço \
                 amarelo por fora",
                if traco { 46 } else { 45 }
            );
            PASSO.store(2, Ordering::Relaxed);
        }
        // A forma entra em Edit um quadro depois do desenho; só então a selecção assenta nela.
        2 if vec.edit.objects.first().copied() == vec.entities.get(ids.first()?).copied() => {
            hero.gizmo
                .replace_selection(vec.entities.get(ids.first()?).copied());
            PASSO.store(3, Ordering::Relaxed);
        }
        3 => return abre(hero, traco),
        _ => {}
    }
    None
}

/// Rola o Inspector até a secção Fill ficar no alto e abre o painel que a cena mostra.
fn abre(hero: &mut HeroScreen, traco: bool) -> Option<u8> {
    let painel = ph2d_editor_core::ids::VECTOR_PANEL;
    let area = hero.store.panel_rect(painel)?;
    let cabeca = hero
        .hit_index
        .rect_for(ph2d_tool_vector::ids::VECTOR_SECTION_FILL)
        .filter(|r| r.h > 0.0);
    let alvo = area.y + area.h * 0.15;
    let Some(c) = cabeca else {
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
    let id = if traco {
        ph2d_panel_vector::ids::VECTOR_STROKE_SETTINGS
    } else {
        ph2d_panel_vector::ids::VECTOR_FILL_ROW
    };
    hero.hit_index.rect_for(id)?;
    if let Some(InteractiveState::Dropdown { open, .. }) = hero.store.get_mut(id) {
        *open = true;
    }
    Some(9)
}
