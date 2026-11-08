//! ⭐ **O cartão User Data** (Pixel P12 U, `docs/PixelArt/P12_plano.md` §9.5): cor, texto e a tabela
//! `name · value · tipo` de um dono (sprite, camada, cel, tag, tileset, slice). Mais que o Aseprite,
//! que na tela só tem cor e texto (O17): aqui a tabela também se edita.
//!
//! O cartão não conhece o documento: recebe uma [`UserDataView`] e devolve, por [`hit`], o gesto em
//! índices de linha. Os ids nascem de um prefixo por dono, para dois cartões não colidirem.

use super::{CardCtx, Cell, Widget};
use crate::interaction::{InteractiveState, WidgetStore};
use crate::widget::{ButtonState, TextInputState};
use ph2d_a11y::NodeId;
use ph2d_i18n::tr;
use ph2d_tool_registry::hash_node_id_runtime;

/// Uma linha da tabela, já escrita.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropRow {
    pub name: String,
    pub value: String,
    /// O nome do tipo no chip (`Bool`, `Int`, `Float`, `Text`, ou o do `.ase`).
    pub kind: &'static str,
    /// `false`: um tipo do `.ase` que a tela não edita; mostra-se, não se toca.
    pub editable: bool,
}

/// O que o cartão mostra.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UserDataView {
    pub color: [u8; 4],
    pub text: String,
    /// O mapa do artista (`""`).
    pub rows: Vec<PropRow>,
    /// Os mapas de extensão, só de leitura, sob o nome da extensão.
    pub extensions: Vec<(String, Vec<PropRow>)>,
}

/// Os ids de um cartão, a partir do prefixo do dono.
#[derive(Debug, Clone)]
pub struct UserDataIds {
    prefix: String,
    pub toggle: NodeId,
    pub color: NodeId,
    pub text: NodeId,
    pub add: NodeId,
}

impl UserDataIds {
    #[must_use]
    pub fn new(prefix: &str) -> Self {
        let id = |s: &str| hash_node_id_runtime(&format!("{prefix}.user_data.{s}"));
        Self {
            prefix: prefix.to_owned(),
            toggle: id("toggle"),
            color: id("color"),
            text: id("text"),
            add: id("add"),
        }
    }

    fn row(&self, part: &str, i: usize) -> NodeId {
        hash_node_id_runtime(&format!("{}.user_data.{part}.{i}", self.prefix))
    }

    #[must_use]
    pub fn name(&self, i: usize) -> NodeId {
        self.row("name", i)
    }

    #[must_use]
    pub fn value(&self, i: usize) -> NodeId {
        self.row("value", i)
    }

    #[must_use]
    pub fn kind(&self, i: usize) -> NodeId {
        self.row("kind", i)
    }

    #[must_use]
    pub fn remove(&self, i: usize) -> NodeId {
        self.row("remove", i)
    }
}

/// O gesto que um id do cartão quer dizer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hit {
    Toggle,
    Color,
    Text,
    Add,
    Name(usize),
    Value(usize),
    Kind(usize),
    Remove(usize),
}

/// O gesto de `id`, se é deste cartão (`n` = linhas do mapa do artista).
#[must_use]
pub fn hit(ids: &UserDataIds, n: usize, id: NodeId) -> Option<Hit> {
    let fixos = [
        (ids.toggle, Hit::Toggle),
        (ids.color, Hit::Color),
        (ids.text, Hit::Text),
        (ids.add, Hit::Add),
    ];
    if let Some((_, h)) = fixos.iter().find(|(i, _)| *i == id) {
        return Some(*h);
    }
    (0..n).find_map(|i| {
        [
            (ids.name(i), Hit::Name(i)),
            (ids.value(i), Hit::Value(i)),
            (ids.kind(i), Hit::Kind(i)),
            (ids.remove(i), Hit::Remove(i)),
        ]
        .into_iter()
        .find(|(x, _)| *x == id)
        .map(|(_, h)| h)
    })
}

fn texto(store: &mut WidgetStore, id: NodeId) {
    if store.get(id).is_none() {
        store.register(
            id,
            InteractiveState::TextInput {
                state: TextInputState::Normal,
                text: String::new(),
                caret: 0,
                selection_anchor: None,
            },
        );
        store.mark_cancel_on_escape(id);
    }
}

