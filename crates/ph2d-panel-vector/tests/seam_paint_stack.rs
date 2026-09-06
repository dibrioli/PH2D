//! ⭐⭐⭐ **SEAM DA PILHA DE APARÊNCIA** (estudo 42 item 4) — o gesto REAL sobre cada controlo de
//! uma camada, e o que ele faz chegar ao barramento.
//!
//! ⚠️ **Nasceu do report do Enio de 2026-09-05:** *"O olho não funciona. Clicar no nome não exibe
//! a largura, a opacidade e a mistura. Demais botões não funcionam"* — a pilha shipou com o
//! modelo, o transform, o renderer, o exportador e o painel gateados, e **zero** gates no caminho
//! do clique. Os controlos eram pintados, hit-indexados, registados no `populate` (logo focáveis,
//! logo ACENDIAM sob o rato) e o `Click` morria no `apply_event` do painel, porque nenhum id da
//! família estava na allowlist que o encaminha.
//!
//! ⛔ **Nenhum gate existente podia ver isto**, e a cegueira é estrutural: o
//! `hit_indexed_ids_are_registered` mede FOCABILIDADE (e estava verde — os ids estão todos
//! registados), e os testes de unidade do `vec_paint_stack` medem o que a shell faz **depois** de
//! receber o verbo. Entre os dois há um passo que nenhum deles atravessa: *o clique sai do
//! painel?* É esse que este ficheiro mede, e é o único sítio onde ele é observável.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_vector::state::{Appearance, PaintRow, VectorPanelState};
use ph2d_panel_vector::{VectorPanel, ids, state};
use ph2d_ui_testkit::MockPanelHost;
use ph2d_vec_scene::BlendMode;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};
const SEC: u128 = 1_000_000_000;

fn pointer(kind: PointerKind, x: f32, y: f32, t: u128) -> PointerEvent {
    PointerEvent {
        kind,
        x,
        y,
        button: PointerButton::Primary,
        source: PointerSource::Mouse,
        pressure: 1.0,
        timestamp_ns: t,
    }
}

/// Uma camada de contorno (a que tem as TRÊS propriedades — a de preenchimento não tem largura).
fn traco(w: f64) -> PaintRow {
    PaintRow {
        is_fill: false,
        color: [0x33, 0x66, 0xCC, 0xFF],
        width: w,
        enabled: true,
        opacity: 1.0,
        blend: BlendMode::Normal,
        offset: [0.0, 0.0],
    }
}

fn tinta() -> PaintRow {
    PaintRow {
        is_fill: true,
        color: [0xCC, 0x33, 0x33, 0xFF],
        width: 0.0,
        enabled: true,
        opacity: 1.0,
        blend: BlendMode::Normal,
        offset: [0.0, 0.0],
    }
}

/// Publica uma pilha de duas camadas (uma tinta no chão, um traço por cima).
fn publica(layers: Vec<PaintRow>) {
    state::set_current_appearance(Some(Appearance {
        opacity: 1.0,
        blend: BlendMode::Normal,
        layers,
    }));
}

/// **O gesto REAL sobre um retângulo pintado**, e o que ele deixa no barramento.
///
/// Down+Up de verdade, e não um `WidgetEvent::Click` sintético: o sintético prova a allowlist do
/// painel mas **pula a checagem de focabilidade no store** — as duas metades falham de maneiras
/// diferentes e um gate que só faz uma delas fica verde sobre a outra.
fn clica(id: ph2d_a11y::NodeId, what: &str) -> Vec<EditorAction> {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    let r = host
        .painted_rect::<VectorPanel>(&mut st, VIEWPORT, id)
        .unwrap_or_else(|| panic!("{what} nao foi PINTADO com area clicavel"));
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
    assert!(
        evs.iter()
            .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
        "{what}: o ponteiro sobre o retangulo nao virou Click — ele esta' desenhado e nao existe \
         para o dispatcher (falta o `register` no populate)"
    );
    for ev in evs {
        host.apply_panel_event::<VectorPanel>(&mut st, ev);
    }
    host.drained_actions()
}

