//! ⭐ **«Destaca, depois escolhe»** (Add Object, dono 07/11, `docs/GameEngineFuncional/24_add_object_plano.md`
//! §2.1–2.2): um clique destaca o item e mostra a dica numa FAIXA fixa no fundo da janela; dois cliques (ou
//! Enter) escolhem. Só para o modelo que traz [`PaletteSelect`] — os outros consumidores da paleta escolhem
//! num clique, como sempre. A 6.ª responsabilidade da paleta, irmã pelo tecto de 500 LOC dos primitivos.
//!
//! ⚠️ **A faixa tem a altura da dica MAIS ALTA** do modelo, medida: trocar de destaque nunca faz a janela
//! saltar.

use super::{PaletteModel, PaletteTip};
use crate::interaction::HitIndex;
use crate::paint::{fill_rounded_rect, paint_text_block, resolve};
use crate::widget::lista_flutuante::Superficie;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_tool_registry::hash_node_id;
use ph2d_vector::VectorScene;

/// O botão do ensino («Start with a Sprite»): limpa a busca e destaca o [`PaletteTeach::start`].
pub const CMD_PALETTE_TEACH_START: NodeId = hash_node_id("command_palette.teach_start");

/// A largura da moldura do desenho na faixa (a altura é metade: a moldura 2:1 das dicas).
const DOCK_PICTURE_W: f32 = 176.0; // LITERAL-PX-OK: geometria do modal, moldura 2:1 do desenho da dica

/// ⭐ **O gesto «destaca, depois escolhe»** de um modelo.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaletteSelect {
    /// O item destacado ao abrir.
    pub initial: NodeId,
    /// O que a busca que não acha ensina.
    pub teach: Vec<PaletteTeach>,
}

/// ⭐ **Uma busca que não acha, a ensinar** (plano 24 §2.5): «coin» não é um objecto, e a frase diz de que
/// peças se monta.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaletteTeach {
    /// As palavras que a disparam, por vírgula, inteiras («coin, coins, money»).
    pub words: &'static str,
    /// A frase.
    pub text: String,
    /// O item por onde se começa, e o rótulo do botão que o destaca.
    pub start: NodeId,
    pub start_label: String,
}

/// O ensino para `query`, se alguma palavra dele é a busca inteira (sem caixa, sem espaços nas pontas).
#[must_use]
pub fn teach_for<'a>(model: &'a PaletteModel, query: &str) -> Option<&'a PaletteTeach> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return None;
    }
    model
        .select
        .as_ref()?
        .teach
        .iter()
        .find(|t| t.words.split(',').any(|w| w.trim().to_lowercase() == q))
}

/// Os ids dos itens de `model`, na ordem em que se pintam (grupo, fatia, item) — a ordem das setas.
#[must_use]
pub fn item_ids(model: &PaletteModel) -> Vec<NodeId> {
    model
        .groups
        .iter()
        .flat_map(|g| &g.subs)
        .flat_map(|s| &s.items)
        .map(|it| it.id)
        .collect()
}

/// A altura da faixa da dica para `content_w`: `0` sem [`PaletteSelect`] ou sem dicas.
pub(super) fn dock_height(ts: &mut TextSystem, model: &PaletteModel, content_w: f32) -> f32 {
    if model.select.is_none() || model.tips.is_empty() {
        return 0.0;
    }
    let pad = Spacing::Md.px();
    let (text_w, picture_h) = dock_split(content_w, pad);
    let mut scratch = VectorScene::new();
    let tallest = model
        .tips
        .iter()
        .map(|t| text_height(ts, &mut scratch, t, text_w))
        .fold(0.0_f32, f32::max);
    tallest.max(picture_h) + pad * 2.0
}

/// A largura do texto e a altura do desenho na faixa.
fn dock_split(content_w: f32, pad: f32) -> (f32, f32) {
    let picture_w = DOCK_PICTURE_W.min(content_w * 0.4);
    (
        (content_w - picture_w - pad * 3.0).max(0.0),
        picture_w * 0.5,
    )
}

fn text_height(ts: &mut TextSystem, scene: &mut VectorScene, tip: &PaletteTip, w: f32) -> f32 {
    paint_texts(
        ts,
        scene,
        tip,
        0.0,
        0.0,
        w,
        [ph2d_vector::Color::TRANSPARENT; 2],
    )
}

