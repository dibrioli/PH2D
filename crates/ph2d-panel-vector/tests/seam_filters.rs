//! Behavioral SEAM test for the **Filters** section (a pilha de FX raster, plano 24) — irmão do
//! `seam.rs` (que bateu o teto de LOC).
//!
//! Prova o que os testes de unidade + o `architecture_panel_wiring_parity` não provam: que o
//! GESTO de montar a pilha chega ao bus. ⚠️ E aqui a cobertura do gate de wiring é **nula por
//! construção**: os ids desta seção são derivados por LINHA (`filter_add_id(k)` /
//! `filter_remove_id(r)` …) e aquele gate só coleta `.register(ids::LITERAL` — então este arquivo
//! é a ÚNICA coisa entre um card inteiro pintado e MORTO sob o mouse.
//!
//! (A cor do halo vai pelo picker OKLCH partilhado — coberta por `register_picker_swatch` + o
//! dispatch genérico, como as swatches de Fill/Contour. Os sliders viajam como `SetValue`,
//! cobertos pela delegação em `event.rs`.)

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_vector::state::VectorPanelState;
use ph2d_panel_vector::{FilterKindView, FilterRowView, VectorPanel, ids};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};
const SEC: u128 = 1_000_000_000;

fn pointer(kind: PointerKind, x: f32, y: f32, t: u128) -> PointerEvent {
    PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: PointerSource::Mouse,
        button: PointerButton::Primary,
        timestamp_ns: t,
    }
}

/// Uma linha de pilha para o fixture.
fn row(kind: u8) -> FilterRowView {
    FilterRowView {
        kind,
        mode: 0,
        enabled: true,
        radius: 0.12,
        offx: 0.12,
        offy: -0.12,
        color: [0, 0, 0, 255],
        color_b: [255, 255, 255, 255],
        opacity: 1.0,
        blend: 0,
        scale: 0.25,
        detail: 3,
        seed: 0,
        grow: 0.06,
        hue: 0.0,
        sat: 0.0,
        bright: 0.0,
        // Uma rampa de TRÊS stops: as duas pontas mais um do meio. ⚠️ O do meio é o que faz o
        // fixture conter o fenômeno — com só as pontas, um trilho que só desenhasse as bordas
        // passaria, e é entre stops que uma rampa de N pontos pode estar errada.
        stop_pos: [0.0, 0.5, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        stop_colors: [
            [0, 0, 0, 255],
            [200, 60, 60, 255],
            [255, 255, 255, 255],
            [0; 4],
            [0; 4],
            [0; 4],
            [0; 4],
            [0; 4],
        ],
        stop_count: 3,
        ramp_preview: [[128; 3]; ph2d_panel_vector::RAMP_PREVIEW_N],
    }
}

/// A tabela de tipos que a shell publica — espelho do `ph2d_ecs::FxOp::SPECS`, que este crate
/// **não alcança** (vive de snapshots).
///
/// ⚠️ O que este arquivo prova é que **o painel OBEDECE a tabela** — presença e ausência de cada
/// controle, para TODO tipo. Que a tabela chegue inteira do motor é outra pergunta, e ela tem uma
/// resposta estrutural: o publish é uma única `map` sobre `FxOp::SPECS` (e o gate do shell pina
/// que os TETOS dos dois lados concordam).
fn kinds_table() -> Vec<FilterKindView> {
    let k = |name, radius_label, offset_labels, color_label, modes, takes_blend| FilterKindView {
        name,
        radius_label,
        offset_labels,
        color_label,
        color_b_label: None,
        modes,
        takes_blend,
        takes_ramp: false,
        noise_labels: None,
        grow_label: None,
        adjust_labels: None,
    };
    const INNER: &[&str] = &["Proximity", "Contour"];
    const OFF: Option<(&str, &str)> = Some(("Offset X", "Offset Y"));
    const LIGHT: Option<(&str, &str)> = Some(("Light X", "Light Y"));
    const COL: Option<&str> = Some("Color");
    const NOISE_MODES: &[&str] = &["Smooth", "Creased"];
    const NOISE: Option<(&str, &str, &str)> = Some(("Size", "Detail", "Seed"));
    vec![
        k("Blur", Some("Radius"), None, None, &[], false),
        k("Glow", Some("Radius"), None, COL, &[], false),
        k("Drop Shadow", Some("Radius"), OFF, COL, &[], false),
        k("Inner Shadow", Some("Radius"), OFF, COL, INNER, true),
        k("Inner Glow", Some("Radius"), None, COL, INNER, true),
        k("Outline", Some("Width"), None, COL, &[], false),
        k("Feather", Some("Feather"), None, None, &[], false),
        k("Bevel", Some("Depth"), LIGHT, Some("Shadow"), &[], true),
        k("Color Overlay", None, None, COL, &[], true),
        FilterKindView {
            noise_labels: NOISE,
            ..k("Turbulence", Some("Amount"), None, None, NOISE_MODES, false)
        },
        FilterKindView {
            // ⚠️ SEM `radius_label`: o único comprimento deste tipo é o Amount BIPOLAR, e é ele
            // que a linha oferece.
            grow_label: Some("Amount"),
            ..k("Grow / Shrink", None, None, None, &[], false)
        },
        FilterKindView {
            // ⚠️ Sem raio, sem cor: o Color Adjust é PONTUAL e ajusta a cor que já está lá — uma
            // swatch aqui seria a segunda porta para *recolorir*, que o Color Overlay já é.
            adjust_labels: Some(("Hue", "Saturation", "Brightness")),
            ..k("Color Adjust", None, None, None, &[], false)
        },
        FilterKindView {
            // ⚠️ DUAS swatches: a rampa tem duas pontas, e cada uma abre o picker por conta
            // própria. É o único tipo com `color_b_label`, e é o que este fixture prova que o
            // painel oferece — presença E ausência, como todo o resto da tabela.
            color_b_label: Some("Highlights"),
            ..k("Duotone", None, None, Some("Shadows"), &[], true)
        },
        // ⚠️ NADA além da Opacity: o Luma to Alpha não tem knob nenhum, e um card com uma linha
        // só é exactamente o que a tabela manda desenhar.
        k("Luma to Alpha", None, None, None, &[], false),
    ]
}

/// Os nomes das leis de mistura que a shell publica — espelho dos vinte primeiros códigos do
/// `BlendMode`, que este crate também não alcança. Só os dois primeiros e o último importam aos
/// gates; a lista tem de ter o COMPRIMENTO certo, porque é ela que decide quantas opções o popover
/// oferece.
fn blend_names_table() -> Vec<&'static str> {
    let mut v = vec!["Normal", "Multiply"];
    while v.len() < ph2d_panel_vector::ids::MAX_FILTER_BLENDS - 1 {
        v.push("…");
    }
    v.push("Luminosity");
    v
}

