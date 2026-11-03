//! ⭐ **O cartão PRESO a um osso** (T7, `docs/CartoesUI/02_especificacao.md` §8): a shell publica o
//! osso em foco nos statics de [`super`]; para cada osso preso por um cartão solto ela publica-o
//! outra vez e o painel guarda o RETRATO (todos os statics do osso). Pintar o cartão preso = repor
//! o retrato dele à volta da pintura e devolver o do foco a seguir.

use super::*;
use std::collections::BTreeMap;

thread_local! {
    static CURRENT_NAME: RefCell<String> = const { RefCell::new(String::new()) };
    static PRESOS: RefCell<BTreeMap<u64, Retrato>> = const { RefCell::new(BTreeMap::new()) };
}

/// O nome do osso em foco (o subtítulo do cartão solto, «Bone 3»). A identidade vem à parte, pelo
/// [`set_current_bone_subject`].
pub fn set_current_bone_name(nome: String) {
    CURRENT_NAME.with(|c| *c.borrow_mut() = nome);
}

/// O osso em foco: os bits e o nome.
pub(crate) fn current_bone_subject() -> Option<(u64, String)> {
    let bits = CURRENT_BONE_SUBJECT.with(Cell::get)?;
    Some((bits, CURRENT_NAME.with(|c| c.borrow().clone())))
}

/// Tudo o que a shell publica de UM osso.
#[derive(Clone)]
struct Retrato {
    sujeito: Option<(u64, String)>,
    osso: Option<ph2d_skeleton::bend::BoneSpec>,
    px: f64,
    handles: Option<usize>,
    ik: (bool, f64, f64, f64, usize),
    aim: Option<f64>,
    auto_side: Option<usize>,
    limit: Option<(f64, f64)>,
    smart: Option<SmartBoneView>,
    actions: Vec<String>,
    tip: Option<TipView>,
}

fn retrato() -> Retrato {
    Retrato {
        sujeito: current_bone_subject(),
        osso: current_bone(),
        px: bone_px_per_unit(),
        handles: current_bone_handles(),
        ik: (
            CURRENT_HAS_IK.with(Cell::get),
            CURRENT_IK_MIX.with(Cell::get),
            CURRENT_IK_SOFTNESS.with(Cell::get),
            CURRENT_IK_CHAIN.with(Cell::get),
            CURRENT_IK_BEND.with(Cell::get),
        ),
        aim: current_bone_aim(),
        auto_side: current_bone_ik_auto_side(),
        limit: current_bone_limit(),
        smart: current_bone_smart(),
        actions: bone_actions(),
        tip: bone_tip(),
    }
}

fn repoe(r: &Retrato) {
    set_current_bone_subject(r.sujeito.as_ref().map(|s| s.0));
    set_current_bone_name(r.sujeito.as_ref().map_or_else(String::new, |s| s.1.clone()));
    set_current_bone(r.osso);
    set_current_bone_px_per_unit(r.px);
    set_current_bone_handles(r.handles);
    let (tem, mix, soft, chain, bend) = r.ik;
    CURRENT_HAS_IK.with(|c| c.set(tem));
    CURRENT_IK_MIX.with(|c| c.set(mix));
    CURRENT_IK_SOFTNESS.with(|c| c.set(soft));
    CURRENT_IK_CHAIN.with(|c| c.set(chain));
    CURRENT_IK_BEND.with(|c| c.set(bend));
    set_current_bone_aim(r.aim);
    set_current_bone_ik_auto_side(r.auto_side);
    set_current_bone_limit(r.limit);
    set_current_bone_smart(r.smart.clone());
    set_current_bone_actions(r.actions.clone());
    set_current_bone_tip(r.tip.clone());
}

/// ⭐ A shell, depois de publicar o osso em foco: `publica(b)` publica o osso preso `b` pelas MESMAS
/// portas; o retrato de cada um fica guardado e o do foco volta.
pub fn publica_presos(presos: &[u64], mut publica: impl FnMut(u64)) {
    let foco = retrato();
    let mut m = BTreeMap::new();
    for &b in presos {
        publica(b);
        m.insert(b, retrato());
    }
    repoe(&foco);
    PRESOS.with(|p| *p.borrow_mut() = m);
}

/// Corre `f` com o retrato do osso preso `b` no lugar do foco (`None` = a shell ainda não o
/// publicou).
pub(crate) fn com_preso<R>(b: u64, f: impl FnOnce() -> R) -> Option<R> {
    let r = PRESOS.with(|p| p.borrow().get(&b).cloned())?;
    let foco = retrato();
    repoe(&r);
    let out = f();
    repoe(&foco);
    Some(out)
}
