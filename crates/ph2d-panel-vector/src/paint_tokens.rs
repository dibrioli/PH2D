//! **A row de TOKEN** — o chip que diz de que token esta propriedade segue, e o popover que o
//! escolhe (plano UI/UX §4/W4).
//!
//! ⚠️ **Ela mora DENTRO das seções Fill e Stroke, ao lado da swatch**, e não numa seção própria.
//! É essa adjacência que responde à pior pergunta que a feature pode gerar — *"por que a cor que
//! eu escolhi não aparece?"* —, porque a resposta fica no mesmo lugar em que a pergunta nasce.
//! Uma seção "Tokens" à parte deixaria a swatch mentindo sozinha lá em cima.

use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{
    DROPDOWN_SCROLLBAR_ID, Dropdown, DropdownOption, paint_dropdown_chip,
    paint_dropdown_popover_scrolled,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, Spacing, Theme};

use crate::ids;
use crate::paint_sections::{BodyCtx, LABEL_COL_W};
use crate::state;

/// O rótulo que o chip mostra: a chave do token, ou o travessão de *não preso*.
fn chip_label(bound: Option<&str>) -> &str {
    bound.unwrap_or("—")
}

/// As opções do popover: **Unbind primeiro**, depois os tokens na ordem da tabela.
///
/// ⚠️ A lista é `ColorToken::ALL`, e nunca uma cópia escrita aqui: um token novo entra na tabela
/// da macro e aparece no picker de graça. Uma segunda lista nasceria desatualizada no primeiro
/// token acrescentado — e o modo de falha é o artista não conseguir escolher uma cor que existe.
fn options(prop: u16) -> Vec<DropdownOption<usize>> {
    std::iter::once(DropdownOption::new(
        ids::vector_token_option_id(prop, 0),
        0,
        tr("panel.vector.token.none"),
    ))
    .chain(ColorToken::ALL.iter().enumerate().map(|(i, t)| {
        DropdownOption::new(ids::vector_token_option_id(prop, i + 1), i + 1, t.key())
    }))
    .collect()
}

/// A espessura da rachura, em px de TELA.
const SLASH_W_PX: f32 = 2.0; // LITERAL-PX-OK: espessura de chrome, irmã do 1.0 do foco

impl BodyCtx<'_> {
    /// **A rachura sobre uma swatch cujo valor um token cobre.**
    ///
    /// Porta ÚNICA das duas swatches: elas fazem a mesma afirmação (*o que se vê aqui não é o que
    /// a arte desenha*), e duas chamadas com números diferentes seriam duas marcas para o mesmo
    /// facto.
    pub(crate) fn token_slash(&mut self, rect: Rect) {
        ph2d_editor_core::paint_shapes::fill_slash(
            self.scene,
            rect,
            SLASH_W_PX,
            ph2d_editor_core::paint::resolve(ColorToken::Danger, self.theme),
        );
    }

    /// A fileira `<rótulo> [token ▾]` de uma propriedade. Devolve o `y` seguinte.
    ///
    /// Mesmo desenho da fileira de Blend de um filtro — a mesma estética para a mesma pergunta
    /// *"qual destes?"*.
    pub(crate) fn token_row(
        &mut self,
        id: ph2d_a11y::NodeId,
        prop: u16,
        bound: Option<&str>,
        y: f32,
    ) -> f32 {
        let gap = Spacing::Sm.px();
        ph2d_editor_core::paint::paint_text(
            self.text_system,
            self.scene,
            tr("panel.vector.token"),
            self.inner_x,
            y + (self.row_h - self.font) * 0.5,
            self.font,
            LABEL_COL_W,
            ph2d_editor_core::paint::resolve(ColorToken::Text1, self.theme),
        );
        let chip = Rect::new(
            self.inner_x + LABEL_COL_W + gap,
            y,
            (self.inner_w - LABEL_COL_W - gap).max(1.0),
            self.row_h,
        );
        let open = matches!(
            self.store.get(id),
            Some(InteractiveState::Dropdown { open: true, .. })
        );
        let dd = Dropdown::new(id, "", vec![DropdownOption::new(id, (), chip_label(bound))])
            .selected(())
            .open(open);
        paint_dropdown_chip(&dd, chip, self.scene, self.text_system, self.theme);
        self.hit_index.register(id, chip);
        if open {
            state::set_pending_token_dd(Some((prop, chip)));
        }
        y + self.row_h + self.row_gap
    }
}

