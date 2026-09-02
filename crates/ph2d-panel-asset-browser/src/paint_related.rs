//! ⭐⭐ **A FAIXA DA PERGUNTA DE RELAÇÃO** (plano 07 D9) — irmã por assunto do [`super::paint`],
//! que está a poucas linhas do tecto dos painéis.
//!
//! # O que ela é, e porque não é um chip no cabeçalho
//!
//! Os outros dois filtros da grade têm controlo **permanente** — a fileira de família e a coluna
//! de catálogos —, e por isso vê-se sempre o que está ligado. Este nasce de um **item de menu**:
//! sem uma faixa, o artista fica com uma grade encolhida e nenhum sítio que diga porquê.
//!
//! ⛔⛔ *Um filtro que só um menu liga tem de trazer o próprio interruptor de desligar.* É por isso
//! que a faixa carrega o `✕` e não apenas o texto — e é a razão de o `ASSET_RELATED_CLEAR` existir.
//!
//! ⚠️ **Ela NOMEIA a âncora**, e o nome é lido do índice deste quadro. Se a âncora sair da
//! biblioteca no meio (alguém a tirou), a faixa cai para *«this asset»* e a grade fica vazia —
//! porque o [`ph2d_asset_index::AssetIndex::query`] falha FECHADO sobre uma âncora que não existe.
//! ⛔ A alternativa (guardar o nome ao ligar o filtro) daria uma faixa a nomear um asset que já não
//! está lá, sobre uma grade vazia: a etiqueta certa em cima da resposta errada.

use crate::ids;
use crate::paint::pad;
use crate::state::{AssetBrowserState, with_index};
use ph2d_asset_index::Relation;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{IconButtonStyle, IconGlyph, paint_icon_button};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Density, Radius, Spacing, TypeToken};

/// Pinta a faixa se houver filtro, e devolve o `y` de baixo (igual ao de cima quando não há).
pub(crate) fn paint(state: &AssetBrowserState, ctx: &mut PaintCtx, rect: Rect, y: f32) -> f32 {
    let Some((anchor, dir)) = state.related else {
        return y;
    };
    let theme = ctx.host.theme();
    let x = rect.x + pad();
    let w = rect.w - pad() * 2.0;
    let row_h = Density::Compact.row_h_px();
    let band = Rect::new(x, y, w, row_h);

    fill_rounded_rect(
        ctx.scene,
        band,
        Radius::Sm.px(),
        resolve(ColorToken::AccentSoft, theme),
    );

    // ⭐ O `✕`, à direita e dentro da faixa — pela PRIMITIVA canónica (`paint_icon_button`), e não
    // por um `paint_icon` cru.
    //
    // ⛔⛔ **O gate da HR-12 apanhou-me a fazê-lo à mão**, e a cura não era a lista de dispensa: um
    // ícone desenhado directamente é **mudo para a árvore de acessibilidade** e não veste o hover
    // vivo, então ele nasceria a pintar uma cor dura no meio de vizinhos que deslizam. *Uma
    // dispensa serve a um ficheiro sem semântica de utilizador; este é um botão.*
    let close = Rect::new(band.x + band.w - row_h, band.y, row_h, row_h);
    paint_icon_button(
        close,
        IconGlyph::Builtin(ph2d_editor_core::icons::IconId::Close),
        IconButtonStyle::Plain,
        ctx.host.store().button_visual(ids::ASSET_RELATED_CLEAR),
        ctx.scene,
        theme,
    );
    ctx.host
        .hit_index_mut()
        .register(ids::ASSET_RELATED_CLEAR, close);

    let name = with_index(|ix| ix.get(&anchor).map(|e| e.name.clone()));
    let name = name.unwrap_or_else(|| "this asset".to_string());
    let label = match dir {
        Relation::Uses => format!("What \u{201c}{name}\u{201d} uses"),
        Relation::UsedBy => format!("What uses \u{201c}{name}\u{201d}"),
    };
    paint_text(
        ctx.text_system,
        ctx.scene,
        &label,
        band.x + Spacing::Xs.px(),
        band.y + row_h - Spacing::Xs.px(),
        TypeToken::Sm.px(),
        (band.w - row_h - Spacing::Xs.px() * 2.0).max(0.0),
        resolve(ColorToken::Text1, theme),
    );

    y + row_h + Spacing::Xs.px()
}