/// Título e texto, um sobre o outro; devolve a altura que ocuparam.
fn paint_texts(
    ts: &mut TextSystem,
    scene: &mut VectorScene,
    tip: &PaletteTip,
    x: f32,
    y: f32,
    w: f32,
    [title_c, body_c]: [ph2d_vector::Color; 2],
) -> f32 {
    let title_h = paint_text_block(ts, scene, &tip.title, x, y, TypeToken::Md.px(), w, title_c);
    let gap = Spacing::Xs.px();
    let body_h = paint_text_block(
        ts,
        scene,
        &tip.body,
        x,
        y + title_h + gap,
        TypeToken::Sm.px(),
        w,
        body_c,
    );
    title_h + gap + body_h
}

/// ⭐ **Pinta a faixa** em `dock` com a dica de `chosen` (vazia se ele não tem dica nem é visível).
pub(super) fn paint_dock(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    model: &PaletteModel,
    chosen: Option<NodeId>,
    dock: Rect,
) {
    let superficie = Superficie::de(theme);
    let cor = |antes: ColorToken, cartao: ColorToken| match superficie.tinta() {
        Some(ink) => ink.color(cartao),
        None => resolve(antes, theme),
    };
    let radius = crate::paint::frame_radius(theme, Radius::Md.px());
    fill_rounded_rect(
        scene,
        dock,
        radius,
        cor(ColorToken::Bg2, ColorToken::KanbanCard),
    );
    let Some(tip) = chosen.and_then(|id| model.tips.iter().find(|t| t.id == id)) else {
        return;
    };
    let pad = Spacing::Md.px();
    let (text_w, picture_h) = dock_split(dock.w, pad);
    let frame = Rect::new(dock.x + pad, dock.y + pad, picture_h * 2.0, picture_h);
    fill_rounded_rect(
        scene,
        frame,
        crate::paint::frame_radius(theme, Radius::Sm.px()),
        cor(ColorToken::Bg3, ColorToken::CardField),
    );
    crate::widget::illustration::paint_illustration(scene, frame, tip.picture, theme);
    let colors = [
        cor(ColorToken::Text1, ColorToken::KanbanCardFg),
        cor(ColorToken::Text2, ColorToken::CardLabel),
    ];
    let text_x = frame.x + frame.w + pad;
    paint_texts(ts, scene, tip, text_x, dock.y + pad, text_w, colors);
}

/// ⭐ **A busca que não acha, no modo de seleção:** «Nothing is called “x”.», o ensino se houver, e o botão
/// que leva ao primeiro item. Regista o botão.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_teach(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    model: &PaletteModel,
    query: &str,
    at: (f32, f32),
    w: f32,
) -> f32 {
    if model.select.is_none() {
        return 0.0;
    }
    let superficie = Superficie::de(theme);
    let (x, y0) = at;
    let mut y = y0;
    let nothing = ph2d_i18n::tr("chrome.palette.nothing_called").replace("{query}", query.trim());
    let font = TypeToken::Md.px();
    y += paint_text_block(
        ts,
        scene,
        &nothing,
        x,
        y,
        font,
        w,
        superficie.cor(ColorToken::Text1),
    );
    let Some(t) = teach_for(model, query) else {
        return y - y0;
    };
    y += Spacing::Xs.px();
    let c = superficie.cor(ColorToken::Text2);
    y += paint_text_block(ts, scene, &t.text, x, y, TypeToken::Sm.px(), w, c);
    y += Spacing::Sm.px();
    let label_w = ts.prefix_width(&t.start_label, TypeToken::Sm.px());
    let button = Rect::new(
        x,
        y,
        label_w + Spacing::Lg.px() * 2.0,
        ph2d_card_stack::metrics::FIELD_H,
    );
    let fg = superficie.botao(scene, button, true, false);
    let label_y = button.y + superficie.folga(button.h, TypeToken::Sm.px());
    let label_x = button.x + Spacing::Lg.px();
    crate::paint::paint_text(
        ts,
        scene,
        &t.start_label,
        label_x,
        label_y,
        TypeToken::Sm.px(),
        label_w + 1.0,
        fg,
    );
    hit_index.register(CMD_PALETTE_TEACH_START, button);
    button.y + button.h - y0
}

/// A altura que [`paint_teach`] ocupa — medida pintando, numa cena à parte (o corpo da janela tem de a
/// conter, senão o recorte da rolagem engole o clique do botão).
pub(super) fn teach_height(ts: &mut TextSystem, model: &PaletteModel, query: &str, w: f32) -> f32 {
    let (mut scene, mut hits) = (VectorScene::new(), HitIndex::new());
    let theme = Theme::default();
    paint_teach(
        &mut scene,
        ts,
        theme,
        &mut hits,
        model,
        query,
        (0.0, 0.0),
        w,
    )
}

#[cfg(test)]
#[path = "selecao_tests.rs"]
mod tests;
