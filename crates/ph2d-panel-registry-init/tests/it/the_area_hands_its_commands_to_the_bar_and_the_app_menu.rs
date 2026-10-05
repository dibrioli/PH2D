//! ⭐⭐⭐ **O PAINEL DEIXA DE SER O DEPÓSITO** — cada comando vai ao sítio que a **D2** lhe dá.
//!
//! A D2 (`docs/UI_New_and_Simple/00_DECISOES_DO_ENIO.md`) corta os comandos de um módulo por
//! ÂMBITO: *«se o comando vale em todo o app vai à barra; se vale só naquele editor vai ao
//! cabeçalho dele»*. A porta é UMA — `WidgetStore::set_area_commands(menus, contrib)` —, escrita em
//! todo quadro pelo módulo que tem o canvas:
//!
//! | metade | o que é | onde aparece |
//! |---|---|---|
//! | `menus` | os pulldowns da área | um chip por pulldown na fila de ferramentas |
//! | `contrib` | linhas a SOMAR a um menu do app | hoje o *File* |
//!
//! ⚠️ **O produtor aqui é uma FIXTURA**, e de propósito: o 1.º produtor (o modelador 3D) saiu do
//! produto com o ADR-0179, e a lei é do MECANISMO, não dele. A fixtura regista os ids das linhas
//! como `Button` (é o que o `populate` de um painel faz) e escreve o `ButtonState` de cada uma —
//! *fiar o clique não é fiar o ESTADO*.
//!
//! # ⚠️ E a fila não cresce
//!
//! > Enio, 2026-08-31: *«esse app tem tablets e iPad como alvo. Não podemos ir perdendo espaço.»*
//!
//! Uma faixa própria (o *cabeçalho de área*) foi construída e revertida no mesmo dia (`28 px` de
//! altura permanente). ⭐ **O orçamento medido é `3` chips de área** (sonda de 2026-09-01: com `4` o
//! iPad 11 e o mini passam a duas linhas).
//!
//! # ⚠️ E o gate carrega num PIXEL
//!
//! Um `apply_event(Click(id))` sintético passaria com o chip **morto sob o dedo**. Quem fecha um
//! menu ao servir, sob o ponteiro, é a regra genérica do store (`pointer_down` fecha todo menu
//! aberto num Down primário que não lhe pertença) — a única excepção é a **paleta de comandos**, que
//! tem gate próprio no fim deste ficheiro.

use ph2d_editor_core::NodeId;
use ph2d_editor_core::interaction::{
    AreaMenu, ContextMenuKind, InteractiveState, WidgetEvent, WidgetStore,
};
use ph2d_editor_core::screens::hero::{HeroScreen, tool_bar};
use ph2d_editor_core::widget::{ButtonState, RailButtonSize, ToolRailEntry};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind};
use ph2d_i18n::tr;
use ph2d_text::TextSystem;

const TABLETS: [(&str, f32, f32); 3] = [
    ("iPad 12.9", 1366.0, 1024.0),
    ("iPad 11", 1194.0, 834.0),
    ("iPad mini", 1133.0, 744.0),
];

/// Os rótulos das linhas da fixtura — chaves REAIS (HR-15), cuja escolha não pesa na lei.
const ROTULOS: [&str; 4] = [
    "object_mode.object",
    "object_mode.paint",
    "object_mode.draw",
    "object_mode.edit",
];

/// As fileiras da fixtura: `(família, quantas linhas)`. O pulldown 0 tem a `vista`; o 1 tem TRÊS
/// fileiras (dois riscos entre elas); o *File* recebe a `saida`.
const VISTA: (&str, usize) = ("vista", 3);
const MODO: (&str, usize) = ("modo", 2);
const OLHAR: (&str, usize) = ("olhar", 2);
const EXPOSICAO: (&str, usize) = ("exposicao", 5);
const SAIDA: (&str, usize) = ("saida", 3);

/// O id da linha `i` da fileira `familia` — cunhado em runtime, só para este gate.
fn row_id(familia: &str, i: usize) -> NodeId {
    ph2d_tool_registry::hash_node_id_runtime(&format!("test.area_commands.{familia}.{i}"))
}

/// Qual linha de cada fileira está ACESA no retrato publicado.
#[derive(Clone, Copy, Default)]
struct Acesos {
    vista: Option<usize>,
    modo: Option<usize>,
    olhar: Option<usize>,
    exposicao: Option<usize>,
}

