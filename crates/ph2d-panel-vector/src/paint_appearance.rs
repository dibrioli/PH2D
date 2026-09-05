//! ⭐⭐⭐ **A seção APPEARANCE** — a opacidade e o modo de mistura do OBJECTO (estudo 42 item 2, v19
//! do schema).
//!
//! É o que o Illustrator põe no painel *Transparency* e o Figma na fileira de baixo do *Fill*: as
//! duas propriedades que descrevem **a forma inteira** em vez de uma tinta dela.
//!
//! # ⚠️ Por que ela não é uma row da seção *Fill*
//!
//! Aquela seção já tem uma row `Opacity`, e ela é **outra coisa**: o alfa da TINTA que a ferramenta
//! tem na mão. As duas convivem em todo editor sério, e a diferença vê-se onde a forma desenha mais
//! de uma marca — meia-opacidade no preenchimento **e** no traço deixa o traço transparecer sobre o
//! preenchimento; meia-opacidade no OBJECTO compõe a forma uma vez e desvanece o resultado.
//!
//! ⇒ Seção própria, com o nome que o artista procura, e as duas rows a viver onde o sujeito delas
//! vive.
//!
//! # A lista de modos é DERIVADA
//!
//! ⛔ O dropdown **não** escreve a lista: ele percorre a [`ph2d_vec_render::blend::offered`], que é
//! o vocabulário do app menos os modos que o Vello não exprime. *Um painel derivado de uma tabela
//! não tem onde esconder um knob morto* — e aqui o morto seria o pior tipo: um modo que grava no
//! documento e desenha `Normal`.

use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{
    DROPDOWN_SCROLLBAR_ID, Dropdown, DropdownOption, paint_dropdown_chip,
    paint_dropdown_popover_scrolled, scrollbar_is_needed, scrollbar_track_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::Theme;
use ph2d_vec_render::blend;

use crate::ids;
use crate::paint_sections::BodyCtx;
use crate::state;

/// O rótulo de um modo, como o artista o lê (a tabela do vocabulário, não uma cópia).
fn nome(m: ph2d_vec_scene::BlendMode) -> &'static str {
    m.name()
}

/// Em que linha da lista OFERECIDA este modo está.
///
/// ⚠️ **`0` para um modo sem tradução** — um documento pode trazer um modo que este renderer não
/// exprime (o Painter tem-nos), e o chip mostraria uma linha que não existe. `0` é `Normal`, que é
/// exactamente o que o desenho faz nesse caso: *o chip diz o que se vê*.
fn linha_de(m: ph2d_vec_scene::BlendMode) -> usize {
    blend::offered().position(|o| o == m).unwrap_or(0)
}

impl BodyCtx<'_> {
    /// **A seção APPEARANCE** — some inteira sem forma selecionada.
    pub(crate) fn appearance_section(&mut self, y: f32) -> f32 {
        let Some(a) = state::current_appearance() else {
            return y;
        };
        let (mut y, collapsed) = self.section_header(
            ids::VECTOR_SECTION_APPEARANCE,
            tr("panel.vector.section.appearance"),
            y,
        );
        if collapsed {
            return y;
        }
        // ⚠️ **O valor VIVO ganha do documento enquanto o dedo arrasta** (`live_track`, a mesma
        // porta das outras rows desta janela): sem isso o slider saltaria de volta a cada quadro,
        // porque a shell republica o documento e o documento só muda quando o arrasto termina.
        let track = self.live_track(ids::VECTOR_OBJ_OPACITY, a.opacity);
        let pct = f64::from(track) * 100.0; // LITERAL-PX-OK: fraction→percent for the opacity chip
        y = self.slider_row(
            tr("panel.vector.appearance.opacity"),
            ids::VECTOR_OBJ_OPACITY,
            ids::VECTOR_OBJ_OPACITY_NUM,
            track,
            pct,
            &format!("{}", pct.round() as i64),
            y,
        );
        let y = self.blend_row(a.blend, y);
        // ⭐⭐⭐ **E A PILHA** (v20) — as camadas por cima do chão, do topo para baixo.
        //
        // ⚠️ **Depois das duas do OBJECTO de propósito:** elas descrevem como a forma INTEIRA se
        // compõe com o que está atrás dela, e a pilha é o que está DENTRO — ler de cima para baixo
        // é ler de fora para dentro.
        self.paint_stack(&a.layers, y)
    }

    /// O chip do modo de mistura. Abre o popover no passe DIFERIDO — sem ele o `push_clip` do
    /// scroll da seção cortaria a lista na borda (a lei dos outros cinco popovers desta janela).
    fn blend_row(&mut self, atual: ph2d_vec_scene::BlendMode, y: f32) -> f32 {
        let chip = Rect::new(self.inner_x, y, self.inner_w, self.row_h);
        let open = matches!(
            self.store.get(ids::VECTOR_OBJ_BLEND),
            Some(InteractiveState::Dropdown { open: true, .. })
        );
        let visual = self.store.dropdown_visual(ids::VECTOR_OBJ_BLEND);
        let dd = Dropdown::new(
            ids::VECTOR_OBJ_BLEND,
            tr("panel.vector.appearance.blend"),
            vec![DropdownOption::new(ids::VECTOR_OBJ_BLEND, (), nome(atual))],
        )
        .selected(())
        .open(open)
        .visual(visual);
        paint_dropdown_chip(&dd, chip, self.scene, self.text_system, self.theme);
        self.hit_index.register(ids::VECTOR_OBJ_BLEND, chip);
        if open {
            state::set_pending_obj_blend_dd(Some(chip));
        }
        y + self.row_h + self.row_gap
    }
}