/// ⭐⭐⭐ **OS CINCO CONTROLOS DE CADA LINHA, E OS DOIS QUE FAZEM A PILHA CRESCER, ATRAVESSAM O
/// BARRAMENTO.**
///
/// ⚠️ **O oráculo é o `EditorAction`, nunca o `WidgetEvent`.** Um controlo que produz `Click` e não
/// produz `ToolPanelEvent` acende sob o rato, consome o gesto e não faz nada — que é literalmente
/// o report: *"o olho não funciona"*.
///
/// ⚠️ **As DUAS camadas são exercitadas**, e não só a `0`: os ids são derivados do índice, e uma
/// allowlist escrita à mão que casasse só o primeiro deixaria toda pilha de duas camadas para
/// cima meio morta.
#[test]
fn every_control_of_a_layer_row_reaches_the_bus() {
    publica(vec![tinta(), traco(2.0)]);
    let mut alvos: Vec<(ph2d_a11y::NodeId, String)> = vec![
        (ids::VECTOR_PAINT_ADD_FILL, "+ Fill".into()),
        (ids::VECTOR_PAINT_ADD_STROKE, "+ Stroke".into()),
    ];
    for i in 0..2usize {
        alvos.push((ids::vector_paint_eye_id(i), format!("o olho da camada {i}")));
        alvos.push((ids::vector_paint_row_id(i), format!("o nome da camada {i}")));
        alvos.push((ids::vector_paint_up_id(i), format!("o subir da camada {i}")));
        alvos.push((
            ids::vector_paint_down_id(i),
            format!("o descer da camada {i}"),
        ));
        alvos.push((
            ids::vector_paint_del_id(i),
            format!("o apagar da camada {i}"),
        ));
    }
    for (id, what) in alvos {
        let acoes = clica(id, &what);
        assert!(
            acoes.iter().any(|a| matches!(
                a,
                EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if *c == id
            )),
            "{what}: o Click nao chegou ao barramento — ele acende sob o rato e nao faz nada \
             (falta a familia na allowlist do `event_clicks::forwards_plain_click`)"
        );
    }
    state::set_current_appearance(None);
}

/// **ABRIR UMA CAMADA MOSTRA AS TRÊS PROPRIEDADES DELA — e fechá-la esconde-as.**
///
/// ⚠️ As duas metades, porque um gate que só afirmasse a presença ficaria verde sobre um painel
/// que as mostra SEMPRE — e aí o artista veria a largura de uma camada enquanto olha para outra.
#[test]
fn opening_a_layer_paints_its_three_properties() {
    publica(vec![traco(2.0)]);
    state::close_open_layer();
    let props = [
        (ids::VECTOR_PAINT_WIDTH, "a largura"),
        (ids::VECTOR_PAINT_OPACITY, "a opacidade"),
        (ids::VECTOR_PAINT_BLEND, "a mistura"),
    ];
    for (id, what) in props {
        let mut host = MockPanelHost::with_panel::<VectorPanel>();
        let mut st = VectorPanelState;
        assert!(
            host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, id)
                .is_none(),
            "{what} foi pintada com a camada FECHADA"
        );
    }
    state::toggle_open_layer(0);
    for (id, what) in props {
        let mut host = MockPanelHost::with_panel::<VectorPanel>();
        let mut st = VectorPanelState;
        assert!(
            host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, id)
                .is_some(),
            "{what} nao foi pintada com a camada ABERTA"
        );
    }
    state::close_open_layer();
    state::set_current_appearance(None);
}

