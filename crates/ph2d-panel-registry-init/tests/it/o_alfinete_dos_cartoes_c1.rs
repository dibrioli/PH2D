//! ⭐ **C1 — o alfinete em Object, Appearance, Ordering e Material** (`docs/CartoesUI/09`), por gesto
//! real no hero montado: prende-se a A, escolhe-se outra coisa, edita-se no solto, e o pedido é de A
//! e não se espalha, com a seleção em B, em B+C e em A+B. A aplicação no mundo e o desfazer têm o
//! gate deles na shell (`fase_bus_inspector_preso_tests.rs`).
#![cfg(all(feature = "panel-inspector", feature = "panel-hierarchy"))]

use super::cartoes_mao::{centro, clica, mao, pinta, solta};
use super::o_bloco5_abertos::escolhe;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::action_bus::{EditorAction, HierRequest};
use ph2d_editor_core::ids::{INSP_ENTITY_NAME, INSP_PANEL, hier_lock_companion};
use ph2d_editor_core::interaction::{dispatch_text_input, loose_pin_of, loose_rect_of};
use ph2d_editor_core::screens::hero::{
    BlendFieldEdit, HeroScreen, InspectorBlendInfo, InspectorOrderingInfo, InspectorVisibilityInfo,
    OrderingFieldEdit,
};
use ph2d_editor_core::tool::PanelEvent;
use ph2d_host::PointerKind;
use ph2d_panel_inspector as insp;
use ph2d_panel_inspector::ids as pids;
use ph2d_panel_inspector::{CARD_APPEARANCE, CARD_MATERIAL, CARD_OBJECT, CARD_ORDERING, DD_BLEND};
use ph2d_tool_vector::appearance::{self, Appearance};
use ph2d_tool_vector::ids as vids;
use ph2d_vec_scene::BlendMode;

const A: u64 = 21;
const B: u64 = 22;
const C: u64 = 23;

fn linha(b: u64) -> NodeId {
    NodeId(0xC100 + b)
}

/// O que a shell publica de um objecto; A difere de B e C em tudo o que os cartões mostram.
fn arma(b: u64) {
    let a = b == A;
    escolhe(b, if a { "Circle" } else { "Square" }, b as f32);
    insp::set_current_inspector_ordering(Some(InspectorOrderingInfo {
        entity_bits: b,
        z_index: a.then_some(5),
        z_as_relative: false,
        show_behind_parent: false,
        sorting_layer: 0,
        order_in_layer: 0,
        y_sort_enabled: false,
        y_sort_point: 0,
        y_sort_axis: [0.0, 0.0],
        sorting_group: false,
        sort_at_root: false,
        top_level: false,
        selected_count: 1,
        mixed: Default::default(),
    }));
    insp::set_current_inspector_blend(Some(InspectorBlendInfo {
        entity_bits: b,
        blend_tag: u8::from(a),
        selected_count: 1,
        mixed: Default::default(),
    }));
    insp::set_current_inspector_visibility(Some(InspectorVisibilityInfo {
        entity_bits: b,
        visible: !a,
        mixed: false,
    }));
    appearance::set_current_appearance(Some(Appearance {
        opacity: if a { 0.3 } else { 0.8 },
        blend: if a {
            BlendMode::Multiply
        } else {
            BlendMode::Normal
        },
        layers: Vec::new(),
        mixed_opacity: false,
        mixed_blend: false,
        layers_differ: false,
        order: Vec::new(),
    }));
}

/// A seleção de `sel` publicada como a shell faz (primário à frente) e o retrato de cada preso.
fn quadros(h: &mut HeroScreen, sel: &[u64], n: usize) {
    for _ in 0..n {
        insp::com_cada_preso(&h.store.loose_pins_of(INSP_PANEL), arma);
        arma(sel[0]);
        insp::set_current_inspector_selecionados(sel.len());
        pinta(h, 1);
    }
}

