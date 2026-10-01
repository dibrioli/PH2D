//! Showcase orchestrator: paint_showcase_body.
//!
//! Extracted from showcase/mod.rs in Wave 6+7 Phase 1.C.

use super::*;

/// O corpo da galeria ABERTO — o recorte da PORTA da rolagem continua de pé até ele ser fechado
/// (`crate::panel::scroll_area::close_showcase_body` ou [`close_showcase_body_with`]).
///
/// ⚠️ Existe porque quem pinta a galeria só tem o `store` IMUTÁVEL (`store_and_hit_index_mut`), e
/// fechar a porta publica as alturas e o dono da barra — ⇒ abre-se aqui, fecha-se lá fora.
#[must_use = "o corpo da galeria tem de ser fechado — senão o recorte fica aberto"]
pub struct ShowcaseBody {
    area: crate::widget::scroll_area::ScrollArea,
    content_h: f32,
    rect: Rect,
}

impl ShowcaseBody {
    /// As três partes, para quem fecha com um `PaintCtx` (`crate::panel::scroll_area`).
    #[must_use = "a área aberta tem de ser fechada, senão os recortes ficam na pilha"]
    pub fn into_parts(self) -> (crate::widget::scroll_area::ScrollArea, f32, Rect) {
        (self.area, self.content_h, self.rect)
    }
}

/// Como uma secção da galeria se pinta — o tema que recebe é o DELA.
type PintaSeccao =
    fn(&mut VectorScene, &mut TextSystem, Theme, &mut HitIndex, &WidgetStore, f32, f32, f32) -> f32;

/// ⭐ **As secções da galeria, pela ordem NATURAL** — o par `(id, pintor)`. ⚠️ Os ids são os de
/// [`SECTION_IDS`], pela mesma ordem (há gate a amarrá-los).
pub(crate) const SECCOES: [(NodeId, PintaSeccao); 11] = [
    (ids::INSP_SECTION_INPUTS, paint_inputs_section),
    (ids::INSP_SECTION_SLIDER, paint_slider_section),
    (ids::INSP_SECTION_SWITCHES, paint_switches_section),
    (ids::INSP_SECTION_LISTS, paint_lists_section),
    (ids::INSP_SECTION_VECTOR, paint_vector_section),
    (ids::INSP_SECTION_STATUS, paint_status_section),
    (ids::INSP_SECTION_COLOR, paint_color_section),
    (ids::INSP_SECTION_ACTIONS, paint_actions_section),
    (ids::INSP_SECTION_IDENTITY, paint_identity_section),
    (ids::INSP_SECTION_CARD, paint_card_section),
    (ids::INSP_SECTION_W6, paint_inspector_w6_section),
];

