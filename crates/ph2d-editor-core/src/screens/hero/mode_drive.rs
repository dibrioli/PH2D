//! ⭐⭐ **O QUADRO DO MODO** (spec/06 F2) — os modos do tipo do activo, a rede de segurança do modo
//! em curso, o pedido do quadro (seletor, `Tab`, aba de cima) e o seletor publicado.
//!
//! As leis são puras e moram no [`crate::object_mode`]; aqui elas correm sobre o Hero e o registo
//! de ferramentas. ⭐ **Cada família declara o seu modo e as portas dele** ([`ModeFamily`]) na crate
//! dela, e a shell só compõe a lista — o drop-crate do ADR-0075. O tipo de um objecto pergunta-se a
//! quem chama (`kind_of`): a fundação não conhece os marcadores.

use super::HeroScreen;
use crate::object_mode::{self, ModeRequest, ObjectMode, Step};
use crate::toast::{Toast, ToastQueue};
use crate::tool::ToolRegistry;
use ph2d_component_desc::ObjectKind;
use ph2d_i18n::tr_with;

/// ⭐ **Um modo que uma família abre** — os pares (tipo, modo) que ela declara e as três portas do
/// módulo dela. Uma família sem modo de criação não declara nada.
#[derive(Copy, Clone)]
pub struct ModeFamily {
    /// Os modos que ela abre, por tipo (D6).
    pub modes: &'static [(ObjectKind, ObjectMode)],
    /// O módulo tem a entidade em mãos?
    pub holds: fn(ObjectMode, &mut ToolRegistry) -> bool,
    /// Abre o módulo (a selecção já foi colapsada ao activo). `false` = recusou.
    pub enter: fn(ObjectMode, &mut ToolRegistry) -> bool,
    /// Larga o módulo.
    pub leave: fn(ObjectMode, &mut ToolRegistry),
}

fn family(families: &[ModeFamily], kind: ObjectKind, mode: ObjectMode) -> Option<&ModeFamily> {
    families.iter().find(|f| f.modes.contains(&(kind, mode)))
}

/// Volta a Object e larga o módulo do modo que estava (se ele ainda o tem).
fn leave_current(
    families: &[ModeFamily],
    kind_of: &dyn Fn(u64) -> ObjectKind,
    tools: &mut ToolRegistry,
    hero: &mut HeroScreen,
) {
    if let Some(prev) = hero.gizmo.mode.leave()
        && let Some(f) = family(families, kind_of(prev.entity), prev.mode)
        && (f.holds)(prev.mode, tools)
    {
        (f.leave)(prev.mode, tools);
    }
}

/// ⭐⭐ **O quadro do modo.** `kind_of`/`name_of` respondem pelos bits de uma entidade. Devolve se o
/// modo mudou.
pub fn drive(
    families: &[ModeFamily],
    kind_of: &dyn Fn(u64) -> ObjectKind,
    name_of: &dyn Fn(u64) -> String,
    tools: &mut ToolRegistry,
    hero: &mut HeroScreen,
    toasts: &mut ToastQueue,
    request: Option<ModeRequest>,
) -> bool {
    let mut changed = false;
    // 1. O activo e os modos que o TIPO dele declara.
    let active = object_mode::active_of(hero.gizmo.selection, &hero.gizmo.extra_selection);
    let modes: Vec<ObjectMode> = active.map_or_else(Vec::new, |bits| {
        let kind = kind_of(bits);
        let declared = families.iter().flat_map(|f| f.modes.iter());
        declared
            .filter(|(k, _)| *k == kind)
            .map(|(_, m)| *m)
            .collect()
    });
    hero.gizmo.mode.publish(active, &modes);
    // 2. A rede de segurança: quem perdeu a entidade (por qualquer porta) volta a Object.
    if let Some(current) = hero.gizmo.mode.active() {
        let held = family(families, kind_of(current.entity), current.mode)
            .is_some_and(|f| (f.holds)(current.mode, tools));
        let (sel, extras) = (hero.gizmo.selection, hero.gizmo.extra_selection.len());
        if !hero.gizmo.mode.still_holds(sel, extras, held) {
            leave_current(families, kind_of, tools, hero);
            changed = true;
        }
    }
    // 3. O pedido do quadro.
    if let Some(req) = request {
        match hero.gizmo.mode.resolve(req) {
            Step::Stay => {}
            Step::Enter(m) => {
                leave_current(families, kind_of, tools, hero);
                if let Some(bits) = active
                    && let Some(f) = family(families, kind_of(bits), m)
                {
                    // ⚠️ Colapsar ANTES de abrir: o módulo lê a selecção ao abrir (o Sculpt do
                    // Blender toma só o activo, e o Painter o documento seleccionado).
                    hero.gizmo.replace_selection(Some(bits));
                    if (f.enter)(m, tools) {
                        hero.gizmo.mode.enter(bits, m);
                        let label = m.label_key().tr();
                        toasts.push(Toast::info(tr_with(
                            "object_mode.entered",
                            &[("mode", &label)],
                        )));
                    }
                }
            }
            Step::Leave => {
                leave_current(families, kind_of, tools, hero);
                toasts.push(Toast::info(ph2d_i18n::tr("object_mode.left")));
            }
            Step::LeaveToDefault => {
                leave_current(families, kind_of, tools, hero);
                tools.activate_default();
            }
            Step::Refuse => {
                toasts.push(Toast::info(match active {
                    None => ph2d_i18n::tr("object_mode.nothing_selected").to_string(),
                    Some(bits) => tr_with("object_mode.only_object", &[("name", &name_of(bits))]),
                }));
            }
        }
        changed = true;
    }
    // 4. O seletor — em todo quadro, vazio incluído.
    let menu = hero.gizmo.mode.menu(&mut hero.store);
    hero.store.publish_mode_menu(menu);
    changed
}

/// ⭐ **O cadeado numa porta de selecção** — `true` = a troca foi recusada (e o aviso já foi dado).
/// As três portas (o clique no canvas, o laço, a linha da Hierarquia) perguntam AQUI.
pub fn refused(
    hero: &HeroScreen,
    target: Option<u64>,
    additive: bool,
    toasts: &mut ToastQueue,
) -> bool {
    let locked = hero.gizmo.mode.locked_entity();
    let refuse = object_mode::decide(locked, target, additive) == object_mode::Decision::Refuse;
    if refuse {
        toasts.push(Toast::warning(object_mode::refusal(&hero.gizmo.mode)));
    }
    refuse
}

/// ⭐ **O botão direito no canvas livre** (spec/06 escolha 3): em Object abre o menu Add — o mesmo
/// pedido do `+` da Hierarquia —; num modo de criação é do módulo. `true` = tomou-o.
pub fn right_click_on_canvas(hero: &mut HeroScreen) -> bool {
    if hero.gizmo.mode.active().is_some() {
        return false;
    }
    use crate::action_bus::{EditorAction, HierRequest};
    hero.bus.push(EditorAction::Hierarchy(HierRequest::AddRoot));
    true
}

#[cfg(test)]
#[path = "mode_drive_tests.rs"]
mod tests;