/// Monta as entradas de uma lista de fileiras **e escreve o `ButtonState` de cada uma** — a metade
/// que um produtor esquece. Um risco só nasce com algo por cima E algo por baixo.
fn entries(
    store: &mut WidgetStore,
    fileiras: &[((&str, usize), Option<usize>)],
) -> Vec<ToolRailEntry> {
    let mut out = Vec::new();
    for ((familia, n), aceso) in fileiras {
        if !out.is_empty() && *n > 0 {
            out.push(ToolRailEntry::Divider);
        }
        for i in 0..*n {
            let id = row_id(familia, i);
            let label = tr(ROTULOS[i % ROTULOS.len()]);
            if store.get(id).is_none() {
                store.register(
                    id,
                    InteractiveState::Button {
                        state: ButtonState::Normal,
                    },
                );
            }
            if let Some(InteractiveState::Button { state }) = store.get_mut(id) {
                *state = if *aceso == Some(i) {
                    ButtonState::Pressed
                } else {
                    ButtonState::Normal
                };
            }
            out.push(ToolRailEntry::compound(id, label, label, ""));
        }
    }
    out
}

/// ⭐ **O produtor de teste** — publica (ou, desarmado, apaga) os dois pulldowns e a contribuição ao
/// *File*, pela porta única. Chamado em todo «quadro», vazio incluído, como um módulo real.
fn publica(store: &mut WidgetStore, armed: bool, acesos: Acesos) {
    if !armed {
        store.set_area_commands(Vec::new(), Vec::new());
        return;
    }
    let menu = |rotulo: &str, face: &str, rows| AreaMenu {
        label: tr(rotulo).to_string(),
        face: tr(face).to_string(),
        faces: ROTULOS.iter().map(|k| tr(k).to_string()).collect(),
        rows,
    };
    let vista = entries(store, &[(VISTA, acesos.vista)]);
    let tres = entries(
        store,
        &[
            (MODO, acesos.modo),
            (OLHAR, acesos.olhar),
            (EXPOSICAO, acesos.exposicao),
        ],
    );
    let saida = entries(store, &[(SAIDA, None)]);
    store.set_area_commands(
        vec![
            menu("object_mode.menu", "object_mode.object", vista),
            menu("object_mode.menu", "object_mode.edit", tres),
        ],
        vec![(ContextMenuKind::MenuBarFile, saida)],
    );
}

fn hero(w: f32, h: f32, armed: bool) -> (HeroScreen, Rect) {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut hero = HeroScreen::new(NodeId(1));
    hero.view.legacy_chrome = false;
    publica(&mut hero.store, armed, Acesos::default());
    (hero, Rect::new(0.0, 0.0, w, h))
}

fn paint(h: &mut HeroScreen, vp: Rect) {
    let _ = paint_counting(h, vp);
}

/// [`paint`], devolvendo **quantos segmentos de caminho a cena de facto recebeu**.
///
/// ⛔⛔ **Um gate que lê o `ButtonState` mede a PUBLICAÇÃO, não a PINTURA.** ⚠️ O número absoluto
/// não significa nada — o que significa é a DIFERENÇA entre dois retratos que só discordam no que se
/// está a medir.
fn paint_counting(h: &mut HeroScreen, vp: Rect) -> u32 {
    let mut scene = ph2d_vector::VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    for _ in 0..2 {
        ph2d_editor_core::screens::hero::paint_hero_screen(h, vp, &mut scene, &mut text);
    }
    scene.inner().encoding().n_path_segments
}

fn pointer(kind: PointerKind, x: f32, y: f32) -> PointerEvent {
    PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: ph2d_host::PointerSource::Mouse,
        button: PointerButton::Primary,
        timestamp_ns: 0,
    }
}

/// ⚠️ Down **e** Up pelo `dispatch_pointer` — um `Click` sintético salta exactamente a metade que
/// falha (o `HitIndex` resolver o ponto). Devolve os eventos que o dedo levantou.
fn click_at(h: &mut HeroScreen, r: Rect) -> Vec<WidgetEvent> {
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let arena = bumpalo::Bump::new();
    let mut todos = Vec::new();
    for ev in [
        pointer(PointerKind::Down, x, y),
        pointer(PointerKind::Up, x, y),
    ] {
        let events =
            ph2d_editor_core::interaction::dispatch_pointer(&mut h.store, &h.hit_index, ev, &arena);
        let evs: Vec<_> = events.to_vec();
        for e in evs {
            h.apply_event(e);
            todos.push(e);
        }
    }
    todos
}

