//! **Os parâmetros da forma em foco — na FILA ②, com a mesma lei de visibilidade** (F9, 11/10).
//!
//! Pedido do dono: *«tirar do inspector as opções de customização de cada shape e colocar nos
//! gizmos»*. A secção `SHAPE_PARAMS` saiu do painel; a alça mora no canvas e o NÚMERO exacto na
//! fila ② (a Context Toolbar do Affinity), com os MESMOS ids do store — um comando, um id.
//!
//! ⚠️ **A lei que esta secção guardava viaja inteira** — o bug do Enio *"ao selecionar a seta, os
//! parâmetros da última forma criada continuam aparecendo"*: sem forma VIVA em foco, os campos
//! caíam no catálogo e penduravam "Star: Points 5" sobre um conector. **É uma classe, não um caso.**
//! O sinal agora é o que o produtor da fila publica ([`bars::tool_bars`]); reverter a regra do foco
//! para `unwrap_or(active)` sai VERMELHO aqui, como saía no painel.

use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::interaction::{ToolBars, WidgetEvent};
use ph2d_editor_core::panel::{ErasedPanel, EventOutcome, Panel, PanelRegistry};
use ph2d_editor_core::screens::hero::{HERO_VIEWPORT_H, HERO_VIEWPORT_W};
use ph2d_editor_core::screens::paint_hero_screen;
use ph2d_editor_core::widget::ToolRailEntry;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_vector::{
    ConnectorSnapshot, VectorPanel, VectorPanelState, bars, set_current_connector,
    set_current_selection_count, set_current_shape_focus, set_current_vector_style,
};
use ph2d_text::TextSystem;
use ph2d_tool_vector::VectorStyleSnapshot;
use ph2d_tool_vector::ids::{MAX_SHAPE_FIELD_SLOTS, vector_shape_choice_id, vector_shape_field_id};
use ph2d_tool_vector::params::DrawMode;
use ph2d_vec_scene::ShapeKind;
use ph2d_vector::VectorScene;
use std::sync::Once;

/// O estado que a shell publica num frame: a forma VIVA em foco (`None` = não é forma), a
/// contagem da seleção, o modo e a forma ATIVA do catálogo.
fn publish(live: Option<ShapeKind>, selected: usize, mode: DrawMode, active: ShapeKind) {
    set_current_shape_focus(live);
    set_current_selection_count(selected);
    set_current_vector_style(Some(VectorStyleSnapshot {
        shape: active,
        mode,
        ..VectorStyleSnapshot::default()
    }));
}

/// A fila ② deste frame, pelo produtor real e sobre o store real do painel.
fn row(host: &ph2d_ui_testkit::MockPanelHost) -> ToolBars {
    bars::tool_bars(host.store(), None, Vec::new())
}

fn entries(b: &ToolBars) -> impl Iterator<Item = &ToolRailEntry> {
    b.row_lead.iter().chain(&b.row_tail)
}

/// Quantos parâmetros de forma a fila leva (o slot numérico OU o chip gémeo de uma escolha).
fn shape_entries(b: &ToolBars) -> usize {
    entries(b)
        .filter_map(ToolRailEntry::node_id)
        .filter(|id| {
            (0..MAX_SHAPE_FIELD_SLOTS)
                .any(|i| *id == vector_shape_field_id(i) || *id == vector_shape_choice_id(i))
        })
        .count()
}

fn shape_row(live: Option<ShapeKind>, selected: usize, mode: DrawMode, active: ShapeKind) -> usize {
    publish(live, selected, mode, active);
    shape_entries(&row(&crate::host_aberto()))
}

/// **O BUG.** Um conector selecionado (a "seta" do Enio) não é forma viva — a fila não leva campo
/// de forma nenhum, em vez de cair no catálogo mostrando os campos da última forma criada.
#[test]
fn selecting_a_connector_hides_the_shape_params_instead_of_showing_the_last_shapes() {
    set_current_connector(Some(ConnectorSnapshot {
        route: 1,
        jetty: 0.35,
        spread: 0.0,
        corner: 0.0,
        curve: 1.0 / 3.0,
    }));
    assert_eq!(
        shape_row(None, 1, DrawMode::Select, ShapeKind::Star),
        0,
        "o conector selecionado NAO e uma forma viva: o campo 'Points' da Estrela editaria o \
         default de desenho enquanto finge editar o objeto selecionado"
    );
    set_current_connector(None);
}

