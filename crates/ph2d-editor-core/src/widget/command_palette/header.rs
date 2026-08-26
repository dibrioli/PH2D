//! **A BANDA do cabeçalho da paleta** — título · busca · contagem · a caixa *Show all* · o X.
//!
//! Irmã de [`super`] pelo teto de 500 LOC dos primitivos, e o corte é por **responsabilidade**: o
//! pai responde *que forma tem o cartão e como os grupos entram*; isto responde *o que a banda de
//! cima diz e oferece*.
//!
//! ⚠️ **Ela nasceu com a F3** ([ADR-0166]): a caixa *Show all* levaria o pai a 508 linhas contra um
//! teto de 500, e *ficar no mesmo sítio não é encolher* — a banda inteira mudou-se, não só o que
//! era novo.
//!
//! [ADR-0166]: ../../../../../docs/architecture/decisions/0166-the-inspector-shows-what-the-object-has-and-components-attach-through-one-palette-filtered-by-object-type.md

use super::{CLOSE_W, HEADER_H, MIN_COL_W, PILL_H, PaletteModel};
use crate::interaction::HitIndex;
use crate::paint::{fill_rounded_rect, paint_text, resolve, stroke_rounded_rect};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_tool_registry::hash_node_id;
use ph2d_vector::VectorScene;

/// ⭐ **A caixa da banda** — hoje é a *Show all* da paleta de componentes (ADR-0166 / F3).
///
/// ⚠️ **É um `Option` no [`PaletteModel`], e por isso os outros dois consumidores da paleta (a
/// biblioteca de nós do Motion e o `Ctrl+K`) não pagam nada por ela** — eles passam `None` e a
/// banda desenha-se como sempre. Um campo obrigatório fá-los-ia carregar um controlo que não têm.
///
/// ⚠️ **Quem a lê é quem abriu a paleta**, como o `id` de um item: este widget desenha o estado e
/// regista o rect; o significado de *"mostrar tudo"* pertence a quem construiu o modelo.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaletteToggle {
    /// O rótulo já localizado (HR-15: inglês).
    pub label: String,
    /// Ligada?
    pub on: bool,
}

/// A caixa da banda. Um clique aqui pertence a quem abriu a paleta — ele reabre o modelo com o
/// estado trocado.
pub const CMD_PALETTE_SHOW_ALL: NodeId = hash_node_id("command_palette.show_all");

/// O lado do quadrado da caixa.
const BOX_W: f32 = 16.0; // LITERAL-PX-OK: lado da caixa de marcar da banda

