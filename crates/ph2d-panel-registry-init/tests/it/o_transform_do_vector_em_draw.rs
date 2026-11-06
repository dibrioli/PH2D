//! ⭐ **Dono 05/11: «faça o transform de vector no modo draw ser o mesmo do modo object»** e *«o transform
//! dividiu o painel. Deveria ficar dentro do painel»*. No hero real, com o Vector à frente e o Inspector na
//! aba de trás: o cartão Transform É o do Inspector, o primeiro do corpo da aba Drawing, na moldura do Vector.
#![cfg(all(feature = "panel-inspector", feature = "panel-vector"))]

use super::cartoes_mao::{arrasta, centro, clica, mao, pinta, solta};
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

/// O cabeçalho do Transform: o que sobra dele à vista no recorte da coluna.
fn cabeca_do_transform(h: &HeroScreen) -> ph2d_editor_core::zones::Rect {
    h.hit_index.rect_for(CARD_TRANSFORM).expect("o Transform")
}

/// *Mutações: sem o convite ⇒ o Transform não aparece. A lista curta com todas as secções ⇒ o Object vem junto.
/// O anfitrião sem descer ⇒ os cartões sobrepõem-se. A barreira depois do passo ⇒ os campos não clicam. O
/// assento sem as abas ⇒ o cartão sobe por cima delas.*
#[test]
fn em_draw_o_transform_e_o_primeiro_cartao_dentro_do_painel() {
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
    assert!(
        h.store.panel_rect(core_ids::INSP_PANEL).is_none(),
        "o Transform não tem moldura própria"
    );
    let skew = h
        .hit_index
        .rect_for(insp::INSP_TRANSFORM_SKEW_X)
        .expect("Skew");
    let shape_y = h
        .hit_index
        .rect_for(t::VECTOR_SECTION_TRANSFORM)
        .expect("o Shape")
        .y;
    assert!(
        skew.y + skew.h <= shape_y,
        "os cartões sobrepõem-se: {skew:?} × {shape_y}"
    );
    let vector = h.store.panel_rect(core_ids::VECTOR_PANEL).expect("o rect");
    let cabeca = cabeca_do_transform(&h);
    let abas = h
        .hit_index
        .rect_for(p::VECTOR_TAB_DRAWING)
        .expect("as abas");
    assert!(
        cabeca.y >= abas.y + abas.h,
        "abaixo das abas: {cabeca:?} {abas:?}"
    );
    assert!(
        cabeca.x >= vector.x
            && cabeca.x + cabeca.w <= vector.x + vector.w
            && cabeca.y + cabeca.h <= vector.y + vector.h,
        "dentro do painel: {cabeca:?} {vector:?}"
    );
    let outro = h
        .hit_index
        .rect_for(t::VECTOR_SECTION_TRANSFORM)
        .expect("o cartão Shape");
    assert_eq!(
        (cabeca.x, cabeca.w),
        (outro.x, outro.w),
        "na mesma coluna dos cartões do Vector"
    );
    let campo = h.hit_index.rect_for(insp::INSP_TRANSFORM_POS_X).expect("X");
    assert_eq!(
        h.hit_index.hit(campo.x + 2.0, campo.y + campo.h * 0.5),
        Some(insp::INSP_TRANSFORM_POS_X),
        "o clique é do campo, não da barreira do painel"
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

/// Na aba Interface o Transform não aparece; de volta à Drawing, sim.
/// *Mutação: o convite sem olhar a aba ⇒ ele fica na Interface.*
#[test]
fn na_aba_interface_o_transform_nao_aparece() {
    let mut h = em_draw();
    clica(&mut h, p::VECTOR_TAB_INTERFACE);
    quadros(&mut h, 40);
    assert!(!pintado(&h, insp::INSP_TRANSFORM_POS_X));
    assert_eq!(h.store.card_guest_h_over(core_ids::VECTOR_PANEL), 0.0);
    clica(&mut h, p::VECTOR_TAB_DRAWING);
    quadros(&mut h, 40);
    assert!(pintado(&h, insp::INSP_TRANSFORM_POS_X));
}

/// Um popover do Vector continua por cima do Transform: a lista de Blend, que passa da janela, sobe até o cobrir.
/// *Mutação: o passo do convidado depois da pintura do anfitrião ⇒ o clique na zona comum vai ao Transform.*
#[test]
fn o_popover_do_vector_fica_por_cima_do_transform() {
    state::set_current_appearance(Some(state::Appearance {
        opacity: 1.0,
        blend: ph2d_vec_scene::BlendMode::Normal,
        layers: Vec::new(),
        mixed_opacity: false,
        mixed_blend: false,
        layers_differ: false,
        order: Vec::new(),
    }));
    let mut h = em_draw();
    h.store.set_collapsed(p::VECTOR_SECTION_APPEARANCE, false);
    quadros(&mut h, 40);
    clica(&mut h, p::VECTOR_OBJ_BLEND);
    quadros(&mut h, 5);
    let linhas: Vec<_> = (0..64).map(p::vector_obj_blend_option_id).collect();
    let campos = [
        insp::INSP_TRANSFORM_POS_X,
        insp::INSP_TRANSFORM_POS_Y,
        insp::INSP_TRANSFORM_ROT,
        insp::INSP_TRANSFORM_SCALE_X,
        insp::INSP_TRANSFORM_SKEW_X,
    ];
    let mut cobertos = 0;
    for id in campos {
        let r = h.hit_index.rect_for(id).expect("o campo");
        let (x, y) = centro(r);
        let quem = h.hit_index.hit(x, y);
        if quem != Some(id) {
            cobertos += 1;
            assert!(
                quem.is_some_and(|q| linhas.contains(&q)),
                "{quem:?} em cima de {id:?}"
            );
        }
    }
    assert!(
        cobertos > 0,
        "a lista não chegou ao Transform: a prova não vale"
    );
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
    quadros(&mut h, 40);
    // A faixa «on canvas» fica no lugar dele, sem moldura separada.
    assert_eq!(
        h.store.card_guest_h_over(core_ids::VECTOR_PANEL),
        ph2d_card_stack::metrics::LOOSE_GHOST_H
    );
    assert!(h.store.panel_rect(core_ids::INSP_PANEL).is_none());
    let vector = h.store.panel_rect(core_ids::VECTOR_PANEL).expect("o rect");
    let shape = h
        .hit_index
        .rect_for(t::VECTOR_SECTION_TRANSFORM)
        .expect("o Shape");
    assert!(
        shape.y > vector.y,
        "o Vector continua: {shape:?} {vector:?}"
    );
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
/// *Mutação: o passo a deixar o rect do Inspector ⇒ a pressão traz o Inspector à frente e a coluna perde o Vector.*
#[test]
fn premir_o_transform_na_coluna_nao_troca_a_aba() {
    let mut h = em_draw();
    let dentro = centro(cabeca_do_transform(&h));
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

/// A roda sobre o Transform rola a coluna do Vector, e ele sobe junto, o mesmo tanto que os cartões do Vector.
/// *Mutação: o assento sem a rolagem do anfitrião ⇒ ele fica parado enquanto o resto sobe.*
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
    let (x, y) = centro(cabeca_do_transform(&h));
    assert!(h.chrome_panel_at(x, y).is_some(), "a shell daria zoom");
    let y_de = |h: &HeroScreen, id| h.hit_index.rect_for(id).map(|r| r.y).expect("o campo");
    let (x0, w0) = (
        y_de(&h, insp::INSP_TRANSFORM_SKEW_X),
        y_de(&h, t::VECTOR_TRANSFORM_W),
    );
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
    let rolou = h.store.panel_scroll(v);
    assert!(rolou > 0.0, "a roda foi a outro");
    let (x1, w1) = (
        y_de(&h, insp::INSP_TRANSFORM_SKEW_X),
        y_de(&h, t::VECTOR_TRANSFORM_W),
    );
    assert!(x1 < x0, "o Transform não subiu: {x0} → {x1}");
    // O que rolou para cima das abas não se clica: o recorte do convidado é o da coluna.
    let abas = h
        .hit_index
        .rect_for(p::VECTOR_TAB_DRAWING)
        .expect("as abas");
    for id in [
        CARD_TRANSFORM,
        insp::INSP_TRANSFORM_RESET,
        insp::INSP_TRANSFORM_POS_X,
        insp::INSP_TRANSFORM_SKEW_X,
    ] {
        if let Some(r) = h.hit_index.rect_for(id) {
            assert!(r.y >= abas.y + abas.h, "{id:?} acima das abas: {r:?}");
        }
    }
    // Com a barra de rolagem à vista, a largura é a dos cartões do Vector.
    let (skew, largura) = (
        h.hit_index
            .rect_for(insp::INSP_TRANSFORM_SKEW_X)
            .expect("Skew"),
        h.hit_index.rect_for(t::VECTOR_TRANSFORM_W).expect("W"),
    );
    assert_eq!(skew.x, largura.x, "{skew:?} × {largura:?}");
    assert_eq!(skew.w, largura.w, "{skew:?} × {largura:?}");
    assert!(
        (x0 - x1 - (w0 - w1)).abs() < 0.01,
        "o Transform e o Shape subiram tanto: {} × {}",
        x0 - x1,
        w0 - w1
    );
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

/// O `y` do título de um cartão: o da pega, que é o que a mão agarra.
fn pega(h: &HeroScreen, secao: NodeId) -> ph2d_editor_core::zones::Rect {
    h.hit_index.rect_for(grip_de(secao)).expect("o título")
}

fn transform_abaixo_do_shape(h: &HeroScreen) -> bool {
    pega(h, CARD_TRANSFORM).y > pega(h, t::VECTOR_SECTION_TRANSFORM).y
}

/// Agarra o título de `secao` e anda `dy` px na coluna.
fn arrasta_na_coluna(h: &mut HeroScreen, secao: NodeId, dy: f32) {
    let r = pega(h, secao);
    let de = (r.x + 12.0, r.y + r.h * 0.5);
    arrasta(h, de, (de.0, de.1 + dy));
    quadros(h, 40);
}

/// A distância que leva o Transform para baixo do Shape: a altura dos dois.
fn abaixo_do_shape(h: &HeroScreen) -> f32 {
    let shape = pega(h, t::VECTOR_SECTION_TRANSFORM);
    let lista = h.store.panel_rect(core_ids::VECTOR_PANEL).expect("o rect");
    let seguinte = pega(h, t::VECTOR_SECTION_SNAP).y.min(lista.y + lista.h);
    seguinte - shape.y
}

/// O dono troca o Transform com o Shape pelo título: o lugar muda, os campos clicam no lugar novo e a coluna
/// não fica com dois vãos.
/// *Mutações: o convidado fora da lista do anfitrião ⇒ não desce. `land` sem a ordem ⇒ volta ao topo. A mão que
/// não conduz a pilha do anfitrião ⇒ não desce. O passo sem o `y` do lugar ⇒ o clique fica no topo.*
#[test]
fn arrastar_o_titulo_do_transform_troca_de_lugar_com_o_shape() {
    let mut h = em_draw();
    assert!(!transform_abaixo_do_shape(&h), "nasce o primeiro");
    let dy = abaixo_do_shape(&h);
    arrasta_na_coluna(&mut h, CARD_TRANSFORM, dy);
    assert!(transform_abaixo_do_shape(&h), "não trocou de lugar");
    assert!(h.store.loose_card(CARD_TRANSFORM).is_none(), "não soltou");

    let cabeca = pega(&h, CARD_TRANSFORM);
    let largura = h.hit_index.rect_for(t::VECTOR_TRANSFORM_W).expect("W");
    assert!(
        largura.y + largura.h <= cabeca.y,
        "o Shape inteiro por cima: {largura:?} {cabeca:?}"
    );
    let (corpo, shape) = (
        cabeca_do_transform(&h),
        h.hit_index
            .rect_for(t::VECTOR_SECTION_TRANSFORM)
            .expect("o Shape"),
    );
    assert_eq!((corpo.x, corpo.w), (shape.x, shape.w), "a mesma coluna");
    for id in [insp::INSP_TRANSFORM_POS_X, insp::INSP_TRANSFORM_SKEW_X] {
        let r = h.hit_index.rect_for(id).expect("o campo");
        assert!(r.y > cabeca.y, "{id:?} acima do título dele: {r:?}");
        let (x, y) = centro(r);
        assert_eq!(h.hit_index.hit(x, y), Some(id), "o campo não clica");
    }
    clica(&mut h, insp::INSP_TRANSFORM_RESET);
    assert!(
        h.bus
            .drain()
            .any(|a| matches!(a, EditorAction::InspectorTransformEdit(_))),
        "o reset não pediu nada no lugar novo"
    );
}

/// O vizinho passa por ele: o Shape arrastado para cima do Transform fica por cima.
/// *Mutação: o lugar sem altura ⇒ o vizinho não o passa.*
#[test]
fn arrastar_o_shape_para_cima_do_transform_passa_por_ele() {
    let mut h = em_draw();
    let dy = pega(&h, CARD_TRANSFORM).y - pega(&h, t::VECTOR_SECTION_TRANSFORM).y;
    arrasta_na_coluna(&mut h, t::VECTOR_SECTION_TRANSFORM, dy - 8.0);
    assert!(transform_abaixo_do_shape(&h), "o Shape não passou");
    let largura = h.hit_index.rect_for(t::VECTOR_TRANSFORM_W).expect("W");
    let cabeca = pega(&h, CARD_TRANSFORM);
    assert!(
        largura.y + largura.h <= cabeca.y,
        "sobrepostos: {largura:?} {cabeca:?}"
    );
}

/// A ordem é a das secções, como a dos outros: ao reabrir, fica; e a lista do Inspector não muda.
/// *Mutações: a nova ordem sem a chave do lugar ⇒ reabre no topo. A chave do lugar = a secção do Inspector ⇒ a
/// ordem da lista do Inspector mexe.*
#[test]
fn ao_reabrir_a_ordem_do_transform_fica_e_a_do_inspector_nao_muda() {
    let mut h = em_draw();
    let dy = abaixo_do_shape(&h);
    arrasta_na_coluna(&mut h, CARD_TRANSFORM, dy);
    assert!(
        !h.store.section_order().contains(&CARD_TRANSFORM),
        "a ordem da lista do Inspector mexeu"
    );
    let disco = ph2d_editor_core::interaction::section_prefs_text(&h.store);

    let mut nova = em_draw();
    assert!(!transform_abaixo_do_shape(&nova), "o de fábrica é o topo");
    ph2d_editor_core::interaction::restore_section_prefs_text(&mut nova.store, &disco);
    quadros(&mut nova, 40);
    assert!(transform_abaixo_do_shape(&nova), "a ordem não ficou");
}

/// Depois de trocar de lugar, soltar para fora continua a soltar para a tela, e a faixa fica no lugar novo. Solto,
/// voltar à coluna repõe-no ali, sem mexer na ordem. O cancelar da queda dentro da coluna não troca nada.
/// *Mutações: `larga` com a chave do lugar ⇒ `solta` falha. A queda de fora sem o `detach` do convidado ⇒ idem.*
#[test]
fn soltar_para_fora_continua_a_soltar_e_a_faixa_fica_no_lugar_novo() {
    let mut h = em_draw();
    let dy = abaixo_do_shape(&h);
    arrasta_na_coluna(&mut h, CARD_TRANSFORM, dy);
    solta(&mut h, CARD_TRANSFORM);
    quadros(&mut h, 40);
    assert_eq!(
        h.store.card_guest_h_over(core_ids::VECTOR_PANEL),
        ph2d_card_stack::metrics::LOOSE_GHOST_H
    );
    assert!(
        pintado(&h, loose_pin_of(CARD_TRANSFORM)),
        "alfinete no solto"
    );
    let shape = pega(&h, t::VECTOR_SECTION_TRANSFORM);
    let snap = pega(&h, t::VECTOR_SECTION_SNAP);
    assert!(shape.y < snap.y, "o Vector continua: {shape:?} {snap:?}");

    assert!(
        h.store.loose_cards_of(core_ids::VECTOR_PANEL).is_empty(),
        "o lugar do convidado virou um solto do Vector"
    );

    // Solto, a mão trá-lo de volta à coluna: o lugar é o que ele tinha, e a lista dele não grava nada.
    let rect = h
        .store
        .panel_rect(loose_rect_of(CARD_TRANSFORM))
        .expect("o solto");
    let fantasma = pega(&h, t::VECTOR_SECTION_TRANSFORM);
    arrasta(
        &mut h,
        (rect.x + 40.0, rect.y + 10.0),
        (fantasma.x + 40.0, fantasma.y + 60.0),
    );
    quadros(&mut h, 40);
    assert!(h.store.loose_card(CARD_TRANSFORM).is_none(), "não voltou");
    assert!(pintado(&h, insp::INSP_TRANSFORM_POS_X), "o cartão inteiro");
    assert!(transform_abaixo_do_shape(&h), "voltou a outro lugar");
    assert!(
        !h.store.section_order().contains(&CARD_TRANSFORM),
        "a volta gravou a secção do Inspector"
    );
}

/// O menu do painel repõe as posições: o lugar do convidado é uma entrada da lista, e volta ao topo.
/// *Mutação: as alturas da lista sem o lugar ⇒ a reposição não o vê.*
#[test]
fn repor_as_posicoes_devolve_o_transform_ao_topo() {
    let mut h = em_draw();
    let dy = abaixo_do_shape(&h);
    arrasta_na_coluna(&mut h, CARD_TRANSFORM, dy);
    assert!(transform_abaixo_do_shape(&h));
    h.store
        .reset_card_list(core_ids::VECTOR_PANEL, true, false, false);
    quadros(&mut h, 40);
    assert!(!transform_abaixo_do_shape(&h), "não voltou ao topo");
    assert!(h.store.section_order().is_empty(), "a ordem ficou gravada");
}

/// Ao largar, quem assenta é a mola da lista do Vector, uma só: o cartão pinta-se sempre na origem do lugar.
/// *Mutação: o convidado a assentar com a mola dele ⇒ o título afasta-se do lugar enquanto assenta.*
#[test]
fn ao_largar_o_cartao_assenta_com_uma_mola_so() {
    let mut h = em_draw();
    let v = core_ids::VECTOR_PANEL;
    let base = |h: &HeroScreen, y: f32| {
        let s = h.store.card_guest_seat().expect("o assento");
        s.body_top - h.store.panel_scroll(v).max(0.0) + y
    };
    let y_do_lugar = |h: &HeroScreen| h.store.card_guest_seat().expect("o assento").y;
    let folga = cabeca_do_transform(&h).y - base(&h, y_do_lugar(&h));
    let dy = abaixo_do_shape(&h);
    let r = pega(&h, CARD_TRANSFORM);
    let de = (r.x + 12.0, r.y + r.h * 0.5);
    mao(&mut h, PointerKind::Down, de);
    quadros(&mut h, 2);
    mao(&mut h, PointerKind::Move, (de.0, de.1 + dy));
    quadros(&mut h, 20);
    mao(&mut h, PointerKind::Up, (de.0, de.1 + dy));
    let mut andou = 0.0_f32;
    for _ in 0..20 {
        let antes = y_do_lugar(&h);
        quadros(&mut h, 1);
        let y = cabeca_do_transform(&h).y;
        assert!(
            (y - base(&h, antes) - folga).abs() < 0.01,
            "{y} × {} + {folga}",
            base(&h, antes)
        );
        andou = andou.max((y_do_lugar(&h) - antes).abs());
    }
    assert!(andou > 1.0, "nada assentou: a prova não vale");
}

/// Esc a meio do arrasto: nada muda, nem a ordem gravada, e o cartão volta inteiro. Longe da coluna, Esc também não solta.
/// *Mutações: o anfitrião a pousar a queda cancelada ⇒ a ordem muda. O convidado sem o `!d.cancelled` ⇒ solta.*
#[test]
fn esc_a_meio_do_arrasto_nao_troca_nem_solta() {
    let mut h = em_draw();
    let dy = abaixo_do_shape(&h);
    let canvas = h.last_layout.expect("houve um quadro").canvas;
    let r = pega(&h, CARD_TRANSFORM);
    let de = (r.x + 12.0, r.y + r.h * 0.5);
    for ate in [(de.0, de.1 + dy), (canvas.x + 50.0, canvas.y + 50.0)] {
        mao(&mut h, PointerKind::Down, de);
        quadros(&mut h, 2);
        mao(&mut h, PointerKind::Move, ate);
        quadros(&mut h, 4);
        let _ = h.store.end_card_drag(true);
        quadros(&mut h, 40);
        assert!(!transform_abaixo_do_shape(&h), "trocou com Esc");
        assert!(h.store.section_order().is_empty(), "gravou com Esc");
        assert!(
            h.store.loose_card(CARD_TRANSFORM).is_none(),
            "soltou com Esc"
        );
        assert!(pintado(&h, insp::INSP_TRANSFORM_POS_X), "o cartão voltou");
    }
}

/// A cena do quadro leva os cartões do Transform: dobrá-lo tira o corpo dele do que se pinta.
/// *Mutação: a lista do anfitrião sem colar a cena do convidado ⇒ dobrar não muda nada, e o cartão não se vê.*
#[test]
fn o_cartao_do_transform_entra_na_cena_do_quadro() {
    let caminhos = |h: &mut HeroScreen| {
        let mut texto = ph2d_text::TextSystem::without_system_fonts();
        let mut cena = ph2d_vector::VectorScene::new();
        let janela = ph2d_editor_core::zones::Rect::new(0.0, 0.0, 1600.0, 900.0);
        ph2d_editor_core::aquecimento::com_passo_fixo(
            ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
            || ph2d_editor_core::screens::hero::paint_hero_screen(h, janela, &mut cena, &mut texto),
        );
        cena.inner().encoding().n_paths
    };
    let mut h = em_draw();
    let aberto = caminhos(&mut h);
    h.store.set_collapsed(CARD_TRANSFORM, true);
    quadros(&mut h, 40);
    let dobrado = caminhos(&mut h);
    assert!(aberto > dobrado, "{aberto} × {dobrado}");
}