/// Publica o estado que a shell publicaria: forma selecionada + a tabela + a pilha dada.
fn publish(rows: Vec<FilterRowView>) {
    ph2d_panel_vector::set_current_filter_can_add(true);
    ph2d_panel_vector::set_filter_kinds(kinds_table());
    ph2d_panel_vector::set_filter_blend_names(blend_names_table());
    ph2d_panel_vector::set_current_filters(rows);
}

/// Clica o widget `id` pelo caminho inteiro e diz se ele chegou ao bus como `Click`.
fn click_reaches_bus(id: ph2d_a11y::NodeId, what: &str) {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    let r = host
        .painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
        .unwrap_or_else(|| panic!("{what} nao foi PINTADO com area clicavel na secao Filters"));
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
    assert!(
        evs.iter()
            .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
        "o ponteiro sobre {what} nao virou Click — falta `button()` no `populate` \
         (esta desenhado, mas nao existe para o dispatcher)"
    );
    for ev in evs {
        host.apply_panel_event::<VectorPanel>(&mut panel_state, ev);
    }
    let reached = host
        .drained_actions()
        .into_iter()
        .any(|a| matches!(a, EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if c == id));
    assert!(
        reached,
        "o Click em {what} nao chegou ao bus — falta o id no `forwards_plain_click` do \
         `event.rs` (o controle e clicavel e MORTO, e o drain da shell nunca edita a pilha)"
    );
}

/// A tabela com a rampa ARMADA no tipo `kind` — o publish real a traz do `FxKindSpec::takes_ramp`.
fn kinds_table_with_ramp(kind: usize) -> Vec<FilterKindView> {
    let mut t = kinds_table();
    if let Some(k) = t.get_mut(kind) {
        k.takes_ramp = true;
    }
    t
}

/// **O TRILHO da rampa: oferecido por quem carrega uma, e por mais ninguém.** Presença E ausência no
/// mesmo gate, porque uma metade sozinha é verde por acidente.
#[test]
fn the_ramp_rail_is_offered_only_by_the_kind_that_carries_one() {
    const RAMPED: usize = 0;
    const PLAIN: usize = 1;
    for (kind, want) in [(RAMPED, true), (PLAIN, false)] {
        let mut host = MockPanelHost::with_panel::<VectorPanel>();
        let mut st = VectorPanelState;
        ph2d_panel_vector::set_current_filter_can_add(true);
        ph2d_panel_vector::set_filter_kinds(kinds_table_with_ramp(RAMPED));
        ph2d_panel_vector::set_filter_blend_names(blend_names_table());
        #[allow(clippy::cast_possible_truncation)]
        ph2d_panel_vector::set_current_filters(vec![row(kind as u8)]);
        // Os quatro controles do trilho: os dois botões, um punho, e a swatch do stop.
        for (id, what) in [
            (
                ph2d_panel_vector::ids::filter_stop_add_id(0),
                "o + do trilho",
            ),
            (
                ph2d_panel_vector::ids::filter_stop_remove_id(0),
                "o - do trilho",
            ),
            (
                ph2d_panel_vector::ids::filter_stop_id(0, 1),
                "o punho do stop do meio",
            ),
            (
                ph2d_panel_vector::ids::filter_stop_color_id(0),
                "a swatch do stop",
            ),
        ] {
            let painted = host
                .painted_rect::<VectorPanel>(&mut st, VIEWPORT, id)
                .is_some();
            assert_eq!(
                painted, want,
                "{what}: pintado={painted} para um tipo com takes_ramp={want} — um controle que \
                 aparece onde o tipo não tem rampa é um knob morto, e um que falta onde tem é a \
                 feature inalcançável"
            );
        }
    }
}

