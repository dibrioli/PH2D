//! [`KeyValueList`] — an editable list of `key → value` rows with a
//! per-row remove button and a trailing "add" button.
//!
//! Sprite Inspector v2 W6 (spec §15.7, T6.8). Backs the
//! `InstanceShaderParams` editor (W4): each row maps a string key to a
//! value preview (the value control itself — a [`VariantEditor`] or a
//! scalar field — is wired by the consuming section against the ids
//! this widget exposes). The widget owns layout + paint + a11y.

use super::button::{Button, ButtonKind, paint_button};
use crate::icons::IconId;
use crate::paint::{fill_rounded_rect, paint_text, resolve};
use crate::zones::Rect;
use ph2d_a11y::{Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Fraction of the row (minus the remove button) given to the key
/// column; the remainder is the value column.
const KEY_FRACTION: f32 = 0.4; // LITERAL-PX-OK: layout proportion (key column share of the row)

#[derive(Clone, Debug)]
pub struct KeyValueEntry {
    pub key_id: NodeId,
    pub value_id: NodeId,
    pub remove_id: NodeId,
    pub key: String,
    /// One-line preview of the value (the live value control is the
    /// consumer's responsibility, registered against `value_id`).
    pub value: String,
}

impl KeyValueEntry {
    pub fn new(
        key_id: NodeId,
        value_id: NodeId,
        remove_id: NodeId,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        Self {
            key_id,
            value_id,
            remove_id,
            key: key.into(),
            value: value.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct KeyValueList {
    pub id: NodeId,
    pub label: String,
    pub entries: Vec<KeyValueEntry>,
    /// Id of the trailing "add row" button.
    pub add_id: NodeId,
}

impl KeyValueList {
    pub fn new(
        id: NodeId,
        label: impl Into<String>,
        entries: Vec<KeyValueEntry>,
        add_id: NodeId,
    ) -> Self {
        Self {
            id,
            label: label.into(),
            entries,
            add_id,
        }
    }

    /// Square side of the per-row remove button.
    fn remove_w(row_h: f32) -> f32 {
        row_h
    }

    pub fn row_rect(host: Rect, row_h: f32, i: usize) -> Rect {
        Rect::new(
            host.x,
            host.y + i as f32 * (row_h + Spacing::Xs.px()),
            host.w,
            row_h,
        )
    }

    pub fn key_rect(row: Rect, row_h: f32) -> Rect {
        let avail = (row.w - Self::remove_w(row_h) - Spacing::Xs.px() * 2.0).max(0.0);
        Rect::new(row.x, row.y, avail * KEY_FRACTION, row.h)
    }

    pub fn value_rect(row: Rect, row_h: f32) -> Rect {
        let avail = (row.w - Self::remove_w(row_h) - Spacing::Xs.px() * 2.0).max(0.0);
        let key_w = avail * KEY_FRACTION;
        Rect::new(
            row.x + key_w + Spacing::Xs.px(),
            row.y,
            avail - key_w,
            row.h,
        )
    }

    pub fn remove_rect(row: Rect, row_h: f32) -> Rect {
        let w = Self::remove_w(row_h);
        Rect::new(row.x + row.w - w, row.y, w, row.h)
    }

    /// Rect of the trailing "add" button (a row-height square at the
    /// left of the row below the last entry).
    pub fn add_button_rect(&self, host: Rect, row_h: f32) -> Rect {
        let y = host.y + self.entries.len() as f32 * (row_h + Spacing::Xs.px());
        Rect::new(host.x, y, row_h, row_h)
    }

    /// Total height occupied (entries + the add button row).
    pub fn total_height(&self, row_h: f32) -> f32 {
        (self.entries.len() + 1) as f32 * (row_h + Spacing::Xs.px())
    }

    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        let mut b = NodeBuilder::new(Role::Group)
            .label(&self.label)
            .bounds(x, y, w, h);
        for e in &self.entries {
            b = b.child(e.key_id).child(e.value_id).child(e.remove_id);
        }
        b.child(self.add_id).build()
    }
}

fn paint_field(
    text: &str,
    rect: Rect,
    placeholder: bool,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    // ⭐ Raio e moldura pela porta do TEMA: o campo é plano num tema moderno.
    let radius = crate::paint::frame_radius(theme, Radius::Sm.px());
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::Bg3, theme));
    crate::paint::stroke_frame(
        scene,
        rect,
        radius,
        theme,
        ph2d_tokens::visuals::Feel::Rest,
        StrokeToken::Default.px(),
        resolve(ColorToken::Border, theme),
    );
    let font = TypeToken::Sm.px();
    let color = if placeholder {
        ColorToken::Text3
    } else {
        ColorToken::Text1
    };
    paint_text(
        text_system,
        scene,
        text,
        rect.x + Spacing::Sm.px(),
        rect.y + (rect.h - font) * 0.5,
        font,
        // ⚠️ **`INFINITY`, e a lição é literalmente a do `TextInput`:** *"um campo de texto é uma
        // LINHA, então ele RECORTA — ele não quebra"*. Um `max_width` finito faz o `paint_text`
        // QUEBRAR o texto, e o transbordo deixa de ser horizontal (aparado pelo recorte da
        // célula, no laço) para ser VERTICAL — medido, uma chave de 53 caracteres numa row de 24
        // px punha glifos de `y = 11` a `y = 55`, três linhas, **sobre as rows seguintes**.
        //
        // ⚠️ E a diferença não é só de gosto: o recorte da célula apara o excesso horizontal, mas
        // **não apara o vertical de forma observável** — a cena CODIFICA os glifos de qualquer
        // maneira e o recorte só age na rasterização, então uma segunda linha é tinta que nenhum
        // gate desta camada consegue ver. Com uma linha só, a propriedade *"a chave fica na banda
        // da row"* passa a ser mensurável.
        f32::INFINITY,
        resolve(color, theme),
    );
}

/// Paint the list. The caller registers each entry's `key_id`,
/// `value_id`, `remove_id` (via the `*_rect` helpers) plus `add_id`
/// (via [`KeyValueList::add_button_rect`]) for hit testing.
pub fn paint_key_value_list(
    list: &KeyValueList,
    host: Rect,
    row_h: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    for (i, entry) in list.entries.iter().enumerate() {
        let row = KeyValueList::row_rect(host, row_h, i);
        let key_rect = KeyValueList::key_rect(row, row_h);
        let value_rect = KeyValueList::value_rect(row, row_h);
        let remove_rect = KeyValueList::remove_rect(row, row_h);
        // ⚠️ **O RECORTE, e a lição é a mesma do `TextInput`:** o `paint_field` passa um
        // `max_width` ao `paint_text`, e o `paint_text` **QUEBRA** o texto nesse limite em vez de
        // o cortar. O transbordo aqui não é horizontal — é VERTICAL: medido, uma chave de 53
        // caracteres numa row de 24 px pinta glifos de `y = 11` a `y = 55`, ou seja três linhas,
        // **por cima das rows seguintes**. E uma lista de pares chave/valor é precisamente onde o
        // artista digita nomes que ele inventou.
        //
        // O recorte é da CÉLULA e não da row: assim a chave também não invade a coluna do valor
        // se a régua das colunas mudar. Numa row cujo texto cabe isto é no-op.
        //
        // ⚠️ **MEDIDO: removê-lo NÃO derruba gate nenhum, e isso é sobre o ORÁCULO, não sobre o
        // valor dele.** A cena codifica os glifos independentemente de qualquer recorte — a camada
        // só age na rasterização —, então nenhum gate desta altura consegue distinguir *aparado*
        // de *não aparado*. Ele é load-bearing no PRODUTO (uma chave de uma linha mais larga que a
        // célula pinta por cima da coluna do valor sem ele) e só seria gateável com rasterização,
        // que esta camada não tem. Quem tem gate aqui é a metade que se vê no encoding: o texto
        // ser UMA linha.
        scene.push_clip(&crate::paint::rect_to_vello(key_rect));
        paint_field(
            &entry.key,
            key_rect,
            entry.key.is_empty(),
            scene,
            text_system,
            theme,
        );
        scene.pop_layer();
        scene.push_clip(&crate::paint::rect_to_vello(value_rect));
        paint_field(
            &entry.value,
            value_rect,
            entry.value.is_empty(),
            scene,
            text_system,
            theme,
        );
        scene.pop_layer();
        let remove = Button::new(entry.remove_id, "Remove").kind(ButtonKind::IconOnly {
            icon: IconId::Trash,
        });
        paint_button(&remove, remove_rect, scene, text_system, theme);
    }
    let add =
        Button::new(list.add_id, "Add parameter").kind(ButtonKind::IconOnly { icon: IconId::Add });
    paint_button(
        &add,
        list.add_button_rect(host, row_h),
        scene,
        text_system,
        theme,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> KeyValueList {
        KeyValueList::new(
            NodeId(1),
            "Shader Params",
            vec![
                KeyValueEntry::new(NodeId(10), NodeId(11), NodeId(12), "hue_shift", "0.25"),
                KeyValueEntry::new(NodeId(20), NodeId(21), NodeId(22), "outline", "#FF0000FF"),
            ],
            NodeId(99),
        )
    }

    #[test]
    fn columns_partition_row_without_overlap() {
        let row = KeyValueList::row_rect(Rect::new(0.0, 0.0, 300.0, 24.0), 24.0, 0);
        let k = KeyValueList::key_rect(row, 24.0);
        let v = KeyValueList::value_rect(row, 24.0);
        let r = KeyValueList::remove_rect(row, 24.0);
        assert!(k.x + k.w <= v.x + 0.01);
        assert!(v.x + v.w <= r.x + 0.01);
        assert!(r.x + r.w <= row.x + row.w + 0.01);
    }

    #[test]
    fn add_button_sits_below_last_entry() {
        let list = fixture();
        let host = Rect::new(0.0, 0.0, 300.0, 200.0);
        let add = list.add_button_rect(host, 24.0);
        let last = KeyValueList::row_rect(host, 24.0, list.entries.len() - 1);
        assert!(add.y > last.y);
    }

    #[test]
    fn total_height_counts_add_row() {
        let list = fixture();
        let rows = (list.entries.len() + 1) as f32;
        assert_eq!(list.total_height(24.0), rows * (24.0 + Spacing::Xs.px()));
    }

    #[test]
    fn a11y_role_is_group() {
        assert_eq!(
            fixture().build_a11y(0.0, 0.0, 300.0, 200.0).role(),
            Role::Group
        );
    }

    #[test]
    fn paint_smoke() {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        paint_key_value_list(
            &fixture(),
            Rect::new(0.0, 0.0, 300.0, 200.0),
            24.0,
            &mut scene,
            &mut text,
            Theme::Forge,
        );
    }

    #[test]
    fn paint_smoke_empty() {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        let empty = KeyValueList::new(NodeId(1), "Empty", vec![], NodeId(99));
        paint_key_value_list(
            &empty,
            Rect::new(0.0, 0.0, 300.0, 60.0),
            24.0,
            &mut scene,
            &mut text,
            Theme::Blueprint,
        );
    }

    /// **Uma chave comprida fica DENTRO da célula dela** — não desce sobre as rows seguintes.
    ///
    /// ⚠️ O `paint_field` passa um `max_width`, e o `paint_text` **quebra** o texto nele em vez de
    /// o cortar: medido antes do recorte, uma chave de 53 caracteres numa row de 24 px pintava
    /// glifos de `y = 11` a `y = 55` — três linhas, sobre as duas rows seguintes.
    ///
    /// ⚠️ **O oráculo são os GLIFOS que a cena emite**, não o retângulo que o layout devolve: o
    /// layout sempre esteve certo (a célula tem a altura da row); quem saía era a tinta. E a
    /// primeira metade é o CONTROLE — sem uma chave que de facto transborde, o gate mede uma
    /// lista que caberia de qualquer maneira.
    #[test]
    fn a_long_key_stays_inside_its_cell() {
        let mut ts = TextSystem::without_system_fonts();
        let row_h = 24.0;
        let host = Rect::new(0.0, 0.0, 300.0, 60.0);
        let long = "um nome de chave bastante comprido para a coluna dela";

        let glyph_span = |key: &str, ts: &mut TextSystem| {
            let mut scene = VectorScene::new();
            let list = KeyValueList::new(
                NodeId(1),
                "Custom",
                vec![KeyValueEntry::new(
                    NodeId(2),
                    NodeId(3),
                    NodeId(4),
                    key,
                    "v",
                )],
                NodeId(5),
            );
            paint_key_value_list(&list, host, row_h, &mut scene, ts, Theme::Forge);
            let ys: Vec<f32> = scene
                .inner()
                .encoding()
                .resources
                .glyphs
                .iter()
                .map(|g| g.y)
                .collect();
            let lo = ys.iter().cloned().fold(f32::INFINITY, f32::min);
            let hi = ys.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            (lo, hi, ys.len())
        };

        // CONTROLE: a chave escolhida de facto quebra em mais de uma linha sem recorte — senão
        // este gate mediria uma lista que cabia de qualquer maneira.
        let short = glyph_span("k", &mut ts);
        let (lo, hi, n) = glyph_span(long, &mut ts);
        assert!(
            n > short.2 + 10,
            "a fixture nao contem o fenomeno: a chave longa emitiu {n} glifos contra {} da curta",
            short.2
        );

        let row = KeyValueList::row_rect(host, row_h, 0);
        assert!(
            lo >= row.y - 1e-3 && hi <= row.y + row.h + 1e-3,
            "os glifos da chave vao de y={lo} a y={hi}, fora da row [{}, {}] — o texto desceu \
             sobre as rows seguintes",
            row.y,
            row.y + row.h
        );
    }
}
