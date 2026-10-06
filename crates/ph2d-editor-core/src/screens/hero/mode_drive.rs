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
/// Painter só precisa do registo de ferramentas; o vetor, da cena e das formas do objecto). A
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
    /// a entidade e entra. Também a porta antiga: a ferramenta de um modo que chegou à mão sem ele
    /// (por isso o registo).
    fn wants(&mut self, _tools: &mut ToolRegistry) -> Option<(u64, ObjectMode)> {
        None
    }
    /// ⭐ **O que se edita DENTRO de `entity`** no modo em curso: `None` = o modo edita o objecto
    /// inteiro (o cadeado exige a selecção exacta); `Some` = as partes que a selecção pode ter sem o
    /// modo cair (o Edit do vetor: as formas do objecto são linhas próprias na Hierarquia).
    fn parts(&mut self, _entity: u64) -> Option<Vec<u64>> {
        None
    }
    /// ⭐ **De que objecto desta família `bits` é parte** — o seletor e o `Tab` sobre uma parte
    /// respondem pelo dono (uma forma seleccionada na Hierarquia oferece o Edit do objecto dela).
    fn owner_of(&mut self, _bits: u64) -> Option<u64> {
        None
    }
    /// ⭐ **A selecção transforma-se pelo gizmo de objecto** neste modo (o Edit do vetor: mover,
    /// girar e escalar a forma — a trancada e as que entraram com ela). `false` = o gizmo só existe
    /// em Object (D6: num modo de criação seria um controlo por cima do módulo).
    fn parts_take_the_object_gizmo(&self, _mode: ObjectMode) -> bool {
        false
    }
    /// ⭐ **Multi-objecto** (o Edit do Blender): `true` = entrar em `mode` leva junto os objectos
    /// do MESMO tipo seleccionados — chegam a [`Self::enter_with`], voltam como [`Self::parts`], e
    /// sair devolve-os à selecção. `false` = a selecção colapsa no activo.
    fn joins(&self, _mode: ObjectMode) -> bool {
        false
    }
    /// ⭐⭐ **O modo é do TIPO** (dono, 05/10: *«o modo de edição significa que todos os objetos daquele
    /// tipo estão em modo de edição. Ao clicar num objeto de outro tipo, o objeto deve ser
    /// selecionado mas em modo object»*): `true` = as [`Self::parts`] são todos os objectos do tipo,
    /// e escolher um de outro tipo não é recusado — a selecção muda e a rede `still_holds` volta a
    /// Object.
    fn holds_the_whole_kind(&self, _mode: ObjectMode) -> bool {
        false
    }
    /// ⭐ **A HERDEIRA** — a entidade trancada (`locked`) morreu (o Soldar consome as formas, juntar
    /// dois caminhos apaga um): `Some` = outra entidade da família continua o modo, sem tocar a
    /// selecção. Só chamada quando o modo em curso deixou de se segurar.
    fn heir(&mut self, _mode: ObjectMode, _locked: u64, _tools: &mut ToolRegistry) -> Option<u64> {
        None
    }
    /// ⭐ **A selecção que era só PARTES do objecto sobrevive à entrada** em `mode` (o Edit/Pose do
    /// esqueleto: o osso escolhido é o pai do próximo e o que se pose). `false` = colapsa no objecto.
    fn keeps_parts_selected(&self, _mode: ObjectMode) -> bool {
        false
    }
    /// Abre o módulo sobre `entity` e os `joined` (só chamada quando [`Self::joins`]).
    fn enter_with(
        &mut self,
        mode: ObjectMode,
        entity: u64,
        _joined: &[u64],
        tools: &mut ToolRegistry,
    ) -> bool {
        self.enter(mode, entity, tools)
    }
}

/// Os objectos seleccionados do tipo de `active`, sem ele — os que um modo que
/// [junta](ModeFamily::joins) leva consigo.
fn same_kind_selected(
    hero: &HeroScreen,
    kind_of: &dyn Fn(u64) -> ObjectKind,
    active: u64,
) -> Vec<u64> {
    let kind = kind_of(active);
    let selected = hero
        .gizmo
        .selection
        .iter()
        .chain(&hero.gizmo.extra_selection);
    selected
        .copied()
        .filter(|b| *b != active && kind_of(*b) == kind)
        .collect()
}

/// Deixa seleccionados `joined` e, por último (o activo do Blender), `active`.
fn select_together(hero: &mut HeroScreen, active: u64, joined: &[u64]) {
    hero.gizmo
        .replace_selection(joined.first().copied().or(Some(active)));
    for b in joined.iter().skip(1).chain([&active]) {
        hero.gizmo.add_to_selection(*b);
    }
}