/// **E não é um caso, é uma CLASSE.** Uma curva comum (convertida — sem `VecShape`) também não
/// tem parâmetro nenhum: mesma regra, mesmo resultado.
#[test]
fn selecting_a_plain_curve_hides_the_shape_params_too() {
    set_current_connector(None);
    assert_eq!(shape_row(None, 1, DrawMode::Select, ShapeKind::Star), 0);
}

/// Com a seleção VAZIA **e a ferramenta a desenhar uma forma**, os campos são os da forma armada
/// — o default do próximo traço.
#[test]
fn with_an_empty_selection_the_armed_fields_still_show() {
    set_current_connector(None);
    assert_eq!(
        shape_row(None, 0, DrawMode::Shape, ShapeKind::Star),
        ph2d_tool_vector::shapes::desc(ShapeKind::Star).fields.len(),
        "armado para desenhar, os campos sao o default do proximo traco"
    );
}

/// ⭐⭐⭐ **O REPORT (Enio, 2026-08-31, com foto):** *"Mesmo com outras ferramentas selecionadas, as
/// Shapes ficam expostas e as propriedades das shapes também."* A pergunta nunca foi quantos
/// objetos estão selecionados: é se a ferramenta na mão vai desenhar uma forma.
#[test]
fn an_empty_selection_in_a_tool_that_draws_nothing_shows_no_shape_fields() {
    set_current_connector(None);
    for mode in [
        DrawMode::Select,
        DrawMode::Node,
        DrawMode::Pen,
        DrawMode::Pencil,
        DrawMode::Cut,
        DrawMode::Width,
    ] {
        assert_eq!(
            shape_row(None, 0, mode, ShapeKind::Star),
            0,
            "{mode:?} nao desenha forma nenhuma — os parametros da Estrela nao tem o que descrever"
        );
    }
    // Controle: a MESMA seleção vazia nas duas ferramentas que DESENHAM uma forma mostra os
    // campos — senão este gate passaria por a fila estar morta, e não pela lei.
    for mode in [DrawMode::Shape, DrawMode::Frame] {
        assert!(
            shape_row(None, 0, mode, ShapeKind::Star) > 0,
            "{mode:?} desenha uma forma — os campos dela tem de estar na fila"
        );
    }
}

/// A exceção: **armado para desenhar** (modo Shape), os campos são o default do próximo traço
/// mesmo com algo selecionado — escolher a forma no catálogo já põe a tool em Shape, sem limpar a
/// seleção.
#[test]
fn arming_a_shape_keeps_the_catalog_fields_even_with_a_connector_selected() {
    set_current_connector(Some(ConnectorSnapshot {
        route: 0,
        jetty: 0.35,
        spread: 0.0,
        corner: 0.0,
        curve: 1.0 / 3.0,
    }));
    assert!(shape_row(None, 1, DrawMode::Shape, ShapeKind::Star) > 0);
    set_current_connector(None);
}

/// O caso que define a feature Live Shape: uma forma VIVA selecionada traz os campos DELA, mesmo
/// na ferramenta Select (a seleção manda sobre o catálogo).
#[test]
fn a_selected_live_shape_still_shows_its_own_fields() {
    set_current_connector(None);
    assert_eq!(
        shape_row(
            Some(ShapeKind::Star),
            1,
            DrawMode::Select,
            ShapeKind::Polygon
        ),
        ph2d_tool_vector::shapes::desc(ShapeKind::Star).fields.len(),
        "a forma viva selecionada tem de trazer os campos DELA (o ciclo Live Shape), e nao os do \
         Poligono do catalogo"
    );
}

