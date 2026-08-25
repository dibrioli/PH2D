//! ⭐ **AS SETAS do Morph** (plano 32 W4) — irmã de [`super::paint_states`] e de
//! [`super::paint_signals`] pelo mesmo corte: ali *que poses esta forma tem* e *o que a faz mudar
//! de pose*; aqui *entre que formas ela transita, e o que dispara cada passagem*.
//!
//! # Por que ela vive na MESMA seção
//!
//! Enio, 2026-08-24: a máquina *"deverá funcionar à seção states do módulo vector"*. E a lei da
//! casa concorda: **o Inspector mostra o que o objecto TEM** (ADR-0166). Um objecto raramente é as
//! duas coisas — uma forma-hospedeiro tem **poses**, um Morph tem **setas** —, então a seção
//! mostra uma ou outra sem que ninguém tenha de escolher uma aba.
//!
//! # Duas linhas por seta, e a segunda é a que evita a mentira
//!
//! A primeira diz **de onde para onde** e traz os dois verbos (percorrer, apagar); a segunda traz
//! a **condição**. É o mesmo corte do `paint_signals`, e pela mesma razão: espremer as duas numa
//! linha só daria um chip de meia dúzia de caracteres, e o artista não conseguiria ler a acção que
//! escolheu.
//!
//! ⚠️ **A condição é um MENU das acções do Input Map, nunca um campo de texto.** Um nome digitado à
//! mão pode não existir, e uma seta que espera uma acção inexistente **nunca dispara** — sem uma
//! palavra na tela a dizer porquê. *Um modelo que aceita o que o painel não mostra produz estado
//! inalcançável.*

use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::widget::{
    Dropdown, DropdownOption, IconButtonStyle, IconGlyph, paint_dropdown_chip, paint_icon_button,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::Spacing;

use crate::ids;
use crate::paint_sections::{BodyCtx, LABEL_COL_W};
use crate::state::{self, MorphStatesState};

/// O lado de cada botão de ícone ao fim da linha da seta.
const ICON_W: f32 = 28.0; // LITERAL-PX-OK: um botão de ícone quadrado na altura da row

impl BodyCtx<'_> {
    /// **As setas desta máquina** — ou a face vazia que diz como desenhar a primeira.
    pub(crate) fn morph_arrow_rows(&mut self, s: &MorphStatesState, y: f32) -> f32 {
        let mut y = self.label_line(tr("panel.vector.morph.arrows"), y);

        // ⭐ **O READOUT: em que forma a máquina está AGORA.**
        //
        // ⚠️ Ele sai da MESMA máquina que escreve o mundo — um readout derivado noutro sítio diria
        // uma forma e a cena mostraria outra. É o argumento, palavra por palavra, do `live` da
        // seção de poses.
        if let Some(cur) = s.current.as_ref() {
            y = self.label_line(&format!("{} {cur}", tr("panel.vector.morph.current")), y);
        }

        // ⭐ **A FACE VAZIA — e ela diz o GESTO, não só a ausência.**
        //
        // ⚠️ Sem esta linha o artista vê um cabeçalho e nada por baixo, e *"não há setas"* e
        // *"esta janela está partida"* leem-se igual. A frase nomeia o pill e o movimento da mão.
        if s.rows.is_empty() {
            return self.label_line(tr("panel.vector.morph.arrows.empty"), y);
        }

        let shown = s.rows.len().min(ids::MAX_MORPH_ARROWS);
        for (i, row) in s.rows.iter().enumerate().take(shown) {
            y = self.arrow_head_row(i, row, y);
            y = self.arrow_when_row(i, row, &s.actions, y);
        }
        // ⚠️ **Acima do tecto a lista DIZ o que ficou de fora**, em vez de o esconder: o grafo
        // continua a funcionar (a máquina percorre todas as setas), e é o painel que não as mostra.
        // Um corte silencioso far-lhe-ia procurar uma seta que ele tem a certeza de ter desenhado.
        if s.rows.len() > shown {
            y = self.label_line(
                &format!(
                    "{} {}",
                    s.rows.len() - shown,
                    tr("panel.vector.morph.beyond")
                ),
                y,
            );
        }
        y
    }

    /// A linha de CIMA: `de -> para`, o botão de percorrer e o de apagar.
    fn arrow_head_row(&mut self, i: usize, row: &state::MorphArrowRow, y: f32) -> f32 {
        let gap = Spacing::Xs.px();
        let label_w = (self.inner_w - ICON_W - gap).max(0.0);
        // ⛔ **`->` em ASCII, e não a seta tipográfica.** A fonte da casa não cobre o bloco de
        // setas do Unicode (U+2190..U+21FF) e o glifo sairia como uma caixa vazia — há gate a
        // varrer isto (`no_tofu_glyphs`), e ele já mordeu três vezes neste repo.
        self.label_line_in(
            &format!("{} -> {}", row.from, row.to),
            Rect::new(self.inner_x, y, label_w, self.row_h),
        );

        // ⛔ **A seta NÃO tem botão de «percorrer» nesta wave, e a ausência é deliberada:** o que
        // ele faria é pôr a máquina VIVA a andar, e a máquina viva nasce na W5. Um botão pintado
        // antes disso seria um clique que não faz nada — *é assim que o artista aprende a não
        // confiar nos botões desta seção*, e é a lei que a própria seção de poses já escreve
        // ("Show e Clear só existem depois do Rec").
        let _ = row.live;
        let del = ids::morph_arrow_delete_id(i);
        let del_rect = Rect::new(self.inner_x + label_w + gap, y, ICON_W, self.row_h);
        self.hit_index.register(del, del_rect);
        paint_icon_button(
            del_rect,
            IconGlyph::Builtin(IconId::Trash),
            IconButtonStyle::Plain,
            self.store.button_visual(del),
            self.scene,
            self.theme,
        );
        y + self.row_h + gap
    }

    /// A linha de BAIXO: a CONDIÇÃO, num menu das acções do Input Map.
    fn arrow_when_row(
        &mut self,
        i: usize,
        row: &state::MorphArrowRow,
        actions: &[String],
        y: f32,
    ) -> f32 {
        let gap = Spacing::Xs.px();
        self.label_line_in(
            tr("panel.vector.morph.when"),
            Rect::new(self.inner_x, y, LABEL_COL_W, self.row_h),
        );
        let chip = Rect::new(
            self.inner_x + LABEL_COL_W + gap,
            y,
            (self.inner_w - LABEL_COL_W - gap).max(1.0),
            self.row_h,
        );
        let id = ids::morph_arrow_when_id(i);
        let open = matches!(
            self.store.get(id),
            Some(InteractiveState::Dropdown { open: true, .. })
        );
        // ⚠️ **Vazio mostra o traço, e não uma string vazia.** Uma célula em branco lê-se como um
        // controlo por carregar; o traço diz *"sem condição, de propósito"*.
        let shown = if row.when.is_empty() {
            tr("panel.vector.morph.when.none")
        } else {
            row.when.as_str()
        };
        let dd = Dropdown::new(id, "", vec![DropdownOption::new(id, (), shown)])
            .selected(())
            .open(open)
            .visual(self.store.dropdown_visual(id));
        paint_dropdown_chip(&dd, chip, self.scene, self.text_system, self.theme);
        self.hit_index.register(id, chip);
        if open {
            let _ = actions;
            state::set_pending_morph_when_dd(Some((i, chip)));
        }
        y + self.row_h + gap
    }
}

