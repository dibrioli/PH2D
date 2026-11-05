//! ⭐ **Dono 05/11: «faça o transform de vector no modo draw ser o mesmo do modo object»**. No hero real,
//! com o Vector à frente e o Inspector na aba de trás: o cartão Transform É o do Inspector, por cima do
//! cartão Shape (Size, Resize Box, Set Center), com o que o de Object tem.
#![cfg(all(feature = "panel-inspector", feature = "panel-vector"))]

use super::cartoes_mao::{centro, clica, mao, pinta, solta};
use super::o_bloco5_abertos::escolhe;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids::{self as core_ids, grip_de};
use ph2d_editor_core::interaction::{loose_pin_of, loose_rect_of};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_host::PointerKind;
use ph2d_panel_inspector::{CARD_TRANSFORM, ids as insp};
use ph2d_panel_vector::{cards, ids as p, state};
use ph2d_tool_vector::ids as t;

const FORMA_A: u64 = 21;
const FORMA_B: u64 = 22;

fn publica(h: &HeroScreen) {
    let pins = h.store.loose_pins_of(core_ids::INSP_PANEL);
    ph2d_panel_inspector::com_cada_preso(&pins, |b| {
        escolhe(
            b,
            if b == FORMA_A { "Shape A" } else { "Shape B" },
            b as f32,
        );
    });
}

fn quadros(h: &mut HeroScreen, n: usize) {
    for _ in 0..n {
        publica(h);
        pinta(h, 1);
    }
}

/// O Vector à frente com a forma A escolhida, como a ponte deixa o Draw.
fn em_draw() -> HeroScreen {
    let _ = ph2d_panel_registry_init::register_all_panels();
    ph2d_panel_inspector::set_cards_on(true);
    ph2d_panel_inspector::set_current_inspector_sprite(None);
    cards::force_cards_on_this_thread(Some(true));
    state::set_length_suffix("px");
    state::set_current_transform(Some([10.0, 20.0, 100.0, 40.0]));
    state::set_resize_box(Some(false));
    escolhe(FORMA_A, "Shape A", 21.0);
    let mut h = HeroScreen::new(NodeId(1));
    let ids: Vec<&'static str> = ph2d_editor_core::panel::with_registry_ref(|reg| {
        reg.panels().iter().map(|p| p.manifest.id).collect()
    });
    for n in ids {
        h.panel_visibility
            .insert(n, n == "inspector" || n == "vector");
    }
    h.store.set_collapsed(CARD_TRANSFORM, false);
    h.store.set_collapsed(t::VECTOR_SECTION_TRANSFORM, false);
    quadros(&mut h, 2);
    // A ponte traz a aba à frente quando a ferramenta entra: a ordem z nasce no primeiro quadro.
    h.store.bump_panel_z(core_ids::VECTOR_PANEL);
    quadros(&mut h, 40);
    h
}

fn pintado(h: &HeroScreen, id: NodeId) -> bool {
    h.hit_index.rect_for(id).is_some()
}

fn por_cima(h: &HeroScreen, a: NodeId, b: NodeId) -> bool {
    let y = |id| h.hit_index.rect_for(id).map(|r| r.y);
    matches!((y(a), y(b)), (Some(a), Some(b)) if a < b)
}

/// *Mutações: sem o convite ⇒ o Transform não aparece. A lista curta com todas as secções ⇒ o Object vem junto.
/// O anfitrião sem descer ⇒ os cartões sobrepõem-se.*
#[test]
fn em_draw_o_transform_e_o_do_inspector_por_cima_do_shape() {
    let h = em_draw();
    assert!(
        por_cima(&h, insp::INSP_TRANSFORM_POS_X, t::VECTOR_TRANSFORM_W),
        "o Transform do Inspector por cima do Shape"
    );
    for id in [
        insp::INSP_TRANSFORM_ROT,
        insp::INSP_TRANSFORM_SCALE_X,
        insp::INSP_TRANSFORM_SKEW_X,
        insp::INSP_TRANSFORM_RESET,
        p::VECTOR_PIVOT_EDIT,
        p::VECTOR_TRANSFORM_RESIZE_BOX,
    ] {
        assert!(pintado(&h, id), "{id:?}");
    }
    for id in [
        t::VECTOR_TRANSFORM_X,
        t::VECTOR_TRANSFORM_Y,
        p::VECTOR_TRANSFORM_R,
    ] {
        assert!(!pintado(&h, id), "a segunda posição voltou: {id:?}");
    }
    let lista_curta = h.store.panel_rect(core_ids::INSP_PANEL).expect("o rect");
    let vector = h.store.panel_rect(core_ids::VECTOR_PANEL).expect("o rect");
    assert!(
        lista_curta.y + lista_curta.h <= vector.y,
        "as duas listas sobrepõem-se: {lista_curta:?} {vector:?}"
    );
    assert!(
        !pintado(&h, grip_de(ph2d_panel_inspector::CARD_OBJECT)),
        "só o Transform vem à coluna do Draw"
    );
}