/// **Os punhos são ALCANÇÁVEIS e não se sobrepõem** — e é isto que mede o recurso de que o teto de
/// stops é (a caixa de agarre), em vez de o afirmar em prosa.
#[test]
fn every_stop_handle_is_reachable_and_the_grabs_do_not_overlap() {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    ph2d_panel_vector::set_current_filter_can_add(true);
    ph2d_panel_vector::set_filter_kinds(kinds_table_with_ramp(0));
    ph2d_panel_vector::set_filter_blend_names(blend_names_table());
    ph2d_panel_vector::set_current_filters(vec![row(0)]);
    let mut rects = Vec::new();
    for stop in 0..3 {
        let id = ph2d_panel_vector::ids::filter_stop_id(0, stop);
        let r = host
            .painted_rect::<VectorPanel>(&mut st, VIEWPORT, id)
            .unwrap_or_else(|| panic!("o punho do stop {stop} nao foi PINTADO com area clicavel"));
        rects.push(r);
    }
    for (i, a) in rects.iter().enumerate() {
        for b in rects.iter().skip(i + 1) {
            let overlap = a.x < b.x + b.w && b.x < a.x + a.w;
            assert!(
                !overlap,
                "duas caixas de agarre se sobrepõem ({a:?} e {b:?}) — o punho de baixo fica \
                 inalcançável, que é exactamente o recurso de que o teto de stops é"
            );
        }
    }
    // E o punho do stop do MEIO fica no meio da barra: sem isto, três punhos empilhados na borda
    // passariam no teste de sobreposição por acidente.
    assert!(
        rects[1].x > rects[0].x && rects[1].x < rects[2].x,
        "os punhos não estão na ordem das posições ({rects:?}) — a barra não os está a mapear"
    );
}

/// A tabela com um tipo que espelha o **Gradient Map REAL**: sem raio, sem cores, com blend, com
/// DOIS modos, e com rampa.
///
/// ⚠️ **A fixture do irmão forçava `takes_ramp` num tipo que TEM raio**, e é um card de outra altura
/// e outra ordem de rows — exactamente a classe de fixture que não contém o fenômeno.
fn kinds_table_real_gradient_map() -> Vec<FilterKindView> {
    let mut t = kinds_table();
    if let Some(k) = t.get_mut(0) {
        k.name = "Gradient Map";
        k.radius_label = None;
        k.offset_labels = None;
        k.color_label = None;
        k.color_b_label = None;
        k.modes = &["Linear", "Smooth"];
        k.takes_blend = true;
        k.takes_ramp = true;
    }
    t
}

/// **ARRASTAR um punho chega ao bus, no card do Gradient Map REAL.**
#[test]
fn dragging_a_stop_handle_works_on_the_real_gradient_map_card() {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    ph2d_panel_vector::set_current_filter_can_add(true);
    ph2d_panel_vector::set_filter_kinds(kinds_table_real_gradient_map());
    ph2d_panel_vector::set_filter_blend_names(blend_names_table());
    ph2d_panel_vector::set_current_filters(vec![row(0)]);
    let id = ph2d_panel_vector::ids::filter_stop_id(0, 1);
    let r = host
        .painted_rect::<VectorPanel>(&mut st, VIEWPORT, id)
        .expect("o punho do stop 1 nao foi PINTADO no card do Gradient Map real");
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let mut evs = host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    evs.extend(host.dispatch_pointer_event(pointer(
        PointerKind::Move,
        cx + 40.0,
        cy,
        SEC + SEC / 100,
    )));
    let ramp = ph2d_panel_vector::ids::filter_ramp_id(0);
    assert!(
        evs.iter()
            .any(|e| matches!(e, WidgetEvent::ValueChanged(c) if *c == ramp)),
        "o arrasto no punho nao virou `ValueChanged` do TRILHO no card REAL — algum widget vizinho \
         roubou o hit, ou o `CurvePoint` nao esta no store"
    );
    for ev in evs {
        host.apply_panel_event::<VectorPanel>(&mut st, ev);
    }
    assert!(
        host.drained_actions().into_iter().any(
            |a| matches!(a, EditorAction::ToolPanelEvent(PanelEvent::SelectOption(c, _)) if c == ramp)
        ),
        "o arrasto nao chegou ao bus no card REAL"
    );
}