/// O cartão `sec` solto e preso a A, com a seleção em `sel`.
fn preso(sec: NodeId, sel: &[u64]) -> HeroScreen {
    let _ = ph2d_panel_registry_init::register_all_panels();
    insp::set_cards_on(true);
    insp::set_current_inspector_sprite(None);
    arma(A);
    let mut h = HeroScreen::new(NodeId(1));
    let ids: Vec<&'static str> = ph2d_editor_core::panel::with_registry_ref(|reg| {
        reg.panels().iter().map(|p| p.manifest.id).collect()
    });
    for n in ids {
        h.panel_visibility.insert(n, n == "inspector");
    }
    let mut filas = std::collections::BTreeMap::new();
    for b in [A, B, C] {
        let mut l = ph2d_editor_core::screens::hero::fixture::hierarchy()
            .into_iter()
            .next()
            .expect("a fixtura tem uma linha");
        l.entity = Some(b);
        filas.insert(linha(b), l);
    }
    let ordem = [linha(A), linha(B), linha(C)];
    ph2d_panel_hierarchy::sync_from_hierarchy(&mut h.store, &ordem, filas);
    h.gizmo.replace_selection(Some(A));
    h.store
        .set_card_subject(INSP_PANEL, Some((A, "Circle".to_owned())));
    h.store.set_collapsed(CARD_ORDERING, false);
    h.store.set_collapsed(CARD_MATERIAL, false);
    // Os cartões de cima fechados: Ordering e Material ficam acima da dobra para a mão os apanhar.
    h.store
        .set_collapsed(insp::CARD_TRANSFORM, sec != insp::CARD_TRANSFORM);
    h.store
        .set_collapsed(CARD_APPEARANCE, sec != CARD_APPEARANCE);
    pinta(&mut h, 30);
    solta(&mut h, sec);
    clica(&mut h, loose_pin_of(sec));
    assert_eq!(h.store.loose_card(sec).and_then(|c| c.pin), Some(A));
    h.gizmo.replace_selection(Some(sel[0]));
    for &b in &sel[1..] {
        h.gizmo.add_to_selection(b);
    }
    quadros(&mut h, sel, 6);
    h
}

/// O centro de `id` no solto `sec` (se o rect não cair nele, o gesto não é do preso).
fn no_solto(h: &HeroScreen, sec: NodeId, id: NodeId) -> (f32, f32) {
    let r = h
        .hit_index
        .rect_for(id)
        .unwrap_or_else(|| panic!("{id:?} não pintou"));
    let s = h.store.panel_rect(loose_rect_of(sec)).expect("o solto");
    let c = centro(r);
    assert!(
        s.contains(c.0, c.1),
        "{id:?} em {r:?} não está no solto {s:?}"
    );
    c
}

/// A guarda: num gesto do preso, a seleção não recebe nada e as formas são a dele.
fn nao_se_espalha(h: &HeroScreen, sel: &[u64]) {
    assert_eq!(
        h.store.card_edit_subject(),
        Some(A),
        "o gesto não é do preso"
    );
    let n = sel.len();
    let fan = h.store.card_fan_out(n, sel.iter().copied());
    assert!(fan.is_empty(), "espalha por {fan:?}");
}

fn drena(h: &mut HeroScreen) -> Vec<EditorAction> {
    h.bus.drain().collect()
}

/// Foca a caixa `id` do solto com um clique e digita `v`.
fn digita(h: &mut HeroScreen, sec: NodeId, id: NodeId, v: &str) {
    let c = no_solto(h, sec, id);
    mao(h, PointerKind::Down, c);
    mao(h, PointerKind::Up, c);
    assert_eq!(h.store.focus_id(), Some(id), "a caixa não tomou o foco");
    let arena = bumpalo::Bump::new();
    for ch in v.chars() {
        let evs = dispatch_text_input(&mut h.store, ch, &arena).to_vec();
        for e in evs {
            h.apply_event(e);
        }
    }
}

/// Larga o foco como a mão larga: uma pressão no canvas (é ela que dispara o commit).
fn clica_no_canvas(h: &mut HeroScreen) {
    let c = h.last_layout.expect("houve um quadro").canvas;
    let fora = (c.x + c.w * 0.9, c.y + c.h * 0.9);
    mao(h, PointerKind::Down, fora);
    mao(h, PointerKind::Up, fora);
}

