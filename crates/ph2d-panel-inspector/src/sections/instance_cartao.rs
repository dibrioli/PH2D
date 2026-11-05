//! Os cartões Instance e Properties na lista de cartões: frases, botões com face e escolhas no ritmo
//! de toda linha (05 §4), sem caixa dentro da caixa. Mesmos ids e mesma ordem do cartão de antes
//! ([`super::instance`]), que continua a ser o de `PH2D_CARDS=0`.

use super::rows::aviso_em;
use super::*;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::panel::cards::Seg;
use ph2d_editor_core::panel::cards::corpo::choices;
use ph2d_editor_core::property_row::{abaixo_do_botao, caixa_do_botao};
use ph2d_editor_core::screens::hero::{InspectorInstanceInfo, InspectorPropertiesInfo};
use ph2d_i18n::{tr, tr_with};

/// A mão que pinta: tudo o que as linhas partilham.
struct Mao<'a> {
    scene: &'a mut VectorScene,
    text: &'a mut TextSystem,
    theme: Theme,
    hit: &'a mut HitIndex,
    store: &'a WidgetStore,
    x: f32,
    w: f32,
}

impl Mao<'_> {
    #[track_caller]
    fn frase(&mut self, y: f32, t: &str, token: ColorToken, tamanho: TypeToken) -> f32 {
        aviso_em(
            self.scene, self.text, self.theme, self.x, self.w, y, t, token, tamanho,
        )
    }

    /// A primeira linha do corpo: o cartão já traz a margem, não leva o vão de cima.
    #[track_caller]
    fn primeira(&mut self, y: f32, t: &str, token: ColorToken, tamanho: TypeToken) -> f32 {
        self.frase(y - m::FIELD_ROW_GAP, t, token, tamanho)
    }

    fn botao(&mut self, y: f32, id: NodeId, rotulo: &str) -> f32 {
        let caixa = caixa_do_botao(self.text, self.x, self.w, y, rotulo);
        self.hit.register(id, caixa);
        let b = Button::new(id, rotulo.to_owned())
            .kind(ButtonKind::Default)
            .visual(self.store.button_visual(id));
        paint_button(&b, caixa, self.scene, self.text, self.theme);
        abaixo_do_botao(caixa)
    }

    /// Uma exceção sem alvo: a frase, e o «×» dela na mesma fila, da altura de todo controlo.
    fn orfa(&mut self, y: f32, t: &str, id: Option<NodeId>) -> f32 {
        let tw = (self.w - m::FIELD_H - m::FIELD_ROW_GAP).max(0.0);
        let font = TypeToken::Sm.px();
        let alta = super::text_h(self.text, t, font, tw, font);
        let desce = ((m::FIELD_H - alta) * 0.5).max(0.0);
        let fundo = aviso_em(
            self.scene,
            self.text,
            self.theme,
            self.x,
            tw,
            y + desce,
            t,
            ColorToken::Text2,
            TypeToken::Sm,
        );
        let topo = y + m::FIELD_ROW_GAP;
        if let Some(id) = id {
            let host = Rect::new(self.x + self.w - m::FIELD_H, topo, m::FIELD_H, m::FIELD_H);
            self.hit.register(id, host);
            paint_icon_button(
                host,
                IconGlyph::Builtin(IconId::Close),
                IconButtonStyle::Compact,
                self.store.button_visual(id),
                self.scene,
                self.theme,
            );
        }
        fundo.max(topo + m::FIELD_H)
    }
}

