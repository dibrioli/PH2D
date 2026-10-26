//! ⭐ **A tool Regras do lado do painel** (W4.6, `docs/BlenderConstraints/04_W4_cad.md` §3.7): os ids dos botões das
//! regras possíveis (a ④, junto da escolha) e do número da cota escolhida (a ②), o que a crate do vetor publica em
//! cada quadro ([`set_cad_vista`]) e a FILA do que os botões pediram — `ph2d_app_vec::cad_tool::frame` drena-a
//! ([`drena`]); a shell não a vê.

use std::cell::RefCell;

use ph2d_a11y::NodeId;
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::{BarButton, BarFace, BarFamily, WidgetEvent, WidgetStore};
use ph2d_editor_core::widget::ToolRailEntry;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tool_registry::hash_node_id;

/// Uma regra que a escolha permite — o botão da ④ e o selo no canvas (o mesmo glifo, [`Possivel::icon`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Possivel {
    Vertical,
    Horizontal,
    Coincidente,
    Igual,
    Fixo,
    Distancia,
    Angulo,
    Raio,
}

/// Os botões da ④, um por [`Possivel`] (pela ordem de [`Possivel::ALL`]).
pub const VECTOR_CAD_RULE: [NodeId; 8] = [
    hash_node_id("vector.cad.rule.vertical"),
    hash_node_id("vector.cad.rule.horizontal"),
    hash_node_id("vector.cad.rule.coincident"),
    hash_node_id("vector.cad.rule.equal"),
    hash_node_id("vector.cad.rule.fixed"),
    hash_node_id("vector.cad.rule.distance"),
    hash_node_id("vector.cad.rule.angle"),
    hash_node_id("vector.cad.rule.radius"),
];
/// O número da cota escolhida, na ②.
pub const VECTOR_CAD_VALUE: NodeId = hash_node_id("vector.cad.value");
/// Apagar a regra escolhida, na ④ (o mesmo que o Delete).
pub const VECTOR_CAD_DELETE: NodeId = hash_node_id("vector.cad.delete");

impl Possivel {
    pub const ALL: [Self; 8] = [
        Self::Vertical,
        Self::Horizontal,
        Self::Coincidente,
        Self::Igual,
        Self::Fixo,
        Self::Distancia,
        Self::Angulo,
        Self::Raio,
    ];

    #[must_use]
    pub fn id(self) -> NodeId {
        VECTOR_CAD_RULE[self as usize]
    }

    #[must_use]
    pub fn de_id(id: NodeId) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.id() == id)
    }

    #[must_use]
    pub fn icon(self) -> IconId {
        match self {
            Self::Vertical => IconId::CadVertical,
            Self::Horizontal => IconId::CadHorizontal,
            Self::Coincidente => IconId::CadCoincident,
            Self::Igual => IconId::CadEqual,
            Self::Fixo => IconId::Lock,
            Self::Distancia => IconId::CadDistance,
            Self::Angulo => IconId::CadAngle,
            Self::Raio => IconId::CadRadius,
        }
    }

    /// O nome (as chaves que o vocabulário do CAD já tem).
    #[must_use]
    pub fn chave(self) -> &'static str {
        match self {
            Self::Vertical => "app.vec.cad.lei.vertical",
            Self::Horizontal => "app.vec.cad.lei.horizontal",
            Self::Coincidente => "app.vec.cad.lei.coincidente",
            Self::Igual => "app.vec.cad.lei.igual",
            Self::Fixo => "app.vec.cad.lei.fixo",
            Self::Distancia => "app.vec.cad.lei.distancia",
            Self::Angulo => "app.vec.cad.lei.angulo",
            Self::Raio => "app.vec.cad.lei.raio",
        }
    }

    /// Uma medida (a cota tem número).
    #[must_use]
    pub fn mede(self) -> bool {
        matches!(self, Self::Distancia | Self::Angulo | Self::Raio)
    }
}

/// A regra escolhida no canvas.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Escolhida {
    /// `None` = uma lei fora da lista da tool (a caneta cria Paralelo…): escolhe-se e apaga-se, sem número.
    pub tipo: Option<Possivel>,
    /// REF: a cota mostra o que o desenho mede e não se escreve.
    pub referencia: bool,
}

