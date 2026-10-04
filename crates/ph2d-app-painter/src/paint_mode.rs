//! ⭐⭐ **IMAGE ▸ PAINT · MASK** — os modos que esta família declara e como eles abrem e largam o
//! Painter sobre a imagem activa (spec/06 F2 + a F3 da Imagem; D6: *imagem → Object · Paint · Mask*).
//!
//! ⭐ **Os dois modos são o MESMO Painter com outro alvo:** Paint pinta a camada, Mask pinta a
//! MÁSCARA dela (criada branca se faltar — escolha do dono, 04/10). Trocar entre eles nunca larga o
//! Painter (largá-lo assa a imagem e desmonta a tela): quem o larga é o [`ModeFamily::follow`],
//! quando nenhum dos dois ficou. ⚠️ O alvo põe-se no `follow` e não no `enter`: ao entrar, o
//! Painter ainda não recebeu a imagem (a shell dá-lha mais tarde no quadro).
//!
//! O Painter já é «o desta entidade» (`PainterTool::bound_doc` = a selecção): entrar no modo é pôr
//! o Painter em mãos com a selecção colapsada ao activo (`screens::hero::mode_drive`), e o cadeado
//! da selecção (`ph2d_editor_core::object_mode::decide`) guarda-a lá enquanto o modo durar.
//!
//! ⚠️ **A porta do IMG não se aplica aqui:** o modo É a porta do Painter. O botão dele saiu da
//! barra IMG na mesma fase (dois caminhos para o mesmo módulo divergem — spec/06 §5).

use ph2d_component_desc::ObjectKind;
use ph2d_editor_core::object_mode::{ActiveMode, ObjectMode};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::screens::hero::mode_drive::ModeFamily;
use ph2d_editor_core::{ToolId, ToolRegistry};
use ph2d_tool_painter::PainterTool;

/// ⭐ **O modo que esta família declara, e as portas dele** — a shell junta-a às outras. Só
/// precisa do registo de ferramentas: o Painter já guarda o documento da selecção.
pub struct Family;

impl ModeFamily for Family {
    fn modes(&self) -> &'static [(ObjectKind, ObjectMode)] {
        &[
            (ObjectKind::Image, ObjectMode::Paint),
            (ObjectKind::Image, ObjectMode::Mask),
        ]
    }
    fn holds(&mut self, _: ObjectMode, _: u64, tools: &mut ToolRegistry) -> bool {
        holds_an_image(tools)
    }
    fn enter(&mut self, _: ObjectMode, _: u64, tools: &mut ToolRegistry) -> bool {
        enter(tools)
    }
    /// ⚠️ Não larga o Painter: o modo seguinte pode ser o irmão (ver o cabeçalho).
    fn leave(&mut self, _: ObjectMode, _: u64, _: &mut ToolRegistry) {}
    fn follow(&mut self, current: Option<ActiveMode>, tools: &mut ToolRegistry) {
        // ⚠️ Só com o Painter sobre uma IMAGEM: o Paint da escultura também se chama `Paint`, e é
        // da família dela (a tela da peça).
        let ours = current.filter(|a| matches!(a.mode, ObjectMode::Paint | ObjectMode::Mask));
        let before = FOLLOWING.get();
        let next = match (ours, painter_on_image(tools)) {
            (Some(a), Some(p)) => Some(aim(p, a, before.filter(|b| b.mode == a))),
            (None, Some(_)) if before.is_some() => {
                leave(tools);
                None
            }
            _ => None,
        };
        FOLLOWING.set(next);
    }
    fn refusal(&mut self) -> Option<String> {
        let wrong = FOLLOWING.get().is_some_and(|f| f.wrong);
        wrong.then(|| ph2d_i18n::tr("object_mode.mask_needs_a_layer").to_string())
    }
    /// ⭐ **O artista mudou o alvo à mão** (a linha *Mask* do painel de camadas em Paint, ou uma
    /// camada de cor em Mask): o modo segue-o — e um Mask sem máscara possível volta a Paint.
    fn wants(&mut self, tools: &mut ToolRegistry) -> Option<(u64, ObjectMode)> {
        let f = FOLLOWING.get()?;
        if f.wrong {
            return Some((f.mode.entity, ObjectMode::Paint));
        }
        f.target?;
        let on_mask = painter_on_image(tools).and_then(|p| active_is_mask(p))?;
        let want = if on_mask {
            ObjectMode::Mask
        } else {
            ObjectMode::Paint
        };
        (want != f.mode.mode).then_some((f.mode.entity, want))
    }
}

/// O que o [`Family::follow`] deixou no quadro anterior: o modo e o alvo que ele pôs.
#[derive(Clone, Copy)]
struct Following {
    mode: ActiveMode,
    /// A camada que o modo pôs como alvo — `None` enquanto o Painter não tem a imagem.
    target: Option<ph2d_tool_painter::LayerId>,
    /// Mask sem máscara possível (o tecto de camadas): o modo devolve-se a Paint, com o aviso.
    wrong: bool,
}

thread_local! {
    static FOLLOWING: std::cell::Cell<Option<Following>> = const { std::cell::Cell::new(None) };
}

/// O Painter em mãos sobre uma imagem.
fn painter_on_image(tools: &mut ToolRegistry) -> Option<&mut PainterTool> {
    tools
        .active_mut()
        .and_then(|t| t.as_any_mut().downcast_mut::<PainterTool>())
        .filter(|p| !p.on_screen_canvas())
}