/// **ARRASTAR um punho chega ao bus como uma posição nova.**
///
/// ⚠️ **Este gate faltava, e o report do Enio ("não é possível arrastar os pontos de cor") é
/// exactamente o vão que ele deixou:** os irmãos provavam que os punhos são PINTADOS e que os dois
/// botões despacham, e nenhum dirigia o GESTO. Um punho pintado, hit-registrado e sem estado de
/// arrasto no store é *desenhado e morto sob o mouse* — a quarta condição da regra de UI (a
/// SEQUÊNCIA leva a algum lugar) não é implicada pelas outras três.
#[test]
fn dragging_a_stop_handle_reaches_the_bus_with_a_new_position() {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    ph2d_panel_vector::set_current_filter_can_add(true);
    ph2d_panel_vector::set_filter_kinds(kinds_table_with_ramp(0));
    ph2d_panel_vector::set_filter_blend_names(blend_names_table());
    ph2d_panel_vector::set_current_filters(vec![row(0)]);
    // O punho do MEIO (posição 0,5) — nem a primeira nem a última, senão um off-by-one no índice
    // acertaria por acidente.
    let id = ph2d_panel_vector::ids::filter_stop_id(0, 1);
    let r = host
        .painted_rect::<VectorPanel>(&mut st, VIEWPORT, id)
        .expect("o punho do stop 1 nao foi PINTADO com area clicavel");
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    // Down no punho, e ARRASTA para a direita — o Down já aplica a posição (click-to-move).
    let mut evs = host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    evs.extend(host.dispatch_pointer_event(pointer(
        PointerKind::Move,
        cx + 40.0,
        cy,
        SEC + SEC / 100,
    )));
    let ramp = ph2d_panel_vector::ids::filter_ramp_id(0);
    assert!(
        evs.iter()
            .any(|e| matches!(e, WidgetEvent::ValueChanged(c) if *c == ramp)),
        "o arrasto no punho nao virou `ValueChanged` do TRILHO — o `InteractiveState::CurvePoint` \
         nao esta no store, entao o punho esta desenhado e MORTO sob o mouse"
    );
    for ev in evs {
        host.apply_panel_event::<VectorPanel>(&mut st, ev);
    }
    let sent: Vec<String> = host
        .drained_actions()
        .into_iter()
        .filter_map(|a| match a {
            EditorAction::ToolPanelEvent(PanelEvent::SelectOption(c, v)) if c == ramp => Some(v),
            _ => None,
        })
        .collect();
    assert!(
        !sent.is_empty(),
        "o arrasto nao chegou ao bus — o painel nao encaminha o `ValueChanged` do trilho, e a shell \
         nunca move o stop"
    );
    // O formato é `linha:idx:x`, e o `idx` tem de ser o do punho AGARRADO.
    let last = sent.last().expect("ha pelo menos um");
    let parts: Vec<&str> = last.split(':').collect();
    assert_eq!(parts.len(), 3, "o formato do arrasto mudou: {last:?}");
    assert_eq!(parts[0], "0", "a LINHA errada: {last:?}");
    assert_eq!(
        parts[1], "1",
        "o INDICE errado ({last:?}) — o arrasto moveria OUTRO stop, e o punho escaparia do dedo"
    );
    let x: f32 = parts[2].parse().expect("o x e um numero");
    assert!(
        x > 0.5,
        "arrastar para a DIREITA devolveu x={x} (o stop estava em 0,5) — o gesto move o stop para o \
         lado errado, ou nao o move"
    );
}

/// **O `+` e o `−` do trilho chegam ao bus.** Sem isto o artista clica e a rampa nunca ganha (nem
/// perde) um stop, com tudo pintado.
#[test]
fn the_plus_and_minus_of_the_rail_reach_the_bus() {
    ph2d_panel_vector::set_current_filter_can_add(true);
    ph2d_panel_vector::set_filter_kinds(kinds_table_with_ramp(0));
    ph2d_panel_vector::set_filter_blend_names(blend_names_table());
    ph2d_panel_vector::set_current_filters(vec![row(0)]);
    click_reaches_bus(
        ph2d_panel_vector::ids::filter_stop_add_id(0),
        "o + do trilho",
    );
    click_reaches_bus(
        ph2d_panel_vector::ids::filter_stop_remove_id(0),
        "o - do trilho",
    );
}

/// **Os "Add" chegam ao bus.** Com uma forma selecionada e a pilha VAZIA, os três botões (um por
/// tipo publicado pelo motor) têm de existir e despachar — é o único gesto que CRIA uma pilha.
#[test]
fn every_add_button_reaches_the_bus_when_clicked() {
    for kind in 0..kinds_table().len() {
        publish(Vec::new());
        click_reaches_bus(ids::filter_add_id(kind), &format!("Add do tipo {kind}"));
    }
    assert!(
        kinds_table().len() <= ids::MAX_FILTER_KINDS,
        "o teto de ids nao cobre a tabela — os ultimos tipos ficariam sem botao, em silencio"
    );
}

/// **Os ícones do card chegam ao bus** — ✕ / ↑ / ↓ / 👁. O fixture monta uma
/// pilha de DOIS Drop Shadows: só assim a linha 1 existe (o ↑ da primeira e o ↓ da última não são
/// desenhados de propósito), e o Drop Shadow é o tipo SUPERSET (é o único com Offset X/Y, e tem
/// cor).
#[test]
fn every_card_icon_reaches_the_bus_when_clicked() {
    for id_of in [
        ids::filter_remove_id as fn(usize) -> ph2d_a11y::NodeId,
        ids::filter_hide_id,
    ] {
        for r in 0..2usize {
            publish(vec![row(2), row(2)]);
            click_reaches_bus(id_of(r), &format!("icone da linha {r}"));
        }
    }
    // ↓ existe na linha 0 (há uma abaixo) e ↑ na linha 1 (há uma acima) — as pontas são mudas.
    publish(vec![row(2), row(2)]);
    click_reaches_bus(ids::filter_down_id(0), "seta para BAIXO da linha 0");
    publish(vec![row(2), row(2)]);
    click_reaches_bus(ids::filter_up_id(1), "seta para CIMA da linha 1");
}

