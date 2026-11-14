//! ⭐ **A marca do estado corrente nos menus do botão direito** (dono, 13/11).
//!
//! O pintor do menu (editor-core) não conhece interpolação nem extrapolação; quem conhece é este
//! painel, que tem o snapshot. A cada quadro com um menu da timeline aberto ele publica as linhas
//! que DIZEM o estado de agora (`WidgetStore::set_menu_marks`) e o pintor acende-as.

use ph2d_anim::{EasingMode, Interp};
use ph2d_editor_core::NodeId;
use ph2d_editor_core::WidgetStore;
use ph2d_editor_core::ids as c;
use ph2d_editor_core::interaction::{ContextMenuKind as K, TimelineInterpScope as S};
use ph2d_timeline::{Extrap, TimelineViewSnapshot};

use crate::presets::{Preset, preset_for};

/// Publica as linhas acesas do menu aberto, se ele for da timeline.
pub(crate) fn publica(store: &mut WidgetStore, snap: &TimelineViewSnapshot) {
    let Some(kind) = store.context_menu().map(|r| r.kind) else {
        return;
    };
    if let Some(marcas) = linhas_acesas(kind, snap) {
        store.set_menu_marks(kind, &marcas, &titulo(kind, snap));
        store.set_ease_sampler(amostra_ease);
    }
}

/// A régua que os botões do cartão da chave desenham: a forma `item` no modo `mode`, pela MESMA
/// conta do documento (`Easing::eval`). `None` para o que não é forma.
pub(crate) fn amostra_ease(item: NodeId, mode: u8, u: f32) -> Option<f32> {
    match preset_for(item, mode)? {
        Preset::Eased(e) => Some(e.eval(f64::from(u)) as f32),
        _ => None,
    }
}

/// As linhas do menu `kind` que são o estado de agora. `None` = não é menu da timeline com estado.
#[must_use]
pub(crate) fn linhas_acesas(kind: K, snap: &TimelineViewSnapshot) -> Option<Vec<NodeId>> {
    match kind {
        K::TimelineSegment { scope } => {
            if matches!(scope, S::StripFade { .. }) {
                return None;
            }
            // ⚠️ Chaves que discordam (uma coluna mista) publicam o TÍTULO sem marca nenhuma.
            let Some((interp, rove)) = interp_de(scope, snap) else {
                return Some(Vec::new());
            };
            let mut v = vec![linha_da_interp(interp)];
            if rove {
                v.push(c::CTX_MENU_TL_ROVE);
            }
            // A FORMA também, para o cartão dizer «Cubic» e o Ease in/out mantê-la.
            if let Interp::Eased(e) = interp {
                v.extend(familia(e));
            }
            Some(v)
        }
        K::TimelineSegmentEase { scope, mode } => {
            let (interp, _) = interp_de(scope, snap)?;
            let Interp::Eased(e) = interp else {
                return Some(Vec::new());
            };
            let modo = match mode {
                c::TL_EASE_MODE_IN => EasingMode::In,
                c::TL_EASE_MODE_OUT => EasingMode::Out,
                _ => EasingMode::InOut,
            };
            Some(if e.mode == modo {
                familia(e).collect()
            } else {
                Vec::new()
            })
        }
        K::TimelineExtrap { target, side, .. } => {
            let t = snap.tracks.iter().find(|t| t.target.get() == target)?;
            let modo = if side == crate::ids::TL_EXTRAP_SIDE_PRE {
                t.pre
            } else {
                t.post
            };
            Some(vec![match modo {
                Extrap::Hold => c::CTX_MENU_TL_EXTRAP_HOLD,
                Extrap::Loop => c::CTX_MENU_TL_EXTRAP_LOOP,
                Extrap::PingPong => c::CTX_MENU_TL_EXTRAP_PINGPONG,
                Extrap::Continue => c::CTX_MENU_TL_EXTRAP_CONTINUE,
            }])
        }
        _ => None,
    }
}

/// O id da forma `e.family` no submenu das formas.
fn familia(e: ph2d_anim::Easing) -> impl Iterator<Item = NodeId> {
    c::TIMELINE_EASE_MENU
        .iter()
        .map(|&(id, ..)| id)
        .filter(move |&id| {
            preset_for(id, ph2d_editor_core::ids::TL_EASE_MODE_IN)
                .is_some_and(|p| matches!(p, Preset::Eased(x) if x.family == e.family))
        })
}

/// O cabeçalho do cartão da chave: «Key at frame 12 · Position Y», ou «Keys at frame 12» numa
/// coluna do Summary. Vazio para os outros menus.
fn titulo(kind: K, snap: &TimelineViewSnapshot) -> String {
    let quadro = |t: f64| (t * snap.fps).round() as i64;
    match kind {
        K::TimelineSegment {
            scope: S::Key { target, key },
        } => snap
            .tracks
            .iter()
            .find(|t| t.target.get() == target)
            .and_then(|t| {
                let k = t.keys.iter().find(|k| k.id.get() == key)?;
                let canal = snap
                    .channel_name(t.prop)
                    .map_or_else(|| crate::nome_do_canal(t.prop), str::to_owned);
                Some(ph2d_i18n::tr_with(
                    "chrome.timeline_menu.key_at",
                    &[("n", &quadro(k.t_seconds)), ("canal", &canal)],
                ))
            })
            .unwrap_or_default(),
        K::TimelineSegment {
            scope: S::Column { t_bits },
        } => ph2d_i18n::tr_with(
            "chrome.timeline_menu.keys_at",
            &[("n", &quadro(f64::from_bits(t_bits)))],
        ),
        _ => String::new(),
    }
}

/// A interpolação que SAI das chaves do escopo, e se elas rovam. Uma coluna (Summary) só tem
/// estado quando todas as chaves dela concordam — senão nenhuma linha acende.
fn interp_de(scope: S, snap: &TimelineViewSnapshot) -> Option<(Interp, bool)> {
    let mut chaves = snap.tracks.iter().flat_map(|t| {
        t.keys.iter().filter(move |k| match scope {
            S::Key { target, key } => t.target.get() == target && k.id.get() == key,
            S::Column { t_bits } => k.t_seconds.to_bits() == t_bits,
            S::StripFade { .. } => false,
        })
    });
    let first = chaves.next()?;
    let (interp, rove) = (first.interp, first.roving);
    chaves
        .all(|k| k.interp == interp && k.roving == rove)
        .then_some((interp, rove))
}

/// A linha de topo do menu da chave que diz esta interpolação.
fn linha_da_interp(interp: Interp) -> NodeId {
    match interp {
        Interp::Hold => c::CTX_MENU_TL_HOLD,
        Interp::Nearest => c::CTX_MENU_TL_NEAREST,
        Interp::Linear => c::CTX_MENU_TL_LINEAR,
        Interp::Eased(e) => match e.mode {
            EasingMode::In => c::CTX_MENU_TL_EASE_IN,
            EasingMode::Out => c::CTX_MENU_TL_EASE_OUT,
            EasingMode::InOut => c::CTX_MENU_TL_EASE_INOUT,
        },
        Interp::Bezier { .. } | Interp::BezierW { .. } => c::CTX_MENU_TL_CUSTOM,
    }
}

#[cfg(test)]
#[path = "menu_marcas_tests.rs"]
mod tests;