/// Abre o chip do pulldown `slot` — com o dedo, não com um `Click` fabricado.
fn open_area_menu(h: &mut HeroScreen, vp: Rect, slot: u32) {
    let chip = h
        .hit_index
        .rect_for(ph2d_editor_core::ids::area_menu_button(slot))
        .unwrap_or_else(|| {
            panic!("o pulldown {slot} da area nao foi registado no indice — nasceu invisivel")
        });
    let _ = click_at(h, chip);
    paint(h, vp);
}

/// ⭐⭐⭐ **O pulldown da área abre, serve, e fecha-se ao servir.**
#[test]
fn the_area_pulldown_opens_serves_a_row_and_closes() {
    let (mut h, vp) = hero(1194.0, 834.0, true);
    paint(&mut h, vp);
    let linha = row_id(VISTA.0, 0);

    // ⛔ **Antes de abrir, as linhas NÃO estão em lado nenhum** — é isto que prova que o comando
    // vive no pulldown e não ganhou um segundo sítio.
    assert!(
        h.hit_index.rect_for(linha).is_none(),
        "a linha continua registada com o menu fechado — ela ganhou um 2.o sitio"
    );

    open_area_menu(&mut h, vp, 0);
    assert!(
        matches!(
            h.store.context_menu().map(|r| r.kind),
            Some(ContextMenuKind::AreaCommands { slot: 0 })
        ),
        "o chip nao abriu o pulldown 0 — ele esta' morto sob o dedo"
    );

    let row = h
        .hit_index
        .rect_for(linha)
        .expect("a 1.a linha nao foi pintada no menu aberto");
    let eventos = click_at(&mut h, row);
    assert!(
        eventos.contains(&WidgetEvent::Click(linha)),
        "carregar na linha nao chegou ao barramento como um clique dela: {eventos:?}"
    );
    assert!(
        h.store.context_menu().is_none(),
        "o menu ficou aberto POR CIMA da coisa que o clique acabou de fazer"
    );
}

/// Quantos chips a ÁREA acrescenta à fila, e em quantas linhas ela cabe.
fn area_chips_and_lines(h: &HeroScreen, vp: Rect) -> (usize, usize) {
    let bands = ph2d_editor_core::screens::layout::ChromeBands {
        rail_w: 0.0,
        top_bar_h: ph2d_editor_core::screens::hero::menu_bar::MENU_BAR_H,
        tool_bar_h: tool_bar::tool_bar_h(RailButtonSize::Small, 1),
        ..ph2d_editor_core::screens::layout::ChromeBands::DEFAULT
    };
    let area_w = ph2d_editor_core::screens::layout::HeroLayout::for_viewport_bands(
        vp,
        false,
        bands,
        ph2d_editor_core::screens::layout::CenterSplit::None,
        ph2d_editor_core::screens::layout::DockSides::BOTH,
    )
    .draw_area
    .w;
    let (rail, over) = tool_bar::bar_split(
        &h.store,
        &mut TextSystem::without_system_fonts(),
        false,
        false,
        area_w,
    );
    let area_ids: Vec<_> = (0..ph2d_editor_core::ids::MAX_AREA_MENUS)
        .map(ph2d_editor_core::ids::area_menu_button)
        .collect();
    let added = rail
        .entries
        .iter()
        .chain(over.iter())
        .filter(|e| e.node_id().is_some_and(|id| area_ids.contains(&id)))
        .count();
    let lines =
        ph2d_editor_core::widget::horizontal_lines(&rail, area_w - 16.0, RailButtonSize::Small);
    (added, lines)
}

/// ⭐⭐⭐ **Dois pulldowns, e a fila continua numa linha nos três tablets.** O orçamento medido é
/// `3` (2026-09-01); este gate mede o que a fixtura de facto gasta, e imprime-o.
#[test]
fn two_area_chips_and_the_bar_is_still_one_line() {
    for (name, w, ht) in TABLETS {
        let (h, vp) = hero(w, ht, true);
        let (added, lines) = area_chips_and_lines(&h, vp);
        println!("{name:11} a area acrescenta {added} chip(s) e a fila cabe em {lines} linha(s)");
        assert_eq!(added, 2, "{name}: a fixtura publica DOIS pulldowns de area");
        assert_eq!(lines, 1, "{name}: a fila precisa de {lines} linhas");
    }
}

