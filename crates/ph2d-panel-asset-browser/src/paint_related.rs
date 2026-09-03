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
//! # ⛔⛔ Ela é a faixa da GRADE, não do painel (report do Enio, 2026-09-02: *«layout ruim»*)
//!
//! A 1.ª versão desenhava-a **à largura toda**, por cima da coluna de catálogos — e o resultado é
//! a foto: o rótulo por baixo do `+ Catalog`, e o `✕` dela empilhado mesmo debaixo do `✕` que
//! FECHA o painel. Dois `✕` na mesma coluna, um a fechar a janela e outro a largar um filtro.
//!
//! ⇒ ela ocupa **exactamente a faixa da grade** (à direita da coluna), que é o que ela filtra.
//! ⚠️ *A largura de um controlo é uma afirmação sobre o que ele manda* — à largura toda, ela dizia
//! que filtrava também a coluna, e não filtra.
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

thread_local! {
    /// **O rectângulo que a faixa desenhou neste quadro** — só para os gates.
    ///
    /// ⛔⛔ **Ela existe porque o report do Enio (*«layout ruim»*) foi um defeito de GEOMETRIA, e
    /// nenhum dos cinco gates desta fatia media geometria nenhuma.** Eles perguntavam *«o id está
    /// no índice de toque?»* e *«o clique chega ao estado?»* — as duas verdadeiras enquanto a
    /// faixa nascia por cima da coluna de catálogos, com o rótulo debaixo de um botão. *Um
    /// controlo pode estar vivo, alcançável e no sítio errado, e as três perguntas são diferentes.*
    static BAND: std::cell::Cell<Option<Rect>> = const { std::cell::Cell::new(None) };
}

/// O rectângulo da faixa neste quadro, ou `None` se ela não foi pintada.
#[must_use]
pub fn probe_band_rect() -> Option<Rect> {
    BAND.with(std::cell::Cell::get)
}

/// Pinta a faixa se houver filtro, e devolve o `y` de baixo (igual ao de cima quando não há).
///
/// ⚠️ **`col_w` é a largura que a coluna de catálogos ocupou neste quadro** — a mesma que o
/// `paint_grid` recebe. Os dois têm de sair do MESMO número: uma faixa medida sobre a largura
/// inteira nasce por cima da coluna, que é o report do *«layout ruim»*.
pub(crate) fn paint(
    state: &AssetBrowserState,
    ctx: &mut PaintCtx,
    rect: Rect,
    y: f32,
    col_w: f32,
) -> f32 {
    BAND.with(|b| b.set(None));
    let Some((anchor, dir)) = state.related else {
        return y;
    };
    let theme = ctx.host.theme();
    let x = rect.x + col_w + pad();
    let w = (rect.x + rect.w - pad() - x).max(0.0);
    let row_h = Density::Compact.row_h_px();
    if w <= row_h {
        // ⛔ Sem largura para o rótulo E o `✕`, a faixa não nasce — e o `✕` não é registado, logo
        // não fica um botão invisível a comer cliques onde nada se pinta. ⚠️ O filtro **continua
        // ligado**: quem o largou não foi o artista, foi a geometria.
        return y;
    }
    let band = Rect::new(x, y, w, row_h);
    BAND.with(|b| b.set(Some(band)));

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
        // ⛔ **A MESMA centragem do `paint_list_item`**, que é o primitivo desta forma — e é a
        // conta que a coluna de catálogos ao lado já usa. A 1.ª versão punha a linha de base em
        // `y + row_h − Xs`, ou seja **no bordo de baixo**: o rótulo descia para fora da faixa e ia
        // colidir com o botão da fileira seguinte. *Uma linha de base não é uma margem.*
        band.y + (row_h - TypeToken::Sm.px()) * 0.5,
        TypeToken::Sm.px(),
        (band.w - row_h - Spacing::Xs.px() * 2.0).max(0.0),
        resolve(ColorToken::Text1, theme),
    );

    y + row_h + Spacing::Xs.px()
}