/// **O popover dos modos de mistura** — pintado no passe diferido do `paint.rs`, por cima de todas
/// as seções. Espelho exacto do `paint_filters_blend::paint_blend_popover`, com uma diferença: a
/// lista sai da porta que TRADUZ, e não de uma tabela publicada.
pub(crate) fn paint_blend_popover(ctx: &mut PaintCtx, chip: Rect, theme: Theme) {
    let modos: Vec<ph2d_vec_scene::BlendMode> = blend::offered().collect();
    if modos.is_empty() {
        return;
    }
    let sel = state::current_appearance().map_or(0, |a| linha_de(a.blend));
    let options: Vec<DropdownOption<usize>> = modos
        .iter()
        .enumerate()
        .map(|(i, m)| DropdownOption::new(ids::vector_obj_blend_option_id(i), i, nome(*m)))
        .collect();
    let dd = Dropdown::new(ids::VECTOR_OBJ_BLEND, "", options)
        .selected(sel)
        .open(true);

    let panel = dd.popover_rect_clamped(chip, ctx.layout.popover_region());
    let content_h = dd.content_height(chip.h);
    let visible_h = panel.h;
    let max_scroll = (content_h - visible_h).max(0.0);
    {
        let store = ctx.host.store_mut();
        store.set_dropdown_popover(ids::VECTOR_OBJ_BLEND, panel);
        store.set_panel_content_h(ids::VECTOR_OBJ_BLEND, content_h);
        store.set_panel_visible_h(ids::VECTOR_OBJ_BLEND, visible_h);
        if store.panel_scroll(ids::VECTOR_OBJ_BLEND) > max_scroll {
            store.set_panel_scroll(ids::VECTOR_OBJ_BLEND, max_scroll);
        }
    }
    let scroll = ctx
        .host
        .store()
        .panel_scroll(ids::VECTOR_OBJ_BLEND)
        .clamp(0.0, max_scroll); // CLAMP-OK: 0.0 literal; max_scroll is a non-negative px extent
    paint_dropdown_popover_scrolled(
        &dd,
        chip,
        panel,
        scroll,
        ctx.host
            .store()
            .scrollbar_visual_for(DROPDOWN_SCROLLBAR_ID, Some(ids::VECTOR_OBJ_BLEND)),
        ctx.scene,
        ctx.text_system,
        theme,
    );

    // Hit-register só a parte VISÍVEL de cada linha — a mesma lei do popover irmão.
    let hit_index = ctx.host.hit_index_mut();
    for i in 0..modos.len() {
        let r = dd.option_rect_in_scrolled(chip, panel, i, scroll);
        let top = r.y.max(panel.y);
        let bot = (r.y + r.h).min(panel.y + panel.h);
        if bot - top >= 1.0 {
            hit_index.register(
                ids::vector_obj_blend_option_id(i),
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
