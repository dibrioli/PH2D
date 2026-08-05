//! Gates dos **verbos da PELE** — arquivo irmão de `vec_widget_edit.rs` (plano UI/UX W6.2).
//!
//! ⚠️ O gate que este arquivo existe para ter é o do **ROTEADOR**. A W5c pagou esta lição: os
//! chips estavam pintados, registados e o clique chegava ao barramento — e virava **nada**,
//! porque `component_edit_for_id` não os conhecia. Doze gates de projeção e quatro de seam
//! ficaram verdes sobre isso.

use super::*;

fn scene_with(worn: Option<u16>) -> (SimWorld, VecEntityMap, Vec<VecPathId>) {
    let mut sim = SimWorld::default();
    let mut map = VecEntityMap::new();
    let id: VecPathId = 1;
    let mut e = sim
        .world_mut()
        .spawn((ph2d_ecs::Transform::IDENTITY, ph2d_ecs::VecPathRef(id)));
    if let Some(k) = worn {
        e.insert(VecWidget { kind: k });
    }
    map.insert(id, e.id().to_bits());
    (sim, map, vec![id])
}

fn worn_kind(sim: &SimWorld, map: &VecEntityMap, id: VecPathId) -> Option<u16> {
    let &bits = map.get(&id)?;
    sim.world()
        .get::<VecWidget>(Entity::from_bits(bits))
        .map(|w| w.kind)
}

/// **Todo id que a seção pinta VIRA um verbo.** O gate do roteador.
#[test]
fn every_painted_id_becomes_a_verb() {
    assert_eq!(
        widget_edit_for_id(ph2d_editor::ids::VECTOR_WIDGET_WEAR),
        Some(WidgetEdit::Wear)
    );
    assert_eq!(
        widget_edit_for_id(ph2d_editor::ids::VECTOR_WIDGET_REMOVE),
        Some(WidgetEdit::Remove)
    );
    for i in 0..ph2d_editor::ids::MAX_WIDGET_KINDS {
        assert_eq!(
            widget_edit_for_id(ph2d_editor::ids::vector_widget_kind_id(i)),
            Some(WidgetEdit::Kind(i)),
            "o chip {i} chega ao barramento e vira NADA"
        );
    }
    // E um id alheio não é sequestrado.
    assert_eq!(
        widget_edit_for_id(ph2d_editor::ids::VECTOR_COMPONENT_SWAP),
        None
    );
}

/// **A tabela de ids cobre o catálogo INTEIRO.**
///
/// ⚠️ Diferente do irmão `MAX_VARIANT_VALUES`, aqui um teto curto não é uma lista truncada com
/// escape: os tipos além dele ficam **inalcançáveis pelo mouse**, sem conta-gotas por trás.
#[test]
fn the_id_table_covers_the_whole_catalogue() {
    assert!(
        ph2d_editor::ids::MAX_WIDGET_KINDS >= WidgetKind::ALL.len(),
        "o catalogo tem {} tipos e a tabela de ids endereca {} — os do fim seriam inalcancaveis",
        WidgetKind::ALL.len(),
        ph2d_editor::ids::MAX_WIDGET_KINDS
    );
}

/// **Vestir, trocar e despir chegam ao ECS** — a sequência leva a algum lugar.
#[test]
fn the_three_verbs_reach_the_world() {
    let (mut sim, map, sel) = scene_with(None);
    let id = sel[0];
    assert_eq!(worn_kind(&sim, &map, id), None, "nasce sem pele");

    apply(&mut sim, &map, &sel, WidgetEdit::Wear);
    assert_eq!(
        worn_kind(&sim, &map, id),
        Some(WidgetKind::Button.code()),
        "Wear nao vestiu, ou vestiu outro tipo"
    );

    let toggle = WidgetKind::ALL
        .iter()
        .position(|k| *k == WidgetKind::Toggle);
    apply(&mut sim, &map, &sel, WidgetEdit::Kind(toggle.unwrap()));
    assert_eq!(worn_kind(&sim, &map, id), Some(WidgetKind::Toggle.code()));

    apply(&mut sim, &map, &sel, WidgetEdit::Remove);
    assert_eq!(worn_kind(&sim, &map, id), None, "Remove nao despiu");
}

