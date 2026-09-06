//! ⭐⭐⭐ **A LISTA DA PILHA DE APARÊNCIA** — N preenchimentos e N contornos numa forma (estudo 42
//! item 4, v20). Módulo irmão do [`crate::paint_appearance`] pelo tecto de 600 LOC do painel, e o
//! corte é por RESPONSABILIDADE: ali moram as duas propriedades do OBJECTO; aqui, a pilha que está
//! **por cima** do chão dele.
//!
//! # ⚠️ O TOPO EM CIMA, e a inversão vive num sítio só
//!
//! O documento guarda a pilha do chão para o topo (é a ordem de desenho); o painel mostra-a ao
//! contrário, como o *Appearance panel* do Illustrator — o que está à frente lê-se primeiro. A
//! conversão é uma linha (`n - 1 - k`) e **não se repete**: com duas, o artista carrega em «subir»
//! e a camada desce.
//!
//! # A linha, e por que ela tem estes cinco controlos
//!
//! `[olho] Rótulo [swatch] [↑] [↓] [✕]` — e clicar no RÓTULO abre a camada, mostrando por baixo a
//! largura (num contorno), a opacidade e a mistura DELA. ⛔ Pôr as três em toda linha encheria a
//! seção com três vezes mais controlos do que o artista olha, e é o que o Illustrator resolve com
//! o mesmo gesto (o triângulo de expandir).

