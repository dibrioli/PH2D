//! ⭐ O que a barra do vetor abre não depende da aba: com o Inspector à frente do Vector (o estado de
//! fábrica) o «Stroke ▾» acendia o chip e nada aparecia, porque o painel escondido não pintava
//! (visto em janela real, F8). E a barra de rolagem do painel do traço tem faixa própria.

use super::o_cartao_solto_no_canvas::clica;
use ph2d_editor_core::screens::hero::{HERO_VIEWPORT_W, paint_hero_screen, slot_tabs};
use ph2d_editor_core::screens::slot::Slot;
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{HeroScreen, NodeId, ids};
use ph2d_panel_vector::ids::{VECTOR_BAR_PROFILE, VECTOR_BAR_STROKE};
use ph2d_tool_vector::{DrawMode, VectorStyleSnapshot};

fn publica(h: &mut HeroScreen) {
    ph2d_panel_vector::set_current_vector_style(Some(VectorStyleSnapshot {
        mode: DrawMode::Select,
        ..VectorStyleSnapshot::default()
    }));
    ph2d_panel_vector::set_current_selection_count(1);
    ph2d_panel_vector::set_current_vertex_count(0);
    let barras = ph2d_panel_vector::bars::tool_bars(&h.store, None, Vec::new());
    h.store.set_tool_bars(Some(barras));
}

fn quadros(h: &mut HeroScreen, ts: &mut ph2d_text::TextSystem, altura: f32, n: usize) {
    let vp = Rect::new(0.0, 0.0, HERO_VIEWPORT_W, altura);
    for _ in 0..n {
        publica(h);
        let mut cena = ph2d_vector::VectorScene::new();
        paint_hero_screen(h, vp, &mut cena, ts);
    }
}

/// O Vector atrás do Inspector, no mesmo encaixe.
fn com_o_vector_atras(ts: &mut ph2d_text::TextSystem) -> HeroScreen {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut h = HeroScreen::new(NodeId(1));
    let nomes: Vec<&'static str> = ph2d_editor_core::panel::with_registry_ref(|reg| {
        reg.panels().iter().map(|p| p.manifest.id).collect()
    });
    for n in nomes {
        h.panel_visibility
            .insert(n, n == "inspector" || n == "vector");
    }
    // O painel que abre vem à frente: o Inspector sobe depois do 1.º quadro.
    quadros(&mut h, ts, 900.0, 2);
    h.store.bump_panel_z(ids::INSP_PANEL);
    quadros(&mut h, ts, 900.0, 2);
    assert!(
        slot_tabs::hidden_by_tabs(&h).contains(&ids::VECTOR_PANEL),
        "a fixtura tem de pôr a aba do Vector atrás ({:?})",
        slot_tabs::chosen(&h, Slot::RightTop)
    );
    h
}

fn limpa() {
    ph2d_panel_vector::set_current_vector_style(None);
    ph2d_panel_vector::set_current_selection_count(0);
}

/// *Mutação: tirar `pinta_com_a_aba_atras` da condição do `panel_walk` ⇒ vermelho.*
#[test]
fn the_stroke_panel_opens_with_the_tab_behind() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut h = com_o_vector_atras(&mut ts);
    assert!(h.hit_index.rect_for(ph2d_tool_vector::ids::VECTOR_DASH_IDS[0]).is_none());
    clica(&mut h, VECTOR_BAR_STROKE);
    quadros(&mut h, &mut ts, 900.0, 3);
    let (id, rect) = h.store.dropdown_popover().expect("o painel do traço abriu");
    assert_eq!(id, VECTOR_BAR_STROKE);
    assert!(
        rect.w >= ph2d_editor_core::interaction::WidgetStore::DOCK_W_MIN,
        "largura {} — a lista escondida tem largura zero",
        rect.w
    );
    assert!(
        h.hit_index.rect_for(ph2d_tool_vector::ids::VECTOR_DASH_IDS[0]).is_some(),
        "as linhas do traço têm de responder ao clique"
    );
    assert!(
        h.store.panel_rect(ids::VECTOR_PANEL).is_none(),
        "a aba atrás não reserva coluna"
    );
    clica(&mut h, VECTOR_BAR_STROKE);
    limpa();
}

/// A outra entrada da mesma família: as listas da barra.
#[test]
fn the_profile_list_opens_with_the_tab_behind() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut h = com_o_vector_atras(&mut ts);
    clica(&mut h, VECTOR_BAR_PROFILE);
    quadros(&mut h, &mut ts, 900.0, 3);
    let (id, _) = h.store.dropdown_popover().expect("a lista abriu");
    assert_eq!(id, VECTOR_BAR_PROFILE);
    clica(&mut h, VECTOR_BAR_PROFILE);
    limpa();
}

/// ⭐ Janela baixa: a barra aparece e as linhas não colam nela; janela alta: sem barra, sem faixa.
/// *Mutação: tirar o `if conteudo > rect.h` de `flutua` ⇒ vermelho na janela baixa.*
#[test]
fn the_scrollbar_of_the_stroke_panel_has_its_own_lane() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut h = com_o_vector_atras(&mut ts);
    clica(&mut h, VECTOR_BAR_STROKE);
    let direita = |h: &HeroScreen| {
        let r = h
            .hit_index
            .rect_for(ph2d_tool_vector::ids::VECTOR_GAP_IDS[0])
            .expect("o Gap do tracejado");
        r.x + r.w
    };
    quadros(&mut h, &mut ts, 1600.0, 3);
    let (_, alto) = h.store.dropdown_popover().expect("aberto");
    let m = ph2d_card_stack::metrics::CARD_PAD_X;
    assert!(
        (alto.x + alto.w - m - direita(&h)).abs() < 1.0,
        "sem barra, a margem é a do cartão"
    );
    quadros(&mut h, &mut ts, 640.0, 3);
    let (_, baixo) = h.store.dropdown_popover().expect("aberto");
    let faixa = ph2d_editor_core::widget::scrollbar_track_rect(baixo).x;
    let vao = ph2d_tokens::Spacing::Xs.px();
    assert!(
        direita(&h) <= faixa - vao + 0.5,
        "o controlo acaba em {} e a barra começa em {faixa}",
        direita(&h)
    );
    clica(&mut h, VECTOR_BAR_STROKE);
    limpa();
}
