//! ⭐ **A ferramenta do arranjo do HUD do lado do painel** (W5.5, ADR-0194): os ids dos botões da ④ com uma seta
//! escolhida (como ela prende: ao lado · não invade · centrar no eixo cruzado · apagar), o que a crate do vetor
//! publica em cada quadro ([`set_hud_vista`]) e a FILA do que os botões pediram — `ph2d_app_vec::hud_tool::frame`
//! drena-a ([`drena`]). O irmão da tool Regras ([`crate::state_cad`]).

use std::cell::RefCell;

use ph2d_a11y::NodeId;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::{BarButton, BarFace, BarFamily, WidgetEvent, WidgetStore};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tool_registry::hash_node_id;

pub const VECTOR_HUD_BESIDE: NodeId = hash_node_id("vector.hud.beside");
pub const VECTOR_HUD_APART: NodeId = hash_node_id("vector.hud.apart");
pub const VECTOR_HUD_CENTRE: NodeId = hash_node_id("vector.hud.centre");
pub const VECTOR_HUD_DELETE: NodeId = hash_node_id("vector.hud.delete");

/// Como a seta prende o par no eixo dela.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TipoSeta {
    /// Encostado, com a folga de quando a seta nasceu.
    AoLado,
    /// Pode afastar-se, nunca passar por cima.
    NaoInvade,
}

/// ⭐ O que a crate do vetor publica em cada quadro com a ferramenta na mão.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HudVista {
    /// A seta escolhida: como prende, e se o par também está centrado no eixo cruzado. `None` = nada escolhido.
    pub escolhida: Option<(TipoSeta, bool)>,
    /// A última seta foi recusada (fecharia um círculo de lados): a dica di-lo.
    pub recusada: bool,
    /// A caixa da seta escolhida no ECRÃ — onde a ④ pousa.
    pub ancora: Option<Rect>,
}

/// O que um botão pediu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PedidoHud {
    Tipo(TipoSeta),
    /// Liga/desliga o centrado no eixo cruzado.
    Centro,
    Apaga,
}

thread_local! {
    static VISTA: RefCell<HudVista> = RefCell::new(HudVista::default());
    static PEDIDOS: RefCell<Vec<PedidoHud>> = const { RefCell::new(Vec::new()) };
}

pub fn set_hud_vista(v: HudVista) {
    VISTA.with(|s| *s.borrow_mut() = v);
}

#[must_use]
pub fn hud_vista() -> HudVista {
    VISTA.with(|s| s.borrow().clone())
}

pub fn pede(p: PedidoHud) {
    PEDIDOS.with(|q| q.borrow_mut().push(p));
}

/// Os pedidos desde o último dreno, pela ordem do clique.
#[must_use]
pub fn drena() -> Vec<PedidoHud> {
    PEDIDOS.with(|q| std::mem::take(&mut *q.borrow_mut()))
}

const BOTOES: [(NodeId, IconId, &str); 4] = [
    (VECTOR_HUD_BESIDE, IconId::Link, "panel.vector.hud.beside"),
    (
        VECTOR_HUD_APART,
        IconId::DistributeH,
        "panel.vector.hud.apart",
    ),
    (
        VECTOR_HUD_CENTRE,
        IconId::AlignMiddle,
        "panel.vector.hud.centre",
    ),
    (VECTOR_HUD_DELETE, IconId::Trash, "panel.vector.hud.delete"),
];

/// Regista os botões (sem isto o clique morre sob o rato: o `wiring_parity`).
pub(crate) fn populate(store: &mut WidgetStore) {
    for (id, _, _) in BOTOES {
        crate::populate::button(store, id);
    }
}

/// ④ com uma seta escolhida: os quatro botões, aceso o que ela é.
#[must_use]
pub(crate) fn acoes() -> Vec<Vec<BarButton>> {
    let Some((tipo, centrado)) = hud_vista().escolhida else {
        return Vec::new();
    };
    let aceso = |id| match id {
        x if x == VECTOR_HUD_BESIDE => tipo == TipoSeta::AoLado,
        x if x == VECTOR_HUD_APART => tipo == TipoSeta::NaoInvade,
        x if x == VECTOR_HUD_CENTRE => centrado,
        _ => false,
    };
    let botao = |(id, icon, key): (NodeId, IconId, &str)| BarButton {
        id,
        face: BarFace::Icon(icon),
        label: tr(key).to_owned(),
        on: aceso(id),
        family: BarFamily::Build,
    };
    vec![
        BOTOES[..3].iter().copied().map(botao).collect(),
        vec![botao(BOTOES[3])],
    ]
}

/// A dica: a recusa tem voz própria até ao próximo gesto.
#[must_use]
pub(crate) fn dica() -> &'static str {
    if hud_vista().recusada {
        "panel.vector.hint.hud_refused"
    } else {
        "panel.vector.hint.hud"
    }
}

/// O clique num botão da seta: vira um pedido. `true` = consumido.
pub(crate) fn evento(_store: &WidgetStore, ev: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = ev else {
        return false;
    };
    let p = match id {
        x if x == VECTOR_HUD_BESIDE => PedidoHud::Tipo(TipoSeta::AoLado),
        x if x == VECTOR_HUD_APART => PedidoHud::Tipo(TipoSeta::NaoInvade),
        x if x == VECTOR_HUD_CENTRE => PedidoHud::Centro,
        x if x == VECTOR_HUD_DELETE => PedidoHud::Apaga,
        _ => return false,
    };
    pede(p);
    true
}