/// Abre o pulldown 1 (as três fileiras) com este retrato e devolve o `HeroScreen` **com o menu
/// aberto**.
fn hero_with_rows_menu_open(acesos: Acesos) -> (HeroScreen, Rect) {
    let (mut h, vp) = hero(1194.0, 834.0, true);
    publica(&mut h.store, true, acesos);
    paint(&mut h, vp);
    open_area_menu(&mut h, vp, 1);
    // ⚠️ **Re-publicar não é cerimónia:** o produtor corre em TODO quadro, e o `Down` que abriu o
    // menu escreveu `Pressed` no chip sob o dedo.
    publica(&mut h.store, true, acesos);
    (h, vp)
}

/// ⭐⭐⭐ **A MARCA CHEGA AO PIXEL, e um ponto é um ESTADO** (report do dono, 2026-09-13: *«coloque a
/// marca de seleção no render selecionado»*).
///
/// ⚠️ **A régua é a DIFERENÇA entre três retratos que só discordam no `active`**, e a 2.ª asserção
/// separa *«pintou alguma coisa»* de *«pintou UM ponto por fileira acesa»*.
#[test]
fn the_mark_reaches_the_pixel_and_one_bullet_is_one_state() {
    let conta = |acesos| {
        let (mut h, vp) = hero_with_rows_menu_open(acesos);
        paint_counting(&mut h, vp)
    };
    let nenhuma = conta(Acesos::default());
    let uma = conta(Acesos {
        modo: Some(1),
        ..Acesos::default()
    });
    let tres = conta(Acesos {
        modo: Some(1),
        olhar: Some(1),
        exposicao: Some(4),
        ..Acesos::default()
    });
    println!("segmentos: 0 acesas {nenhuma} · 1 acesa {uma} · 3 acesas {tres}");
    assert!(
        uma > nenhuma,
        "a linha ACESA nao pintou nada a mais do que uma apagada ({uma} contra {nenhuma}) — o menu \
         abre com as linhas todas iguais, que e' o report do dono"
    );
    assert_eq!(
        tres - nenhuma,
        (uma - nenhuma) * 3,
        "tres fileiras acesas nao custaram tres pontos ({tres} - {nenhuma} contra 3 x ({uma} - \
         {nenhuma})) — a marca nao e' por FILEIRA"
    );
}

/// ⭐⭐⭐ **UM RISCO ENTRE AS FILEIRAS** — a prova de que nove linhas são **três perguntas**.
///
/// ⚠️ **A régua é o índice de acerto**: duas linhas da mesma fileira ficam a `ROW_H` uma da outra;
/// atravessar uma fronteira custa `ROW_H` **mais a banda do risco** — e os dois riscos têm de medir o
/// mesmo, senão não é uma lei, são dois números.
///
/// **Mutação que deve sangrar:** o braço `MenuRow::Divider` do pintor — as três distâncias colapsam
/// em `ROW_H`.
#[test]
fn a_rule_between_the_rows_says_they_are_three_questions() {
    let (h, _vp) = hero_with_rows_menu_open(Acesos {
        modo: Some(0),
        olhar: Some(0),
        exposicao: Some(2),
        ..Acesos::default()
    });
    let y = |id| {
        h.hit_index
            .rect_for(id)
            .unwrap_or_else(|| panic!("a linha {id:?} nao foi pintada no menu aberto"))
            .y
    };
    let modo0 = y(row_id(MODO.0, 0));
    let modo1 = y(row_id(MODO.0, 1));
    let olhar0 = y(row_id(OLHAR.0, 0));
    let olhar1 = y(row_id(OLHAR.0, 1));
    let exp0 = y(row_id(EXPOSICAO.0, 0));

    let dentro = modo1 - modo0;
    let primeiro = olhar0 - modo1 - dentro;
    let segundo = exp0 - olhar1 - dentro;
    println!("passo dentro da fileira {dentro} · riscos {primeiro} e {segundo}");
    assert!(
        primeiro > 0.0,
        "nao ha nada entre as duas fileiras: elas leem-se como uma lista so"
    );
    assert!(
        (primeiro - segundo).abs() < 0.01,
        "os dois riscos medem diferente ({primeiro} contra {segundo}) — nao e' uma lei, sao dois \
         numeros"
    );
}