/// *Mutações: o convite sem olhar a seleção ⇒ a lista curta fica sem forma. A travessia sem filtrar quem está
/// à frente ⇒ dois Transforms.*
#[test]
fn sem_forma_ou_com_o_inspector_a_frente_o_convite_nao_vale() {
    let mut h = em_draw();
    state::set_current_transform(None);
    quadros(&mut h, 40);
    assert!(
        !pintado(&h, insp::INSP_TRANSFORM_POS_X),
        "sem forma no Draw, sem Transform"
    );
    assert_eq!(h.store.card_guest_h_over(core_ids::VECTOR_PANEL), 0.0);

    state::set_current_transform(Some([10.0, 20.0, 100.0, 40.0]));
    // O solto do Vector faz a aba de trás pintar (e convidar): quem está à frente manda.
    h.store.detach_card(
        t::VECTOR_SECTION_SNAP,
        core_ids::VECTOR_PANEL,
        (400.0, 300.0),
        300.0,
    );
    h.store.bump_panel_z(core_ids::INSP_PANEL);
    quadros(&mut h, 40);
    assert!(pintado(&h, insp::INSP_TRANSFORM_POS_X));
    assert!(
        pintado(&h, grip_de(ph2d_panel_inspector::CARD_OBJECT)),
        "o Inspector à frente é a lista inteira"
    );
    assert!(!pintado(&h, t::VECTOR_TRANSFORM_W));
}

/// O Inspector sem Transform a mostrar (nada com pose): a lista curta não ocupa a coluna.
/// *Mutação: a altura = a do fundo vazio ⇒ o Vector desce por uma lista invisível.*
#[test]
fn sem_cartao_a_lista_curta_nao_ocupa_nada() {
    let mut h = em_draw();
    ph2d_panel_inspector::set_current_inspector_transform(None);
    quadros(&mut h, 40);
    assert_eq!(h.store.card_guest_h_over(core_ids::VECTOR_PANEL), 0.0);
    assert!(h.store.panel_rect(core_ids::INSP_PANEL).is_none());
    assert!(pintado(&h, t::VECTOR_TRANSFORM_W));
}

/// Solto no Draw ele é o de Object: nome, reset, alfinete; preso a A, com B em foco, o arrasto publica só para A.
/// *Mutações: a lista curta sem o passo do solto ⇒ `solta` falha. `com_retrato` sem o preso ⇒ o pedido vai a B.*
#[test]
fn solto_no_draw_tem_nome_reset_alfinete_e_o_arrasto_vai_so_ao_preso() {
    let mut h = em_draw();
    solta(&mut h, CARD_TRANSFORM);
    quadros(&mut h, 5);
    assert!(
        pintado(&h, loose_pin_of(CARD_TRANSFORM)),
        "alfinete no solto"
    );
    assert!(pintado(&h, insp::INSP_TRANSFORM_RESET), "reset no solto");
    clica(&mut h, loose_pin_of(CARD_TRANSFORM));
    assert_eq!(
        h.store.loose_card(CARD_TRANSFORM).and_then(|c| c.pin),
        Some(FORMA_A)
    );
    assert_eq!(h.store.card_subject_name(FORMA_A), Some("Shape A"));
    clica(&mut h, loose_pin_of(CARD_TRANSFORM));
    assert_eq!(
        h.store.loose_card(CARD_TRANSFORM).map(|c| (c.pin, c.rope)),
        Some((Some(FORMA_A), true)),
        "o 2.º clique dá a corda"
    );

    escolhe(FORMA_B, "Shape B", 22.0);
    quadros(&mut h, 40);
    let r = h
        .store
        .panel_rect(loose_rect_of(CARD_TRANSFORM))
        .expect("o preso está no canvas");
    let dentro = (r.x + r.w * 0.5, r.y + r.h - 3.0);
    mao(&mut h, PointerKind::Down, dentro);
    mao(&mut h, PointerKind::Up, dentro);
    let _ = h.bus.drain().count();
    h.store.set_number_value(insp::INSP_TRANSFORM_POS_X, 250.0);
    h.apply_event(ph2d_editor_core::interaction::WidgetEvent::ValueChanged(
        insp::INSP_TRANSFORM_POS_X,
    ));
    let pedidos: Vec<u64> = h
        .bus
        .drain()
        .filter_map(|a| match a {
            EditorAction::InspectorTransformEdit(i) => Some(i.entity_bits),
            _ => None,
        })
        .collect();
    assert_eq!(pedidos, vec![FORMA_A], "só o preso");
    assert!(
        h.store.card_guest_h_over(core_ids::VECTOR_PANEL) > ph2d_card_stack::metrics::LOOSE_GHOST_H,
        "a seleção B continua com o Transform inteiro na coluna"
    );
}

/// Premir o Transform na coluna não muda a aba: quem está à frente continua a ser o Vector.
/// *Mutação: `bump_panel_z(panel_id)` na pressão ⇒ o Inspector vem à frente e a coluna perde o Vector.*
#[test]
fn premir_o_transform_na_coluna_nao_troca_a_aba() {
    let mut h = em_draw();
    let r = h.store.panel_rect(core_ids::INSP_PANEL).expect("o rect");
    let dentro = (r.x + r.w * 0.5, r.y + 2.0);
    mao(&mut h, PointerKind::Down, dentro);
    mao(&mut h, PointerKind::Up, dentro);
    quadros(&mut h, 3);
    assert!(
        pintado(&h, t::VECTOR_TRANSFORM_W),
        "o Vector saiu da frente"
    );
}