/// **As setas das PONTAS não são desenhadas** — subir na primeira linha e descer na última não
/// fazem nada, e um ícone inerte ensina o artista a desconfiar dos que funcionam. É a metade
/// AUSENTE, e ela é o que impede a mutação "desenhe as quatro sempre" de passar.
#[test]
fn the_arrows_at_the_ends_are_not_painted() {
    publish(vec![row(0), row(0)]);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    assert!(
        host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, ids::filter_up_id(0))
            .is_none(),
        "a seta para CIMA da primeira linha nao pode ser desenhada (nao ha para onde subir)"
    );
    assert!(
        host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, ids::filter_down_id(1))
            .is_none(),
        "a seta para BAIXO da ultima linha nao pode ser desenhada"
    );
}

/// **Cada linha oferece só os controles do TIPO dela — TODOS os tipos, presença E ausência.**
///
/// ⚠️ O laço varre a tabela inteira e pergunta a ELA o que exigir. A W2 comparava dois tipos
/// escritos à mão (Blur × Drop Shadow) e teria ficado **verde** sobre os quatro tipos desta wave:
/// um Color Overlay com slider de Radius (knob morto — o tipo é pontual) ou um Outline sem cor
/// passariam sem que nada falhasse.
#[test]
fn a_row_paints_only_the_controls_its_kind_uses() {
    let table = kinds_table();
    for (kind, spec) in table.iter().enumerate() {
        publish(vec![row(u8::try_from(kind).expect("kind cabe em u8"))]);
        let mut host = MockPanelHost::with_panel::<VectorPanel>();
        let mut st = VectorPanelState;
        let painted = |host: &mut MockPanelHost, st: &mut VectorPanelState, id| {
            host.painted_rect::<VectorPanel>(st, VIEWPORT, id).is_some()
        };
        // Opacity: em TODA linha (é o "quanto" de qualquer degrau).
        assert!(
            painted(&mut host, &mut st, ids::filter_opacity_id(0)),
            "Opacity tem de existir no {}",
            spec.name
        );
        assert_eq!(
            painted(&mut host, &mut st, ids::filter_radius_id(0)),
            spec.radius_label.is_some(),
            "o Radius do {} discorda da tabela",
            spec.name
        );
        for id in [ids::filter_offx_id(0), ids::filter_offy_id(0)] {
            assert_eq!(
                painted(&mut host, &mut st, id),
                spec.offset_labels.is_some(),
                "o Offset do {} discorda da tabela",
                spec.name
            );
        }
        assert_eq!(
            painted(&mut host, &mut st, ids::filter_color_id(0)),
            spec.color_label.is_some(),
            "a cor do {} discorda da tabela",
            spec.name
        );
        // A SEGUNDA ponta da rampa: existe exactamente onde a tabela diz, e em mais lugar
        // nenhum. ⚠️ A pergunta é separada da primeira de propósito — um tipo com `color_b_label`
        // e sem `color_label` seria meia rampa, e é a comparação com a TABELA que o proíbe.
        assert_eq!(
            painted(&mut host, &mut st, ids::filter_color_b_id(0)),
            spec.color_b_label.is_some(),
            "a segunda cor do {} discorda da tabela",
            spec.name
        );
        // A LEI DE MISTURA: o chip existe exactamente onde a tabela diz que a cor do degrau
        // pousa em conteúdo que já existe — e em mais lugar nenhum.
        assert_eq!(
            painted(&mut host, &mut st, ids::filter_blend_id(0)),
            spec.takes_blend,
            "o chip de mistura do {} discorda da tabela",
            spec.name
        );
        // Os TRÊS knobs do ruído aparecem em BLOCO, exactamente onde a tabela diz que há um
        // campo de ruído a descrever — e em mais lugar nenhum. Um Size sem Detail (ou num tipo
        // que não lê ruído) é um controle que não descreve nada.
        for id in [
            ids::filter_scale_id(0),
            ids::filter_detail_id(0),
            ids::filter_seed_id(0),
        ] {
            assert_eq!(
                painted(&mut host, &mut st, id),
                spec.noise_labels.is_some(),
                "um knob de ruído do {} discorda da tabela",
                spec.name
            );
        }
        // O **Amount** do Grow / Shrink: exactamente onde a tabela diz que o tipo engorda algo.
        assert_eq!(
            painted(&mut host, &mut st, ids::filter_grow_id(0)),
            spec.grow_label.is_some(),
            "o Amount do {} discorda da tabela",
            spec.name
        );
        // Os TRÊS do ajuste de cor, também em BLOCO: eles descrevem *como esta cor é ajustada*
        // em três eixos, e um sem os outros deixaria a arte cinzenta sem knob vivo nenhum.
        for id in [
            ids::filter_hue_id(0),
            ids::filter_sat_id(0),
            ids::filter_bright_id(0),
        ] {
            assert_eq!(
                painted(&mut host, &mut st, id),
                spec.adjust_labels.is_some(),
                "um knob de ajuste de cor do {} discorda da tabela",
                spec.name
            );
        }
        // Os chips de MODO: um por modo publicado, e NENHUM em quem não tem escolha a fazer.
        for m in 0..ids::MAX_FILTER_MODES {
            assert_eq!(
                painted(&mut host, &mut st, ids::filter_mode_id(0, m)),
                m < spec.modes.len(),
                "o chip de modo {m} do {} discorda da tabela",
                spec.name
            );
        }
    }
}

