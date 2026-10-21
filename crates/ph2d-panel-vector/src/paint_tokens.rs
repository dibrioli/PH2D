//! **A row de TOKEN** — o chip que diz de que token esta propriedade segue, e o popover que o
//! escolhe (plano UI/UX §4/W4).
//!
//! ⚠️ **Ela mora DENTRO das seções Fill e Stroke, ao lado da swatch**, e não numa seção própria.
//! É essa adjacência que responde à pior pergunta que a feature pode gerar — *"por que a cor que
//! eu escolhi não aparece?"* —, porque a resposta fica no mesmo lugar em que a pergunta nasce.
//! Uma seção "Tokens" à parte deixaria a swatch mentindo sozinha lá em cima.

use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{Dropdown, DropdownOption};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, Theme};

use crate::ids;
use crate::paint_sections::{BodyCtx, label_col_w};
use crate::state;

/// O rótulo que o chip mostra: a chave do token, ou o travessão de *não preso*.
fn chip_label(bound: Option<&str>) -> &str {
    bound.unwrap_or("—")
}

/// **Que tabela este slot lista** — a porta ÚNICA de *"que lista o picker mostra?"*.
///
/// ⚠️ Um código desconhecido cai na tabela de COR em vez de entrar em pânico: isto corre no laço
/// de PINTURA, e derrubar a janela por um id que não existe seria trocar um controle inerte por um
/// app fechado. Quem impede o caso é o gate, não um `unwrap`.
fn table_of(prop: u16) -> ids::TokenTable {
    ids::token_slot(prop).map_or(ids::TokenTable::Colour, |s| s.table)
}

/// As opções do popover: **Unbind primeiro**, depois os tokens na ordem da tabela DO SLOT.
///
/// ⚠️ A lista vem do `ph2d-tokens` pela [`ids::TokenTable`], e nunca de uma cópia escrita aqui: um
/// token novo entra na macro e aparece no picker de graça. Uma segunda lista nasceria desatualizada
/// no primeiro token acrescentado — e o modo de falha é o artista não conseguir escolher um valor
/// que existe.
fn options(prop: u16) -> Vec<DropdownOption<usize>> {
    let table = table_of(prop);
    std::iter::once(DropdownOption::new(
        ids::vector_token_option_id(prop, 0),
        0,
        tr("panel.vector.token.none"),
    ))
    .chain((0..table.len()).filter_map(move |i| {
        Some(DropdownOption::new(
            ids::vector_token_option_id(prop, i + 1),
            i + 1,
            table.key(i)?,
        ))
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

    /// A fileira `<rótulo> [token ▾]` de um SLOT. Devolve o `y` seguinte.
    ///
    /// Mesmo desenho da fileira de Blend de um filtro — a mesma estética para a mesma pergunta
    /// *"qual destes?"*.
    ///
    /// ⚠️ **Toma o CHIP e DERIVA o código**, em vez de receber o par: os dois viajavam soltos e
    /// nada impedia que uma row nova pintasse o chip de um slot com o código de outro — o popover
    /// abriria a lista errada, e a escolha do artista pousaria na propriedade errada. Derivado da
    /// tabela, o descasamento deixa de ser exprimível.
    ///
    /// ⚠️ Um chip que não é de slot nenhum não pinta nada e devolve o `y` intacto — pelo mesmo
    /// motivo do [`table_of`]: isto corre no laço de pintura, e um `unwrap` trocaria um controle
    /// ausente por uma janela fechada.
    pub(crate) fn token_row(&mut self, id: ph2d_a11y::NodeId, bound: Option<&str>, y: f32) -> f32 {
        let Some(slot) = ids::token_slot_of(id) else {
            return y;
        };
        ph2d_editor_core::paint::paint_text(
            self.text_system,
            self.scene,
            tr("panel.vector.token"),
            self.inner_x,
            y + (self.row_h - self.font) * 0.5,
            self.font,
            label_col_w(self.inner_x, self.inner_w),
            ph2d_editor_core::paint::resolve(ColorToken::Text1, self.theme),
        );
        let chip = ph2d_editor_core::panel::value_col(self.inner_x, self.inner_w, y, self.row_h);
        if let Some(chip) = self.dropdown_row(id, "", chip_label(bound), chip) {
            state::set_pending_token_dd(Some((slot.code, chip)));
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
    b.of_slot(prop)
        .and_then(|k| table_of(prop).position(&k))
        .map_or(0, |i| i + 1)
}

/// **O popover dos tokens** — pintado no passe DIFERIDO, por cima de todas as seções.
///
/// Espelho exato do popover de mistura dos filtros, e pelo mesmo motivo: são oitenta linhas, e o
/// card tem de rolar e ficar acima do scroll da seção.
///
/// ⚠️ **A região do clamp é a BANDA DE CHROME, nunca a janela** ([`HeroLayout::popover_region`]).
/// Com oitenta e uma linhas nada cabe em lado nenhum, então o clamp escolhe *o lado com mais
/// espaço* — e contra a JANELA isso encostava a lista na borda de cima: a linha de SOLTAR nascia a
/// 2 px do topo, por cima da barra de ferramentas e 64 px acima do painel dela (reportado no smoke
/// de 2026-08-02: *"selecionar None só funciona se clicar na parte de baixo do nome"*).
pub(crate) fn paint_token_popover(ctx: &mut PaintCtx, prop: u16, chip: Rect, theme: Theme) {
    // ⚠️ O chip sai da MESMA tabela que nomeia as opções — um `if prop == 0 {…} else {…}` aqui
    // mandaria o popover do terceiro slot escrever o estado de rolagem do segundo.
    let Some(id) = ids::token_slot(prop).map(|s| s.chip) else {
        return;
    };
    let opts = options(prop);
    // ⚠️ A linha DESTACADA é a do token vigente, nunca um índice cravado. Com `0` cravado o
    // popover dizia "None" mesmo sobre uma forma bindada — e então escolher None de volta parecia
    // um no-op, porque o picker já afirmava estar lá (reportado no smoke de 2026-08-02).
    let dd = Dropdown::new(id, "", opts)
        .selected(selected_row(prop))
        .open(true);

    let region = ctx.layout.popover_region();
    ph2d_editor_core::panel::paint_dropdown_list(ctx, &dd, chip, region, theme, None);
}