/// **O menu das acções do Input Map** para a condição da seta `row` — pintado no passe DIFERIDO,
/// por cima de todas as seções. Espelho exacto do `paint_filters_blend::paint_blend_popover`.
///
/// ⚠️ **A opção `0` é o «—»** (sem condição): tirar a condição tem de ser um gesto, senão o artista
/// só poderia apagar a seta inteira para se arrepender.
///
/// ⚠️ **As acções saem da lista PUBLICADA**, nunca de uma leitura do painel: elas são conteúdo
/// autorado do projecto, e uma segunda leitura aqui envelheceria no dia em que ele criasse uma.
pub(crate) fn paint_when_popover(
    ctx: &mut ph2d_editor_core::panel::PaintCtx,
    row: usize,
    chip: Rect,
    theme: ph2d_tokens::Theme,
) {
    use ph2d_editor_core::widget::{
        DROPDOWN_SCROLLBAR_ID, paint_dropdown_popover_scrolled, scrollbar_is_needed,
        scrollbar_track_rect,
    };
    let Some(s) = state::morph_states_state() else {
        return;
    };
    let Some(arrow) = s.rows.get(row) else {
        return;
    };
    let id = ids::morph_arrow_when_id(row);
    // O «—» à frente, e depois as acções — ATÉ ao pool de ids que o `populate` registou.
    let mut labels: Vec<&str> = vec![tr("panel.vector.morph.when.none")];
    labels.extend(
        s.actions
            .iter()
            .map(String::as_str)
            .take(ids::MAX_MORPH_ACTIONS - 1),
    );
    let sel = labels.iter().position(|l| *l == arrow.when).unwrap_or(0);
    let options: Vec<DropdownOption<usize>> = labels
        .iter()
        .enumerate()
        .map(|(i, l)| DropdownOption::new(ids::morph_arrow_when_option_id(row, i), i, *l))
        .collect();
    let dd = Dropdown::new(id, "", options).selected(sel).open(true);

    let panel = dd.popover_rect_clamped(chip, ctx.layout.popover_region());
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
    paint_dropdown_popover_scrolled(
        &dd,
        chip,
        panel,
        scroll,
        ctx.host
            .store()
            .scrollbar_visual_for(DROPDOWN_SCROLLBAR_ID, Some(id)),
        ctx.scene,
        ctx.text_system,
        theme,
    );
    // Hit-register só a parte VISÍVEL de cada linha — a barra de rolagem é o alvo do arrasto.
    let hit_index = ctx.host.hit_index_mut();
    for i in 0..labels.len() {
        let r = dd.option_rect_in_scrolled(chip, panel, i, scroll);
        let top = r.y.max(panel.y);
        let bot = (r.y + r.h).min(panel.y + panel.h);
        if bot - top >= 1.0 {
            hit_index.register(
                ids::morph_arrow_when_option_id(row, i),
                Rect::new(r.x, top, r.w, bot - top),
            );
        }
    }
    if scrollbar_is_needed(content_h, visible_h) {
        ctx.host
            .hit_index_mut()
            .register(DROPDOWN_SCROLLBAR_ID, scrollbar_track_rect(panel));
    }
}