/// **Os chips de MODO chegam ao bus** — e é o único gesto que troca a LEI de um degrau (a banda
/// que segue o contorno × a proximidade do lado de fora). Sem isto o card mostraria a escolha e o
/// clique não mudaria nada.
#[test]
fn every_mode_chip_reaches_the_bus_when_clicked() {
    let table = kinds_table();
    for (kind, spec) in table.iter().enumerate() {
        for m in 0..spec.modes.len() {
            publish(vec![row(u8::try_from(kind).expect("kind cabe em u8"))]);
            click_reaches_bus(
                ids::filter_mode_id(0, m),
                &format!("modo {} do {}", spec.modes[m], spec.name),
            );
        }
    }
}

// ⚠️ **Não há gate para "o card usa o nome que a tabela dá".** O testkit de painel não tem oráculo
// de TEXTO (só rects), e a garantia aqui é melhor que um gate: o `FilterRowView` **não tem campo
// de rótulo** — o nome só pode vir da tabela, porque não existe outro sítio de onde ele possa vir.
// Divergência inexprimível > divergência testada.

/// A seção **NÃO** é oferecida sem seleção nem pilha viva — o cabeçalho nem sobe. É a metade
/// AUSENTE do seam: um botão que aparece sobre a tela vazia editaria a forma errada.
#[test]
fn the_filters_section_is_not_offered_without_a_shape() {
    ph2d_panel_vector::set_current_filter_can_add(false);
    ph2d_panel_vector::set_filter_kinds(kinds_table());
    ph2d_panel_vector::set_filter_blend_names(blend_names_table());
    ph2d_panel_vector::set_current_filters(Vec::new());
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    assert!(
        host.painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, ids::filter_add_id(0))
            .is_none(),
        "o botao Add foi pintado sem forma selecionada — a secao Filters vazou"
    );
}

/// **A swatch de cor é um alvo de PICKER, não um botão** — e essa distinção é dura: um id só pode
/// ter UM tipo de widget no store, e registá-la como `button()` fez este arquivo ficar vermelho
/// (o Down abre o picker e nenhum `Click` sai). O que ela precisa é de ser PINTADA com hit-rect e
/// estar no conjunto de swatches de picker; quem lê a cor de volta é a ponte, pelo
/// `picker_target()`.
///
/// Sem ESTE gate, tirar a swatch do `paint` deixaria a cor do halo sem porta nenhuma, e os outros
/// gates continuariam verdes.
#[test]
fn the_colour_swatch_is_painted_and_is_a_picker_target() {
    publish(vec![row(1)]);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    let id = ids::filter_color_id(0);
    assert!(
        host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, id)
            .is_some(),
        "a swatch de cor do Glow tem de ser pintada com area clicavel"
    );
    assert!(
        host.store().is_picker_swatch(id),
        "a swatch tem de estar no conjunto de picker (senao o Down nao abre o OKLCH)"
    );
}

/// **A SEGUNDA ponta da rampa é uma swatch de picker POR CONTA PRÓPRIA.**
///
/// ⚠️ Não basta ela ser pintada: se o id não estiver no conjunto de picker, o Down cai no dispatch
/// genérico e o OKLCH **nunca abre** — a swatch fica desenhada, hit-registrada e morta sob o mouse.
/// É a mesma metade que o irmão da primeira ponta prova, e ela é registada num LAÇO pelo teto de
/// linhas, então o gate é o único a vê-la.
#[test]
fn the_second_swatch_of_the_ramp_is_its_own_picker_target() {
    let duotone = kinds_table()
        .iter()
        .position(|k| k.color_b_label.is_some())
        .expect("a tabela tem de ter um tipo de rampa");
    publish(vec![row(u8::try_from(duotone).expect("kind cabe em u8"))]);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    let (a, b) = (ids::filter_color_id(0), ids::filter_color_b_id(0));
    assert_ne!(a, b, "as duas pontas nao podem partilhar o id");
    for (which, id) in [("escura", a), ("clara", b)] {
        assert!(
            host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, id)
                .is_some(),
            "a ponta {which} da rampa tem de ser pintada com area clicavel"
        );
        assert!(
            host.store().is_picker_swatch(id),
            "a ponta {which} tem de estar no conjunto de picker (senao o Down nao abre o OKLCH)"
        );
    }
}