use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::widget::{
    Button, ColorSwatch, Dropdown, DropdownOption, IconButtonStyle, IconGlyph, SwatchSize,
    paint_button, paint_color_swatch, paint_dropdown_chip, paint_icon_button,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::Spacing;
use ph2d_vec_render::blend;

use crate::ids;
use crate::paint_sections::BodyCtx;
use crate::state;
use crate::state::PaintRow;

/// Um botão de ícone quadrado na altura da row — o mesmo da fileira do morph.
const ICON_W: f32 = 28.0; // LITERAL-PX-OK: um botão de ícone quadrado na altura da row

impl BodyCtx<'_> {
    /// **A pilha inteira**, do topo para o chão, mais os dois botões que a fazem crescer.
    pub(crate) fn paint_stack(&mut self, layers: &[PaintRow], mut y: f32) -> f32 {
        let n = layers.len();
        for k in 0..n {
            // ⚠️ **A inversão, e ela mora AQUI e em mais lado nenhum**: `i` é o índice do
            // DOCUMENTO (chão → topo) e é o que viaja no clique; `k` é a linha pintada.
            let i = n - 1 - k;
            y = self.layer_row(i, &layers[i], y);
            if state::open_layer() == Some(i) {
                y = self.layer_props(&layers[i], y);
            }
        }
        self.add_buttons(y)
    }

    /// Uma linha da pilha.
    fn layer_row(&mut self, i: usize, row: &PaintRow, y: f32) -> f32 {
        let gap = Spacing::Xs.px();
        let sw = SwatchSize::Sm.px();
        // Da direita para a esquerda: os três verbos, depois a swatch. O que sobra é o rótulo.
        let mut x = self.inner_x + self.inner_w;
        for (id, icon) in [
            (ids::vector_paint_del_id(i), IconId::Trash),
            (ids::vector_paint_down_id(i), IconId::ChevronDown),
            (ids::vector_paint_up_id(i), IconId::ChevronUp),
        ] {
            x -= ICON_W;
            let r = Rect::new(x, y, ICON_W, self.row_h);
            self.hit_index.register(id, r);
            paint_icon_button(
                r,
                IconGlyph::Builtin(icon),
                IconButtonStyle::Plain,
                self.store.button_visual(id),
                self.scene,
                self.theme,
            );
            x -= gap;
        }
        x -= sw;
        let sr = Rect::new(x, y, sw, self.row_h);
        let sid = ids::vector_paint_swatch_id(i);
        paint_color_swatch(
            &ColorSwatch::new(sid, "Layer color", row.color).size(SwatchSize::Sm),
            sr,
            self.scene,
            self.theme,
        );
        self.hit_index.register(sid, sr);

        // O OLHO, à esquerda de tudo.
        let eid = ids::vector_paint_eye_id(i);
        let er = Rect::new(self.inner_x, y, ICON_W, self.row_h);
        self.hit_index.register(eid, er);
        paint_icon_button(
            er,
            IconGlyph::Builtin(if row.enabled {
                IconId::Eye
            } else {
                IconId::EyeClosed
            }),
            IconButtonStyle::Plain,
            self.store.button_visual(eid),
            self.scene,
            self.theme,
        );

        // O RÓTULO ocupa o que sobra — e é ele que ABRE a camada.
        let lx = self.inner_x + ICON_W + gap;
        let lw = (x - gap - lx).max(1.0);
        let rid = ids::vector_paint_row_id(i);
        let lr = Rect::new(lx, y, lw, self.row_h);
        self.label_line_in(
            tr(if row.is_fill {
                "panel.vector.paint.fill"
            } else {
                "panel.vector.paint.stroke"
            }),
            lr,
        );
        self.hit_index.register(rid, lr);
        y + self.row_h + self.row_gap
    }

    /// As propriedades da camada ABERTA, recuadas por baixo dela.
    fn layer_props(&mut self, row: &PaintRow, y: f32) -> f32 {
        let mut y = y;
        if !row.is_fill {
            y = self.lone_number_row(tr("panel.vector.paint.width"), ids::VECTOR_PAINT_WIDTH, y);
        }
        // ⭐⭐⭐ **ONDE ela desenha** (v21) — o par `X`/`Y`, na MESMA `number_row` que o Transform e
        // o Vertex usam para as coordenadas deles.
        //
        // ⚠️ **Aparece nas DUAS espécies**, ao contrário da largura: um preenchimento deslocado é
        // precisamente o caso que motivou isto (a sombra dura), e escondê-lo num preenchimento
        // deixaria de fora o pedido que a wave responde.
        y = self.number_row(
            tr("panel.vector.paint.dx"),
            ids::VECTOR_PAINT_DX,
            tr("panel.vector.paint.dy"),
            ids::VECTOR_PAINT_DY,
            y,
        );
        // ⭐⭐⭐ **O OFFSET DE CAD** (v22) — a silhueta cresce (`>0`) ou encolhe (`<0`).
        y = self.lone_number_row(tr("panel.vector.paint.dilate"), ids::VECTOR_PAINT_DILATE, y);
        // ⚠️ **A QUINA só aparece com o offset ARMADO** — é a lei do «nenhum controlo mudo»: com
        // `dilate = 0` não há esquina nenhuma a formar, e três chips que não mudam nada são a
        // definição de um controlo morto sob o dedo.
        if row.dilate != 0.0 {
            y = self.segmented3(
                tr("panel.vector.paint.join"),
                [
                    (ids::VECTOR_PAINT_JOIN_MITER, "Miter", row.dilate_join == 0),
                    (ids::VECTOR_PAINT_JOIN_ROUND, "Round", row.dilate_join == 1),
                    (ids::VECTOR_PAINT_JOIN_BEVEL, "Bevel", row.dilate_join == 2),
                ],
                y,
            );
        }
        let track = self.live_track(ids::VECTOR_PAINT_OPACITY, row.opacity);
        let pct = f64::from(track) * 100.0; // LITERAL-PX-OK: fraction→percent
        #[expect(
            clippy::cast_possible_truncation,
            reason = "o chip mostra a percentagem inteira, como os irmaos dele"
        )]
        let rotulo = format!("{}", pct.round() as i64);
        y = self.slider_row(
            tr("panel.vector.paint.opacity"),
            ids::VECTOR_PAINT_OPACITY,
            ids::VECTOR_PAINT_OPACITY_NUM,
            track,
            pct,
            &rotulo,
            y,
        );
        self.layer_blend_row(row.blend, y)
    }

    /// O chip de mistura da camada aberta. O popover abre no passe DIFERIDO — a lei dos outros
    /// seis desta janela.
    fn layer_blend_row(&mut self, atual: ph2d_vec_scene::BlendMode, y: f32) -> f32 {
        let chip = Rect::new(self.inner_x, y, self.inner_w, self.row_h);
        let open = matches!(
            self.store.get(ids::VECTOR_PAINT_BLEND),
            Some(InteractiveState::Dropdown { open: true, .. })
        );
        let dd = Dropdown::new(
            ids::VECTOR_PAINT_BLEND,
            tr("panel.vector.paint.blend"),
            vec![DropdownOption::new(
                ids::VECTOR_PAINT_BLEND,
                (),
                atual.name(),
            )],
        )
        .selected(())
        .open(open)
        .visual(self.store.dropdown_visual(ids::VECTOR_PAINT_BLEND));
        paint_dropdown_chip(&dd, chip, self.scene, self.text_system, self.theme);
        self.hit_index.register(ids::VECTOR_PAINT_BLEND, chip);
        if open {
            state::set_pending_paint_blend_dd(Some(chip));
        }
        y + self.row_h + self.row_gap
    }

    /// **+ Fill** e **+ Stroke**, lado a lado — uma camada nova nasce no TOPO, que é onde o
    /// artista a espera ver (a lei do Illustrator).
    ///
    /// ⛔ Os dois somem quando a pilha está no tecto ([`ph2d_vec_scene::MAX_PAINT_LAYERS`]): um
    /// botão que consome o clique e não faz nada é pior que um ausente.
    fn add_buttons(&mut self, y: f32) -> f32 {
        if state::current_appearance().is_some_and(|a| a.layers.len() >= MAX_ROWS) {
            return self.hint_line(tr("panel.vector.paint.full"), y);
        }
        let cw = self.half_cell_w();
        let gap = Spacing::Sm.px();
        for (k, (id, label)) in [
            (ids::VECTOR_PAINT_ADD_FILL, "panel.vector.paint.add_fill"),
            (
                ids::VECTOR_PAINT_ADD_STROKE,
                "panel.vector.paint.add_stroke",
            ),
        ]
        .into_iter()
        .enumerate()
        {
            #[expect(clippy::cast_precision_loss, reason = "k in 0..2")]
            let x = self.inner_x + k as f32 * (cw + gap);
            let r = Rect::new(x, y, cw, self.row_h);
            let btn = Button::new(id, tr(label)).visual(self.store.button_visual(id));
            paint_button(&btn, r, self.scene, self.text_system, self.theme);
            self.hit_index.register(id, r);
        }
        y + self.row_h + self.row_gap
    }
}