/// **Um índice fora do catálogo não escreve nada** — a forma não pode desaparecer por um chip
/// que o painel nem pintou.
#[test]
fn an_out_of_range_chip_writes_nothing() {
    let (mut sim, map, sel) = scene_with(Some(WidgetKind::Card.code()));
    apply(&mut sim, &map, &sel, WidgetEdit::Kind(9999));
    assert_eq!(
        worn_kind(&sim, &map, sel[0]),
        Some(WidgetKind::Card.code()),
        "um indice invalido reescreveu o tipo"
    );
}

/// **A seção é oferecida para uma forma NUA** — a face vazia, que é a importante.
///
/// ⚠️ Se ela só existisse sobre uma forma já vestida, a feature seria alcançável apenas onde já
/// foi usada — ou seja, em lugar nenhum. É a lei que a seção de física do Inspector documenta.
#[test]
fn the_section_is_offered_to_a_bare_shape() {
    let (sim, map, sel) = scene_with(None);
    let (state, beyond) = publish(&sim, &map, &sel).expect("a forma nua tem seção");
    assert_eq!(state.selected, None, "uma forma nua nao tem tipo aceso");
    assert!(!state.unknown);
    assert_eq!(state.kinds.len(), WidgetKind::ALL.len());
    assert_eq!(beyond, 0, "a tabela de ids cobre o catalogo");
    // ⚠️ Os rótulos passam pelo i18n: um chip que mostrasse a CHAVE crua seria a falha que a
    // regra de HR-15 existe para impedir, e ela não é visível em nenhum outro gate.
    assert!(
        state.kinds.iter().all(|s| !s.contains("panel.vector")),
        "um chip mostrou a chave i18n crua: {:?}",
        state.kinds
    );
}

/// **Vestida com um tipo do FUTURO é um terceiro estado.**
///
/// ⚠️ Colapsá-lo em *"não vestida"* faria o painel oferecer *Wear*, e um clique nele **apagaria
/// em silêncio** o `kind` que o documento carrega — trabalho perdido sem um erro.
#[test]
fn a_future_kind_is_neither_worn_nor_bare() {
    let (sim, map, sel) = scene_with(Some(9999));
    let (state, _) = publish(&sim, &map, &sel).unwrap();
    assert_eq!(state.selected, None, "o tipo do futuro nao acende chip");
    assert!(state.unknown, "o readout de compatibilidade sumiu");
}

/// **Sem seleção única não há seção** — o `apply` também não escreve.
#[test]
fn a_multi_selection_offers_nothing() {
    let (mut sim, map, sel) = scene_with(None);
    let two = vec![sel[0], 2];
    assert!(publish(&sim, &map, &two).is_none());
    apply(&mut sim, &map, &two, WidgetEdit::Wear);
    assert_eq!(
        worn_kind(&sim, &map, sel[0]),
        None,
        "um verbo com dois selecionados escreveu no primeiro"
    );
}

/// Uma cena com um WIDGET vestido e uma FORMA nua, para os gates do vínculo (W8b.3).
fn scene_pair(kind: WidgetKind) -> (SimWorld, VecEntityMap, VecPathId, VecPathId) {
    let mut sim = SimWorld::default();
    let mut map = VecEntityMap::new();
    let (w_id, t_id): (VecPathId, VecPathId) = (1, 2);
    let t = sim
        .world_mut()
        .spawn((
            ph2d_ecs::Transform::IDENTITY,
            ph2d_ecs::VecPathRef(t_id),
            ph2d_ecs::Name::new("Star"),
        ))
        .id();
    map.insert(t_id, t.to_bits());
    let w = sim
        .world_mut()
        .spawn((
            ph2d_ecs::Transform::IDENTITY,
            ph2d_ecs::VecPathRef(w_id),
            ph2d_ecs::Name::new("Opacity"),
            VecWidget { kind: kind.code() },
        ))
        .id();
    map.insert(w_id, w.to_bits());
    (sim, map, w_id, t_id)
}