/// **A lista de leis de mistura ABRE, e cada opção chega ao bus.** As quatro condições da política
/// de UI, no gesto REAL: o chip existe · é registrado (`populate`) · o clique nele ABRE a lista ·
/// e o clique numa opção sai como `Click` para a ponte traduzir.
///
/// ⚠️ **Sem abrir o chip primeiro, as opções não existem** — o popover é pintado no passe DIFERIDO
/// e só com o `Dropdown` aberto. Um gate que clicasse a opção direto mediria um retângulo que
/// ninguém pinta e passaria sobre uma lista que nunca abre.
#[test]
fn the_blend_list_opens_and_every_law_reaches_the_bus() {
    // O Color Overlay é o card mais MAGRO que toma a lei (sem raio, sem offset) — se o chip
    // sobrevive nele, sobrevive nos outros três.
    const COLOR_OVERLAY: u8 = 8;
    assert!(
        kinds_table()[COLOR_OVERLAY as usize].takes_blend,
        "a fixture tem de conter o fenômeno: este tipo TEM de tomar a lei"
    );

    // 1) O CHIP: clicá-lo abre a lista. É o dispatch genérico do `Dropdown`, então o que se prova
    //    aqui é que ele foi REGISTRADO como um (e não como botão, que abriria coisa nenhuma).
    publish(vec![row(COLOR_OVERLAY)]);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    let chip = host
        .painted_rect::<VectorPanel>(&mut st, VIEWPORT, ids::filter_blend_id(0))
        .expect("o chip de mistura tem de ser PINTADO com área clicável");
    let (cx, cy) = (chip.x + chip.w * 0.5, chip.y + chip.h * 0.5);
    host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
    assert_eq!(
        host.dropdown_is_open(ids::filter_blend_id(0)),
        Some(true),
        "o clique no chip não abriu a lista — ele não está registrado como `Dropdown`"
    );

    // 2) Com a lista ABERTA, cada opção é pintada E chega ao bus.
    let names = blend_names_table();
    for m in [0usize, 1, names.len() - 1] {
        publish(vec![row(COLOR_OVERLAY)]);
        let mut host = MockPanelHost::with_panel::<VectorPanel>();
        let mut st = VectorPanelState;
        host.set_dropdown_open(ids::filter_blend_id(0), true);
        let id = ids::filter_blend_option_id(0, m);
        let r = host
            .painted_rect::<VectorPanel>(&mut st, VIEWPORT, id)
            .unwrap_or_else(|| panic!("a lei {m} ({}) não é pintada com a lista aberta", names[m]));
        let (ox, oy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        host.dispatch_pointer_event(pointer(PointerKind::Down, ox, oy, SEC));
        let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, ox, oy, SEC + SEC / 100));
        assert!(
            evs.iter()
                .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
            "o ponteiro sobre a lei {m} não virou Click — falta `button()` no `populate`"
        );
        for ev in evs {
            host.apply_panel_event::<VectorPanel>(&mut st, ev);
        }
        assert!(
            host.drained_actions().into_iter().any(
                |a| matches!(a, EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if c == id)
            ),
            "o Click na lei {m} não chegou ao bus — falta o id no `is_filter_button` do \
             `event_filters.rs` (a opção é clicável e MORTA)"
        );
    }
}

/// **Nenhuma opção de mistura é pintada com a lista FECHADA** — a metade AUSENTE. Sem ela o gate
/// acima passaria sobre um popover que vive sempre aberto por cima do resto do painel.
#[test]
fn the_blend_list_paints_nothing_while_it_is_closed() {
    publish(vec![row(8)]);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    for m in [0usize, 1, ids::MAX_FILTER_BLENDS - 1] {
        assert!(
            host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, ids::filter_blend_option_id(0, m))
                .is_none(),
            "a lei {m} foi pintada com a lista fechada"
        );
    }
}

/// **Os três knobs do RUÍDO chegam ao barramento** — e é o gesto inteiro da turbulência: sem eles
/// o card desenha três sliders que o artista arrasta e que não mudam um pixel.
///
/// ⚠️ Um slider não chega por `Click`, e sim por `ValueChanged` — é por isso que este gate não usa
/// o [`click_reaches_bus`] dos botões: ele arrasta o trilho de verdade e confere que o valor
/// atravessa a fronteira `forward_track` com o MESMO mapa que o `populate` deu ao chip.
#[test]
fn the_three_noise_knobs_reach_the_bus_when_dragged() {
    // O índice do tipo que lê ruído, perguntado à TABELA — nunca um número escrito à mão, que é o
    // que passa a endereçar outro tipo quando um tipo novo entra no meio da lista.
    let table = kinds_table();
    let kind = table
        .iter()
        .position(|s| s.noise_labels.is_some())
        .expect("a tabela tem um tipo que lê ruído");
    publish(vec![row(u8::try_from(kind).expect("kind cabe em u8"))]);
    for (what, id) in [
        ("Size", ids::filter_scale_id(0)),
        ("Detail", ids::filter_detail_id(0)),
        ("Seed", ids::filter_seed_id(0)),
    ] {
        let mut host = MockPanelHost::with_panel::<VectorPanel>();
        let mut st = VectorPanelState;
        let rect = host
            .painted_rect::<VectorPanel>(&mut st, VIEWPORT, id)
            .unwrap_or_else(|| panic!("o slider {what} nao foi PINTADO"));
        // Arrasta ate ~70% do trilho: longe do valor publicado, para o evento nao poder ser
        // confundido com o estado que ja estava la.
        let (x, y) = (rect.x + rect.w * 0.7, rect.y + rect.h * 0.5);
        // ⚠️ **O `ValueChanged` sai no DOWN**, não no Up — um gate que aplicasse só os eventos do
        // Up ficaria vermelho sobre um slider perfeitamente vivo (foi o que esta função fez na 1ª
        // versão).
        let mut evs = host.dispatch_pointer_event(pointer(PointerKind::Down, x, y, SEC));
        evs.extend(host.dispatch_pointer_event(pointer(PointerKind::Up, x, y, SEC + SEC / 100)));
        for ev in evs {
            host.apply_panel_event::<VectorPanel>(&mut st, ev);
        }
        let reached = host.drained_actions().into_iter().any(|a| {
            matches!(a, EditorAction::ToolPanelEvent(PanelEvent::SetValue(got, _)) if got == id)
        });
        assert!(
            reached,
            "arrastar o {what} nao emitiu valor nenhum ao barramento — o slider esta pintado e \
             inerte"
        );
    }
}

