//! ⭐⭐ **O QUADRO DO MODO** (spec/06 F2) — os modos do tipo do activo, a rede de segurança do modo
//! em curso, o pedido do quadro (seletor, `Tab`, aba de cima) e o seletor publicado.
//!
//! As leis são puras e moram no [`crate::object_mode`]; aqui elas correm sobre o Hero e o registo
//! de ferramentas. ⭐ **Cada família declara o seu modo e as portas dele** ([`ModeFamily`]) na crate
//! dela, e a shell só compõe a lista — o drop-crate do ADR-0075. O tipo de um objecto pergunta-se a
//! quem chama (`kind_of`): a fundação não conhece os marcadores.

use super::HeroScreen;
use crate::object_mode::{self, ActiveMode, ModeRequest, ObjectMode, Step};
use crate::toast::{Toast, ToastQueue};
use crate::tool::ToolRegistry;
use ph2d_component_desc::ObjectKind;
use ph2d_i18n::tr_with;

/// ⭐ **Uma família que abre modos** — os pares (tipo, modo) que ela declara e as portas do módulo
/// dela. Uma família sem modo de criação não declara nada.
///
/// ⚠️ **Um trait, e não uma tabela de `fn`:** cada família abre o modo com os recursos DELA (o
/// Painter só precisa do registo de ferramentas; a escultura, da cena e do mapa peça↔entidade). A
/// shell constrói as famílias em cada quadro com o que cada uma empresta — a assinatura comum de
/// recursos seria uma mentira (o mesmo desvio do `object_add`, spec/06 F1).
pub trait ModeFamily {
    /// Os modos que ela abre, por tipo (D6).
    fn modes(&self) -> &'static [(ObjectKind, ObjectMode)];
    /// O módulo tem `entity` em mãos, neste modo?
    fn holds(&mut self, mode: ObjectMode, entity: u64, tools: &mut ToolRegistry) -> bool;
    /// Abre o módulo sobre `entity` (a selecção já foi colapsada a ela). `false` = recusou.
    fn enter(&mut self, mode: ObjectMode, entity: u64, tools: &mut ToolRegistry) -> bool;
    /// Larga o módulo.
    fn leave(&mut self, mode: ObjectMode, entity: u64, tools: &mut ToolRegistry);
    /// ⭐ **O módulo SEGUE o modo**, em todo quadro, depois do pedido: o que ele tem em mãos sem o
    /// modo o declarar é largado aqui (a outra metade da rede `still_holds`).
    fn follow(&mut self, _current: Option<ActiveMode>, _tools: &mut ToolRegistry) {}
    /// ⭐ **Um objecto que NASCEU num modo** pede-o uma vez: `(entidade, modo)`. O quadro selecciona
    /// a entidade e entra.
    fn wants(&mut self) -> Option<(u64, ObjectMode)> {
        None
    }
}

fn family<'a>(
    families: &'a mut [&mut dyn ModeFamily],
    kind: ObjectKind,
    mode: ObjectMode,
) -> Option<&'a mut dyn ModeFamily> {
    families
        .iter_mut()
        .find(|f| f.modes().contains(&(kind, mode)))
        .map(|f| &mut **f as &mut dyn ModeFamily)
}

/// Volta a Object e larga o módulo do modo que estava (se ele ainda o tem).
fn leave_current(
    families: &mut [&mut dyn ModeFamily],
    kind_of: &dyn Fn(u64) -> ObjectKind,
    tools: &mut ToolRegistry,
    hero: &mut HeroScreen,
) {
    if let Some(prev) = hero.gizmo.mode.leave()
        && let Some(f) = family(families, kind_of(prev.entity), prev.mode)
        && f.holds(prev.mode, prev.entity, tools)
    {
        f.leave(prev.mode, prev.entity, tools);
    }
}

/// ⭐⭐ **O quadro do modo.** `kind_of`/`name_of` respondem pelos bits de uma entidade. Devolve se o
/// modo mudou.
pub fn drive(
    families: &mut [&mut dyn ModeFamily],
    kind_of: &dyn Fn(u64) -> ObjectKind,
    name_of: &dyn Fn(u64) -> String,
    tools: &mut ToolRegistry,
    hero: &mut HeroScreen,
    toasts: &mut ToastQueue,
    request: Option<ModeRequest>,
) -> bool {
    let mut changed = false;
    // 0. Um objecto que nasceu num modo pede-o (só sem pedido do artista neste quadro).
    let request = request.or_else(|| {
        let (bits, mode) = families.iter_mut().find_map(|f| f.wants())?;
        hero.gizmo.replace_selection(Some(bits));
        Some(ModeRequest::Enter(mode))
    });
    // 1. O activo e os modos que o TIPO dele declara.
    let active = object_mode::active_of(hero.gizmo.selection, &hero.gizmo.extra_selection);
    let modes: Vec<ObjectMode> = active.map_or_else(Vec::new, |bits| {
        let kind = kind_of(bits);
        let declared = families.iter().flat_map(|f| f.modes().iter());
        declared
            .filter(|(k, _)| *k == kind)
            .map(|(_, m)| *m)
            .collect()
    });
    hero.gizmo.mode.publish(active, &modes);
    // 2. A rede de segurança: quem perdeu a entidade (por qualquer porta) volta a Object.
    if let Some(current) = hero.gizmo.mode.active() {
        let held = family(families, kind_of(current.entity), current.mode)
            .is_some_and(|f| f.holds(current.mode, current.entity, tools));
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
                    if f.enter(m, bits, tools) {
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
    // 4. Cada módulo segue o modo que ficou.
    let current = hero.gizmo.mode.active();
    for f in families.iter_mut() {
        f.follow(current, tools);
    }
    // 5. O seletor — em todo quadro, vazio incluído.
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

/// ⭐ **O gizmo de transformação É o modo Object** (D6): num modo de criação (Paint, Sculpt) ele
/// seria um controlo que não responde por cima do módulo. A shell pergunta aqui ao decidir se o
/// pinta; a selecção fica armada.
#[must_use]
pub fn object_gizmo_shows(hero: &HeroScreen) -> bool {
    hero.gizmo.mode.active().is_none()
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