/// De quem é cópia, o que tem de diferente, e os gestos de cada diferença. Devolve o `y` de baixo.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_instance(
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
    hit: &mut HitIndex,
    store: &WidgetStore,
    info: &InspectorInstanceInfo,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let mut c = Mao {
        scene,
        text,
        theme,
        hit,
        store,
        x,
        w,
    };
    let mut y = c.primeira(y, &info.provenance(), ColorToken::Text1, TypeToken::Base);
    y = c.frase(y, &info.summary(), ColorToken::Text2, TypeToken::Sm);
    y = c.botao(
        y,
        ids::INSP_INSTANCE_OPEN_PREFAB,
        &tr("panel.inspector.instance.edit_prefab"),
    );
    for name in &info.overridden {
        y = c.frase(
            y,
            &format!("\u{2022} {name}"),
            ColorToken::Text1,
            TypeToken::Sm,
        );
    }
    let ladder = info.apply_rows();
    for (choice, &id) in ladder.iter().zip(ids::INSP_INSTANCE_APPLY_LEVEL.iter()) {
        y = c.botao(y, id, &choice.label());
    }
    if !ladder.is_empty() && info.apply_levels_beyond > 0 {
        let t = tr_with(
            "panel.inspector.instance.deeper",
            &[("n", &info.apply_levels_beyond)],
        );
        y = c.frase(y, &t, ColorToken::Text2, TypeToken::Sm);
    }
    for (row, &id) in info
        .added_rows
        .iter()
        .zip(ids::INSP_INSTANCE_APPLY_ADDED.iter())
    {
        y = c.botao(y, id, &row.label());
    }
    let sobra = instance_added::buttonless(info);
    if sobra > 0 {
        let t = tr_with(
            "panel.inspector.instance.added_more",
            &[("left_out", &sobra)],
        );
        y = c.frase(y, &t, ColorToken::Text2, TypeToken::Sm);
    }
    for (row, &id) in info
        .removed_rows
        .iter()
        .zip(ids::INSP_INSTANCE_RESTORE_PIECE.iter())
    {
        y = c.botao(y, id, &row.label());
    }
    let sobra = instance_removed::buttonless(info);
    if sobra > 0 {
        let t = tr_with(
            "panel.inspector.instance.removed_more",
            &[("left_out", &sobra)],
        );
        y = c.frase(y, &t, ColorToken::Text2, TypeToken::Sm);
    }
    for (i, row) in info.orphan_rows.iter().enumerate() {
        let id = ids::INSP_INSTANCE_DROP_ORPHAN.get(i).copied();
        y = c.orfa(y, &format!("\u{2022} {}", row.label()), id);
    }
    let sobra = instance_orphans::dropless(info);
    if sobra > 0 {
        let t = tr_with(
            "panel.inspector.instance.orphans_more",
            &[("left_out", &sobra)],
        );
        y = c.frase(y, &t, ColorToken::Text2, TypeToken::Sm);
    }
    if info.orphans() > 0 {
        let t = tr_with(
            "panel.inspector.instance.clear_orphans",
            &[("n", &info.orphans())],
        );
        y = c.botao(y, ids::INSP_INSTANCE_CLEAR_ORPHANS, &t);
    }
    y
}

/// O título e uma escolha por eixo, rótulo em cima. Devolve o `y` de baixo.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_properties(
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
    hit: &mut HitIndex,
    store: &WidgetStore,
    info: &InspectorPropertiesInfo,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let mut c = Mao {
        scene,
        text,
        theme,
        hit,
        store,
        x,
        w,
    };
    let Some(ink) = ph2d_editor_core::widget::card_ink::card_body() else {
        return y;
    };
    let titulo = properties::card_title(info);
    let mut y = c.primeira(y, &titulo, ColorToken::Text2, TypeToken::Sm);
    for (ax, row_ids) in info.rows.iter().zip(ids::INSP_INSTANCE_AXIS_OPTION.iter()) {
        let rotulo = if ax.name.is_empty() {
            properties::FLAT_AXIS_LABEL.tr()
        } else {
            ax.name.as_str()
        };
        if ax.options.len() < 2 {
            y = super::rows::rotulo_do_bloco(c.scene, c.text, c.theme, x, w, y, rotulo);
            if let Some(so) = ax.options.first() {
                // `rotulo_do_bloco` já traz o vão de baixo.
                y = c.frase(
                    y - m::FIELD_ROW_GAP,
                    &so.label,
                    ColorToken::Text1,
                    TypeToken::Base,
                );
            }
            continue;
        }
        let segs: Vec<Seg<'_>> = ax
            .options
            .iter()
            .zip(row_ids.iter())
            .map(|(v, id)| Seg {
                id: *id,
                icon: None,
                text: v.label.as_str(),
                on: v.current,
            })
            .collect();
        // Sem ids de marca: escolher uma variante não se anima.
        y = choices(
            c.scene,
            c.text,
            store,
            c.hit,
            ink,
            x,
            w,
            y,
            rotulo,
            &[],
            &segs,
        );
    }
    if info.beyond > 0 {
        let t = tr_with(
            "panel.inspector.properties.more_not_shown",
            &[("n", &info.beyond)],
        );
        y = c.frase(y, &t, ColorToken::Text2, TypeToken::Sm);
    }
    y
}
