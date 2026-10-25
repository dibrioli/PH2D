//! ⭐⭐⭐ **A lista aberta tem a largura dos NOMES, não a do chip** (report do dono 25/10: num painel
//! estreito saíam «Standing...», «Key held:...», «Took dam...»).
//!
//! Pela porta real (`panel::paint_dropdown_list`): a lista alarga até o rótulo mais largo MEDIDO
//! caber, a banda clicável de cada opção é a linha pintada, e junto à borda direita da região a
//! lista desloca-se para a esquerda e fica dentro dela.

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::{PaintCtx, paint_dropdown_list};
use ph2d_editor_core::screens::HeroLayout;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::widget::{
    Dropdown, DropdownOption, DropdownState, PaintedPopover, PopoverRow,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{Theme, TypeToken};
use ph2d_vector::VectorScene;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1200.0,
    h: 800.0,
};
const DD: NodeId = NodeId(9_200_000);
const LONGA: usize = 1;
const NOMES: [&str; 3] = ["Idle", "Key held: a very long action name", "Run"];

fn opt(i: usize) -> NodeId {
    NodeId(9_200_001 + i as u64)
}

fn lista() -> Dropdown<usize> {
    let options = NOMES
        .iter()
        .enumerate()
        .map(|(i, n)| DropdownOption::new(opt(i), i, *n))
        .collect();
    Dropdown::new(DD, "", options).selected(0).open(true)
}

fn hero() -> HeroScreen {
    ph2d_editor_core::test_support::ensure_panel_registry();
    let mut hero = HeroScreen::new(NodeId(1));
    hero.store.register(
        DD,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: true,
            selected_index: Some(0),
        },
    );
    hero
}

/// Pinta pela porta; devolve o publicado, as linhas pintadas e a largura medida do nome longo.
fn pinta(
    hero: &mut HeroScreen,
    chip: Rect,
    region: Rect,
) -> (PaintedPopover, Vec<PopoverRow>, f32) {
    hero.hit_index.clear_for_frame();
    let layout = HeroLayout::for_viewport(VIEWPORT);
    let mut scene = VectorScene::new();
    let mut text_system = TextSystem::without_system_fonts();
    let medido = ph2d_editor_core::paint::label_column_width(
        &mut text_system,
        TypeToken::Base.px(),
        [NOMES[LONGA]],
    );
    let mut vistas = Vec::new();
    let mut linha = |_: &mut VectorScene, _: &mut TextSystem, r: PopoverRow| {
        vistas.push(r);
    };
    let painted = {
        let mut ctx = PaintCtx {
            host: hero,
            layout: &layout,
            slot: VIEWPORT,
            viewport: VIEWPORT,
            scene: &mut scene,
            text_system: &mut text_system,
        };
        paint_dropdown_list(
            &mut ctx,
            &lista(),
            chip,
            region,
            Theme::Forge,
            Some(&mut linha),
        )
    };
    (painted, vistas, medido)
}

fn hit_de(hero: &HeroScreen, id: NodeId) -> Rect {
    hero.hit_index
        .iter_registrations()
        .find(|(i, _)| *i == id)
        .map(|(_, r)| r)
        .unwrap_or_else(|| panic!("{id:?} não foi registada"))
}

/// (a) mais larga que o chip e o nome inteiro cabe no orçamento da linha; (b) a banda clicável
/// é a linha pintada.
///
/// **Mutações que devem sangrar:** a porta voltar ao `popover_rect_clamped` (largura do chip) ·
/// o `list_width` ignorar os rótulos.
#[test]
fn a_lista_alarga_ate_o_nome_inteiro_e_o_clique_acompanha() {
    let chip = Rect::new(40.0, 120.0, 72.0, 24.0);
    let region = Rect::new(0.0, 0.0, 1200.0, 800.0);
    let mut hero = hero();
    let (p, vistas, medido) = pinta(&mut hero, chip, region);
    assert!(
        medido > chip.w,
        "a fixtura não é comprida: {medido} em {}",
        chip.w
    );
    assert!(
        p.panel.w > chip.w,
        "a lista ficou na largura do chip: {} ≤ {}",
        p.panel.w,
        chip.w
    );
    assert_eq!(p.panel.x, chip.x, "com espaço, a lista ancora no chip");
    let row = vistas
        .iter()
        .find(|r| r.index == LONGA)
        .expect("a opção longa foi pintada");
    assert!(
        ph2d_editor_core::paint::label_budget(row.rect.w) >= medido,
        "o nome não cabe na linha: orçamento {} < {medido}",
        ph2d_editor_core::paint::label_budget(row.rect.w)
    );
    let band = hit_de(&hero, opt(LONGA));
    assert_eq!(
        (band.x, band.w),
        (row.rect.x, row.rect.w),
        "a banda clicável diverge da linha pintada"
    );
    assert_eq!(hero.store.dropdown_popover(), Some((DD, p.panel)));
}

/// (c) junto à borda direita da região a lista desloca-se para a esquerda e fica dentro dela; o
/// clique acompanha.
#[test]
fn junto_a_borda_direita_a_lista_desloca_se_e_fica_dentro() {
    let region = Rect::new(0.0, 0.0, 400.0, 800.0);
    let chip = Rect::new(region.w - 80.0, 120.0, 72.0, 24.0);
    let mut hero = hero();
    let (p, vistas, medido) = pinta(&mut hero, chip, region);
    assert!(p.panel.w > chip.w, "a lista não alargou");
    assert!(
        p.panel.x < chip.x,
        "a lista não se deslocou para a esquerda"
    );
    assert!(
        p.panel.x >= region.x && p.panel.x + p.panel.w <= region.x + region.w + 1.0e-3,
        "a lista saiu da região: {:?} em {region:?}",
        p.panel
    );
    let row = vistas.iter().find(|r| r.index == LONGA).unwrap();
    assert!(ph2d_editor_core::paint::label_budget(row.rect.w) >= medido);
    let band = hit_de(&hero, opt(LONGA));
    assert_eq!((band.x, band.w), (row.rect.x, row.rect.w));
}

/// Uma região mais estreita que o nome: a lista ocupa a região inteira e não sai dela (só aí
/// elide); uma lista que cabe no chip fica exactamente como era.
#[test]
fn a_regiao_estreita_limita_e_a_lista_curta_nao_muda() {
    let region = Rect::new(0.0, 0.0, 120.0, 800.0);
    let chip = Rect::new(20.0, 120.0, 72.0, 24.0);
    let mut hero = hero();
    let (p, _, _) = pinta(&mut hero, chip, region);
    assert_eq!((p.panel.x, p.panel.w), (region.x, region.w));

    let curta = Dropdown::new(DD, "", vec![DropdownOption::new(opt(0), 0usize, "A")]);
    let wide_chip = Rect::new(20.0, 120.0, 300.0, 24.0);
    let mut ts = TextSystem::without_system_fonts();
    assert_eq!(
        curta.popover_rect_fitted(wide_chip, Some(VIEWPORT), &mut ts),
        curta.popover_rect_clamped(wide_chip, VIEWPORT),
        "uma lista que cabe no chip mudou de rect"
    );
}
