//! ⭐ **A CARA de uma linha de menu** — o ícone, o grupo, a seta de submenu e o vermelho do destrutivo.
//!
//! Irmão do [`super::menu_row_mark`] (que responde *«esta linha é o estado corrente?»*): este responde
//! *«como ela se apresenta?»*. Nasceu com os menus da timeline (dono, 13/11: *«melhorar a aparência
//! dos menus de botão direito»*), e por isso só conhece os ids dela — um id que não está aqui pinta
//! como sempre pintou, e os outros menus do app não mudam. O pintor partilhado
//! ([`super::context_menu_overlay`]) pergunta a ESTE ficheiro, nunca a uma lista por menu.

use crate::icons::IconId;
use crate::ids as c;
use ph2d_a11y::NodeId;

/// O ícone à esquerda do rótulo. `None` = a linha não tem ícone.
#[must_use]
pub(super) fn icone(id: NodeId) -> Option<IconId> {
    const TABELA: &[(NodeId, IconId)] = &[
        // Chave: a interpolação que SAI dela.
        (c::CTX_MENU_TL_HOLD, IconId::CardInterpHold),
        (c::CTX_MENU_TL_NEAREST, IconId::Step),
        (c::CTX_MENU_TL_LINEAR, IconId::CardInterpLinear),
        (c::CTX_MENU_TL_EASE_IN, IconId::CardInterpCubic),
        (c::CTX_MENU_TL_EASE_OUT, IconId::CardInterpCubic),
        (c::CTX_MENU_TL_EASE_INOUT, IconId::CardInterpCubic),
        (c::CTX_MENU_TL_CUSTOM, IconId::VertexSmooth),
        (c::CTX_MENU_TL_ROVE, IconId::Timer),
        (c::CTX_MENU_TL_FADE_SMOOTH, IconId::CardInterpCubic),
        // Faixa.
        (c::CTX_MENU_TL_EXTRAP_PRE, IconId::SkipBack),
        (c::CTX_MENU_TL_EXTRAP_POST, IconId::SkipForward),
        (c::CTX_MENU_TL_EXTRAP_HOLD, IconId::Minus),
        (c::CTX_MENU_TL_EXTRAP_LOOP, IconId::Loop),
        (c::CTX_MENU_TL_EXTRAP_PINGPONG, IconId::PingPong),
        (c::CTX_MENU_TL_EXTRAP_CONTINUE, IconId::ArrowRight),
        (c::CTX_MENU_TL_TO_PATH, IconId::CardPath),
        (c::CTX_MENU_TL_TO_AXES, IconId::Move),
        (c::CTX_MENU_TL_AUTO_ORIENT, IconId::Rotate),
        (c::CTX_MENU_TL_DELETE_TRACK, IconId::Trash),
        // Pista, tira, marcador.
        (c::CTX_MENU_TL_LANE_OVERRIDE, IconId::Layer),
        (c::CTX_MENU_TL_LANE_ADDITIVE, IconId::Plus),
        (c::CTX_MENU_TL_LANE_RENAME, IconId::Text),
        (c::CTX_MENU_TL_LANE_DELETE, IconId::Trash),
        (c::CTX_MENU_TL_STRIP_ENTER, IconId::Open),
        (c::CTX_MENU_TL_STRIP_DUPLICATE, IconId::Duplicate),
        (c::CTX_MENU_TL_STRIP_ONCE, IconId::Play),
        (c::CTX_MENU_TL_STRIP_LOOP, IconId::Loop),
        (c::CTX_MENU_TL_STRIP_PINGPONG, IconId::PingPong),
        (c::CTX_MENU_TL_STRIP_RESET_SPEED, IconId::Reset),
        (c::CTX_MENU_TL_STRIP_DELETE, IconId::Trash),
        (c::CTX_MENU_TL_RENAME_MARKER, IconId::Text),
        (c::CTX_MENU_TL_SET_SIGNAL, IconId::Bolt),
        (c::CTX_MENU_TL_DELETE_MARKER, IconId::Trash),
    ];
    TABELA.iter().find(|(i, _)| *i == id).map(|&(_, ic)| ic)
}

/// A linha ABRE um submenu: leva a seta à direita (e o rótulo não leva «▶»). A pergunta da barra
/// ([`super::menu_cascade::opens_submenu`]: cascatas, Preferências, Theme, linhas do SettingsMenu) mais as
/// curvas e a extrapolação da timeline — UMA porta, nunca duas tabelas.
#[must_use]
pub(super) fn abre_submenu(id: NodeId) -> bool {
    super::menu_cascade::opens_submenu(id)
        || [
            c::CTX_MENU_TL_EASE_IN,
            c::CTX_MENU_TL_EASE_OUT,
            c::CTX_MENU_TL_EASE_INOUT,
            c::CTX_MENU_TL_EXTRAP_PRE,
            c::CTX_MENU_TL_EXTRAP_POST,
        ]
        .contains(&id)
}

/// A linha DESTRÓI: rótulo e ícone em vermelho discreto, e um risco antes dela.
#[must_use]
pub(super) fn destrutiva(id: NodeId) -> bool {
    [
        c::CTX_MENU_TL_DELETE_TRACK,
        c::CTX_MENU_TL_LANE_DELETE,
        c::CTX_MENU_TL_STRIP_DELETE,
        c::CTX_MENU_TL_DELETE_MARKER,
    ]
    .contains(&id)
}

/// Um RISCO antes desta linha: ela abre um grupo novo. ⚠️ Nunca na primeira linha (o pintor
/// pergunta só a partir da segunda), e por isso a pergunta não depende da posição.
#[must_use]
pub(super) fn risco_antes(id: NodeId) -> bool {
    destrutiva(id)
        || super::menu_bar::GROUP_BREAKS.contains(&id)
        || [
            // Chave: degraus · curvas com submenu · à mão.
            c::CTX_MENU_TL_EASE_IN,
            c::CTX_MENU_TL_CUSTOM,
            // Faixa: conversões · extrapolação.
            c::CTX_MENU_TL_EXTRAP_PRE,
            // Pista: modo · nome.
            c::CTX_MENU_TL_LANE_RENAME,
            // Tira: ela mesma · o que a fonte faz no fim.
            c::CTX_MENU_TL_STRIP_ONCE,
        ]
        .contains(&id)
}

#[cfg(test)]
#[path = "menu_row_look_tests.rs"]
mod tests;
