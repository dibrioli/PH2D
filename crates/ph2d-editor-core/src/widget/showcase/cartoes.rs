//! Os corpos da galeria para a lista de CARTÕES (doc `CartoesUI/08` §1.2).
//!
//! Cada secção de hoje parte-se em cabeçalho (só a janela flutuante, `PH2D_CARDS=0`) + corpo; a
//! composição em cartões vive na crate `ph2d-panel-widget-gallery` (o `widget` não chama `panel::`).

use super::*;

/// Como o corpo de uma secção se pinta: `(scene, text, tema, hits, store, x, w, y) -> fundo`.
pub type PintaCorpo =
    fn(&mut VectorScene, &mut TextSystem, Theme, &mut HitIndex, &WidgetStore, f32, f32, f32) -> f32;

/// Uma secção da galeria vista como cartão: o id de sempre (ordem e tema em `sections.txt`
/// sobrevivem), o título do cabeçalho de hoje e o pintor do corpo.
#[derive(Clone, Copy)]
pub struct ShowcaseCard {
    pub section: NodeId,
    pub title: &'static str,
    pub body: PintaCorpo,
}

/// ⭐ As secções da galeria como cartões, pela ordem NATURAL de [`SECTION_IDS`] (há teste).
pub const SHOWCASE_CARDS: [ShowcaseCard; 11] = [
    card(ids::INSP_SECTION_INPUTS, "Inputs", paint_inputs_body),
    card(ids::INSP_SECTION_SLIDER, "Slider", paint_slider_body),
    card(ids::INSP_SECTION_SWITCHES, "Switches", paint_switches_body),
    card(ids::INSP_SECTION_LISTS, "Lists", paint_lists_body),
    card(ids::INSP_SECTION_VECTOR, "Vector", paint_vector_body),
    card(ids::INSP_SECTION_STATUS, "Status", paint_status_body),
    card(ids::INSP_SECTION_COLOR, "Color", paint_color_body),
    card(ids::INSP_SECTION_ACTIONS, "Actions", paint_actions_body),
    card(ids::INSP_SECTION_IDENTITY, "Identity", paint_identity_body),
    card(ids::INSP_SECTION_CARD, "Card", paint_card_body),
    card(
        ids::INSP_SECTION_W6,
        "Inspector v2 (W6)",
        paint_inspector_w6_body,
    ),
];

const fn card(section: NodeId, title: &'static str, body: PintaCorpo) -> ShowcaseCard {
    ShowcaseCard {
        section,
        title,
        body,
    }
}

/// A fase TARDIA do corpo: a lista suspensa aberta da secção Lists, por cima de tudo o que se
/// pintou antes. Quem pinta os corpos chama-a DEPOIS do último (a janela e a lista de cartões).
/// `take_pending_dropdown_chip` é um `thread_local` da galeria; o Inspector não pinta listas
/// suspensas, logo não há disputa.
pub fn paint_showcase_popovers(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
) {
    let Some((sel_idx, chip)) = take_pending_dropdown_chip() else {
        return;
    };
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
    let panel = crate::widget::paint_dropdown_popover(&dd, chip, scene, text_system, theme);
    for (i, opt) in dd.options.iter().enumerate() {
        hit_index.register(opt.id, dd.option_rect_in(chip, panel, i));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐ Os cartões são as secções: os mesmos ids, pela ordem natural, e o mesmo título do
    /// cabeçalho da janela (a tabela `SECCOES` do corpo amarra-se ao `SECTION_IDS` pelo outro lado).
    #[test]
    fn os_cartoes_da_galeria_sao_as_do_section_ids() {
        let ids: Vec<NodeId> = SHOWCASE_CARDS.iter().map(|c| c.section).collect();
        assert_eq!(ids, SECTION_IDS.to_vec());
        assert!(SHOWCASE_CARDS.iter().all(|c| !c.title.is_empty()));
    }
}