/// ⭐⭐⭐ **A LARGURA, A OPACIDADE E A MISTURA DE UMA CAMADA CHEGAM AO BARRAMENTO.**
///
/// ⚠️ **É a segunda metade do report, e ela falha SOZINHA:** mesmo com o clique do nome a
/// atravessar, as três rows abririam e os três controlos seriam mudos — o `ValueChanged` de cada
/// um caía no catch-all do `apply_event`. O oráculo é o VALOR, porque encaminhar o id com o número
/// de outro campo é o mesmo defeito com outra roupa.
#[test]
fn the_layers_width_and_opacity_reach_the_bus() {
    publica(vec![traco(2.0)]);
    state::toggle_open_layer(0);

    // A LARGURA — campo numérico, o valor viaja como o do Z-index (a mesma porta).
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    host.set_number_value(ids::VECTOR_PAINT_WIDTH, 7.0);
    host.apply_panel_event::<VectorPanel>(
        &mut st,
        WidgetEvent::ValueChanged(ids::VECTOR_PAINT_WIDTH),
    );
    let larguras: Vec<f64> = host
        .drained_actions()
        .into_iter()
        .filter_map(|a| match a {
            EditorAction::ToolPanelEvent(PanelEvent::SetValue(id, v))
                if id == ids::VECTOR_PAINT_WIDTH =>
            {
                Some(v)
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        larguras,
        vec![7.0],
        "a largura da camada nao chegou ao barramento: o artista escreve um numero e nada acontece"
    );

    // A OPACIDADE — slider, o valor viaja como TRACK `0..1`.
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    host.set_slider_value(ids::VECTOR_PAINT_OPACITY, 0.25);
    host.apply_panel_event::<VectorPanel>(
        &mut st,
        WidgetEvent::ValueChanged(ids::VECTOR_PAINT_OPACITY),
    );
    let tracks: Vec<f64> = host
        .drained_actions()
        .into_iter()
        .filter_map(|a| match a {
            EditorAction::ToolPanelEvent(PanelEvent::SetValue(id, v))
                if id == ids::VECTOR_PAINT_OPACITY =>
            {
                Some(v)
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        tracks.len(),
        1,
        "a opacidade da camada nao chegou ao barramento (ou chegou em duplicado)"
    );
    assert!(
        (tracks[0] - 0.25).abs() < 1e-6,
        "a opacidade chegou com o valor errado: {}",
        tracks[0]
    );

    state::close_open_layer();
    state::set_current_appearance(None);
}

/// ⭐⭐ **O CAMPO DE LARGURA MOSTRA A LARGURA DA CAMADA ABERTA.**
///
/// ⚠️ Sem a semente ele mostra o que a última edição deixou no store — e o `PaintRow::width` que a
/// shell publica não tem leitor nenhum. Um campo que mostra o número de OUTRA camada é pior que um
/// campo vazio: o artista lê `3`, escreve `4`, e engorda um traço que media `12`.
#[test]
fn the_width_field_shows_the_open_layers_width() {
    publica(vec![traco(9.0)]);
    state::toggle_open_layer(0);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    host.set_number_value(ids::VECTOR_PAINT_WIDTH, 1.0);
    host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, ids::VECTOR_PAINT_WIDTH);
    let visto = host.store().number_value(ids::VECTOR_PAINT_WIDTH);
    assert_eq!(
        visto,
        Some(9.0),
        "o campo de largura nao foi semeado da camada aberta (mostra {visto:?}, a camada mede 9)"
    );
    state::close_open_layer();
    state::set_current_appearance(None);
}

/// ⛔⛔ **A SWATCH DE UMA CAMADA NÃO EMITE `Click` — e isso é a DECISÃO, não um esquecimento.**
///
/// Ela é uma *picker swatch*: o `pointer_down` do editor-core curto-circuita o Down dessas para
/// abrir o picker OKLCH partilhado, e o gesto **nunca** chega a virar um `Click`. Quem lê a
/// escolha é a shell, pelo `layer_of_picker_target`.
///
/// ⚠️ **Este gate existe para que a próxima janela não «conserte» isto** pondo a swatch na
/// allowlist de cliques por simetria com os outros cinco controlos da linha: a linha entraria, o
/// `Click` continuaria a não existir, e ela ficaria com cara de viva. Existiu um `StackVerb::Swatch`
/// exactamente assim — inalcançável por construção, com um teste a chamá-lo à mão.
#[test]
fn a_layer_swatch_opens_the_picker_instead_of_clicking() {
    publica(vec![traco(2.0)]);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    let id = ids::vector_paint_swatch_id(0);
    let r = host
        .painted_rect::<VectorPanel>(&mut st, VIEWPORT, id)
        .expect("a swatch da camada tem de ser pintada");
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
    assert!(
        !evs.iter().any(|e| matches!(e, WidgetEvent::Click(_))),
        "a swatch emitiu Click — ela deixou de abrir o picker, e a shell le' a cor de la'"
    );
    assert_eq!(
        host.store().picker_target(),
        Some(id),
        "o Down na swatch nao armou o picker nesta camada"
    );
    state::set_current_appearance(None);
}

/// ⭐⭐ **A SWATCH SEMEIA O PICKER COM A COR DA CAMADA.**
///
/// ⚠️ O oráculo é a cor que o `pointer_down` vai **ler** (`widget_color`), e não a que a swatch
/// pinta: são duas leituras diferentes do mesmo facto, e era só a segunda que estava ligada. Sem a
/// semente o picker abre no cinzento de fábrica e a primeira mexida carimba cinzento por cima do
/// azul da camada.
#[test]
fn a_layer_swatch_seeds_the_picker_with_that_layers_colour() {
    publica(vec![tinta(), traco(2.0)]);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, ids::vector_paint_swatch_id(0));
    for (i, esperada) in [(0usize, tinta().color), (1usize, traco(2.0).color)] {
        assert_eq!(
            host.store().widget_color(ids::vector_paint_swatch_id(i)),
            Some(esperada),
            "a swatch da camada {i} nao foi semeada com a cor dela — o picker abriria no cinzento"
        );
    }
    state::set_current_appearance(None);
}

/// ⭐⭐⭐ **O CHIP DE MISTURA DA CAMADA ABRE, E A LINHA ESCOLHIDA CHEGA AO BARRAMENTO.**
///
/// ⚠️ São **duas** costuras e o report do Enio não as distingue (*"clicar no nome não exibe … a
/// mistura"*): o chip pode estar morto sob o dedo (não abre) **ou** a lista pode abrir e a escolha
/// morrer no painel. Este gate atravessa as duas, e o oráculo do fim é o **CÓDIGO do modo** — a
/// lista é derivada da tradução para o Vello, e mandar a LINHA faria a shell reconstruí-la.
#[test]
fn the_layer_blend_chip_opens_and_the_pick_reaches_the_bus() {
    publica(vec![traco(2.0)]);
    state::toggle_open_layer(0);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    let r = host
        .painted_rect::<VectorPanel>(&mut st, VIEWPORT, ids::VECTOR_PAINT_BLEND)
        .expect("o chip de mistura da camada tem de ser pintado");
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
    for ev in evs {
        host.apply_panel_event::<VectorPanel>(&mut st, ev);
    }
    let _ = host.drained_actions();
    assert!(
        matches!(
            host.store().get(ids::VECTOR_PAINT_BLEND),
            Some(ph2d_editor_core::interaction::InteractiveState::Dropdown { open: true, .. })
        ),
        "o chip de mistura da camada nao abriu — ele esta' morto sob o dedo"
    );

    // A 2.ª linha da lista OFERECIDA — nunca um código escrito à mão.
    let modo = ph2d_vec_render::blend::offered()
        .nth(1)
        .expect("a lista oferecida tem de ter pelo menos duas linhas");
    host.apply_panel_event::<VectorPanel>(
        &mut st,
        WidgetEvent::Click(ids::vector_paint_blend_option_id(1)),
    );
    let codigos: Vec<f64> = host
        .drained_actions()
        .into_iter()
        .filter_map(|a| match a {
            EditorAction::ToolPanelEvent(PanelEvent::SetValue(id, v))
                if id == ids::VECTOR_PAINT_BLEND =>
            {
                Some(v)
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        codigos,
        vec![f64::from(modo.to_u8())],
        "a mistura escolhida na camada nao chegou ao barramento com o codigo do modo"
    );
    state::close_open_layer();
    state::set_current_appearance(None);
}

/// ⭐⭐⭐ **ONDE A CAMADA DESENHA CHEGA AO BARRAMENTO, NOS DOIS EIXOS.**
///
/// Report do Enio, 2026-09-05: *"o fill não deveria ter um offset? não seria mais útil?"* — sem
/// ele dois preenchimentos desenham nos MESMOS pixels e o de cima só se distingue por cor,
/// opacidade e mistura.
///
/// ⚠️ **Os dois eixos, e não só o `x`:** eles são ids diferentes num braço só, e um braço que
/// casasse apenas o primeiro deixaria metade do controlo mudo — a forma exacta do bug #29.
#[test]
fn the_layers_offset_reaches_the_bus_on_both_axes() {
    publica(vec![tinta()]);
    state::toggle_open_layer(0);
    for (id, valor, eixo) in [
        (ids::VECTOR_PAINT_DX, 3.0, "x"),
        (ids::VECTOR_PAINT_DY, -2.5, "y"),
    ] {
        let mut host = MockPanelHost::with_panel::<VectorPanel>();
        let mut st = VectorPanelState;
        host.set_number_value(id, valor);
        host.apply_panel_event::<VectorPanel>(&mut st, WidgetEvent::ValueChanged(id));
        let sent: Vec<f64> = host
            .drained_actions()
            .into_iter()
            .filter_map(|a| match a {
                EditorAction::ToolPanelEvent(PanelEvent::SetValue(c, v)) if c == id => Some(v),
                _ => None,
            })
            .collect();
        assert_eq!(
            sent,
            vec![valor],
            "o deslocamento em {eixo} nao chegou ao barramento"
        );
    }
    state::close_open_layer();
    state::set_current_appearance(None);
}

/// ⭐⭐ **O PAR X/Y APARECE NUM PREENCHIMENTO — e é o ponto da wave.**
///
/// ⚠️ A LARGURA só existe num contorno (um preenchimento não tem uma), e o deslocamento existe nos
/// **dois**: a sombra dura de um preenchimento é o caso que motivou isto. Um gate que só medisse o
/// contorno ficaria verde sobre um painel que esconde o par exactamente onde ele foi pedido.
#[test]
fn the_offset_pair_shows_on_a_fill_layer_where_the_width_does_not() {
    publica(vec![tinta()]);
    state::toggle_open_layer(0);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    for (id, what) in [(ids::VECTOR_PAINT_DX, "o X"), (ids::VECTOR_PAINT_DY, "o Y")] {
        assert!(
            host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, id)
                .is_some(),
            "{what} nao foi pintado numa camada de PREENCHIMENTO"
        );
    }
    assert!(
        host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, ids::VECTOR_PAINT_WIDTH)
            .is_none(),
        "a largura foi pintada num preenchimento — o controlo nao tem sujeito"
    );
    state::close_open_layer();
    state::set_current_appearance(None);
}

/// ⭐⭐ **O PAR MOSTRA O DESLOCAMENTO DA CAMADA ABERTA.**
///
/// ⚠️ Mesma lei da largura, e a wave anterior pagou-a: sem a semente o campo mostra o que a última
/// edição deixou no store, e o artista lê `0` numa camada que está a `4`.
#[test]
fn the_offset_fields_show_the_open_layers_offset() {
    let mut linha = tinta();
    linha.offset = [4.0, -1.5];
    publica(vec![linha]);
    state::toggle_open_layer(0);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    host.set_number_value(ids::VECTOR_PAINT_DX, 0.0);
    host.set_number_value(ids::VECTOR_PAINT_DY, 0.0);
    host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, ids::VECTOR_PAINT_DX);
    assert_eq!(
        (
            host.store().number_value(ids::VECTOR_PAINT_DX),
            host.store().number_value(ids::VECTOR_PAINT_DY)
        ),
        (Some(4.0), Some(-1.5)),
        "o par nao foi semeado da camada aberta"
    );
    state::close_open_layer();
    state::set_current_appearance(None);
}