const SELECOES: [&[u64]; 3] = [&[B], &[B, C], &[A, B]];

/// ⭐ Material: cada modo edita A e não se espalha. *Mutação: `card_fan_out` sem a guarda ⇒ a seleção recebe.*
#[test]
fn every_material_mode_on_the_pinned_card_edits_only_its_object() {
    for sel in SELECOES {
        let mut h = preso(CARD_MATERIAL, sel);
        for (i, &id) in pids::INSP_SAMPLE_BLEND.iter().enumerate() {
            let c = no_solto(&h, CARD_MATERIAL, id);
            drena(&mut h);
            mao(&mut h, PointerKind::Down, c);
            mao(&mut h, PointerKind::Up, c);
            nao_se_espalha(&h, sel);
            let feitos: Vec<_> = drena(&mut h)
                .into_iter()
                .filter_map(|a| match a {
                    EditorAction::InspectorBlendEdit { entity_bits, edit } => {
                        Some((entity_bits, edit))
                    }
                    _ => None,
                })
                .collect();
            assert_eq!(
                feitos,
                [(A, BlendFieldEdit::Blend(i as u8))],
                "{sel:?}, modo {i}"
            );
            quadros(&mut h, sel, 2);
        }
    }
}

/// ⭐ Ordering: o preso MOSTRA o Z dele, e digitar edita-o a ele, mesmo que o commit venha da pressão
/// fora do cartão. *Mutação: `blur_subject` fora ⇒ o commit vai à seleção com o número de A.*
#[test]
fn the_z_typed_on_the_pinned_card_goes_to_its_object_when_the_focus_leaves() {
    for sel in SELECOES {
        let mut h = preso(CARD_ORDERING, sel);
        let do_preso = h
            .store
            .pinned_states(CARD_ORDERING)
            .and_then(|e| e.estado(pids::INSP_ORDER_Z_INDEX));
        assert!(
            matches!(
                do_preso,
                Some(ph2d_editor_core::interaction::InteractiveState::NumberInput { value, .. }) if *value == 5.0
            ) || sel[0] == A,
            "o Z do preso não é o de A: {do_preso:?}"
        );
        digita(&mut h, CARD_ORDERING, pids::INSP_ORDER_Z_INDEX, "7");
        drena(&mut h);
        clica_no_canvas(&mut h);
        assert_eq!(h.store.card_press_subject(), None, "a pressão foi fora");
        nao_se_espalha(&h, sel);
        let feitos: Vec<_> = drena(&mut h)
            .into_iter()
            .filter_map(|a| match a {
                EditorAction::InspectorOrderingEdit { entity_bits, edit } => {
                    Some((entity_bits, edit))
                }
                _ => None,
            })
            .collect();
        assert_eq!(feitos.len(), 1, "{sel:?}: {feitos:?}");
        assert_eq!(feitos[0].0, A, "{sel:?}");
        assert!(
            matches!(feitos[0].1, OrderingFieldEdit::ZIndex(Some(_))),
            "{feitos:?}"
        );
        h.store.end_card_edit_frame();
        assert_eq!(
            h.store.card_edit_subject(),
            None,
            "o sujeito do commit acabou com o quadro"
        );
    }
}