/// *Mutação: o reset sem o convidado no `apply_event` ⇒ nenhum pedido.*
#[test]
fn o_reset_no_draw_repoe_a_forma() {
    let mut h = em_draw();
    let _ = h.bus.drain().count();
    clica(&mut h, insp::INSP_TRANSFORM_RESET);
    let pedido = h
        .bus
        .drain()
        .find_map(|a| match a {
            EditorAction::InspectorTransformEdit(i) => Some(i),
            _ => None,
        })
        .expect("o reset não pediu nada");
    assert_eq!(pedido.entity_bits, FORMA_A);
    assert_eq!(pedido.translation, [0.0, 0.0]);
    assert_eq!(pedido.scale, [1.0, 1.0]);
    assert_eq!(pedido.rotation_rad, 0.0);
}

/// Size e Set Center continuam a pedir o que pediam, no cartão Shape.
/// *Mutações: `shape` sem o Set Center ou sem o W ⇒ `clica`/`pintado` reprovam.*
#[test]
fn o_cartao_shape_pede_o_efeito_de_antes() {
    let mut h = em_draw();
    let _ = h.bus.drain().count();
    h.store.set_number_value(t::VECTOR_TRANSFORM_W, 75.0);
    h.apply_event(ph2d_editor_core::interaction::WidgetEvent::ValueChanged(
        t::VECTOR_TRANSFORM_W,
    ));
    let largura = h.bus.drain().find_map(|a| match a {
        EditorAction::ToolPanelEvent(PanelEvent::SetValue(id, v))
            if id == t::VECTOR_TRANSFORM_W =>
        {
            Some(v)
        }
        _ => None,
    });
    assert_eq!(largura, Some(75.0));
    clica(&mut h, p::VECTOR_PIVOT_EDIT);
    assert!(
        h.bus.drain().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::Click(id)) if id == p::VECTOR_PIVOT_EDIT
        )),
        "Set Center não pediu nada"
    );
}

/// *Mutação: `column_owner` = o próprio painel ⇒ a roda sobre o Transform não rola o Vector.*
#[test]
fn a_roda_sobre_o_transform_rola_a_coluna_do_vector() {
    let mut h = em_draw();
    for s in p::VECTOR_SECTIONS {
        h.store.set_collapsed(*s, false);
    }
    quadros(&mut h, 40);
    let v = core_ids::VECTOR_PANEL;
    let (c, vista) = (h.store.panel_content_h(v), h.store.panel_visible_h(v));
    assert!(
        c.zip(vista).is_some_and(|(c, vista)| c > vista),
        "o Vector tem de transbordar: {c:?} × {vista:?}"
    );
    let (x, y) = centro(h.store.panel_rect(core_ids::INSP_PANEL).expect("o rect"));
    assert!(h.chrome_panel_at(x, y).is_some(), "a shell daria zoom");
    let arena = bumpalo::Bump::new();
    for _ in 0..5 {
        let _ = h.handle_wheel(
            ph2d_host::WheelEvent {
                x,
                y,
                delta_x: 0.0,
                delta_y: -16.0,
                modifiers: Default::default(),
                timestamp_ns: 0,
            },
            &arena,
        );
        quadros(&mut h, 1);
    }
    assert!(h.store.panel_scroll(v) > 0.0, "a roda foi a outro");
}

/// A roda continua a rolar depois de um clique num campo da coluna: o foco não reescreve o alvo da rolagem.
#[test]
fn a_roda_rola_a_coluna_com_um_campo_em_foco() {
    let mut h = em_draw();
    for s in p::VECTOR_SECTIONS {
        h.store.set_collapsed(*s, false);
    }
    quadros(&mut h, 40);
    let r = h
        .hit_index
        .rect_for(t::VECTOR_TRANSFORM_W)
        .expect("o campo");
    let dentro = (r.x + r.w * 0.6, r.y + r.h * 0.5);
    mao(&mut h, PointerKind::Down, dentro);
    mao(&mut h, PointerKind::Up, dentro);
    quadros(&mut h, 3);
    assert_eq!(h.store.focus_id(), Some(t::VECTOR_TRANSFORM_W), "sem foco");
    let arena = bumpalo::Bump::new();
    for _ in 0..5 {
        let _ = h.handle_wheel(
            ph2d_host::WheelEvent {
                x: dentro.0,
                y: dentro.1,
                delta_x: 0.0,
                delta_y: -16.0,
                modifiers: Default::default(),
                timestamp_ns: 0,
            },
            &arena,
        );
        quadros(&mut h, 1);
    }
    assert!(
        h.store.panel_scroll(core_ids::VECTOR_PANEL) > 0.0,
        "o foco comeu a roda"
    );
}