fn botao(store: &mut WidgetStore, id: NodeId) {
    if store.get(id).is_none() {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
}

fn semeia(store: &mut WidgetStore, id: NodeId, novo: &str) {
    if store.focus_id() == Some(id) {
        return;
    }
    if let Some(InteractiveState::TextInput { text, caret, .. }) = store.get_mut(id)
        && text != novo
    {
        *caret = novo.len();
        novo.clone_into(text);
    }
}

/// Regista os controlos de `v` e escreve neles o documento (salvo o campo com o cursor). Cada quadro,
/// antes de pintar: as linhas da tabela vêm e vão com o documento.
pub fn prepare(store: &mut WidgetStore, ids: &UserDataIds, v: &UserDataView, full: bool) {
    botao(store, ids.toggle);
    botao(store, ids.add);
    if full {
        texto(store, ids.text);
        semeia(store, ids.text, &v.text);
    }
    if full && store.get(ids.color).is_none() {
        store.register_picker_swatch(ids.color);
        store.register(ids.color, InteractiveState::Plain);
    }
    if full && store.picker_target() != Some(ids.color) {
        store.set_widget_color(ids.color, v.color);
    }
    for (i, r) in v.rows.iter().enumerate() {
        if r.editable {
            texto(store, ids.name(i));
            texto(store, ids.value(i));
            botao(store, ids.kind(i));
            semeia(store, ids.name(i), &r.name);
            semeia(store, ids.value(i), &r.value);
        }
        botao(store, ids.remove(i));
    }
}

/// O botão `User Data` que abre e fecha o cartão, para morar ao lado do nome do dono (O17).
#[must_use]
pub fn toggle_cell(ids: &UserDataIds) -> Cell<'static> {
    Cell::new(Widget::Button {
        id: ids.toggle,
        text: tr("card.user_data.toggle"),
        primary: false,
    })
}

fn so_leitura<'a>(c: &mut CardCtx<'_>, r: &'a PropRow, y: f32) -> f32 {
    c.row(
        &[
            Cell::new(Widget::Readout { text: &r.name }),
            Cell::new(Widget::Readout { text: &r.value }),
            Cell::new(Widget::Readout { text: r.kind }),
        ],
        y,
    )
}

/// O corpo aberto: cor e texto (se `full`; a slice já os tem no próprio cartão), a tabela, `+`, e os
/// mapas de extensão por baixo.
pub fn paint_body(
    c: &mut CardCtx<'_>,
    ids: &UserDataIds,
    v: &UserDataView,
    full: bool,
    mut y: f32,
) -> f32 {
    let codigo = format!(
        "#{:02X}{:02X}{:02X}{:02X}",
        v.color[0], v.color[1], v.color[2], v.color[3]
    );
    let texto = Cell::new(Widget::Text {
        id: ids.text,
        placeholder: "",
    })
    .label(tr("card.user_data.text"));
    if full {
        let cor = Cell::new(Widget::Swatch {
            id: ids.color,
            rgba: Some(v.color),
            text: &codigo,
            edit: None,
            detach: None,
        })
        .label(tr("card.user_data.color"));
        y = c.row(&[cor, texto], y);
    }
    for (i, r) in v.rows.iter().enumerate() {
        let primeira = i == 0;
        let remove = Cell::new(Widget::Button {
            id: ids.remove(i),
            text: tr("card.user_data.remove"),
            primary: false,
        });
        if !r.editable {
            y = so_leitura(c, r, y);
            continue;
        }
        let nome = Cell::new(Widget::Text {
            id: ids.name(i),
            placeholder: "",
        });
        let valor = Cell::new(Widget::Text {
            id: ids.value(i),
            placeholder: "",
        });
        let (nome, valor) = if primeira {
            (
                nome.label(tr("card.user_data.name")),
                valor.label(tr("card.user_data.value")),
            )
        } else {
            (nome, valor)
        };
        let tipo = Cell::new(Widget::Button {
            id: ids.kind(i),
            text: r.kind,
            primary: false,
        });
        y = c.row(&[nome, valor, tipo, remove], y);
    }
    y = c.row(
        &[Cell::new(Widget::Button {
            id: ids.add,
            text: tr("card.user_data.add"),
            primary: false,
        })],
        y,
    );
    for (ext, rows) in &v.extensions {
        y = c.hint(ext, y);
        for r in rows {
            y = so_leitura(c, r, y);
        }
    }
    y
}