/// ⭐ Object: nome, olho e cadeado são de A. O olho parte do estado de A (escondido) e não da
/// seleção (visível). *Mutação: o olho a ler a caixa da seleção ⇒ ícone e pedido trocados.*
#[test]
fn name_eye_and_lock_of_the_pinned_object_card_are_of_its_object() {
    for sel in SELECOES {
        let mut h = preso(CARD_OBJECT, sel);
        // Olho.
        let c = no_solto(&h, CARD_OBJECT, pids::INSP_VISIBILITY_CHECK);
        drena(&mut h);
        mao(&mut h, PointerKind::Down, c);
        mao(&mut h, PointerKind::Up, c);
        nao_se_espalha(&h, sel);
        let olho: Vec<_> = drena(&mut h)
            .into_iter()
            .filter_map(|a| match a {
                EditorAction::InspectorVisibilityEdit(i) => Some((i.entity_bits, i.visible)),
                _ => None,
            })
            .collect();
        assert_eq!(
            olho,
            [(A, true)],
            "{sel:?}: A está escondido, o olho mostra-o"
        );
        quadros(&mut h, sel, 2);
        // Cadeado: a linha de A, nunca a da seleção.
        let c = no_solto(&h, CARD_OBJECT, hier_lock_companion(linha(A)));
        drena(&mut h);
        mao(&mut h, PointerKind::Down, c);
        mao(&mut h, PointerKind::Up, c);
        let cadeado: Vec<_> = drena(&mut h)
            .into_iter()
            .filter_map(|a| match a {
                EditorAction::Hierarchy(HierRequest::ToggleLock { row }) => Some(row),
                _ => None,
            })
            .collect();
        assert_eq!(cadeado, [linha(A)], "{sel:?}");
        assert!(
            h.store.tooltip_for(hier_lock_companion(linha(A))).is_some(),
            "o cadeado do preso sem dica"
        );
        quadros(&mut h, sel, 2);
        // Nome: a tecla e o commit na saída do foco.
        digita(&mut h, CARD_OBJECT, INSP_ENTITY_NAME, "z");
        let tecla: Vec<_> = drena(&mut h)
            .into_iter()
            .filter_map(|a| match a {
                EditorAction::InspectorNameEdit(i) => Some(i.entity_bits),
                _ => None,
            })
            .collect();
        assert!(
            !tecla.is_empty() && tecla.iter().all(|b| *b == A),
            "{sel:?}: {tecla:?}"
        );
        clica_no_canvas(&mut h);
        let saida = drena(&mut h);
        assert!(
            saida.iter().all(|a| !matches!(
                a,
                EditorAction::InspectorNameEdit(i) if i.entity_bits != A
            )),
            "{sel:?}: {saida:?}"
        );
    }
}

/// ⭐ Appearance: Opacity e Blend pelo painel Vector (não levam `entity_bits`), o olho pelo Inspector.
/// O preso mostra os de A; a guarda diz que o pedido é dele. *Mutação: `aparencia` fora do retrato
/// ⇒ a caixa mostra a da seleção.*
#[test]
fn opacity_blend_and_eye_of_the_pinned_appearance_card_are_of_its_object() {
    for sel in SELECOES {
        let mut h = preso(CARD_APPEARANCE, sel);
        let num = vids::VECTOR_OBJ_OPACITY_NUM;
        let mostrada = match h
            .store
            .pinned_states(CARD_APPEARANCE)
            .and_then(|e| e.estado(num))
        {
            Some(ph2d_editor_core::interaction::InteractiveState::NumberInput {
                value, ..
            }) => *value,
            _ => h.store.number_value(num).unwrap_or(-1.0),
        };
        assert_eq!(mostrada, 30.0, "{sel:?}: a Opacity do preso não é a de A");
        // A foto C1: com o Vector atrás do Inspector, a caixa da seleção é a da forma escolhida.
        let da_selecao = if sel[0] == A { 30.0 } else { 80.0 };
        assert_eq!(h.store.number_value(num), Some(da_selecao), "{sel:?}");
        // Opacity digitada: a pressão fora dispara o commit, e ele é de A.
        digita(&mut h, CARD_APPEARANCE, num, "4");
        drena(&mut h);
        clica_no_canvas(&mut h);
        nao_se_espalha(&h, sel);
        let op = drena(&mut h).into_iter().find_map(|a| match a {
            EditorAction::ToolPanelEvent(PanelEvent::SetValue(id, v))
                if id == vids::VECTOR_OBJ_OPACITY =>
            {
                Some(v)
            }
            _ => None,
        });
        assert!(op.is_some(), "{sel:?}: a Opacity não pediu nada");
        h.store.end_card_edit_frame();
        quadros(&mut h, sel, 2);
        // Blend: abre a lista no preso, escolhe uma opção.
        let c = no_solto(&h, CARD_APPEARANCE, DD_BLEND);
        mao(&mut h, PointerKind::Down, c);
        mao(&mut h, PointerKind::Up, c);
        quadros(&mut h, sel, 2);
        let opcao = h
            .hit_index
            .rect_for(vids::vector_obj_blend_option_id(1))
            .expect("a lista abriu");
        drena(&mut h);
        mao(&mut h, PointerKind::Down, centro(opcao));
        mao(&mut h, PointerKind::Up, centro(opcao));
        nao_se_espalha(&h, sel);
        let bl = drena(&mut h).into_iter().find_map(|a| match a {
            EditorAction::ToolPanelEvent(PanelEvent::SetValue(id, v))
                if id == vids::VECTOR_OBJ_BLEND =>
            {
                Some(v)
            }
            _ => None,
        });
        assert!(bl.is_some(), "{sel:?}: o Blend não pediu nada");
        quadros(&mut h, sel, 2);
        // Olho no cabeçalho.
        let c = no_solto(&h, CARD_APPEARANCE, pids::INSP_VISIBILITY_CHECK);
        drena(&mut h);
        mao(&mut h, PointerKind::Down, c);
        mao(&mut h, PointerKind::Up, c);
        nao_se_espalha(&h, sel);
        let olho: Vec<_> = drena(&mut h)
            .into_iter()
            .filter_map(|a| match a {
                EditorAction::InspectorVisibilityEdit(i) => Some((i.entity_bits, i.visible)),
                _ => None,
            })
            .collect();
        assert_eq!(olho, [(A, true)], "{sel:?}");
    }
}