pub fn paint_showcase_body(
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
) -> ShowcaseBody {
    crate::widget::panel_chrome::paint_panel_surface_floating(rect, scene, theme);
    // Drag pill + resize gripper hit zones. Visuals are inside
    // `paint_panel_surface` / `paint_panel_corner_dot`; we register
    // the hits here against the gallery's own NodeIds so the
    // BlenderHit dispatch (`DragHandle` / `ResizeHandle`) drives
    // `GAL_PANEL` independently of the Inspector.
    let drag_handle_rect = panel_drag_handle_rect(
        rect,
        crate::widget::panel_chrome::PANEL_HEADER_H_DEFAULT,
        crate::widget::panel_chrome::PANEL_HEADER_CLOSE_RESERVE,
    );
    let resize_handle_rect = panel_resize_handle_rect(rect);
    let resize_handle_bl_rect = crate::widget::panel_chrome::panel_resize_handle_rect_bl(rect);
    hit_index.register(ids::GAL_DRAG_HANDLE, drag_handle_rect);
    hit_index.register(ids::GAL_RESIZE_HANDLE, resize_handle_rect);
    hit_index.register(ids::GAL_RESIZE_HANDLE_BL, resize_handle_bl_rect);

    // Header: title + subtitle + divider. Canonical panel title
    // (single source of truth — `panel_chrome::paint_panel_title`);
    // reserve ≈ICON_BTN_SIZE on the right for the Close button.
    let title_y = rect.y + PANEL_TITLE_BASELINE;
    let title_size = paint_panel_title(rect, "Widget Gallery", 40.0, scene, text_system, theme); // LITERAL-PX-OK: Close-button reserve
    // ⭐⭐ **A legenda QUEBRA, nunca é cortada** (2026-09-19): ela mede `290,76` px e a coluna do
    //    cabeçalho dá `268`, logo saía `…peripheral…`. *Uma frase é PROSA: ela desce uma linha;
    //    quem se elide é um rótulo, que tem uma coluna.* É a mesma cura que a frase de estado
    //    vazio do Inspector e a do painel de Tags levaram em 18/09.
    //
    // ⚠️ E a altura VOLTA: o risco por baixo, o corpo e o recorte saem todos do `div_y`, logo
    //    quebrar sem devolver a altura escreveria a 2.ª linha por cima do risco.
    let subtitle_h = crate::paint::paint_text_block(
        text_system,
        scene,
        "Canonical widget showcase \u{00b7} reference for peripheral agents",
        rect.x + PANEL_HEAD_PAD,
        title_y + title_size + Spacing::Xs.px(),
        TypeToken::Xs.px() - 1.0,
        rect.w - PANEL_HEAD_PAD * 2.0,
        resolve(ColorToken::Text3, theme),
    );
    // Close (X) at top-right of the header strip.
    let close_size = Spacing::Xl2.px();
    let close_rect = Rect::new(
        rect.x + rect.w - PANEL_HEAD_PAD - close_size,
        title_y - 2.0,
        close_size,
        close_size,
    );
    hit_index.register(ids::GAL_CLOSE, close_rect);
    crate::paint::paint_icon(
        scene,
        IconId::Close,
        close_rect,
        resolve(ColorToken::Text2, theme),
        StrokeToken::Default.px(),
    );

    let div_y = title_y + title_size + subtitle_h + Spacing::Xl.px();
    let div = Rect::new(
        rect.x + PANEL_HEAD_PAD,
        div_y,
        rect.w - PANEL_HEAD_PAD * 2.0,
        1.0,
    );
    scene.fill_rect(rect_to_vello(div), resolve(ColorToken::Border, theme));

    // Body clipped to panel rect with wheel-driven scroll offset
    // routed through `GAL_PANEL` (independent of `INSP_PANEL`).
    // Reserve room for the scrollbar even when it isn't visible so
    // the section content width is stable.
    // ⭐ Pela PORTA da rolagem (spec `04_a_rolagem_unica`): o desenho E o clique recortados.
    let content_top = div_y + Spacing::Sm.px();
    let content_bottom = rect.y + rect.h - Spacing::Xs.px();
    let visible_h = (content_bottom - content_top).max(0.0);
    let area = crate::widget::scroll_area::open_with(
        scene,
        hit_index,
        store,
        ids::GAL_PANEL,
        crate::widget::GALLERY_SCROLLBAR_ID,
        Rect::new(rect.x, content_top, rect.w, visible_h),
    );

    let inner_x = rect.x + BODY_PAD;
    let scrollbar_reserve = crate::widget::SCROLLBAR_W + Spacing::Sm.px();
    let inner_w = (rect.w - BODY_PAD * 2.0 - scrollbar_reserve).max(0.0);
    let body_top_y = area.top() + Spacing::Xs.px();
    let mut y = body_top_y;
    // ⭐⭐ **A galeria também mostra o CARTÃO** — ela é a fonte de verdade do cromo (DIRETRIZ
    //    §5.2), logo um risco aqui seria a galeria a ensinar o que o app já não faz.
    crate::widget::section_cards::begin_section_cards(scene, theme, body_top_y);
    // ⭐ As notas — lidas uma vez. Cada uma pinta-se no FIM da secção a que pertence (pela
    //    identidade dela, `NoteData::section`, desde 2026-09-30); as de nenhuma secção da galeria
    //    pintam-se no fim do corpo.
    let all_notes = store.notes_for_panel(ids::GAL_PANEL).to_vec();
    macro_rules! paint_pending_notes {
        ($section_id:expr) => {
            for (slot, note) in all_notes.iter().enumerate() {
                if note.section == Some($section_id) {
                    paint_one_note(
                        scene,
                        text_system,
                        hit_index,
                        store,
                        inner_x,
                        inner_w,
                        &mut y,
                        note,
                        &crate::ids::GAL_NOTES,
                        slot,
                    );
                }
            }
        };
    }
    // ⭐⭐⭐ **O corpo é uma LISTA** (ordem do dono, 2026-09-30: *«siga com os outros painéis»*) —
    //    pintado pela ordem que o artista escolheu, cada secção no tema que ele lhe deu, com a marca
    //    de queda e o fantasma da lei partilhada ([`crate::widget::section_plan`]).
    //
    // Por cada secção: a secção, depois o contorno colorido (se o utilizador escolheu um pelo botão
    // direito → "Section outline"), depois as notas presas a ELA (no fim da secção, ANTES do fecho
    // do cartão — cânone de 2026-05-24: as notas pertencem VISUALMENTE à secção onde nasceram).
    let ordem = crate::widget::section_plan::ordem(&SECTION_IDS, store);
    let arrastada = store.section_drag().filter(|d| d.active).map(|d| d.section);
    let mut faixas: Vec<crate::widget::section_plan::Faixa> = Vec::with_capacity(ordem.len());
    let mut fantasma: Option<VectorScene> = None;
    let mut corredor = crate::widget::section_plan::Corredor::default();
    for section_id in ordem {
        let Some(&(_, pinta)) = SECCOES.iter().find(|(id, _)| *id == section_id) else {
            continue;
        };
        let tema = crate::widget::section_plan::tema_da_seccao(store, section_id, theme);
        y = corredor.antes(scene, theme, inner_x, inner_w, y);
        let y_before = y;
        let mut parte = VectorScene::new();
        let a_parte = arrastada == Some(section_id);
        if a_parte {
            std::mem::swap(scene, &mut parte);
        }
        let new_y = pinta(
            scene,
            text_system,
            tema,
            hit_index,
            store,
            inner_x,
            inner_w,
            y,
        );
        if let Some(color_idx) = store.section_outline_color(section_id) {
            let rgba = crate::widget::panel_chrome::highlighter_rgba(color_idx);
            let pad = Spacing::Xs.px();
            let block = Rect::new(
                inner_x - pad,
                y_before - pad,
                inner_w + pad * 2.0,
                (new_y - y_before + pad * 2.0).max(0.0),
            );
            let outline_color = ph2d_vector::Color::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]); // LITERAL-COLOR-OK: user-color — showcase preview outline from user-stored ColorValue
            // FRAME-RAW-OK: a MARCA de realce que o utilizador escolheu (cor de marcador): conteudo autorado
            crate::paint::stroke_rounded_rect(
                scene,
                block,
                crate::paint::frame_radius(tema, Radius::Md.px()),
                StrokeToken::Thick.px(),
                outline_color,
            );
        }
        y = new_y;
        paint_pending_notes!(section_id);
        if a_parte {
            // ⭐ A arrastada pinta-se numa cena À PARTE, pousada no sítio de sempre e reusada como
            //    o FANTASMA que segue o cursor. ⚠️ Os alvos dela registam-se no sítio real.
            std::mem::swap(scene, &mut parte);
            scene.inner_mut().append(parte.inner(), None);
            fantasma = Some(parte);
        }
        corredor.depois(y_before, y);
        if y > y_before {
            faixas.push((section_id, y_before, y, tema));
        }
    }
    y = corredor.antes(scene, theme, inner_x, inner_w, y);
    // As notas de nenhuma secção da galeria — no fim do corpo.
    for (slot, note) in all_notes.iter().enumerate() {
        if !note.section.is_some_and(|s| SECTION_IDS.contains(&s)) {
            paint_one_note(
                scene,
                text_system,
                hit_index,
                store,
                inner_x,
                inner_w,
                &mut y,
                note,
                &crate::ids::GAL_NOTES,
                slot,
            );
        }
    }
    crate::widget::section_plan::conclui(
        scene,
        store,
        theme,
        &faixas,
        inner_x,
        inner_w,
        SECTION_HEAD_H,
    );
    if let Some(f) =
        crate::widget::section_plan::Fantasma::de(fantasma, &faixas, arrastada, inner_x, inner_w)
    {
        f.pinta(scene, store);
    }
    // ⭐ A nota arrastada — o fantasma por cima de tudo o que o corpo pintou.
    super::paint_note_drag_ghost(scene, text_system, hit_index, store, ids::GAL_PANEL, theme);
    // Publish content + visible heights so the host can clamp the
    // wheel-scroll bound and so the scrollbar's thumb sizes itself
    // correctly. Mirror of the live Inspector's `set_last_inspector_*`
    // pair — kept separate so the two panels can scroll independently.
    let content_h = (y - body_top_y).max(0.0);
    set_last_gallery_content_h(content_h);
    set_last_gallery_visible_h(visible_h);

    // Late-paint phase: open Dropdown popover sits on top of every
    // section that ran before it. `take_pending_dropdown_chip` is a
    // thread_local owned by the showcase; the live Inspector never
    // paints dropdowns so there's no contention.
    if let Some((sel_idx, chip)) = take_pending_dropdown_chip() {
        let labels = ["Front", "Side", "Top"];
        let selected_label = labels.get(sel_idx).copied().unwrap_or("Front");
        let dd = Dropdown::new(
            ids::INSP_SAMPLE_DROPDOWN,
            "View",
            vec![
                DropdownOption::new(ids::INSP_SAMPLE_DD_OPT_A, "front", "Front"),
                DropdownOption::new(ids::INSP_SAMPLE_DD_OPT_B, "side", "Side"),
                DropdownOption::new(ids::INSP_SAMPLE_DD_OPT_C, "top", "Top"),
            ],
        )
        .selected(selected_label)
        .open(true);
        crate::widget::paint_dropdown_popover(&dd, chip, scene, text_system, theme);
        for (i, opt) in dd.options.iter().enumerate() {
            hit_index.register(opt.id, dd.option_rect(chip, i));
        }
    }

    crate::widget::section_cards::end_section_cards(scene);
    ShowcaseBody {
        area,
        content_h,
        rect,
    }
}