/// Pinta a banda inteira e devolve o rect do X de fechar (que o chamador regista POR ÚLTIMO, para
/// ele ganhar dentro do cartão).
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_header(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    model: &PaletteModel,
    query: &str,
    card_x: f32,
    card_w: f32,
    content_x: f32,
    content_w: f32,
    header_y: f32,
    pad: f32,
    font: f32,
) -> Rect {
    // ── Header band: title left, search box in the middle, count centre-right, close-X right. ──
    let title_w = ts.prefix_width(&model.title, font).min(content_w * 0.35); // LITERAL-PX-OK: FRAÇÃO da banda que o título pode tomar antes de ceder ao campo de busca
    paint_text(
        ts,
        scene,
        &model.title,
        content_x,
        header_y + (HEADER_H - font) * 0.5,
        font,
        title_w,
        resolve(ColorToken::Text1, theme),
    );
    let count_str = format!("{} nodes", model.item_count());
    let count_w = ts.prefix_width(&count_str, TypeToken::Sm.px());
    let close_x = card_x + card_w - CLOSE_W - pad;
    let count_x = close_x - count_w - Spacing::Sm.px();
    paint_text(
        ts,
        scene,
        &count_str,
        count_x,
        header_y + (HEADER_H - TypeToken::Sm.px()) * 0.5,
        TypeToken::Sm.px(),
        count_w,
        resolve(ColorToken::Text2, theme),
    );

    // ⭐ **A caixa da banda** (ADR-0166 / F3), à esquerda da contagem — e o que ela toma sai do
    //    campo de busca, senão os dois desenham um por cima do outro.
    let toggle_w = paint_toggle(
        scene,
        ts,
        theme,
        hit_index,
        model.toggle.as_ref(),
        count_x - Spacing::Sm.px(),
        header_y,
    );

    // ── Search box: a Bg3 rounded field showing the typed query (or a "Search" placeholder), the app's
    //    keyboard feeds it while the palette is open (a full-screen modal captures the keys). ──
    let sm = TypeToken::Sm.px();
    let search_h = (HEADER_H - Spacing::Sm.px()).max(PILL_H);
    let sb_x = content_x + title_w + Spacing::Md.px();
    let sb_w = (count_x - toggle_w - Spacing::Md.px() - sb_x).max(MIN_COL_W * 0.5);
    let sb = Rect::new(sb_x, header_y + (HEADER_H - search_h) * 0.5, sb_w, search_h);
    fill_rounded_rect(scene, sb, Radius::Sm.px(), resolve(ColorToken::Bg3, theme));
    stroke_rounded_rect(
        scene,
        sb,
        Radius::Sm.px(),
        1.0,
        resolve(ColorToken::Border, theme),
    );
    let (search_text, search_color) = if query.is_empty() {
        ("Search", resolve(ColorToken::Text2, theme))
    } else {
        (query, resolve(ColorToken::Text1, theme))
    };
    let text_y = sb.y + (sb.h - sm) * 0.5;
    paint_text(
        ts,
        scene,
        search_text,
        sb.x + Spacing::Sm.px(),
        text_y,
        sm,
        sb.w - Spacing::Sm.px() * 2.0,
        search_color,
    );
    // Caret at the end of the typed text (only while there IS a query — the placeholder needs none).
    if !query.is_empty() {
        let caret_x = (sb.x + Spacing::Sm.px() + ts.prefix_width(query, sm))
            .min(sb.x + sb.w - Spacing::Xs.px());
        fill_rounded_rect(
            scene,
            Rect::new(
                caret_x,
                sb.y + Spacing::Xs.px(),
                1.5, // LITERAL-PX-OK: espessura do traço do X de fechar
                sb.h - Spacing::Xs.px() * 2.0,
            ), // LITERAL-PX-OK: 1.5px text caret
            0.0,
            resolve(ColorToken::Text1, theme),
        );
    }
    let close_rect = Rect::new(
        close_x,
        header_y + (HEADER_H - CLOSE_W) * 0.5,
        CLOSE_W,
        CLOSE_W,
    );
    paint_text(
        ts,
        scene,
        "X",
        close_rect.x + CLOSE_W * 0.3, // LITERAL-PX-OK: o X recua 30% para dentro do quadrado (fração, não medida)
        close_rect.y + (CLOSE_W - font) * 0.5,
        font,
        CLOSE_W,
        resolve(ColorToken::Text1, theme),
    );

    close_rect
}

/// ⭐ **A caixa de marcar**, à esquerda da contagem. Devolve a largura que ela tomou (`0` quando o
/// modelo não tem caixa nenhuma), para o campo de busca ceder-lhe o espaço.
///
/// ⚠️ **Ela regista o hit mesmo desligada** — um controlo que só é clicável no estado em que já
/// está seria uma porta de sentido único, e o artista não teria como voltar.
fn paint_toggle(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    toggle: Option<&PaletteToggle>,
    right_x: f32,
    header_y: f32,
) -> f32 {
    let Some(t) = toggle else {
        return 0.0;
    };
    let sm = TypeToken::Sm.px();
    let label_w = ts.prefix_width(&t.label, sm);
    let total = BOX_W + Spacing::Xs.px() + label_w;
    let x = right_x - total;
    let box_rect = Rect::new(x, header_y + (HEADER_H - BOX_W) * 0.5, BOX_W, BOX_W);
    let (bg, fg) = if t.on {
        (ColorToken::Accent, ColorToken::Text1)
    } else {
        (ColorToken::Bg3, ColorToken::Text2)
    };
    fill_rounded_rect(scene, box_rect, Radius::Sm.px(), resolve(bg, theme));
    stroke_rounded_rect(
        scene,
        box_rect,
        Radius::Sm.px(),
        1.0,
        resolve(ColorToken::Border, theme),
    );
    if t.on {
        // O visto, em texto — o mesmo idioma do X de fechar, que também é uma letra.
        paint_text(
            ts,
            scene,
            "x",
            box_rect.x + BOX_W * 0.28, // LITERAL-PX-OK: fração, não medida
            box_rect.y + (BOX_W - sm) * 0.5,
            sm,
            BOX_W,
            resolve(ColorToken::BgElev, theme),
        );
    }
    paint_text(
        ts,
        scene,
        &t.label,
        x + BOX_W + Spacing::Xs.px(),
        header_y + (HEADER_H - sm) * 0.5,
        sm,
        label_w,
        resolve(fg, theme),
    );
    // ⚠️ O hit cobre a caixa **e** o rótulo: um alvo de 16 px é o que faz um artista clicar duas
    // vezes e concluir que o controlo está morto.
    hit_index.register(
        CMD_PALETTE_SHOW_ALL,
        Rect::new(x, header_y, total, HEADER_H),
    );
    total + Spacing::Md.px()
}