/// ⭐⭐ **A MOLDURA desenha um `RoundRect`, e os campos têm de ser DELE** — não os do botão aceso
/// do catálogo (a semente da shell já escreve os VALORES do `RoundRect` nesses mesmos slots).
#[test]
fn the_frame_mode_shows_the_round_rect_fields_not_the_lit_catalog_buttons() {
    use ph2d_tool_vector::shapes;
    let cru = shapes::desc(ShapeKind::Rectangle).fields.len();
    let efectivo = shapes::desc(ShapeKind::RoundRect).fields.len();
    assert_ne!(
        cru, efectivo,
        "a fixture precisa de duas formas distinguiveis"
    );
    set_current_connector(None);
    assert_eq!(
        shape_row(None, 0, DrawMode::Frame, ShapeKind::Rectangle),
        efectivo,
        "a Moldura levou os campos da forma CRUA do catalogo ({cru}) em vez dos do RoundRect \
         que o gesto desenha ({efectivo})"
    );
}

/// ⭐⭐ **A Estrela na fila: Points · Inner · Tip round · Notch round**, cada um um CAMPO numérico no
/// id do slot que a shell semeia (`vector_shape_field_id(0..4)`), pela ordem do descritor — e à
/// frente dos botões do gesto (um campo atrás do `⋯` não se edita).
#[test]
fn a_star_in_focus_puts_its_number_fields_in_the_row() {
    set_current_connector(None);
    publish(Some(ShapeKind::Star), 1, DrawMode::Select, ShapeKind::Star);
    let b = row(&crate::host_aberto());
    let fields: Vec<(NodeId, &str)> = b
        .row_lead
        .iter()
        .filter_map(|e| match e {
            ToolRailEntry::Number { id, label, .. }
                if (0..MAX_SHAPE_FIELD_SLOTS).any(|i| *id == vector_shape_field_id(i)) =>
            {
                Some((*id, label.as_str()))
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        fields,
        vec![
            (vector_shape_field_id(0), "Points"),
            (vector_shape_field_id(1), "Inner"),
            (vector_shape_field_id(2), "Tip round"),
            (vector_shape_field_id(3), "Notch round"),
            // F10: o 5.º (o tipo do canto) é uma ESCOLHA — um chip que cicla, não um número.
            (vector_shape_field_id(5), "Curve"),
            (vector_shape_field_id(6), "Smooth"),
            (vector_shape_field_id(7), "Twist"),
        ]
    );
    assert!(
        b.row_tail
            .iter()
            .filter_map(ToolRailEntry::node_id)
            .all(|id| (0..MAX_SHAPE_FIELD_SLOTS).all(|i| id != vector_shape_field_id(i))),
        "um campo de forma na cauda vai para o `⋯`, onde nada se edita"
    );
}

/// ⭐⭐ **Uma ESCOLHA é um chip que cicla, não um número** — o `Viewed` do cubo isométrico é um
/// chip no id GÉMEO (`vector_shape_choice_id(2)`) cuja face é a opção corrente; o slot numérico
/// fica só como depósito (não vai à fila). E o clique nele, pela porta do painel, cicla o valor e
/// a face seguinte o mostra.
#[test]
fn an_iso_cube_shows_viewed_as_a_cycling_chip_on_the_twin_id() {
    set_current_connector(None);
    publish(
        Some(ShapeKind::IsoCube),
        1,
        DrawMode::Select,
        ShapeKind::IsoCube,
    );
    let mut host = crate::host_aberto();
    let face = |b: &ToolBars| {
        entries(b).find_map(|e| match e {
            ToolRailEntry::Compound { id, face, .. } if *id == vector_shape_choice_id(2) => {
                Some(face.clone())
            }
            _ => None,
        })
    };
    let b = row(&host);
    assert_eq!(face(&b).as_deref(), Some("From above"));
    let ids: Vec<NodeId> = entries(&b).filter_map(ToolRailEntry::node_id).collect();
    assert!(ids.contains(&vector_shape_field_id(0)) && ids.contains(&vector_shape_field_id(1)));
    assert!(
        !ids.contains(&vector_shape_field_id(2)),
        "a escolha nao e um numero: o slot 2 e so o deposito do valor"
    );
    let out = host.apply_panel_event::<VectorPanel>(
        &mut VectorPanelState,
        WidgetEvent::Click(vector_shape_choice_id(2)),
    );
    assert_eq!(
        out,
        EventOutcome::Consumed,
        "o chip pinta na fila e o painel nao o despacha"
    );
    assert!(
        !host.drained_actions().is_empty(),
        "o clique nunca virou ToolPanelEvent"
    );
    assert_eq!(face(&row(&host)).as_deref(), Some("From below"));
}

/// O painel Vector, docado e visível (é o que a shell faz enquanto a tool está ativa).
fn hero_with_vector_panel() -> HeroScreen {
    static INIT: Once = Once::new();
    ph2d_editor_core::test_support::ensure_panel_registry();
    INIT.call_once(|| {
        let mut reg = PanelRegistry::new_empty();
        reg.push(ErasedPanel::new::<VectorPanel>());
        let _ = ph2d_editor_core::panel::install_panel_registry(reg);
    });
    let mut hero = HeroScreen::new(NodeId(1));
    hero.panel_visibility.insert(VectorPanel::ID, true);
    hero
}

/// ⛔ **A secção SAIU do painel** — com uma forma viva em foco (o caso em que ela aparecia), o
/// painel não pinta cabeçalho nem campo de forma: o número exacto mora na fila ②, nunca em duas
/// casas (*um comando, uma porta*). Pelo `paint` de verdade (`paint_hero_screen`, headless).
#[test]
fn the_panel_no_longer_paints_the_shape_params() {
    let mut hero = hero_with_vector_panel();
    set_current_connector(None);
    publish(Some(ShapeKind::Star), 1, DrawMode::Select, ShapeKind::Star);
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    paint_hero_screen(
        &mut hero,
        Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H),
        &mut scene,
        &mut text,
    );
    assert!(
        hero.hit_index
            .rect_for(ph2d_tool_vector::ids::VECTOR_SECTION_SHAPE_PARAMS)
            .is_none()
    );
    for i in 0..MAX_SHAPE_FIELD_SLOTS {
        assert!(
            hero.hit_index.rect_for(vector_shape_field_id(i)).is_none()
                && hero.hit_index.rect_for(vector_shape_choice_id(i)).is_none(),
            "o slot {i} da forma continua pintado no painel"
        );
    }
    // Controle: o MESMO quadro leva os campos na fila — senão o gate passaria por nada existir.
    assert!(shape_entries(&row(&crate::host_aberto())) > 0);
}

/// ⑤ **A dica ensina as alças no modo FORMA** (F10): com a forma VIVA de alças em foco (as alças à
/// vista), a dica diz os modificadores delas e que o vazio desenha outra; armado no catálogo (foco
/// vivo `None`), com uma forma sem parâmetros (o rectângulo) ou no Select, a dica de sempre.
#[test]
fn the_shape_hint_teaches_the_handles_only_while_they_are_on_screen() {
    let snap = |mode| VectorStyleSnapshot {
        mode,
        ..VectorStyleSnapshot::default()
    };
    publish(Some(ShapeKind::Star), 1, DrawMode::Shape, ShapeKind::Star);
    assert_eq!(
        bars::hint_key_for(&snap(DrawMode::Shape)),
        "panel.vector.hint.shape_handles"
    );
    // No Select não há alças de forma (F10): a dica do Select.
    assert_eq!(
        bars::hint_key_for(&snap(DrawMode::Select)),
        "panel.vector.hint.select"
    );
    // Armado no catálogo: a shell publica foco vivo `None` — sem alças, a dica do desenho.
    publish(None, 1, DrawMode::Shape, ShapeKind::Star);
    assert_eq!(
        bars::hint_key_for(&snap(DrawMode::Shape)),
        "panel.vector.hint.shape"
    );
    // Uma forma sem alças (a 1.ª do catálogo que não tem nenhuma, se ainda houver).
    if let Some(&bare) = ph2d_vec_scene::ALL_SHAPES
        .iter()
        .find(|&&k| ph2d_tool_vector::shape_handles::handles_of(k).is_empty())
    {
        publish(Some(bare), 1, DrawMode::Shape, ShapeKind::Star);
        assert_eq!(
            bars::hint_key_for(&snap(DrawMode::Shape)),
            "panel.vector.hint.shape",
            "{bare:?} não tem alças"
        );
    }
    assert_ne!(
        ph2d_i18n::tr("panel.vector.hint.shape_handles"),
        "panel.vector.hint.shape_handles",
        "a dica nova não tem texto"
    );
}
