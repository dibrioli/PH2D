//! **Os primitivos de ROW que o card Brush empresta aos outros** — o rótulo de coluna fixa, a row
//! `rótulo + chip` de dropdown, e o chip nu.
//!
//! ⚠️ **Eles moram aqui porque têm SETE consumidores** (Stroke, Shape, Texture, Paper, Ramp, Line,
//! e o próprio Brush): enquanto viviam dentro do `paint_brush.rs` o arquivo crescia por causa de
//! quem o importava, não de quem ele é — e foi assim que ele cruzou o teto de LOC. O corte é por
//! RESPONSABILIDADE: *a seção Brush* de um lado, *as peças de linha que qualquer seção usa* do
//! outro.

use ph2d_editor_core::IconId;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_icon, paint_text, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::DropdownState;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, TypeToken};

/// ⭐⭐⭐ **AS COLUNAS de uma linha deste painel, derivadas da SECÇÃO a que a chave pertence.**
///
/// ⛔⛔ Até 2026-09-16 a coluna do rótulo era o literal `LABEL_W = 60,0`, escrito no sítio de
/// pintura — o que a spec §3 proíbe por escrito (*«uma largura FIXA está errada por construção: a
/// coluna docada é arrastável»*). Medido nesse dia, com `60`:
///
/// - o rótulo **`Paint Mode`** mede `65,3 px` e saía **cortado em TODA largura de painel**;
/// - e os rótulos destas linhas ficavam encostados à ESQUERDA enquanto as caixas de marcar do mesmo
///   cartão já viviam na coluna da secção ⇒ **duas colunas de nome, alternando linha sim linha
///   não**, que é o defeito que o §6-quinquies da spec existe para matar.
///
/// ⚠️ **O censo que devia ter apanhado isto é CEGO à grafia:** o
/// `the_label_column_is_one_answer` procura `label_col` e este chamava-se `LABEL_W`. *A quinta
/// grafia da mesma pergunta* (§34.6) — a régua foi alargada no mesmo commit.
pub(crate) fn linha_da_chave(
    ctx: &mut PaintCtx,
    x: f32,
    content_w: f32,
    y: f32,
    chave: &str,
) -> ph2d_editor_core::widget::PropertyRow {
    let seccao = crate::seccoes::seccao_da_chave(ctx.text_system, chave);
    ph2d_editor_core::widget::colunas_da_linha(x, content_w, y, ROW_H_PX, seccao)
}

/// O rótulo de uma linha de propriedade: **alinhado à direita da coluna e elidido** (spec §4).
///
/// ⛔ `paint_text` perde as duas coisas — era com ele que este painel pintava, e é por isso que o
/// `Paint Mode` cortava sem reticências e sem ninguém ver.
pub(crate) fn label(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    text: &str,
    row: &ph2d_editor_core::widget::PropertyRow,
    font: f32,
) {
    ph2d_editor_core::widget::paint_property_label(
        ctx.text_system,
        ctx.scene,
        text,
        row.label.x,
        row.label.y + (row.label.h - font) * 0.5,
        font,
        row.label.w,
        resolve(ColorToken::Text2, theme),
    );
}

/// ⭐⭐⭐ **A LINHA DE COR deste painel — pela PORTA da linha de cor**
/// ([`ph2d_editor_core::property_row::paint_color_row`]): o nome à esquerda e a amostra a ENCHER a
/// coluna do valor, com a coluna da SECÇÃO da chave.
///
/// ⛔⛔ **Até 2026-09-23 este painel pintava as suas QUATRO cores de três maneiras:** o Pincel e o
/// Papel com um `fill_rounded_rect` + moldura à mão (já na coluna da secção, e cada uma com a sua
/// cópia do mesmo desenho — o doc da do Papel dizia-se *«Mirror of the Brush section's»*), e a LUZ
/// e a CERA da Impasto como um quadrado de `22 px` encostado a um campo numérico, sem nome nenhum
/// (nem visível, nem acessível). O report do dono de 2026-09-21 — *«os seletores de cor de todo o
/// app precisam ser padronizados»*, com a amostra desenhada como uma BARRA — é esta porta. ⇒ numa
/// Impasto cada cor ganha a SUA linha, e o cartão uma linha a mais (o chamador soma-a no
/// `card_frame`).
///
/// ⚠️ **O clique continua a ser do `event.rs`** (a amostra é um BOTÃO que alterna o selector
/// partilhado, e o chamador regista-o); aqui só se pinta e se põe o hit, que é o que a porta faz.
/// ⚠️ **E a porta lê a cor do `widget_color`**, que o `hero` escreve enquanto o selector está aberto
/// nesta amostra — e deixa lá quando fecha. ⇒ fora desse caso semeia-se a cor do documento, senão
/// um *undo* depois de uma escolha mostraria a cor escolhida sobre um valor que já não é o dela.
#[allow(clippy::too_many_arguments)]
pub(crate) fn color_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    chave: &str,
    id: ph2d_a11y::NodeId,
    rgb: [u8; 3],
) -> f32 {
    let seccao = crate::seccoes::seccao_da_chave(ctx.text_system, chave);
    let rgba = [rgb[0], rgb[1], rgb[2], u8::MAX];
    {
        let store = ctx.host.store_mut();
        if store.picker_target() != Some(id) {
            store.set_widget_color(id, rgba);
        }
    }
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    ph2d_editor_core::property_row::paint_color_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        content_w,
        y,
        tr(chave),
        id,
        rgba,
        false,
        seccao,
    )
}

