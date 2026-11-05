//! As peças do cromo na galeria: a barra de ferramentas e a lista flutuante, pelo gesto real.

use super::cartoes::Galeria;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::screens::hero::{HERO_VIEWPORT_H, HERO_VIEWPORT_W};
use ph2d_editor_core::widget::ContextMenuEntry;
use ph2d_editor_core::zones::Rect;
use ph2d_host::PointerKind;
use ph2d_panel_widget_gallery::WidgetGalleryPanel;
use ph2d_panel_widget_gallery::cards_cromo;
use ph2d_panel_widget_gallery::ids as g;
use ph2d_panel_widget_gallery::state::Pick;
use ph2d_text::TextSystem;

const CHIPS: [NodeId; 3] = [g::BAR_MOVE, g::BAR_ROTATE, g::BAR_TRANSFORM];
const LINHAS: [NodeId; 3] = [g::LIST_COPY, g::LIST_DUPLICATE, g::LIST_DELETE];

/// O que a pintura desenha, com o rato fora de tudo.
fn desenho(gal: &mut Galeria) -> Vec<u32> {
    gal.ponteiro(PointerKind::Move, 0.0, 0.0);
    gal.repinta();
    let tela = Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H * 30.0);
    gal.host
        .paint_draw_data::<WidgetGalleryPanel>(&mut gal.st, tela)
}

/// Trocar a escolha muda a pintura, e voltar devolve-a ao bit (o clique deixa o foco: aqui não há clique).
fn a_escolha_aparece(gal: &mut Galeria, p: Pick) {
    gal.st.choices.choose(p, 0);
    let zero = desenho(gal);
    assert_eq!(zero, desenho(gal), "controlo: a pintura é estável");
    gal.st.choices.choose(p, 2);
    assert_ne!(zero, desenho(gal), "a escolha não apareceu na pintura");
    gal.st.choices.choose(p, 0);
    assert_eq!(zero, desenho(gal), "voltar não devolveu a pintura");
}

/// Cada chip responde no próprio centro, e o aceso aparece na pintura. *Mutação: tirar o registo dos
/// alvos, o `.active()` do chip, ou o controlo da tabela.*
#[test]
fn cada_chip_da_barra_escolhe_e_acende() {
    let mut gal = Galeria::aberta();
    assert_eq!(gal.st.choices.pick(Pick::BarTool), 0, "fixtura: Move");
    for (i, id) in CHIPS.iter().enumerate().rev() {
        gal.clica(*id);
        assert_eq!(usize::from(gal.st.choices.pick(Pick::BarTool)), i);
    }
    a_escolha_aparece(&mut gal, Pick::BarTool);
}

/// Irmãos curtos na mesma fila, nada colado, e a legenda inteira cabe no chip (o nome nunca se corta).
/// *Mutação: tirar `size_row_pulldowns`.*
#[test]
fn a_barra_e_uma_fila_que_cabe_o_nome_inteiro() {
    let gal = Galeria::aberta();
    let rects = CHIPS.map(|id| gal.rect(id).expect("chip pintado"));
    for par in rects.windows(2) {
        assert_eq!(par[0].y, par[1].y, "os chips têm de dividir a fila");
        assert!(par[0].x + par[0].w < par[1].x, "chips grudados");
    }
    let mut text = TextSystem::new();
    let legenda = ph2d_i18n::tr("chrome.rail.caption.xform");
    assert_eq!(legenda, "Transform", "a legenda é o nome inteiro");
    let pede = text.prefix_width(legenda, m::LABEL_PX);
    assert!(
        pede > rects[0].w,
        "controlo: a legenda longa não cabe no chip quadrado"
    );
    assert!(rects[2].w >= pede, "{} contra {pede}", rects[2].w);
}

/// A linha escolhida muda a seleção e a dica. *Mutação: tirar o registo dos alvos ou o `.selected()`.*
#[test]
fn cada_linha_da_lista_escolhe_e_a_dica_muda() {
    let mut gal = Galeria::aberta();
    assert_eq!(gal.st.choices.pick(Pick::ListRow), 0, "fixtura: Copy");
    for (i, id) in LINHAS.iter().enumerate().rev() {
        gal.clica(*id);
        assert_eq!(usize::from(gal.st.choices.pick(Pick::ListRow)), i);
    }
    a_escolha_aparece(&mut gal, Pick::ListRow);
    gal.st.choices.choose(Pick::ListRow, 1);
    let marcadas: Vec<NodeId> = cards_cromo::menu(&gal.st.choices, gal.host.store())
        .entries
        .iter()
        .filter_map(|e| match e {
            ContextMenuEntry::Item(l) if l.selected => Some(l.id),
            _ => None,
        })
        .collect();
    assert_eq!(marcadas, [g::LIST_DUPLICATE], "só a escolhida se marca");
}

/// As linhas têm a altura de um campo, e o separador do menu do app abre o vão antes de Delete.
/// *Mutação: tirar o `Separator`.*
#[test]
fn a_lista_e_o_menu_do_app() {
    let gal = Galeria::aberta();
    let [copy, dup, del] = LINHAS.map(|id| gal.rect(id).expect("linha pintada"));
    assert_eq!(copy.h, m::FIELD_H);
    assert_eq!(dup.y - copy.y, m::FIELD_H, "linhas seguidas encostam");
    assert!(
        del.y - dup.y > m::FIELD_H,
        "falta o separador antes de Delete"
    );
    assert_eq!(copy.x, del.x);
}