/// O tecto, lido da folha do documento — nunca um número escrito aqui.
const MAX_ROWS: usize = ph2d_vec_scene::MAX_PAINT_LAYERS;

/// **O popover dos modos de mistura de uma CAMADA** — pintado no passe diferido, por cima de todas
/// as seções. Espelho do irmão do objecto, com o espaço de ids próprio.
pub(crate) fn paint_layer_blend_popover(
    ctx: &mut ph2d_editor_core::panel::PaintCtx,
    chip: Rect,
    theme: ph2d_tokens::Theme,
) {
    use ph2d_editor_core::widget::{
        DROPDOWN_SCROLLBAR_ID, paint_dropdown_popover_scrolled, scrollbar_is_needed,
        scrollbar_track_rect,
    };
    let modos: Vec<ph2d_vec_scene::BlendMode> = blend::offered().collect();
    if modos.is_empty() {
        return;
    }
    let atual = state::current_appearance()
        .and_then(|a| state::open_layer().and_then(|i| a.layers.get(i).map(|r| r.blend)))
        .unwrap_or_default();
    let sel = modos.iter().position(|m| *m == atual).unwrap_or(0);
    let options: Vec<DropdownOption<usize>> = modos
        .iter()
        .enumerate()
        .map(|(i, m)| DropdownOption::new(ids::vector_paint_blend_option_id(i), i, m.name()))
        .collect();
    let dd = Dropdown::new(ids::VECTOR_PAINT_BLEND, "", options)
        .selected(sel)
        .open(true);

    let panel = dd.popover_rect_clamped(chip, ctx.layout.popover_region());
    let content_h = dd.content_height(chip.h);
    let max_scroll = (content_h - panel.h).max(0.0);
    {
        let store = ctx.host.store_mut();
        store.set_dropdown_popover(ids::VECTOR_PAINT_BLEND, panel);
        store.set_panel_content_h(ids::VECTOR_PAINT_BLEND, content_h);
        store.set_panel_visible_h(ids::VECTOR_PAINT_BLEND, panel.h);
        if store.panel_scroll(ids::VECTOR_PAINT_BLEND) > max_scroll {
            store.set_panel_scroll(ids::VECTOR_PAINT_BLEND, max_scroll);
        }
    }
    let scroll = ctx
        .host
        .store()
        .panel_scroll(ids::VECTOR_PAINT_BLEND)
        .clamp(0.0, max_scroll); // CLAMP-OK: 0.0 literal; max_scroll is a non-negative px extent
    paint_dropdown_popover_scrolled(
        &dd,
        chip,
        panel,
        scroll,
        ctx.host
            .store()
            .scrollbar_visual_for(DROPDOWN_SCROLLBAR_ID, Some(ids::VECTOR_PAINT_BLEND)),
        ctx.scene,
        ctx.text_system,
        theme,
    );
    let hit_index = ctx.host.hit_index_mut();
    for i in 0..modos.len() {
        let r = dd.option_rect_in_scrolled(chip, panel, i, scroll);
        let top = r.y.max(panel.y);
        let bot = (r.y + r.h).min(panel.y + panel.h);
        if bot - top >= 1.0 {
            hit_index.register(
                ids::vector_paint_blend_option_id(i),
                Rect::new(r.x, top, r.w, bot - top),
            );
        }
    }
    if scrollbar_is_needed(content_h, panel.h) {
        ctx.host
            .hit_index_mut()
            .register(DROPDOWN_SCROLLBAR_ID, scrollbar_track_rect(panel));
    }
}