/// Repõe a selecção de antes do quadro se ela era só `parts` (vazia ou com algo de fora: fica).
fn restore_parts(hero: &mut HeroScreen, before: &(Option<u64>, Vec<u64>), parts: Option<&[u64]>) {
    let (Some(primary), Some(parts)) = (before.0, parts) else {
        return;
    };
    if !std::iter::once(&primary)
        .chain(&before.1)
        .all(|b| parts.contains(b))
    {
        return;
    }
    hero.gizmo.replace_selection(Some(primary));
    for b in &before.1 {
        hero.gizmo.add_to_selection(*b);
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

/// Publica o activo e os modos do tipo dele; devolve o activo. Num modo, o activo é a entidade
/// trancada (a selecção pode ser uma parte dela, ou nenhuma); em Object, uma parte responde pelo
/// [dono](ModeFamily::owner_of).
fn publish_active(
    families: &mut [&mut dyn ModeFamily],
    kind_of: &dyn Fn(u64) -> ObjectKind,
    hero: &mut HeroScreen,
) -> Option<u64> {
    let picked = object_mode::active_of(hero.gizmo.selection, &hero.gizmo.extra_selection);
    let active = hero.gizmo.mode.locked_entity().or_else(|| {
        let bits = picked?;
        Some(
            families
                .iter_mut()
                .find_map(|f| f.owner_of(bits))
                .unwrap_or(bits),
        )
    });
    let modes: Vec<ObjectMode> = active.map_or_else(Vec::new, |bits| {
        let kind = kind_of(bits);
        let declared = families.iter().flat_map(|f| f.modes().iter());
        declared
            .filter(|(k, _)| *k == kind)
            .map(|(_, m)| *m)
            .collect()
    });
    hero.gizmo.mode.publish(active, &modes);
    active
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
    let before = (hero.gizmo.selection, hero.gizmo.extra_selection.clone());
    // 0. Um objecto que nasceu num modo pede-o (só sem pedido do artista neste quadro). ⚠️ SÓ ele:
    // a selecção de antes não entra junto — a forma desenhada num Edit levaria a anterior com ela.
    let mut born = None;
    let request = request.or_else(|| {
        let (bits, mode) = families.iter_mut().find_map(|f| f.wants(tools))?;
        select_together(hero, bits, &[]);
        born = Some(bits);
        Some(ModeRequest::Enter(mode))
    });
    // 1. O activo e os modos que o TIPO dele declara.
    let mut active = publish_active(families, kind_of, hero);
    // 2. A rede de segurança: quem perdeu a entidade (por qualquer porta) volta a Object.
    let mut fell_from = None;
    if let Some(current) = hero.gizmo.mode.active() {
        let f = family(families, kind_of(current.entity), current.mode);
        let (held, parts) = f.map_or((false, None), |f| {
            let held = f.holds(current.mode, current.entity, tools);
            (held, f.parts(current.entity))
        });
        hero.gizmo.mode.publish_parts(parts);
        let (sel, extras) = (hero.gizmo.selection, &hero.gizmo.extra_selection);
        if !hero.gizmo.mode.still_holds(sel, extras, held) {
            let heir = families
                .iter_mut()
                .find_map(|f| f.heir(current.mode, current.entity, tools));
            if let Some(h) = heir {
                hero.gizmo.mode.enter(h, current.mode);
                let parts = family(families, kind_of(h), current.mode).and_then(|f| f.parts(h));
                hero.gizmo.mode.publish_parts(parts);
            } else {
                fell_from = Some(current.mode);
                leave_current(families, kind_of, tools, hero);
            }
            active = publish_active(families, kind_of, hero);
            changed = true;
        }
    }
    // 3. O pedido do quadro.
    if let Some(req) = request {
        match hero.gizmo.mode.resolve(req) {
            // ⭐ Num modo do TIPO o nascido já é parte dele (a rede não cai): o modo PASSA a ele.
            Step::Stay => {
                if let (Some(b), Some(a)) = (born, hero.gizmo.mode.active())
                    && b != a.entity
                    && hero.gizmo.mode.whole_kind()
                    && let Some(f) = family(families, kind_of(b), a.mode)
                    && f.enter_with(a.mode, b, &[], tools)
                {
                    hero.gizmo.mode.enter(b, a.mode);
                    hero.gizmo.mode.publish_parts(f.parts(b));
                }
            }
            Step::Enter(m) => {
                leave_current(families, kind_of, tools, hero);
                if let Some(bits) = active
                    && let Some(f) = family(families, kind_of(bits), m)
                {
                    // ⚠️ Colapsar ANTES de abrir: o módulo lê a selecção ao abrir (o Painter toma
                    // só o documento seleccionado) — salvo os do mesmo tipo, num modo que os junta.
                    let joined = if f.joins(m) {
                        same_kind_selected(hero, kind_of, bits)
                    } else {
                        Vec::new()
                    };
                    select_together(hero, bits, &joined);
                    if f.enter_with(m, bits, &joined, tools) {
                        hero.gizmo.mode.enter(bits, m);
                        let parts = f.parts(bits);
                        if f.keeps_parts_selected(m) {
                            restore_parts(hero, &before, parts.as_deref());
                        }
                        hero.gizmo.mode.publish_parts(parts);
                        // O modo que só PASSOU a outro objecto (a forma que nasceu num Edit) não
                        // é notícia: o aviso repetir-se-ia a cada forma desenhada.
                        if fell_from != Some(m) {
                            let label = m.label_key().tr();
                            toasts.push(Toast::info(tr_with(
                                "object_mode.entered",
                                &[("mode", &label)],
                            )));
                        }
                    }
                }
            }
            Step::Leave => {
                // Sair de um modo de partes devolve a selecção ao objecto inteiro (o `Tab` do
                // Blender): uma parte seleccionada em Object não teria o modo de volta.
                // Num modo que JUNTA, os objectos que entraram com ele voltam todos — os das partes
                // do MESMO tipo (as formas de dentro de um objecto vetorial também são partes).
                // Num modo do TIPO as partes são todos os objectos dele: a selecção fica como está
                // (vazia, o trancado — o `Tab` seguinte tem o activo).
                let whole = if hero.gizmo.mode.whole_kind() {
                    hero.gizmo
                        .selection
                        .is_none()
                        .then(|| hero.gizmo.mode.locked_entity())
                        .flatten()
                } else {
                    hero.gizmo.mode.parts().and(hero.gizmo.mode.locked_entity())
                };
                let joined = hero.gizmo.mode.active().and_then(|a| {
                    let f = family(families, kind_of(a.entity), a.mode)?;
                    let parts = (f.joins(a.mode) && !f.holds_the_whole_kind(a.mode))
                        .then(|| hero.gizmo.mode.parts())??;
                    let kind = kind_of(a.entity);
                    Some(
                        parts
                            .iter()
                            .copied()
                            .filter(|b| *b != a.entity && kind_of(*b) == kind)
                            .collect::<Vec<_>>(),
                    )
                });
                leave_current(families, kind_of, tools, hero);
                if let Some(e) = whole {
                    select_together(hero, e, joined.as_deref().unwrap_or_default());
                }
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
    let (part_gizmo, whole_kind) = current
        .and_then(|a| {
            let f = family(families, kind_of(a.entity), a.mode)?;
            Some((
                f.parts_take_the_object_gizmo(a.mode),
                f.holds_the_whole_kind(a.mode),
            ))
        })
        .unwrap_or_default();
    hero.gizmo.mode.publish_part_gizmo(part_gizmo);
    hero.gizmo.mode.publish_whole_kind(whole_kind);
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
    let (locked, parts) = (hero.gizmo.mode.locked_entity(), hero.gizmo.mode.parts());
    // Num modo do TIPO nada se recusa: um objecto de outro tipo escolhido sai do modo (a rede).
    let refuse = !hero.gizmo.mode.whole_kind()
        && object_mode::decide(locked, parts, target, additive) == object_mode::Decision::Refuse;
    if refuse {
        toasts.push(Toast::warning(object_mode::refusal(&hero.gizmo.mode)));
    }
    refuse
}

/// ⭐ **O gizmo de transformação É o modo Object** (D6): num modo de criação (Paint, Draw) ele
/// seria um controlo que não responde por cima do módulo. A shell pergunta aqui ao decidir se o
/// pinta; a selecção fica armada. ⭐ A excepção é o modo cuja família o declara
/// ([`ModeFamily::parts_take_the_object_gizmo`]): a selecção tem o gizmo — a parte E o objecto
/// trancado (o Edit do vetor é sobre a própria forma desde 05/10; com o contentor de 04/10 o
/// trancado não o tinha, e a booleana deixava a forma nova sem gizmo — report do dono).
#[must_use]
pub fn object_gizmo_shows(hero: &HeroScreen) -> bool {
    let mode = &hero.gizmo.mode;
    mode.active().is_none() || (mode.part_gizmo() && hero.gizmo.selection.is_some())
}

/// ⭐ **O cadeado na porta do LAÇO** — num modo de objecto inteiro a multi-selecção é recusada (com
/// o aviso; `false`); num modo de PARTES o laço fica só com as partes e o objecto trancado (o laço
/// do Edit do vetor apanha as formas dele).
pub fn lasso_admits(hero: &HeroScreen, bits: &mut Vec<u64>, toasts: &mut ToastQueue) -> bool {
    let (Some(locked), parts) = (hero.gizmo.mode.locked_entity(), hero.gizmo.mode.parts()) else {
        return true;
    };
    let Some(parts) = parts else {
        toasts.push(Toast::warning(object_mode::refusal(&hero.gizmo.mode)));
        return false;
    };
    bits.retain(|b| *b == locked || parts.contains(b));
    true
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

/// ⭐ **A tecla do modo** (spec/06 §3.2): `Tab` alterna Object ↔ o último modo do objecto;
/// `Ctrl+Tab` (`list`) abre a LISTA — o seletor do cabeçalho pelo clique DELE, logo o 2.º toque
/// fecha. Sem seletor (nenhum activo) nenhuma das duas faz nada, como no Blender.
pub fn mode_key(hero: &mut HeroScreen, list: bool) {
    use crate::action_bus::EditorAction;
    if !list {
        hero.bus.push(EditorAction::ObjectMode(ModeRequest::Toggle));
    } else if hero.store.has_mode_selector() {
        hero.apply_event(crate::interaction::WidgetEvent::Click(
            crate::ids::area_menu_button(0),
        ));
    }
}

#[cfg(test)]
#[path = "mode_drive_tests.rs"]
mod tests;