/// ⭐⭐⭐ **A CONTRIBUIÇÃO vive no menu do APP** — e as linhas dele continuam lá (SOMA, nunca
/// substitui).
#[test]
fn the_file_menu_serves_the_contributed_row_and_keeps_its_own() {
    let (mut h, vp) = hero(1194.0, 834.0, true);
    paint(&mut h, vp);

    let title = h
        .hit_index
        .rect_for(ph2d_editor_core::ids::MENUBAR_FILE)
        .expect("o titulo `File` nao esta' no indice de acerto");
    let _ = click_at(&mut h, title);
    paint(&mut h, vp);

    assert!(
        h.hit_index
            .rect_for(ph2d_editor_core::ids::CTX_MENU_SAVE)
            .is_some(),
        "as linhas que o menu `File` ja' tinha desapareceram — a contribuicao substituiu em vez de somar"
    );

    let linha = row_id(SAIDA.0, 1);
    let row = h
        .hit_index
        .rect_for(linha)
        .expect("a linha contribuida nao foi pintada no menu `File`");
    let eventos = click_at(&mut h, row);
    assert!(
        eventos.contains(&WidgetEvent::Click(linha)),
        "carregar na linha contribuida nao chegou ao barramento: {eventos:?}"
    );
}

/// ⭐⭐ **Desarmar o produtor tira os chips da fila E as linhas do menu do app, no MESMO quadro.**
///
/// ⚠️ É a lei do `⋯`: publicado em todo quadro, vazio incluído. Sem o ramo vazio o chip ficava lá, a
/// despachar para um módulo que já não está — e o *File* ficava com linhas que não fazem nada.
#[test]
fn closing_the_module_takes_the_commands_off_the_bar_and_off_the_file_menu() {
    let (mut h, vp) = hero(1194.0, 834.0, true);
    paint(&mut h, vp);
    assert!(
        h.hit_index
            .rect_for(ph2d_editor_core::ids::area_menu_button(0))
            .is_some(),
        "controlo: com o produtor armado o pulldown tem de estar la'"
    );

    publica(&mut h.store, false, Acesos::default());
    paint(&mut h, vp);
    for slot in 0..ph2d_editor_core::ids::MAX_AREA_MENUS {
        assert!(
            h.hit_index
                .rect_for(ph2d_editor_core::ids::area_menu_button(slot))
                .is_none(),
            "o pulldown {slot} sobreviveu ao fecho do modulo"
        );
    }
    assert!(
        h.store.area_menus().is_empty(),
        "os comandos da area sobreviveram ao fecho do modulo"
    );
    assert!(
        h.store
            .menu_contrib(ContextMenuKind::MenuBarFile)
            .is_empty(),
        "a contribuicao sobreviveu no menu `File` depois de o modulo fechar"
    );

    // E o menu do app abre, sem as linhas do módulo.
    let title = h
        .hit_index
        .rect_for(ph2d_editor_core::ids::MENUBAR_FILE)
        .expect("o titulo `File` nao esta' no indice de acerto");
    let _ = click_at(&mut h, title);
    paint(&mut h, vp);
    assert!(
        h.hit_index
            .rect_for(ph2d_editor_core::ids::CTX_MENU_SAVE)
            .is_some(),
        "controlo: o menu `File` tem de abrir com as linhas dele"
    );
    assert!(
        h.hit_index.rect_for(row_id(SAIDA.0, 0)).is_none(),
        "a linha contribuida continua no menu `File` com o modulo fechado"
    );
}

/// ⭐⭐⭐ **O CAMINHO SEM DEDO** — a paleta de comandos, que não tem `Down` nenhum.
///
/// ⛔⛔ **Este gate existe porque uma MUTAÇÃO sobreviveu.** Sob o ponteiro, quem fecha um menu ao
/// servir é a regra genérica do store, e não o handler do chip: apagar o interruptor de lá deixava o
/// gate de gesto verde. Mas a **paleta de comandos global** levanta `apply_event(Click(id))` sem
/// ponteiro nenhum — e por esse caminho, sem o interruptor, escolher o chip **re-abriria** um menu já
/// aberto.
#[test]
fn the_palette_path_toggles_the_menu_because_it_has_no_pointer_down() {
    let (mut h, vp) = hero(1194.0, 834.0, true);
    paint(&mut h, vp);
    let click = WidgetEvent::Click(ph2d_editor_core::ids::area_menu_button(0));
    assert!(
        h.apply_event(click),
        "controlo: o chip nao respondeu ao clique sintetico"
    );
    assert!(
        matches!(
            h.store.context_menu().map(|r| r.kind),
            Some(ContextMenuKind::AreaCommands { slot: 0 })
        ),
        "a paleta nao abriu o menu"
    );
    assert!(h.apply_event(click));
    assert!(
        h.store.context_menu().is_none(),
        "pela paleta o 2.o pick RE-ABRIU o menu — o interruptor do chip e' a unica coisa que o fecha \
         quando nao ha' Down"
    );
}