/// ⭐ O que a crate do vetor publica em cada quadro com a tool na mão.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CadVista {
    /// As regras que a escolha de agora permite (vazio = nada escolhido, ou nada a oferecer).
    pub possiveis: Vec<Possivel>,
    pub escolhida: Option<Escolhida>,
    /// A caixa da escolha (ou da regra escolhida) no ECRÃ — onde a ④ pousa.
    pub ancora: Option<Rect>,
}

/// O que um botão pediu.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PedidoCad {
    Junta(Possivel),
    /// O valor escrito, na unidade que o chip mostra (px ou graus do projecto).
    Medida(f64),
    Apaga,
}

thread_local! {
    static VISTA: RefCell<CadVista> = RefCell::new(CadVista::default());
    static PEDIDOS: RefCell<Vec<PedidoCad>> = const { RefCell::new(Vec::new()) };
}

pub fn set_cad_vista(v: CadVista) {
    VISTA.with(|s| *s.borrow_mut() = v);
}

#[must_use]
pub fn cad_vista() -> CadVista {
    VISTA.with(|s| s.borrow().clone())
}

pub fn pede(p: PedidoCad) {
    PEDIDOS.with(|q| q.borrow_mut().push(p));
}

/// Os pedidos desde o último dreno, pela ordem do clique.
#[must_use]
pub fn drena() -> Vec<PedidoCad> {
    PEDIDOS.with(|q| std::mem::take(&mut *q.borrow_mut()))
}

/// Regista os botões e a caixa do número (sem isto o clique morre sob o rato: o `wiring_parity`).
pub(crate) fn populate(store: &mut WidgetStore) {
    for id in VECTOR_CAD_RULE {
        crate::populate::button(store, id);
    }
    crate::populate::button(store, VECTOR_CAD_DELETE);
    crate::populate::number_field(store, VECTOR_CAD_VALUE, 0.0, f64::MAX, 1.0, 0.0);
}

/// ④ com a tool Regras: as regras possíveis; com uma regra escolhida, o apagar dela.
#[must_use]
pub(crate) fn acoes() -> Vec<Vec<BarButton>> {
    let v = cad_vista();
    let botao = |id, icon, key: &str| BarButton {
        id,
        face: BarFace::Icon(icon),
        label: tr(key).to_owned(),
        on: false,
        family: BarFamily::Build,
    };
    let mut out = Vec::new();
    if !v.possiveis.is_empty() {
        out.push(
            v.possiveis
                .iter()
                .map(|p| botao(p.id(), p.icon(), p.chave()))
                .collect(),
        );
    }
    if v.escolhida.is_some() {
        out.push(vec![botao(
            VECTOR_CAD_DELETE,
            IconId::Trash,
            "panel.vector.cad.delete",
        )]);
    }
    out
}

/// ② com uma MEDIDA escolhida (e não REF): o número dela.
#[must_use]
pub(crate) fn chips() -> Vec<ToolRailEntry> {
    match cad_vista().escolhida {
        Some(Escolhida {
            tipo: Some(t),
            referencia: false,
        }) if t.mede() => {
            let nome = tr(t.chave());
            vec![ToolRailEntry::number(VECTOR_CAD_VALUE, nome, nome, None)]
        }
        _ => Vec::new(),
    }
}

/// O clique num botão das regras, ou o número escrito: vira um pedido. `true` = consumido.
pub(crate) fn evento(store: &WidgetStore, ev: WidgetEvent) -> bool {
    match ev {
        WidgetEvent::Click(id) if id == VECTOR_CAD_DELETE => pede(PedidoCad::Apaga),
        WidgetEvent::Click(id) => match Possivel::de_id(id) {
            Some(p) => pede(PedidoCad::Junta(p)),
            None => return false,
        },
        WidgetEvent::ValueChanged(id) if id == VECTOR_CAD_VALUE => {
            match store.number_value(id) {
                Some(v) => pede(PedidoCad::Medida(v)),
                None => return false,
            }
        }
        _ => return false,
    }
    true
}

#[cfg(test)]
#[path = "state_cad_tests.rs"]
mod tests;