fn bound_target(sim: &SimWorld, map: &VecEntityMap, w_id: VecPathId) -> Option<VecPathId> {
    let &bits = map.get(&w_id)?;
    sim.world()
        .get::<ph2d_ecs::VecWidgetBind>(Entity::from_bits(bits))
        .map(|b| b.target)
}

/// **O conta-gotas PRENDE, e o Unbind SOLTA.** O par mínimo do gesto.
#[test]
fn the_eyedropper_binds_and_unbind_releases() {
    let (mut sim, map, w, t) = scene_pair(WidgetKind::Slider);
    assert!(bind(&mut sim, &map, w, t), "o vinculo tem de pousar");
    assert_eq!(bound_target(&sim, &map, w), Some(t));
    apply(&mut sim, &map, &[w], WidgetEdit::Unbind);
    assert_eq!(bound_target(&sim, &map, w), None);
}

/// **Um tipo que não DIRIGE recusa o alvo, e o conta-gotas fica armado.**
///
/// ⚠️ A metade que HONRA a pergunta que o painel usa para OFERECER. Aceitar em silêncio daria um
/// vínculo que existe no documento, viaja no save, e não faz nada — pior que a recusa, porque
/// nada na tela diria por quê.
#[test]
fn a_kind_that_cannot_drive_refuses_the_target() {
    let (mut sim, map, w, t) = scene_pair(WidgetKind::Button);
    assert!(!bind(&mut sim, &map, w, t));
    assert_eq!(bound_target(&sim, &map, w), None);
}

/// **Um alvo que VESTE é recusado** — uma row não dirige outra row.
#[test]
fn a_target_that_wears_a_widget_is_refused() {
    let (mut sim, map, w, t) = scene_pair(WidgetKind::Slider);
    let &bits = map.get(&t).expect("o alvo esta' no mapa");
    sim.world_mut()
        .entity_mut(Entity::from_bits(bits))
        .insert(VecWidget {
            kind: WidgetKind::Toggle.code(),
        });
    assert!(!bind(&mut sim, &map, w, t));
    assert_eq!(bound_target(&sim, &map, w), None);
}

/// **DESPIR leva o vínculo junto.**
///
/// ⚠️ Um `VecWidgetBind` sobre uma forma que já não veste é um fio pendurado em nada: ele viajaria
/// no save e ressuscitaria dirigindo no dia em que alguém a vestisse outra vez — uma forma a
/// desvanecer por um controle que ninguém lembra de ter prendido.
#[test]
fn taking_the_skin_off_takes_the_binding_with_it() {
    let (mut sim, map, w, t) = scene_pair(WidgetKind::Slider);
    assert!(bind(&mut sim, &map, w, t));
    apply(&mut sim, &map, &[w], WidgetEdit::Remove);
    assert_eq!(bound_target(&sim, &map, w), None);
}

/// **O painel só oferece a linha do vínculo a quem dirige, e mostra o NOME do alvo.**
#[test]
fn the_panel_offers_the_bind_row_only_where_it_drives() {
    let (mut sim, map, w, t) = scene_pair(WidgetKind::Slider);
    let (st, _) = publish(&sim, &map, &[w]).expect("a secao e' oferecida");
    assert_eq!(st.drives, Some(None), "dirige e ainda nao esta' preso");

    assert!(bind(&mut sim, &map, w, t));
    let (st, _) = publish(&sim, &map, &[w]).expect("a secao e' oferecida");
    assert_eq!(st.drives, Some(Some("Star".to_string())));

    let (mut sim, map, w, _) = scene_pair(WidgetKind::Button);
    let (st, _) = publish(&sim, &map, &[w]).expect("a secao e' oferecida");
    assert_eq!(st.drives, None, "um Button nao produz valor — sem linha");
    let _ = &mut sim;
}
