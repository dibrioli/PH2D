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
        opacity: 1.0,
        blend: 0,
        scale: 0.25,
        detail: 3,
        seed: 0,
        grow: 0.06,
        hue: 0.0,
        sat: 0.0,
        bright: 0.0,
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
        modes,
        takes_blend,
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