/// Paint a "label + dropdown chip" row. Returns `(next_y, Some(chip_rect))` when
/// the chip is open (the caller stashes the rect into the matching pending slot).
/// `pub(crate)` so the Stroke section reuses it for Method + Jitter Unit.
///
/// ⭐ **Ela recebe a CHAVE do rótulo, não o texto** — pela mesma razão que a linha de marcar
/// ([`crate::paint_brush_top::paint_checkbox_row`]): *quem só tem o texto traduzido não sabe a que
/// secção pertence*, e a coluna é uma resposta da secção.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_dropdown_row(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    chave: &str,
    id: ph2d_a11y::NodeId,
    cur_value: u8,
    cur_label: &str,
) -> (f32, Option<Rect>) {
    let row = linha_da_chave(ctx, x, content_w, y, chave);
    label(ctx, theme, tr(chave), &row, TypeToken::Sm.px());
    let rect = row.control;
    let open = paint_dropdown_chip(ctx, theme, id, cur_value, cur_label, rect);
    (y + ph2d_tokens::row_pitch_px(), open.then_some(rect))
}

/// Paint a dropdown chip (registered as a `Dropdown` for the generic open/close
/// dispatch). Returns whether it is open. Shared by the Blend + Falloff chips.
pub(crate) fn paint_dropdown_chip(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    id: ph2d_a11y::NodeId,
    cur_value: u8,
    cur_label: &str,
    rect: Rect,
) -> bool {
    paint_dropdown_chip_activo(ctx, theme, id, cur_value, cur_label, rect, true)
}

/// O mesmo chip, com um estado **INATIVO** — ele continua a ver-se e deixa de ser alcançável.
///
/// ⭐ Ele é um parâmetro e não um segundo pintor: *duas maneiras de desenhar um dropdown divergem
/// no dia em que uma delas ganhar um estado*. Nasceu com o menu do `+` da pilha do Composite
/// (2026-09-21), onde a quota gasta tem de o «inativar» sem o fazer desaparecer.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_dropdown_chip_activo(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    id: ph2d_a11y::NodeId,
    cur_value: u8,
    cur_label: &str,
    rect: Rect,
    activo: bool,
) -> bool {
    ctx.host.store_mut().register_if_absent(
        id,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: Some(cur_value as usize),
        },
    );
    let open = matches!(
        ctx.host.store().get(id),
        Some(InteractiveState::Dropdown { open: true, .. })
    );

    // ⭐ Raio e moldura pela porta do TEMA, com o `Feel` do estado do dropdown — a mesma porta do
    //    `Dropdown` da casa (`dropdown_feel`), pela mesma razão do `chip_border_color` abaixo.
    let radius = ph2d_editor_core::paint::frame_radius(theme, Radius::Sm.px());
    fill_rounded_rect(
        ctx.scene,
        rect,
        radius,
        resolve(
            ph2d_editor_core::widget::section_cards::CardDepth::Subsection.token(),
            theme,
        ),
    );
    // ⚠️ **Pela porta do widget, e não por uma quarta cópia da lei.** Este chip é desenhado à
    // mão (não constrói um `Dropdown`) e por isso carregava a sua própria regra de borda — que
    // não conhecia `BorderEmph` e portanto **nunca acendia sob o ponteiro**.
    let (dd_state, dd_t) = ctx.host.store().dropdown_visual(id);
    ph2d_editor_core::paint::stroke_frame(
        ctx.scene,
        rect,
        radius,
        theme,
        ph2d_editor_core::widget::dropdown_feel(dd_state),
        StrokeToken::Default.px(),
        ph2d_editor_core::widget::chip_border_color(dd_state, dd_t, theme),
    );

    let chevron = Spacing::Md.px();
    let pad = Spacing::Sm.px();
    let chevron_rect = Rect::new(
        rect.x + rect.w - pad - chevron,
        rect.y + (rect.h - chevron) * 0.5,
        chevron,
        chevron,
    );
    let icon = if open {
        IconId::ChevronUp
    } else {
        IconId::ChevronDown
    };
    paint_icon(
        ctx.scene,
        icon,
        chevron_rect,
        resolve(
            if activo {
                ColorToken::Text2
            } else {
                ColorToken::TextDisabled
            },
            theme,
        ),
        StrokeToken::Default.px(),
    );

    let font = TypeToken::Sm.px();
    let text_x = rect.x + pad;
    let text_w = (chevron_rect.x - Spacing::Xs.px() - text_x).max(0.0);
    paint_text(
        ctx.text_system,
        ctx.scene,
        cur_label,
        text_x,
        rect.y + (rect.h - font) * 0.5,
        font,
        text_w,
        resolve(
            if activo {
                ColorToken::Text1
            } else {
                ColorToken::TextDisabled
            },
            theme,
        ),
    );

    // ⛔ Um chip inativo NÃO regista hit rect — é isso que o torna inerte sob o dedo, e não a cor.
    if activo {
        ctx.host.hit_index_mut().register(id, rect);
    }
    activo && open
}