/// A camada activa é uma máscara? `None` = o Painter ainda não tem camadas.
fn active_is_mask(p: &PainterTool) -> Option<bool> {
    let id = p.layers().active()?;
    Some(p.layers().is_mask(id))
}

/// ⭐ **Põe o alvo do modo** uma vez, ao entrar (`kept` = o do quadro anterior, se o modo é o mesmo):
/// Paint pinta a camada dona da máscara; Mask pinta a máscara da camada activa, criando-a se faltar.
fn aim(p: &mut PainterTool, mode: ActiveMode, kept: Option<Following>) -> Following {
    let mut f = Following {
        mode,
        target: None,
        wrong: false,
    };
    if let Some(k) = kept.filter(|k| k.target.is_some()) {
        return k;
    }
    let Some(active) = p.layers().active() else {
        return f;
    };
    let target = match (mode.mode, p.layers().is_mask(active)) {
        (ObjectMode::Mask, false) => p
            .layers()
            .get(active)
            .and_then(|l| l.mask)
            .or_else(|| p.add_mask_to_active()),
        (ObjectMode::Paint, true) => p.layers().owner_of_mask(active),
        _ => Some(active),
    };
    match target {
        Some(t) => {
            p.select_layer(t);
            f.target = Some(t);
        }
        None => f.wrong = true,
    }
    f
}

/// O id do Painter no registo de ferramentas.
const PAINTER: &str = "painter";

/// O Painter está em mãos **sobre uma imagem** — e não sobre a tela da peça 3D, que é a pintura
/// da escultura (`ph2d_app_sculpt3d::painter_na_malha`, o Paint do Sculpt na F3).
#[must_use]
pub fn holds_an_image(tools: &mut ToolRegistry) -> bool {
    tools
        .active_mut()
        .and_then(|t| t.as_any_mut().downcast_mut::<PainterTool>())
        .is_some_and(|p| !p.on_screen_canvas())
}

/// ⭐ **Entra**: põe o Painter em mãos — o quadro do modo já colapsou a selecção ao activo, que é o
/// documento que ele abre. Devolve `false` se o registo recusou.
pub fn enter(tools: &mut ToolRegistry) -> bool {
    holds_an_image(tools) || tools.set_active(&ToolId::new(PAINTER))
}

/// **Larga**: devolve o canvas à ferramenta de omissão, se o Painter ainda o tem.
pub fn leave(tools: &mut ToolRegistry) {
    if tools
        .active()
        .is_some_and(|t| t.id() == ToolId::new(PAINTER))
    {
        tools.activate_default();
    }
}

/// ⭐ **O smoke do modo** — `PH2D_OBJECT_MODE_SMOKE=1` selecciona a 1.ª imagem da cena e abre o
/// seletor *Mode* sobre ela; `=2` entra logo em *Paint Mode*; `=7` em *Mask Mode*. É como a foto do passo do smoke o
/// apanha, já que o clique sintético não chega à tela virtual
/// (`docs/Components/ferramentas/fotografa_cena.sh`). Corre uma vez; devolve o pedido a drenar.
pub fn smoke_step(
    sim: &mut ph2d_ecs::SimWorld,
    hero: &mut HeroScreen,
) -> Option<ph2d_editor_core::object_mode::ModeRequest> {
    use std::sync::atomic::{AtomicU8, Ordering};
    // 0 = por ler · 1 = abrir o seletor · 2 = entrar em Paint · 7 = em Mask · 9 = feito.
    static STAGE: AtomicU8 = AtomicU8::new(0);
    static REQUESTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    let stage = match STAGE.load(Ordering::Relaxed) {
        0 => {
            let want = match std::env::var("PH2D_OBJECT_MODE_SMOKE").as_deref() {
                Ok("1") => 1,
                Ok("2") => 2,
                Ok("7") => 7,
                _ => 9,
            };
            STAGE.store(want, Ordering::Relaxed);
            want
        }
        s => s,
    };
    if stage == 9 {
        return None;
    }
    if hero.gizmo.selection.is_none() {
        let first = sim
            .world_mut()
            .query::<(ph2d_ecs::Entity, &ph2d_render::Sprite)>()
            .iter(sim.world())
            .map(|(e, _)| e.to_bits())
            .min();
        if first.is_some() {
            hero.gizmo.replace_selection(first);
        } else if !REQUESTED.swap(true, Ordering::Relaxed) {
            // A cena de arranque é vazia: a imagem nasce pela porta do `Ctrl+N`, em branco.
            hero.store.set_new_image_bg(2);
            hero.store.request_new_image();
        }
        return None;
    }
    if stage == 2 || stage == 7 {
        STAGE.store(9, Ordering::Relaxed);
        let mode = if stage == 7 {
            ObjectMode::Mask
        } else {
            ObjectMode::Paint
        };
        return Some(ph2d_editor_core::object_mode::ModeRequest::Enter(mode));
    }
    // O chip só se abre depois de pintado (o pulldown ancora-se no rect dele).
    let chip = ph2d_editor_core::ids::area_menu_button(0);
    if hero.hit_index.rect_for(chip).is_some() {
        STAGE.store(9, Ordering::Relaxed);
        hero.apply_event(ph2d_editor_core::interaction::WidgetEvent::Click(chip));
    }
    None
}

#[cfg(test)]
#[path = "paint_mode_tests.rs"]
mod tests;