/// O fecho da galeria com as partes soltas. A forma com `PaintCtx` é
/// `crate::panel::scroll_area::close_showcase_body` (o `widget` não conhece o `panel`).
pub fn close_showcase_body_with(
    body: ShowcaseBody,
    scene: &mut VectorScene,
    hit_index: &mut HitIndex,
    store: &mut WidgetStore,
    theme: Theme,
) {
    crate::widget::scroll_area::close_with(
        body.area,
        scene,
        hit_index,
        store,
        body.content_h,
        theme,
    );
    finish_chrome(body.rect, scene, hit_index, theme);
}

/// O cromo da galeria por cima do corpo (pontos dos cantos, alça, redimensionamentos, fechar).
pub fn finish_chrome(rect: Rect, scene: &mut VectorScene, hit_index: &mut HitIndex, theme: Theme) {
    let drag_handle_rect = panel_drag_handle_rect(
        rect,
        crate::widget::panel_chrome::PANEL_HEADER_H_DEFAULT,
        crate::widget::panel_chrome::PANEL_HEADER_CLOSE_RESERVE,
    );
    let resize_handle_rect = panel_resize_handle_rect(rect);
    let resize_handle_bl_rect = crate::widget::panel_chrome::panel_resize_handle_rect_bl(rect);

    paint_panel_corner_dot(rect, scene, theme);
    crate::widget::panel_chrome::paint_panel_corner_dot_bl(rect, scene, theme);
    // End-of-frame re-registration of the title-bar drag handle so it
    // sits z-on-top of any body widget that scrolled into the header
    // band (prevents click-through behind the title — DIRETRIZ chip-
    // canon work, 2026-05-24). Close button must be re-registered AFTER
    // drag so it wins over the drag rect on its small overlap zone
    // AND wins over any body widget that scrolled into its position
    // (bug reported 2026-05-24: close stopped working after scroll).
    hit_index.register(ids::GAL_DRAG_HANDLE, drag_handle_rect);
    hit_index.register(ids::GAL_RESIZE_HANDLE, resize_handle_rect);
    hit_index.register(ids::GAL_RESIZE_HANDLE_BL, resize_handle_bl_rect);
    hit_index.register(
        ids::GAL_CLOSE,
        crate::widget::panel_chrome::panel_close_button_rect(rect),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐ **A lista de pintores é a lista de secções** — os mesmos ids, pela mesma ordem natural. Sem
    /// isto uma secção nova no `SECTION_IDS` dobrava e recebia o menu e nunca era pintada.
    #[test]
    fn as_seccoes_pintadas_sao_as_do_section_ids() {
        let pintadas: Vec<NodeId> = SECCOES.iter().map(|(id, _)| *id).collect();
        assert_eq!(pintadas, SECTION_IDS.to_vec());
    }
}