/// **Que linha do popover está vigente** — `0` é *Unbind*, `i + 1` é `ColorToken::ALL[i]`.
///
/// ⚠️ Porta ÚNICA das duas perguntas que TÊM de concordar: qual linha o popover destaca, e qual
/// rótulo o chip mostra. Duas contas divergiriam no dia em que a lista ganhasse um membro, e o
/// artista veria o chip a dizer um token e o picker a destacar outro.
pub fn selected_row(prop: u16) -> usize {
    let b = state::token_bindings().unwrap_or_default();
    let key = if prop == 0 { b.fill } else { b.stroke };
    key.and_then(|k| ColorToken::ALL.iter().position(|t| t.key() == k))
        .map_or(0, |i| i + 1)
}

/// **O popover dos tokens** — pintado no passe DIFERIDO, por cima de todas as seções.
///
/// Espelho exato do popover de mistura dos filtros, e pelo mesmo motivo: são oitenta linhas, e o
/// card tem de rolar e ficar acima do scroll da seção.
pub(crate) fn paint_token_popover(ctx: &mut PaintCtx, prop: u16, chip: Rect, theme: Theme) {
    let id = if prop == 0 {
        ids::VECTOR_TOKEN_FILL
    } else {
        ids::VECTOR_TOKEN_STROKE
    };
    let opts = options(prop);
    // ⚠️ A linha DESTACADA é a do token vigente, nunca um índice cravado. Com `0` cravado o
    // popover dizia "None" mesmo sobre uma forma bindada — e então escolher None de volta parecia
    // um no-op, porque o picker já afirmava estar lá (reportado no smoke de 2026-08-02).
    let dd = Dropdown::new(id, "", opts)
        .selected(selected_row(prop))
        .open(true);

    let panel = dd.popover_rect_clamped(chip, ctx.viewport);
    let content_h = dd.content_height(chip.h);
    let visible_h = panel.h;
    let max_scroll = (content_h - visible_h).max(0.0);
    {
        let store = ctx.host.store_mut();
        store.set_dropdown_popover(id, panel);
        store.set_panel_content_h(id, content_h);
        store.set_panel_visible_h(id, visible_h);
        if store.panel_scroll(id) > max_scroll {
            store.set_panel_scroll(id, max_scroll);
        }
    }
    let scroll = ctx.host.store().panel_scroll(id).clamp(0.0, max_scroll); // CLAMP-OK: 0.0 literal; max_scroll is a non-negative px extent
    let scrollbar_active = matches!(ctx.host.store().scrollbar_drag(), Some(d) if d.panel == id);
    paint_dropdown_popover_scrolled(
        &dd,
        chip,
        panel,
        scroll,
        scrollbar_active,
        ctx.scene,
        ctx.text_system,
        theme,
    );

    // Hit-register só a parte VISÍVEL de cada linha — a barra de rolagem é o alvo do drag.
    let n = ColorToken::ALL.len() + 1;
    let hit_index = ctx.host.hit_index_mut();
    for i in 0..n {
        let r = dd.option_rect_in_scrolled(chip, panel, i, scroll);
        let top = r.y.max(panel.y);
        let bot = (r.y + r.h).min(panel.y + panel.h);
        if bot - top >= 1.0 {
            hit_index.register(
                ids::vector_token_option_id(prop, i),
                Rect::new(r.x, top, r.w, bot - top),
            );
        }
    }
    if content_h > visible_h {
        hit_index.register(
            DROPDOWN_SCROLLBAR_ID,
            ph2d_editor_core::widget::scrollbar_track_rect(panel),
        );
    }
}