/// **Os três knobs do ajuste chegam ao bus com a régua BIPOLAR.** Irmão exacto do gate do Amount
/// logo abaixo, e pelo mesmo motivo: o meio do curso é o NEUTRO, e um mapa unipolar herdado do
/// raio faria a metade esquerda de cada um ler como um ajuste positivo pequeno.
#[test]
fn the_three_adjust_knobs_reach_the_bus_with_a_bipolar_map_when_dragged() {
    let table = kinds_table();
    let kind = table
        .iter()
        .position(|s| s.adjust_labels.is_some())
        .expect("a tabela tem um tipo que ajusta cor");
    publish(vec![row(u8::try_from(kind).expect("kind cabe em u8"))]);
    for (id, chip) in [
        (ids::filter_hue_id(0), ids::filter_hue_num_id(0)),
        (ids::filter_sat_id(0), ids::filter_sat_num_id(0)),
        (ids::filter_bright_id(0), ids::filter_bright_num_id(0)),
    ] {
        for (frac, want_negative) in [(0.15_f32, true), (0.85, false)] {
            let mut host = MockPanelHost::with_panel::<VectorPanel>();
            let mut st = VectorPanelState;
            let rect = host
                .painted_rect::<VectorPanel>(&mut st, VIEWPORT, id)
                .expect("um slider de ajuste nao foi PINTADO");
            let (x, y) = (rect.x + rect.w * frac, rect.y + rect.h * 0.5);
            let mut evs = host.dispatch_pointer_event(pointer(PointerKind::Down, x, y, SEC));
            evs.extend(host.dispatch_pointer_event(pointer(
                PointerKind::Up,
                x,
                y,
                SEC + SEC / 100,
            )));
            for ev in evs {
                host.apply_panel_event::<VectorPanel>(&mut st, ev);
            }
            let value = host.drained_actions().into_iter().find_map(|a| match a {
                EditorAction::ToolPanelEvent(PanelEvent::SetValue(got, v)) if got == id => Some(v),
                _ => None,
            });
            let v = value.expect("arrastar um knob de ajuste nao emitiu valor ao barramento");
            assert_eq!(
                v < 0.0,
                want_negative,
                "a {frac:.0}% do trilho o knob saiu {v:.3} — a régua deixou de ser bipolar"
            );
            // ⚠️ **E o READOUT tem de dizer o mesmo número.** Esta metade não é redundante: o
            // valor no bus vem do mapa do `event_filters`, e o número que o artista LÊ vem do
            // link que o `populate` registou — são DUAS portas para a mesma régua, e uma
            // mutação que tornou só o `populate` unipolar passou por este gate sem ela. O
            // sintoma seria o documento receber −90 enquanto o chip mostra 0.
            let shown = host
                .store()
                .number_value(chip)
                .expect("o chip do knob de ajuste nao e' um NumberInput registado");
            assert_eq!(
                shown < 0.0,
                want_negative,
                "a {frac:.0}% do trilho o bus levou {v:.3} e o chip mostra {shown:.3} — as duas \
                 réguas discordam"
            );
        }
    }
}

/// **O Amount do Grow / Shrink chega ao bus quando arrastado** — e o mapa é BIPOLAR.
///
/// ⚠️ O `ValueChanged` sai no DOWN (a lição do gate irmão do ruído), e o meio do curso é o ZERO:
/// arrastar para a esquerda tem de emitir NEGATIVO. Um mapa unipolar herdado do raio faria a
/// metade esquerda do slider ler como um crescimento pequeno em vez de um encolhimento.
#[test]
fn the_grow_knob_reaches_the_bus_with_a_bipolar_map_when_dragged() {
    let table = kinds_table();
    let kind = table
        .iter()
        .position(|s| s.grow_label.is_some())
        .expect("a tabela tem um tipo que engorda");
    publish(vec![row(u8::try_from(kind).expect("kind cabe em u8"))]);
    let id = ids::filter_grow_id(0);
    // 15% do trilho (bem à esquerda do meio) tem de virar um valor NEGATIVO; 85%, positivo.
    for (frac, want_negative) in [(0.15_f32, true), (0.85, false)] {
        let mut host = MockPanelHost::with_panel::<VectorPanel>();
        let mut st = VectorPanelState;
        let rect = host
            .painted_rect::<VectorPanel>(&mut st, VIEWPORT, id)
            .expect("o slider Amount nao foi PINTADO");
        let (x, y) = (rect.x + rect.w * frac, rect.y + rect.h * 0.5);
        let mut evs = host.dispatch_pointer_event(pointer(PointerKind::Down, x, y, SEC));
        evs.extend(host.dispatch_pointer_event(pointer(PointerKind::Up, x, y, SEC + SEC / 100)));
        for ev in evs {
            host.apply_panel_event::<VectorPanel>(&mut st, ev);
        }
        let value = host.drained_actions().into_iter().find_map(|a| match a {
            EditorAction::ToolPanelEvent(PanelEvent::SetValue(got, v)) if got == id => Some(v),
            _ => None,
        });
        let v = value.expect("arrastar o Amount nao emitiu valor nenhum ao barramento");
        assert_eq!(
            v < 0.0,
            want_negative,
            "a {frac:.0}% do trilho o Amount saiu {v:.3} — a régua deixou de ser bipolar"
        );
    }
}