/// Os botões de girar do Transform preso são dele: o pedido sai sem `entity_bits`, e a forma é a do
/// sujeito (`paint_stack::alvos`, gate na shell). *Mutação: voltar a `pen.selected()` na shell.*
#[test]
fn rotate_and_flip_on_the_pinned_transform_are_of_its_object() {
    for sel in SELECOES {
        let mut h = preso(insp::CARD_TRANSFORM, sel);
        for id in [
            vids::VECTOR_ARRANGE_ROTATE_CW,
            vids::VECTOR_ARRANGE_FLIP_H,
            vids::VECTOR_ARRANGE_FLIP_V,
        ] {
            let c = no_solto(&h, insp::CARD_TRANSFORM, id);
            drena(&mut h);
            mao(&mut h, PointerKind::Down, c);
            mao(&mut h, PointerKind::Up, c);
            nao_se_espalha(&h, sel);
            let pediu = drena(&mut h).into_iter().any(
                |a| matches!(a, EditorAction::ToolPanelEvent(PanelEvent::Click(i)) if i == id),
            );
            assert!(pediu, "{sel:?}: {id:?} não pediu nada");
            quadros(&mut h, sel, 2);
        }
    }
}

/// Contraprova: o mesmo cartão SEM alfinete segue a seleção e espalha-se por ela. Objecto apagado:
/// o alfinete cai (a chave durável fica) e o gesto volta a ser da seleção.
#[test]
fn an_unpinned_card_fans_out_and_a_deleted_object_lets_go() {
    let sel: &[u64] = &[B, C];
    let mut h = preso(CARD_MATERIAL, sel);
    h.store.rebind_loose_pin(CARD_MATERIAL, None);
    quadros(&mut h, sel, 3);
    assert_eq!(h.store.loose_card(CARD_MATERIAL).and_then(|c| c.pin), None);
    let c = no_solto(&h, CARD_MATERIAL, pids::INSP_SAMPLE_BLEND[2]);
    drena(&mut h);
    mao(&mut h, PointerKind::Down, c);
    mao(&mut h, PointerKind::Up, c);
    assert_eq!(h.store.card_edit_subject(), None);
    assert_eq!(h.store.card_fan_out(2, sel.iter().copied()), sel);
    let e = drena(&mut h).into_iter().find_map(|a| match a {
        EditorAction::InspectorBlendEdit { entity_bits, .. } => Some(entity_bits),
        _ => None,
    });
    assert_eq!(e, Some(B), "o cartão solto sem alfinete é da seleção");
}
